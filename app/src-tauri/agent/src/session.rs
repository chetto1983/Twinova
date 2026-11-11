//! A client's session (spec 2026-11-11-agent-server-design.md, decisions 2 and 4): the project it opened, the working
//! copy its edits change, the project last built, and that build's engine. An HTTP session owns one; stdio has one.

use std::path::{Path, PathBuf};

use twinova_engine::engine::{Engine, Saved};
use twinova_project::Project;
use twinova_project::storage::open_folder;

#[derive(Default)]
pub struct Session {
    /// The folder the project was opened from: `save_project`'s default.
    pub folder: Option<PathBuf>,
    /// The project the editing tools change; `build` makes it the running one.
    pub working: Option<Project>,
    /// The project the engine was built from, for `reset` and `restore_state`.
    pub built: Option<Project>,
    pub engine: Option<Engine>,
    /// The states `save_state` kept, by id.
    pub saved: Vec<Saved>,
}

/// What `open_project` and `build` answer: the project's name, its instances and Build's warnings.
#[derive(Debug, serde::Serialize)]
pub struct Opened {
    pub name: String,
    pub instances: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

impl Session {
    /// Opens the project in `folder` and builds it.
    pub fn open(&mut self, folder: &Path) -> Result<Opened, String> {
        let opened = open_folder(folder).map_err(|e| e.to_string())?;
        self.folder = Some(folder.to_path_buf());
        self.working = Some(opened.opened.project);
        self.build()
    }

    /// Builds the working project at time zero, as the app's Build does: on failure the running line stays.
    pub fn build(&mut self) -> Result<Opened, String> {
        let working = self.working.as_ref().ok_or(NO_PROJECT)?;
        let (project, engine, warnings) = twinova_engine::build(working)?;
        let opened = Opened {
            name: project.name.clone(),
            instances: (project.scene.instances.iter())
                .map(|i| (i.id.clone(), i.definition.clone()))
                .collect(),
            warnings,
        };
        self.built = Some(project);
        self.engine = Some(engine);
        self.saved.clear();
        Ok(opened)
    }

    /// The working project the editing tools change.
    pub fn working(&mut self) -> Result<&mut Project, String> {
        self.working.as_mut().ok_or_else(|| NO_PROJECT.to_string())
    }

    pub fn engine(&mut self) -> Result<&mut Engine, String> {
        self.engine.as_mut().ok_or_else(|| NO_PROJECT.to_string())
    }

    /// The last built project, at time zero again.
    pub fn reset(&mut self) -> Result<(), String> {
        let built = self.built.as_ref().ok_or(NO_PROJECT)?;
        self.engine = Some(Engine::new(built)?.0);
        Ok(())
    }

    /// Keeps the running line's state; its id for `restore`.
    pub fn save_state(&mut self) -> Result<usize, String> {
        let saved = self.engine()?.save();
        self.saved.push(saved);
        Ok(self.saved.len() - 1)
    }

    /// The line back at the state `save_state` kept as `id`, replayed from time zero.
    pub fn restore_state(&mut self, id: usize) -> Result<(), String> {
        let built = self.built.as_ref().ok_or(NO_PROJECT)?;
        let saved = self
            .saved
            .get(id)
            .ok_or_else(|| format!("no saved state {id}"))?;
        self.engine = Some(Engine::restore(built, saved)?);
        Ok(())
    }
}

const NO_PROJECT: &str = "open a project first";
