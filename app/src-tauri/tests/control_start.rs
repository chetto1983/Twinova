//! `control::start` in the app's own data folder (spec 2026-11-23-app-mcp-design.md, decision 1): the channel opens,
//! its file names the port and the token, and an agent's client is let in with them. On Linux the data folder follows
//! `XDG_DATA_HOME`, so this binary points it at a folder of its own; it has this one test, so nothing else reads the
//! environment while it is set.
#![cfg(target_os = "linux")]

use std::time::Duration;

use tauri::{Listener, Manager};
use twinova_control::{Client, ControlFile};
use twinova_lib::control::{self, Control};

#[tokio::test]
async fn the_channel_opens_in_the_app_s_data_folder_and_lets_the_token_in() {
    let data = tempfile::tempdir().unwrap();
    // SAFETY: the only test of this binary, before the app starts any thread that reads the environment.
    unsafe {
        std::env::set_var("XDG_DATA_HOME", data.path());
        std::env::remove_var("TWINOVA_CONTROL");
    }
    let app = tauri::test::mock_app();
    app.manage(Control::default());
    let folder = app.path().app_data_dir().unwrap();
    assert!(folder.starts_with(data.path()), "{}", folder.display());
    let (opened, port) = tokio::sync::oneshot::channel::<String>();
    let opened = std::sync::Mutex::new(Some(opened));
    app.listen("control-port", move |event| {
        if let Some(sender) = opened.lock().unwrap().take() {
            let _ = sender.send(event.payload().to_string());
        }
    });
    control::start(app.handle()).unwrap();
    let port: u16 = (tokio::time::timeout(Duration::from_secs(10), port).await)
        .expect("the channel opened")
        .unwrap()
        .parse()
        .unwrap();
    let file = ControlFile::read(&folder).unwrap();
    assert_eq!((file.port, file.pid), (port, std::process::id()));
    Client::connect(port, &file.token).await.unwrap();
}
