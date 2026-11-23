//! The app's end of the control channel (spec 2026-11-23-app-mcp-design.md, decisions 1 and 2; crate
//! `twinova-control`): at start the app listens on the first free port of `PORTS` and writes its file; a request goes
//! to the main window as a `control-request` event, where the command registry runs it as the window's own button
//! would, and comes back through `control_reply`. `TWINOVA_CONTROL=off`, or a `control/off` file in the app's data
//! folder, keeps the channel closed.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tokio::sync::{Notify, oneshot};
use twinova_control::{ControlFile, Handler, PORTS, bind, serve, token};

/// How long a command may take in the window, a Build included, before its client is told (SolidCraft's 60 s).
const ANSWER_WITHIN: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct Control {
    waiting: Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>,
    next: AtomicU64,
    file: Mutex<Option<(PathBuf, ControlFile)>>,
    /// The window's registry listens: a request sent before then would be lost, so it waits (`control_ready`).
    ready: AtomicBool,
    readied: Notify,
}

/// A request as the window receives it.
#[derive(Debug, Clone, Serialize)]
pub struct ControlRequest {
    pub id: u64,
    pub method: String,
    pub params: Value,
}

impl Control {
    /// The port the channel listens on, while it does.
    pub fn port(&self) -> Option<u16> {
        self.file.lock().ok()?.as_ref().map(|(_, f)| f.port)
    }

    /// Until the window's registry listens.
    async fn window_ready(&self) {
        loop {
            let readied = self.readied.notified();
            if self.ready.load(Ordering::SeqCst) {
                return;
            }
            readied.await;
        }
    }

    /// The window's registry listens from now on.
    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
        self.readied.notify_waiters();
    }

    /// `method` run in the main window, and its answer; a window that is not ready in time, or does not answer in
    /// time, is said so.
    async fn ask<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        method: String,
        params: Value,
    ) -> Result<Value, String> {
        if tokio::time::timeout(ANSWER_WITHIN, self.window_ready())
            .await
            .is_err()
        {
            return Err(format!(
                "the window was not ready for {method} within {} s",
                ANSWER_WITHIN.as_secs()
            ));
        }
        let id = self.next.fetch_add(1, Ordering::SeqCst) + 1;
        let (sender, answer) = oneshot::channel();
        self.waiting
            .lock()
            .map_err(|_| POISONED)?
            .insert(id, sender);
        let request = ControlRequest {
            id,
            method: method.clone(),
            params,
        };
        if let Err(e) = app.emit_to("main", "control-request", request) {
            self.waiting.lock().map_err(|_| POISONED)?.remove(&id);
            return Err(format!("the window could not be reached: {e}"));
        }
        match tokio::time::timeout(ANSWER_WITHIN, answer).await {
            Ok(Ok(answer)) => answer,
            Ok(Err(_)) => Err(format!("the window dropped {method}")),
            Err(_) => {
                self.waiting.lock().map_err(|_| POISONED)?.remove(&id);
                Err(format!(
                    "the window did not answer {method} within {} s",
                    ANSWER_WITHIN.as_secs()
                ))
            }
        }
    }

    /// Hands request `id` its answer; false when nothing waits for it any more.
    pub fn answer(&self, id: u64, answer: Result<Value, String>) -> bool {
        let sender = self.waiting.lock().ok().and_then(|mut w| w.remove(&id));
        sender.is_some_and(|s| s.send(answer).is_ok())
    }
}

const POISONED: &str = "the control channel's lock is poisoned";

/// Whether the channel is turned off: `TWINOVA_CONTROL=off`, or `control/off` in the app's data folder.
pub fn turned_off(variable: Option<String>, data_dir: &std::path::Path) -> bool {
    variable.is_some_and(|v| v.eq_ignore_ascii_case("off"))
        || data_dir.join("control").join("off").exists()
}

/// Opens the channel for `app`, unless it is turned off: the listener, the file, and every connection served.
pub fn start<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if turned_off(std::env::var("TWINOVA_CONTROL").ok(), &data_dir) {
        return Ok(());
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let opened = open(&app, data_dir).await;
        if let Err(e) = opened {
            eprintln!("Twinova's control channel is closed: {e}");
        }
    });
    Ok(())
}

async fn open<R: Runtime>(app: &AppHandle<R>, data_dir: PathBuf) -> Result<(), String> {
    let listener = bind(PORTS)
        .await
        .map_err(|e| format!("no free port in {PORTS:?}: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let token = token().map_err(|e| e.to_string())?;
    let file = ControlFile {
        port,
        token: token.clone(),
        pid: std::process::id(),
    };
    file.write(&data_dir).map_err(|e| e.to_string())?;
    *app.state::<Control>().file.lock().map_err(|_| POISONED)? = Some((data_dir, file));
    let _ = app.emit("control-port", port);
    serve(listener, token, handler(app.clone())).await;
    Ok(())
}

fn handler<R: Runtime>(app: AppHandle<R>) -> Handler {
    std::sync::Arc::new(move |method, params| {
        let app = app.clone();
        Box::pin(async move { app.state::<Control>().ask(&app, method, params).await })
    })
}

/// Removes the channel's file when the app quits, if it is still this app's.
pub fn close<R: Runtime>(app: &AppHandle<R>) {
    if let Some((data_dir, file)) = app
        .state::<Control>()
        .file
        .lock()
        .ok()
        .and_then(|mut f| f.take())
    {
        file.remove(&data_dir);
    }
}

/// The window's answer to request `id`: `value` when `ok`, else `value` is the error's text.
#[tauri::command]
pub fn control_reply(state: State<'_, Control>, id: u64, ok: bool, value: Value) -> bool {
    let answer = if ok {
        Ok(value)
    } else {
        Err(value
            .as_str()
            .map_or_else(|| value.to_string(), str::to_string))
    };
    state.answer(id, answer)
}

/// The window's command registry listens for requests (registry.ts `listenControl`).
#[tauri::command]
pub fn control_ready(state: State<'_, Control>) {
    state.set_ready();
}

/// The port the channel listens on, or null when it is closed, for the status bar.
#[tauri::command]
pub fn control_port(state: State<'_, Control>) -> Option<u16> {
    state.port()
}

#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
