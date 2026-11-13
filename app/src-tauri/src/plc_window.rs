//! The PLC pane in a window of its own (spec 2026-11-13-plc-window-design.md). The window is opened from Rust, so
//! the webview needs no window-creation permission; the main window keeps the program's text and the window mirrors
//! it through events (src/lib/plcWindow.ts).

use tauri::{
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent,
};

/// The PLC window's label; the webview mounts the PLC pane when it reads this label (main.ts).
pub const LABEL: &str = "plc";
const MAIN: &str = "main";
/// Sent to the main window when the PLC window is gone, so the dock mounts its editor again.
const CLOSED: &str = "plc-closed";

/// Opens the PLC window, or brings it to the front when it is open. Async: on Windows, building a window in a
/// synchronous command deadlocks (WebviewWindowBuilder::new, "Known issues").
#[tauri::command]
pub async fn open_plc_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(LABEL) {
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("index.html".into()))
        .title("Twinova · PLC")
        .inner_size(1000.0, 700.0)
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Closes the PLC window as its title bar does, which frees its label (measured: WebDriver's `closeWindow` does not).
/// Says whether a window was open.
#[tauri::command]
pub async fn close_plc_window<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    match app.get_webview_window(LABEL) {
        Some(window) => window.close().map(|_| true).map_err(|e| e.to_string()),
        None => Ok(false),
    }
}

/// What one window's event asks of the other.
#[derive(Debug, PartialEq)]
pub(crate) enum Follow {
    TellMain,
    ClosePlc,
    Nothing,
}

pub(crate) fn follow(label: &str, event: &WindowEvent) -> Follow {
    match (label, event) {
        (LABEL, WindowEvent::Destroyed) => Follow::TellMain,
        // Otherwise the process stays alive with the PLC window alone (measured, spec "What exists").
        (MAIN, WindowEvent::Destroyed) => Follow::ClosePlc,
        _ => Follow::Nothing,
    }
}

/// The builder's window-event hook.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let app = window.app_handle();
    match follow(window.label(), event) {
        Follow::TellMain => {
            let _ = app.emit_to(MAIN, CLOSED, ());
        }
        Follow::ClosePlc => {
            if let Some(plc) = app.get_webview_window(LABEL) {
                let _ = plc.close();
            }
        }
        Follow::Nothing => {}
    }
}

#[cfg(test)]
#[path = "plc_window_tests.rs"]
mod tests;
