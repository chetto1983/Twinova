//! Tauri commands and the Run loop. The engine lives behind one mutex; Run is a thread that paces it to the wall
//! clock and emits a `snapshot` event about 30 times a second.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Runtime, State};

use crate::engine::{Engine, Saved, Snapshot};
use crate::plc::Plc;

pub const DEFAULT_PROGRAM: &str = include_str!("default_program.st");
const FRAME: Duration = Duration::from_millis(33);
/// Wall time spent scanning per tick when running as fast as possible; the rest of the frame lets the UI draw.
const FAST_BUDGET: Duration = Duration::from_millis(25);

#[derive(Default)]
pub struct AppState {
    engine: Arc<Mutex<Option<Engine>>>,
    source: Mutex<String>,
    running: Arc<AtomicBool>,
    /// One saved state per session, with the program it was taken on (spec 2026-11-08-run-modes-design.md).
    saved: Mutex<Option<(String, Saved)>>,
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
    let engine = Engine::new(Plc::build(&source)?)?;
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
    with_engine(&state, |e| e.press(start, stop))
}

#[tauri::command]
pub fn set_outlet_blocked(state: State<'_, AppState>, blocked: bool) -> Result<Snapshot, String> {
    with_engine(&state, |e| {
        e.set_outlet_blocked(blocked)?;
        Ok(e.snapshot())
    })
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>) {
    state.running.store(false, Ordering::SeqCst);
}

/// Saves the current state in the session's slot.
#[tauri::command]
pub fn save_state(state: State<'_, AppState>) -> Result<(), String> {
    let saved = with_engine(&state, |e| Ok(e.save()))?;
    let source = state
        .source
        .lock()
        .map_err(|_| "source lock poisoned")?
        .clone();
    *state.saved.lock().map_err(|_| "saved lock poisoned")? = Some((source, saved));
    Ok(())
}

/// Replays the saved state on a fresh engine built from the program it was taken on.
#[tauri::command]
pub fn restore_state(state: State<'_, AppState>) -> Result<Snapshot, String> {
    state.running.store(false, Ordering::SeqCst);
    let (source, saved) = state
        .saved
        .lock()
        .map_err(|_| "saved lock poisoned")?
        .clone()
        .ok_or("save a state first")?;
    let engine = Engine::restore(Plc::build(&source)?, &saved)?;
    let snapshot = engine.snapshot();
    *state.engine.lock().map_err(|_| "engine lock poisoned")? = Some(engine);
    *state.source.lock().map_err(|_| "source lock poisoned")? = source;
    Ok(snapshot)
}

/// Starts the Run loop at `speed` times the wall clock, or as fast as possible when `speed` is `None`; a second call
/// while running does nothing.
#[tauri::command]
pub fn run<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    speed: Option<f64>,
) -> Result<(), String> {
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
                    match speed {
                        Some(scale) => e.catch_up(origin_ms, wall_ms, scale)?,
                        None => {
                            let tick = Instant::now();
                            e.lag_ms = 0;
                            while tick.elapsed() < FAST_BUDGET {
                                e.step()?;
                            }
                        }
                    }
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
