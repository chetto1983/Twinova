//! The debugger's commands (specs 2026-11-13-breakpoints-hold-design.md and -breakpoints-complete-design.md):
//! breakpoints with their conditions, hit counts and log messages, Run to cursor, resume, where the program is held, and
//! the logpoints' lines. None waits for the engine, which a held scan holds: the PLC's and the field's debuggers live
//! beside it in `AppState`. The editor asks where the program is held, since a stop sends no event while its scan holds
//! the Run loop.

use tauri::State;
use twinova_engine::debugger::{Breakpoint, Debugger, Hold, Log, Resume};
use twinova_engine::watch::Scope;

use crate::commands::{AppState, HOLD, with_engine};

/// The runtime a file's breakpoints go to: a block's to the field, a program file's to the PLC.
fn scope_of(file: &str) -> Scope {
    match file.starts_with("components/") {
        true => Scope::Field,
        false => Scope::Plc,
    }
}

/// The debugger of `file`'s runtime, attached at its first breakpoint (decision 10).
fn attached(state: &AppState, file: &str) -> Result<Debugger, String> {
    let scope = scope_of(file);
    if let Some(debugger) = state.debugger(scope)? {
        return Ok(debugger);
    }
    let debugger = with_engine(state, |e| match scope {
        Scope::Plc => Ok(e.debugger()),
        Scope::Field => e
            .field_debugger()
            .ok_or_else(|| "no device has a behaviour".to_string()),
    })?;
    state.set_debugger(scope, debugger.clone())?;
    Ok(debugger)
}

/// Sets `file`'s breakpoints and gives them as set, each on its statement's line.
#[tauri::command]
pub fn set_breakpoints(
    state: State<'_, AppState>,
    file: String,
    breakpoints: Vec<Breakpoint>,
) -> Result<Vec<Breakpoint>, String> {
    // Debug on costs even with no breakpoint (`release_idle_field`): nothing to take off attaches nothing.
    let set = match (breakpoints.is_empty(), state.debugger(scope_of(&file))?) {
        (true, None) => Vec::new(),
        _ => attached(&state, &file)?.set(&file, &breakpoints)?,
    };
    let mut kept = state.breakpoints.lock().map_err(|e| e.to_string())?;
    match set.is_empty() {
        true => kept.remove(&file),
        false => kept.insert(file, set.clone()),
    };
    drop(kept);
    release_idle_field(&state)?;
    Ok(set)
}

/// Arms the user's breakpoints in the engine about to scan, attaching a debugger where there are some.
pub(crate) fn arm(state: &AppState) -> Result<(), String> {
    let kept = state.breakpoints.lock().map_err(|e| e.to_string())?.clone();
    for (file, breakpoints) in kept {
        attached(state, &file)?.set(&file, &breakpoints)?;
    }
    release_idle_field(state)
}

/// Turns the field's debug off once no block has a breakpoint: on, its 1 ms scans run 7.7 times slower (measured
/// 2026-11-13, release). The PLC's stays on, since forces and writes use it too. A held line keeps it to the next Run
/// or Step, which arm again.
fn release_idle_field(state: &AppState) -> Result<(), String> {
    let kept = state.breakpoints.lock().map_err(|e| e.to_string())?;
    if kept.keys().any(|file| scope_of(file) == Scope::Field) {
        return Ok(());
    }
    drop(kept);
    if state.debugger(Scope::Field)?.is_none() || state.held().is_some() {
        return Ok(());
    }
    with_engine(state, |e| {
        e.release_field_debugger();
        Ok(())
    })?;
    state.drop_debugger(Scope::Field)
}

#[tauri::command]
pub fn run_to_cursor(state: State<'_, AppState>, file: String, line: u32) -> Result<(), String> {
    attached(&state, &file)?.run_to(&file, line)
}

#[tauri::command]
pub fn resume(state: State<'_, AppState>, how: Resume) -> Result<(), String> {
    let (_, debugger) = state.held().ok_or("the PLC is not held")?;
    debugger.resume(how);
    // A block's last breakpoint taken off while held: its debug goes off now, or at the next Run or Step if the line
    // stops again first.
    match release_idle_field(&state) {
        Err(e) if e == HOLD => Ok(()),
        released => released,
    }
}

/// Where the program is held, with its call stack and, in a block, its device; `None` while it runs.
#[tauri::command]
pub fn hold(state: State<'_, AppState>) -> Result<Option<Hold>, String> {
    Ok(state.held().and_then(|(_, d)| d.held()))
}

/// The lines the logpoints wrote since the last call, the PLC's and the field's.
#[tauri::command]
pub fn debug_logs(state: State<'_, AppState>) -> Vec<Log> {
    state.debuggers().iter().flat_map(Debugger::logs).collect()
}

#[cfg(test)]
#[path = "debug_commands_tests.rs"]
mod tests;
