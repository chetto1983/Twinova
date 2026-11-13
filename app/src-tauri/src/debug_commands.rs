//! The debugger's commands (specs 2026-11-13-breakpoints-hold-design.md and -breakpoints-complete-design.md):
//! breakpoints with their conditions, hit counts and log messages, Run to cursor, resume, where the program is held, and
//! the logpoints' lines. None waits for the engine, which a held scan holds: the PLC's and the field's debuggers live
//! beside it in `AppState`. The editor asks where the program is held, since a stop sends no event while its scan holds
//! the Run loop.

use tauri::State;
use twinova_engine::debugger::{Breakpoint, Debugger, Hold, Log, Resume};
use twinova_engine::watch::Scope;

use crate::commands::{AppState, with_engine};

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
    let set = attached(&state, &file)?.set(&file, &breakpoints)?;
    let mut kept = state.breakpoints.lock().map_err(|e| e.to_string())?;
    match set.is_empty() {
        true => kept.remove(&file),
        false => kept.insert(file, set.clone()),
    };
    Ok(set)
}

/// Arms the user's breakpoints in the engine about to scan, attaching a debugger where there are some.
pub(crate) fn arm(state: &AppState) -> Result<(), String> {
    let kept = state.breakpoints.lock().map_err(|e| e.to_string())?.clone();
    for (file, breakpoints) in kept {
        attached(state, &file)?.set(&file, &breakpoints)?;
    }
    Ok(())
}

#[tauri::command]
pub fn run_to_cursor(state: State<'_, AppState>, file: String, line: u32) -> Result<(), String> {
    attached(&state, &file)?.run_to(&file, line)
}

#[tauri::command]
pub fn resume(state: State<'_, AppState>, how: Resume) -> Result<(), String> {
    let (_, debugger) = state.held().ok_or("the PLC is not held")?;
    debugger.resume(how);
    Ok(())
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
