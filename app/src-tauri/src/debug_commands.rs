//! The debugger's commands (spec 2026-11-13-breakpoints-hold-design.md): breakpoints, Run to cursor, resume, and where
//! the program is held. None waits for the engine, which a held scan holds (decision 4): the debugger lives beside it
//! in `AppState`. The editor asks where the program is held, since a stop sends no event while its scan holds the Run
//! loop.

use tauri::State;
use twinova_engine::debugger::{Debugger, Hold, Resume};

use crate::commands::{AppState, with_engine};

/// The debugger, attached at the first breakpoint (decision 10).
fn attached(state: &AppState) -> Result<Debugger, String> {
    if let Some(debugger) = state.debugger()? {
        return Ok(debugger);
    }
    let debugger = with_engine(state, |e| Ok(e.debugger()))?;
    state.set_debugger(Some(debugger.clone()))?;
    Ok(debugger)
}

/// Sets `file`'s breakpoints to `lines` and gives the lines they landed on, each its statement's.
#[tauri::command]
pub fn set_breakpoints(
    state: State<'_, AppState>,
    file: String,
    lines: Vec<u32>,
) -> Result<Vec<u32>, String> {
    let set = attached(&state)?.set(&file, &lines)?;
    let mut kept = state.breakpoints.lock().map_err(|e| e.to_string())?;
    match set.is_empty() {
        true => kept.remove(&file),
        false => kept.insert(file, set.clone()),
    };
    Ok(set)
}

/// Arms the user's breakpoints in the engine about to scan, attaching the debugger when there are some.
pub(crate) fn arm(state: &AppState) -> Result<(), String> {
    let kept = state.breakpoints.lock().map_err(|e| e.to_string())?.clone();
    if kept.is_empty() {
        return Ok(());
    }
    let debugger = attached(state)?;
    for (file, lines) in kept {
        debugger.set(&file, &lines)?;
    }
    Ok(())
}

#[tauri::command]
pub fn run_to_cursor(state: State<'_, AppState>, file: String, line: u32) -> Result<(), String> {
    attached(&state)?.run_to(&file, line)
}

#[tauri::command]
pub fn resume(state: State<'_, AppState>, how: Resume) -> Result<(), String> {
    state.debugger()?.ok_or("no breakpoint is set")?.resume(how);
    Ok(())
}

/// Where the program is held, with its call stack; `None` while it runs.
#[tauri::command]
pub fn hold(state: State<'_, AppState>) -> Result<Option<Hold>, String> {
    Ok(state.debugger()?.and_then(|d| d.held()))
}

#[cfg(test)]
#[path = "debug_commands_tests.rs"]
mod tests;
