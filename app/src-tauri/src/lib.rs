pub mod belt;
pub mod bench;
pub mod commands;
pub mod engine;
pub mod plc;
pub mod scene;
pub mod webview_locale;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "linux")]
            webview_locale::apply(app)?;
            #[cfg(not(target_os = "linux"))]
            let _ = app;
            Ok(())
        })
        .manage(commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::default_program,
            commands::build,
            commands::reset,
            commands::step,
            commands::press,
            commands::set_outlet_blocked,
            commands::pause,
            commands::run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Twinova");
}
