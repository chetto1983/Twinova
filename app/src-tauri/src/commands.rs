//! Tauri commands and the Run loop. The engine lives behind one mutex; Run is a thread that paces it to the wall
//! clock and emits a `snapshot` event about 30 times a second. Build runs the current project (project_commands.rs)
//! with the ST sources the editor holds (spec 2026-11-09-component-behaviour-design.md, "The app").

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State};
use twinova_project::{Project, encode_project, parse_project};

use crate::engine::{Engine, Saved, Snapshot};
use crate::project_commands::{ProjectState, lines};

/// A tick of the Run loop: its work, then the rest of the frame asleep, so a snapshot is published about 30 times a
/// second (spec 2026-11-08-run-modes-design.md).
const FRAME: Duration = Duration::from_millis(33);
/// Wall time spent scanning per tick when running as fast as possible; the rest of the frame lets the UI draw.
const FAST_BUDGET: Duration = Duration::from_millis(25);

/// What is left of a frame after `spent` of work. Sleeping a whole frame after 25 ms of scans ran as fast as possible
/// 43 % of the time (spec 2026-11-10-run-pace-native-feedback-design.md, measured).
fn rest(spent: Duration) -> Duration {
    FRAME.saturating_sub(spent)
}

#[derive(Default)]
pub struct AppState {
    engine: Arc<Mutex<Option<Engine>>>,
    /// The project the engine was last built from, for Reset.
    built: Mutex<Option<Project>>,
    running: Arc<AtomicBool>,
    /// One saved state per session, with the project it was taken on (spec 2026-11-08-run-modes-design.md).
    saved: Mutex<Option<(Project, Saved)>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Built {
    pub snapshot: Snapshot,
    pub warnings: Vec<String>,
}

fn poisoned<T>(_: T) -> String {
    "a lock is poisoned".to_string()
}

fn with_engine<T>(
    state: &AppState,
    f: impl FnOnce(&mut Engine) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = state.engine.lock().map_err(poisoned)?;
    let engine = guard.as_mut().ok_or("build the project first")?;
    f(engine)
}

/// Puts `engine` in place of the current one, stopped, for `project`.
fn install(state: &AppState, project: Project, engine: Engine) -> Result<Snapshot, String> {
    state.running.store(false, Ordering::SeqCst);
    let snapshot = engine.snapshot();
    *state.engine.lock().map_err(poisoned)? = Some(engine);
    *state.built.lock().map_err(poisoned)? = Some(project);
    Ok(snapshot)
}

/// Checks the current project with `program` and `behaviours` as the format does on open, then replaces the engine
/// with a fresh one at time zero. On success the sources become the project's; on failure the running line stays.
#[tauri::command]
pub fn build(
    state: State<'_, AppState>,
    projects: State<'_, ProjectState>,
    program: Vec<(String, String)>,
    behaviours: BTreeMap<String, String>,
) -> Result<Built, String> {
    let mut project = projects.project()?;
    project.program = program;
    project.behaviours = behaviours;
    let project = parse_project(&encode_project(&project))
        .map_err(|d| lines(&d))?
        .project;
    let (engine, warnings) = Engine::new(&project)?;
    projects.set_sources(&project)?;
    let snapshot = install(&state, project, engine)?;
    Ok(Built { snapshot, warnings })
}

/// Back to time zero with the last built project.
#[tauri::command]
pub fn reset(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let project = state
        .built
        .lock()
        .map_err(poisoned)?
        .clone()
        .ok_or("build the project first")?;
    let (engine, _) = Engine::new(&project)?;
    install(&state, project, engine)
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

/// A panel button, by its input address.
#[tauri::command]
pub fn press(state: State<'_, AppState>, address: String) -> Result<(), String> {
    with_engine(&state, |e| {
        e.press(&address);
        Ok(())
    })
}

#[tauri::command]
pub fn set_sink_blocked(
    state: State<'_, AppState>,
    sink: usize,
    blocked: bool,
) -> Result<Snapshot, String> {
    with_engine(&state, |e| {
        e.set_sink_blocked(sink, blocked);
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
    let project = state
        .built
        .lock()
        .map_err(poisoned)?
        .clone()
        .ok_or("build the project first")?;
    *state.saved.lock().map_err(poisoned)? = Some((project, saved));
    Ok(())
}

/// Replays the saved state on a fresh engine built from the project it was taken on.
#[tauri::command]
pub fn restore_state(state: State<'_, AppState>) -> Result<Snapshot, String> {
    let (project, saved) = state
        .saved
        .lock()
        .map_err(poisoned)?
        .clone()
        .ok_or("save a state first")?;
    let engine = Engine::restore(&project, &saved)?;
    install(&state, project, engine)
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
            let tick_started = Instant::now();
            let wall_ms = i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX);
            let tick = engine.lock().map_err(poisoned).and_then(|mut guard| {
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
            thread::sleep(rest(tick_started.elapsed()));
        }
        let _ = app.emit("stopped", ());
    });
    Ok(())
}

#[cfg(test)]
#[path = "commands_tests.rs"]
mod tests;
