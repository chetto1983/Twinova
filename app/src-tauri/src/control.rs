//! The app's end of the control channel (spec 2026-11-23-app-mcp-design.md, decisions 1 and 2; crate
//! `twinova-control`): at start the app listens on the first free port of `PORTS` and writes its file. A request goes
//! as a `control-request` event to the window whose command registry holds its command (spec
//! 2026-11-24-open-points-design.md, decision 3), where it runs as the window's own button would, and comes back
//! through `control_reply`. `TWINOVA_CONTROL=off`, or a `control/off` file in the app's data folder, keeps the channel
//! closed.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tokio::sync::{Notify, oneshot};
use twinova_control::{ControlFile, Handler, PORTS, bind, serve, token};

/// How long a command may take in the window, a Build included, before its client is told (SolidCraft's 60 s).
const ANSWER_WITHIN: Duration = Duration::from_secs(60);
/// How long a PLC window just opened may take to tell its commands, its pane mounted, before a request for one of them
/// goes to the main window, which refuses it.
const PLC_LOADING: Duration = Duration::from_secs(10);

#[derive(Default)]
pub struct Control {
    waiting: Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>,
    next: AtomicU64,
    file: Mutex<Option<(PathBuf, ControlFile)>>,
    /// The window's registry listens: a request sent before then would be lost, so it waits (`control_ready`).
    ready: AtomicBool,
    readied: Notify,
    /// Each window's commands, as its registry tells them (`control_commands`), by the window's label.
    commands: Mutex<BTreeMap<String, Vec<Listed>>>,
}

/// A command as `list_commands` names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listed {
    pub name: String,
    pub description: String,
}

/// The main window, which answers a request no window registered, and the PLC window, which wins a tie: it holds the
/// pane's commands while it is open.
const MAIN: &str = "main";
const PLC: &str = crate::plc_window::LABEL;

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

    /// Window `label`'s commands from now on; none forgets the window.
    pub fn set_commands(&self, label: &str, commands: Vec<Listed>) {
        if let Ok(mut known) = self.commands.lock() {
            match commands.is_empty() {
                true => known.remove(label),
                false => known.insert(label.to_string(), commands),
            };
        }
    }

    /// Every window's commands, by name.
    fn listed(&self) -> Vec<Listed> {
        let known = self.commands.lock().map(|k| k.clone()).unwrap_or_default();
        let mut merged: BTreeMap<String, Listed> = BTreeMap::new();
        for list in known.into_values() {
            for command in list {
                merged.entry(command.name.clone()).or_insert(command);
            }
        }
        merged.into_values().collect()
    }

    fn holds(&self, label: &str, name: Option<&str>) -> bool {
        let known = self.commands.lock();
        let list = known.as_ref().ok().and_then(|k| k.get(label));
        list.is_some_and(|list| name.is_none_or(|name| list.iter().any(|c| c.name == name)))
    }

    /// The window for command `name`, once a PLC window that is open has told its commands: while it loads, the pane's
    /// commands are nowhere, and the main window, whose pane it took, would refuse them.
    async fn window_for<R: Runtime>(&self, app: &AppHandle<R>, name: &str) -> String {
        let deadline = tokio::time::Instant::now() + PLC_LOADING;
        loop {
            let window = self.window_of(name);
            let loading = window == MAIN
                && !self.holds(MAIN, Some(name))
                && app.get_webview_window(PLC).is_some()
                && !self.holds(PLC, None);
            if !loading || tokio::time::Instant::now() >= deadline {
                return window;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// The window that runs command `name`: the one that registered it, the PLC window on a tie, else the main window.
    fn window_of(&self, name: &str) -> String {
        let known = self.commands.lock().map(|k| k.clone()).unwrap_or_default();
        let mut holders = (known.iter())
            .filter(|(_, list)| list.iter().any(|c| c.name == name))
            .map(|(label, _)| label.as_str());
        let first = holders.next();
        let plc = first == Some(PLC) || holders.any(|label| label == PLC);
        match (plc, first) {
            (true, _) => PLC.to_string(),
            (false, Some(label)) => label.to_string(),
            (false, None) => MAIN.to_string(),
        }
    }

    /// `method` run in the window that holds it, and its answer: `list_commands` here, from what the windows told,
    /// `describe_command` in the window of the command it names. A window that is not ready in time, or does not
    /// answer in time, is said so.
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
        if method == "list_commands" {
            return serde_json::to_value(self.listed()).map_err(|e| e.to_string());
        }
        let name = match method.as_str() {
            "describe_command" => params["name"].as_str().unwrap_or_default(),
            _ => method.as_str(),
        };
        let window = self.window_for(app, name).await;
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
        if let Err(e) = app.emit_to(window.as_str(), "control-request", request) {
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

/// Runs a request: `screenshot` here, in Rust, since the window cannot picture itself; every other in the window.
fn handler<R: Runtime>(app: AppHandle<R>) -> Handler {
    std::sync::Arc::new(move |method, params| {
        let app = app.clone();
        Box::pin(async move {
            if method == "screenshot" {
                let label = params["window"].as_str().unwrap_or("main").to_string();
                let picture = crate::screenshot::take(&app, &label).await?;
                return serde_json::to_value(picture).map_err(|e| e.to_string());
            }
            app.state::<Control>().ask(&app, method, params).await
        })
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

/// The commands the calling window's registry holds now (registry.ts), so its requests reach it.
#[tauri::command]
pub fn control_commands<R: Runtime>(
    window: tauri::Window<R>,
    state: State<'_, Control>,
    commands: Vec<Listed>,
) {
    state.set_commands(window.label(), commands);
}

/// The port the channel listens on, or null when it is closed, for the status bar.
#[tauri::command]
pub fn control_port(state: State<'_, Control>) -> Option<u16> {
    state.port()
}

#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
