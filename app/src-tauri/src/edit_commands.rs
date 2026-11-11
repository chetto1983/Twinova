//! The editor's commands (spec 2026-11-11-editor-basics-design.md): each applies one edit to the open project through
//! its history and returns the view, or the edit's error with the project unchanged. The running line changes only at
//! Build.

use serde::Deserialize;
use serde_json::Value;
use tauri::State;
use twinova_project::Project;
use twinova_project::edit;
use twinova_project::model::{Controller, Quat, Scene, Vec3};

use crate::project_commands::{Current, ProjectState, ProjectView, replace, view};

/// One edit, as the webview sends it.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Edit {
    /// A library family placed at a pose.
    Place {
        family: String,
        position: Vec3,
        rotation: Quat,
    },
    Move {
        id: String,
        position: Vec3,
        rotation: Quat,
    },
    Remove {
        id: String,
    },
    Rename {
        id: String,
        name: String,
    },
    /// `null` takes the definition's default back.
    SetParameter {
        id: String,
        name: String,
        value: Option<Value>,
    },
    /// `null` unbinds it.
    BindSignal {
        id: String,
        signal: String,
        address: Option<String>,
    },
    /// `host` null unmounts the device.
    SetMount {
        device: String,
        host: Option<String>,
        at: f64,
    },
    /// `peer` null leaves the port unconnected.
    SetFlow {
        id: String,
        port: String,
        peer: Option<(String, String)>,
    },
}

fn apply(project: &mut Project, op: Edit) -> Result<(), String> {
    match op {
        Edit::Place {
            family,
            position,
            rotation,
        } => edit::place(project, &family, position, rotation).map(|_| ()),
        Edit::Move {
            id,
            position,
            rotation,
        } => edit::move_instance(project, &id, position, Some(rotation)),
        Edit::Remove { id } => edit::remove_instance(project, &id),
        Edit::Rename { id, name } => edit::set_name(project, &id, &name),
        Edit::SetParameter { id, name, value } => {
            edit::set_parameter(project, &id, &name, value.as_ref())
        }
        Edit::BindSignal {
            id,
            signal,
            address,
        } => {
            let address = address.as_deref().map(str::trim).filter(|a| !a.is_empty());
            edit::bind_signal(project, &id, &signal, address)
        }
        Edit::SetMount { device, host, at } => {
            edit::set_mount(project, &device, host.as_deref().map(|h| (h, at)))
        }
        Edit::SetFlow { id, port, peer } => edit::set_flow(
            project,
            &id,
            &port,
            peer.as_ref().map(|(i, p)| (i.as_str(), p.as_str())),
        ),
    }
}

#[tauri::command]
pub fn edit(state: State<'_, ProjectState>, op: Edit) -> Result<ProjectView, String> {
    let mut guard = state.lock()?;
    let current = &mut *guard;
    current
        .history
        .apply(&mut current.project, |p| apply(p, op))?;
    Ok(view(current, Vec::new(), false))
}

#[tauri::command]
pub fn undo(state: State<'_, ProjectState>) -> Result<ProjectView, String> {
    let mut guard = state.lock()?;
    let current = &mut *guard;
    current.history.undo(&mut current.project);
    Ok(view(current, Vec::new(), false))
}

#[tauri::command]
pub fn redo(state: State<'_, ProjectState>) -> Result<ProjectView, String> {
    let mut guard = state.lock()?;
    let current = &mut *guard;
    current.history.redo(&mut current.project);
    Ok(view(current, Vec::new(), false))
}

/// The program a new project starts with.
const EMPTY_PROGRAM: &str = "PROGRAM Main\nEND_PROGRAM\n";

/// A project with nothing in its scene and an empty program, not yet saved to a folder.
pub fn empty_project() -> Project {
    Project {
        name: "Untitled".into(),
        app: env!("CARGO_PKG_VERSION").into(),
        scene: Scene {
            instances: Vec::new(),
            groups: Vec::new(),
            connections: Vec::new(),
            controller: Controller { panel: Vec::new() },
        },
        definitions: Default::default(),
        program: vec![("plc/Main.st".into(), EMPTY_PROGRAM.into())],
        behaviours: Default::default(),
        assets: Default::default(),
    }
}

#[tauri::command]
pub fn new_project(state: State<'_, ProjectState>) -> Result<ProjectView, String> {
    replace(
        &state,
        Current::new(empty_project(), None),
        Vec::new(),
        false,
    )
}

#[cfg(test)]
#[path = "edit_commands_tests.rs"]
mod tests;
