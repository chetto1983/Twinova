pub mod commands;
pub mod demo;
pub mod edit_commands;
pub mod lsp_files;
pub mod project_commands;
#[cfg(test)]
mod test_fixtures;
pub mod watch;
pub mod webview_locale;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let watch = watch::SourceWatch::new(move |folder| watch::settled(&handle, folder));
            app.state::<project_commands::ProjectState>()
                .set_watch(watch);
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
            commands::change_program,
            commands::reset,
            commands::step,
            commands::press,
            commands::set_sink_blocked,
            commands::pause,
            commands::run,
            commands::save_state,
            commands::restore_state,
            project_commands::current_project,
            project_commands::open_demo,
            project_commands::open_project,
            project_commands::open_launch_project,
            project_commands::save_project,
            project_commands::export_project,
            project_commands::import_project,
            edit_commands::edit,
            edit_commands::undo,
            edit_commands::redo,
            edit_commands::new_project,
            edit_commands::catalogue,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Twinova");
}
