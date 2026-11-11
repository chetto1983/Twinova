//! Tauri commands over the project format (ROADMAP 4.2, spec 2026-11-08-project-format-design.md). The webview
//! receives a view; the validated project stays here. The app starts on the bundled M3 demo (spec
//! 2026-11-09-component-behaviour-design.md), and a project that fails to open never replaces the current one. Build
//! (commands.rs) puts edited ST sources back into the current project, so Save writes them.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use std::collections::BTreeMap;
use tauri::State;
use twinova_engine::diagnostics;
use twinova_project::archive::{export_archive, import_archive};
use twinova_project::model::{Connection, Definition, Instance, PanelButton};
use twinova_project::storage::{open_folder, save_folder};
use twinova_project::{Opened, Project};

pub struct ProjectState {
    current: Mutex<Current>,
    /// The folder named on the command line, until the webview asks for it.
    launch: Mutex<Option<PathBuf>>,
}

impl Default for ProjectState {
    fn default() -> Self {
        Self::launching(None)
    }
}

impl ProjectState {
    pub fn launching(folder: Option<PathBuf>) -> Self {
        Self {
            current: Mutex::new(Current {
                project: crate::demo::demo(),
                folder: None,
            }),
            launch: Mutex::new(folder),
        }
    }

    pub(crate) fn project(&self) -> Result<Project, String> {
        Ok(self.lock()?.project.clone())
    }

    /// The ST sources a Build accepted: they become the project's, for Save.
    pub(crate) fn set_sources(&self, from: &Project) -> Result<(), String> {
        let mut current = self.lock()?;
        current.project.program = from.program.clone();
        current.project.behaviours = from.behaviours.clone();
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Current>, String> {
        self.current
            .lock()
            .map_err(|_| "project lock poisoned".to_string())
    }
}

/// The project folder in a process's arguments (spec, decision 8): the value of `--project=<folder>`, or else the
/// first argument after the program that is not a switch. msedgedriver passes every WebDriver argument as a switch
/// (`[--]name[=value]`, chromedriver's `capabilities.h`), so the E2E names the folder with `--project=` (run
/// 42572872545). A `--project=` value that is not valid UTF-8 is not recognised.
pub fn launch_folder(args: impl IntoIterator<Item = OsString>) -> Option<PathBuf> {
    let args: Vec<OsString> = args.into_iter().skip(1).collect();
    let switch = args
        .iter()
        .find_map(|arg| arg.to_str()?.strip_prefix("--project=").map(PathBuf::from));
    switch.or_else(|| {
        args.iter()
            .find(|arg| !arg.to_string_lossy().starts_with('-'))
            .map(PathBuf::from)
    })
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
    /// The pinned definitions by `id@version`, for units, defaults, ranges and signal directions.
    pub definitions: BTreeMap<String, Definition>,
    /// Files changed outside Twinova since its last save (git, an editor); they were validated like any file.
    pub changed_outside: Vec<String>,
    /// An interrupted save was rolled back while opening.
    pub recovered: bool,
    /// The line program's sources, (path, text), for the editor.
    pub program: Vec<(String, String)>,
    /// The behaviour block of each definition that has one, by `id@version`, for the editor.
    pub behaviours: BTreeMap<String, String>,
    pub panel: Vec<PanelButton>,
    /// Each mounted device's host and position along it, so the view places it as the physics does.
    pub mounts: Vec<MountView>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MountView {
    pub device: String,
    pub host: String,
    pub at: f64,
}

fn mounts(project: &Project) -> Vec<MountView> {
    project
        .scene
        .connections
        .iter()
        .filter_map(|c| match c {
            Connection::Mount {
                host,
                device,
                position,
                ..
            } => Some(MountView {
                device: device.0.clone(),
                host: host.0.clone(),
                at: position[0],
            }),
            _ => None,
        })
        .collect()
}

fn view(current: &Current, changed_outside: Vec<String>, recovered: bool) -> ProjectView {
    ProjectView {
        name: current.project.name.clone(),
        folder: current.folder.as_ref().map(|f| f.display().to_string()),
        instances: current.project.scene.instances.clone(),
        definitions: current.project.definitions.clone(),
        changed_outside,
        recovered,
        program: current.project.program.clone(),
        behaviours: current.project.behaviours.clone(),
        panel: current.project.scene.controller.panel.clone(),
        mounts: mounts(&current.project),
    }
}

fn replace(
    state: &ProjectState,
    current: Current,
    changed: Vec<String>,
    recovered: bool,
) -> Result<ProjectView, String> {
    let shown = view(&current, changed, recovered);
    *state.lock()? = current;
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

fn open(state: &ProjectState, folder: PathBuf) -> Result<ProjectView, String> {
    let result = open_folder(&folder).map_err(|e| e.to_string())?;
    let (current, changed) = opened(Some(folder), result.opened);
    replace(state, current, changed, result.recovered)
}

/// The project the app shows: the demo at start, else the last one opened or imported.
#[tauri::command]
pub fn current_project(state: State<'_, ProjectState>) -> Result<ProjectView, String> {
    Ok(view(&*state.lock()?, Vec::new(), false))
}

/// Back to the bundled demo, as it was shipped.
#[tauri::command]
pub fn open_demo(state: State<'_, ProjectState>) -> Result<ProjectView, String> {
    let current = Current {
        project: crate::demo::demo(),
        folder: None,
    };
    replace(&state, current, Vec::new(), false)
}

#[tauri::command]
pub fn open_project(state: State<'_, ProjectState>, folder: String) -> Result<ProjectView, String> {
    open(&state, PathBuf::from(folder))
}

/// Opens the folder named on the command line, once: a reload of the webview finds nothing to open.
#[tauri::command]
pub fn open_launch_project(state: State<'_, ProjectState>) -> Result<Option<ProjectView>, String> {
    let folder = state
        .launch
        .lock()
        .map_err(|_| "project lock poisoned")?
        .take();
    folder.map(|folder| open(&state, folder)).transpose()
}

/// Saves the current project into `folder`, which becomes its folder.
#[tauri::command]
pub fn save_project(state: State<'_, ProjectState>, folder: String) -> Result<ProjectView, String> {
    let mut current = state.lock()?;
    let folder = PathBuf::from(folder);
    save_folder(&folder, &current.project).map_err(|e| e.to_string())?;
    current.folder = Some(folder);
    Ok(view(&current, Vec::new(), false))
}

#[tauri::command]
pub fn export_project(state: State<'_, ProjectState>, file: String) -> Result<(), String> {
    let current = state.lock()?;
    std::fs::write(Path::new(&file), export_archive(&current.project))
        .map_err(|e| format!("{file}: {e}"))
}

#[tauri::command]
pub fn import_project(state: State<'_, ProjectState>, file: String) -> Result<ProjectView, String> {
    let bytes = std::fs::read(Path::new(&file)).map_err(|e| format!("{file}: {e}"))?;
    let result = import_archive(&bytes).map_err(|d| diagnostics(&d))?;
    let (current, changed) = opened(None, result);
    replace(&state, current, changed, false)
}

#[cfg(test)]
#[path = "project_commands_tests.rs"]
mod tests;
