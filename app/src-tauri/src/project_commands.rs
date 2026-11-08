//! Tauri commands over the project format (ROADMAP 4.2, spec 2026-11-08-project-format-design.md). The webview
//! receives a read-only view; the validated project stays here. A project that fails to open never replaces the
//! current one, and nothing here runs a project: running a line belongs to 4.3–4.5, and the M3 demo keeps its own
//! commands.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;
use twinova_project::archive::{export_archive, import_archive};
use twinova_project::model::Instance;
use twinova_project::storage::{open_folder, save_folder};
use twinova_project::{Opened, Project};

#[derive(Default)]
pub struct ProjectState {
    current: Mutex<Option<Current>>,
}

struct Current {
    project: Project,
    /// The folder it was opened from or saved to; `None` after an import, until it is saved.
    folder: Option<PathBuf>,
}

/// What the webview shows of a project.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectView {
    pub name: String,
    pub folder: Option<String>,
    pub instances: Vec<Instance>,
    /// Files changed outside Twinova since its last save (git, an editor); they were validated like any file.
    pub changed_outside: Vec<String>,
    /// An interrupted save was rolled back while opening.
    pub recovered: bool,
}

fn view(current: &Current, changed_outside: Vec<String>, recovered: bool) -> ProjectView {
    ProjectView {
        name: current.project.name.clone(),
        folder: current.folder.as_ref().map(|f| f.display().to_string()),
        instances: current.project.scene.instances.clone(),
        changed_outside,
        recovered,
    }
}

fn replace(
    state: &ProjectState,
    current: Current,
    changed: Vec<String>,
    recovered: bool,
) -> Result<ProjectView, String> {
    let shown = view(&current, changed, recovered);
    *state.current.lock().map_err(|_| "project lock poisoned")? = Some(current);
    Ok(shown)
}

fn opened(folder: Option<PathBuf>, opened: Opened) -> (Current, Vec<String>) {
    (
        Current {
            project: opened.project,
            folder,
        },
        opened.changed_outside,
    )
}

fn lines(diagnostics: &[twinova_project::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[tauri::command]
pub fn open_project(state: State<'_, ProjectState>, folder: String) -> Result<ProjectView, String> {
    let folder = PathBuf::from(folder);
    let result = open_folder(&folder).map_err(|e| e.to_string())?;
    let (current, changed) = opened(Some(folder), result.opened);
    replace(&state, current, changed, result.recovered)
}

/// Saves the current project into `folder`, which becomes its folder.
#[tauri::command]
pub fn save_project(state: State<'_, ProjectState>, folder: String) -> Result<ProjectView, String> {
    let mut guard = state.current.lock().map_err(|_| "project lock poisoned")?;
    let current = guard.as_mut().ok_or("open or import a project first")?;
    let folder = PathBuf::from(folder);
    save_folder(&folder, &current.project).map_err(|e| e.to_string())?;
    current.folder = Some(folder);
    Ok(view(current, Vec::new(), false))
}

#[tauri::command]
pub fn export_project(state: State<'_, ProjectState>, file: String) -> Result<(), String> {
    let guard = state.current.lock().map_err(|_| "project lock poisoned")?;
    let current = guard.as_ref().ok_or("open or import a project first")?;
    std::fs::write(Path::new(&file), export_archive(&current.project))
        .map_err(|e| format!("{file}: {e}"))
}

#[tauri::command]
pub fn import_project(state: State<'_, ProjectState>, file: String) -> Result<ProjectView, String> {
    let bytes = std::fs::read(Path::new(&file)).map_err(|e| format!("{file}: {e}"))?;
    let result = import_archive(&bytes).map_err(|d| lines(&d))?;
    let (current, changed) = opened(None, result);
    replace(&state, current, changed, false)
}

#[cfg(test)]
#[path = "project_commands_tests.rs"]
mod tests;
