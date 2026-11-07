//! Tauri commands and the Run loop. The engine lives behind one mutex; Run is a thread that paces it to the wall
//! clock and emits a `snapshot` event about 30 times a second.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Runtime, State};

use crate::engine::{Engine, Snapshot};
use crate::plc::Plc;

pub const DEFAULT_PROGRAM: &str = include_str!("default_program.st");
const FRAME: Duration = Duration::from_millis(33);

#[derive(Default)]
pub struct AppState {
    engine: Arc<Mutex<Option<Engine>>>,
    source: Mutex<String>,
    running: Arc<AtomicBool>,
}

fn with_engine<T>(
    state: &AppState,
    f: impl FnOnce(&mut Engine) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = state
        .engine
        .lock()
        .map_err(|_| "engine lock poisoned".to_string())?;
    let engine = guard.as_mut().ok_or("build a program first")?;
    f(engine)
}

#[tauri::command]
pub fn default_program() -> String {
    DEFAULT_PROGRAM.to_string()
}

/// Compiles `source` and replaces the engine with a fresh one at time zero.
#[tauri::command]
pub fn build(state: State<'_, AppState>, source: String) -> Result<Snapshot, String> {
    state.running.store(false, Ordering::SeqCst);
    let engine = Engine::new(Plc::build(&source)?);
    let snapshot = engine.snapshot();
    *state.engine.lock().map_err(|_| "engine lock poisoned")? = Some(engine);
    *state.source.lock().map_err(|_| "source lock poisoned")? = source;
    Ok(snapshot)
}

/// Back to time zero with the last built program.
#[tauri::command]
pub fn reset(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let source = state
        .source
        .lock()
        .map_err(|_| "source lock poisoned")?
        .clone();
    build(state, source)
}

#[tauri::command]
pub fn step(state: State<'_, AppState>) -> Result<Snapshot, String> {
    state.running.store(false, Ordering::SeqCst);
    with_engine(&state, |e| {
        e.lag_ms = 0;
        e.step()?;
        Ok(e.snapshot())
    })
}

#[tauri::command]
pub fn press(state: State<'_, AppState>, start: bool, stop: bool) -> Result<(), String> {
    with_engine(&state, |e| {
        e.press(start, stop);
        Ok(())
    })
}

#[tauri::command]
pub fn set_outlet_blocked(state: State<'_, AppState>, blocked: bool) -> Result<Snapshot, String> {
    with_engine(&state, |e| {
        e.set_outlet_blocked(blocked);
        Ok(e.snapshot())
    })
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>) {
    state.running.store(false, Ordering::SeqCst);
}

/// Starts the Run loop; a second call while running does nothing.
#[tauri::command]
pub fn run<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) -> Result<(), String> {
    let origin_ms = with_engine(&state, |e| Ok(e.time_ms()))?;
    if state.running.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    let (engine, running) = (state.engine.clone(), state.running.clone());
    thread::spawn(move || {
        let started = Instant::now();
        while running.load(Ordering::SeqCst) {
            let wall_ms = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
            let tick = engine
                .lock()
                .map_err(|_| "engine lock poisoned".to_string())
                .and_then(|mut guard| {
                    let e = guard.as_mut().ok_or("no engine")?;
                    e.catch_up(origin_ms, wall_ms)?;
                    Ok(e.snapshot())
                });
            match tick {
                Ok(snapshot) => {
                    let _ = app.emit("snapshot", snapshot);
                }
                Err(error) => {
                    running.store(false, Ordering::SeqCst);
                    let _ = app.emit("engine-error", error);
                }
            }
            thread::sleep(FRAME);
        }
        let _ = app.emit("stopped", ());
    });
    Ok(())
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
