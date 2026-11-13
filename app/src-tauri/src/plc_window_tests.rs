use super::*;
use std::sync::{Arc, Mutex};
use tauri::test::{MockRuntime, mock_app};
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};

fn opened(app: &tauri::App<tauri::test::MockRuntime>) -> usize {
    app.webview_windows()
        .keys()
        .filter(|label| label.as_str() == LABEL)
        .count()
}

#[test]
fn opening_twice_keeps_one_window_and_closing_asks_it_to_close() {
    let app = mock_app();
    tauri::async_runtime::block_on(open_plc_window(app.handle().clone())).unwrap();
    assert_eq!(opened(&app), 1);
    tauri::async_runtime::block_on(open_plc_window(app.handle().clone())).unwrap();
    assert_eq!(
        opened(&app),
        1,
        "an open window is focused, not built again"
    );
    assert_eq!(
        tauri::async_runtime::block_on(close_plc_window(app.handle().clone())),
        Ok(true)
    );
}

#[test]
fn closing_a_window_that_is_not_open_does_nothing() {
    let app = mock_app();
    assert_eq!(
        tauri::async_runtime::block_on(close_plc_window(app.handle().clone())),
        Ok(false)
    );
}

#[test]
fn the_plc_window_gone_is_told_to_main_and_main_gone_closes_it() {
    let destroyed = WindowEvent::Destroyed;
    assert_eq!(follow(LABEL, &destroyed), Follow::TellMain);
    assert_eq!(follow(MAIN, &destroyed), Follow::ClosePlc);
    assert_eq!(follow("other", &destroyed), Follow::Nothing);
    let focused = WindowEvent::Focused(true);
    assert_eq!(follow(LABEL, &focused), Follow::Nothing);
    assert_eq!(follow(MAIN, &focused), Follow::Nothing);
}

fn window(app: &tauri::App<MockRuntime>, label: &str) -> tauri::WebviewWindow<MockRuntime> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::default())
        .build()
        .unwrap()
}

#[test]
fn the_plc_window_destroyed_tells_main() {
    let app = mock_app();
    window(&app, MAIN);
    let plc = window(&app, LABEL);
    let told = Arc::new(Mutex::new(0));
    let count = told.clone();
    app.listen_any(CLOSED, move |_| *count.lock().unwrap() += 1);
    on_window_event(&plc.as_ref().window(), &WindowEvent::Destroyed);
    assert_eq!(*told.lock().unwrap(), 1);
}
