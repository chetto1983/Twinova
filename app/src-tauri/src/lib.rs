pub mod analysis;
pub mod call_templates;
pub mod commands;
pub mod debug_commands;
pub mod declarations;
pub mod demo;
pub mod edit_commands;
mod frontend;
pub mod ladder_commands;
pub mod lsp_files;
pub mod monitor;
pub mod navigation;
pub mod plc_window;
mod positions;
pub mod project_commands;
#[cfg(test)]
mod test_fixtures;
pub mod watch;
pub mod webview_locale;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(plc_window::on_window_event)
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
        .manage(analysis::AnalysisState::default())
        .manage(project_commands::ProjectState::launching(
            project_commands::launch_folder(std::env::args_os()),
        ))
        .invoke_handler(tauri::generate_handler![
            analysis::analyze,
            analysis::hover,
            analysis::completion,
            ladder_commands::ladder_parse,
            ladder_commands::ladder_print,
            ladder_commands::ladder_lowering,
            ladder_commands::ladder_export,
            ladder_commands::ladder_import,
            navigation::definition,
            navigation::references,
            call_templates::call_template,
            call_templates::declare_proposal,
            commands::build,
            commands::scan_timing,
            commands::change_program,
            commands::reset,
            commands::step,
            commands::press,
            commands::set_sink_blocked,
            commands::pause,
            commands::run,
            debug_commands::set_breakpoints,
            debug_commands::run_to_cursor,
            debug_commands::resume,
            debug_commands::hold,
            debug_commands::debug_logs,
            commands::save_state,
            commands::restore_state,
            declarations::declarations_of,
            declarations::edit_declaration,
            declarations::rename_declaration,
            declarations::rename_symbol,
            monitor::watch_range,
            monitor::unwatch,
            monitor::watched,
            monitor::forces,
            monitor::force,
            monitor::release,
            monitor::write_value,
            plc_window::open_plc_window,
            plc_window::close_plc_window,
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
