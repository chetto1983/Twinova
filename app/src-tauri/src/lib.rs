pub mod commands;
pub mod controller;
pub mod demo;
pub mod engine;
pub mod field;
pub mod line;
pub mod project_commands;
pub mod webview_locale;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            #[cfg(target_os = "linux")]
            webview_locale::apply(app)?;
            #[cfg(not(target_os = "linux"))]
            let _ = app;
            Ok(())
        })
        .manage(commands::AppState::default())
        .manage(project_commands::ProjectState::launching(
            project_commands::launch_folder(std::env::args_os()),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::build,
            commands::reset,
            commands::step,
            commands::press,
            commands::set_sink_blocked,
            commands::pause,
            commands::run,
            commands::save_state,
            commands::restore_state,
            project_commands::current_project,
            project_commands::open_project,
            project_commands::open_launch_project,
            project_commands::save_project,
            project_commands::export_project,
            project_commands::import_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Twinova");
}
