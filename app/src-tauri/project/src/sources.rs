//! A project's ST sources as its folder holds them, when they changed outside Twinova (spec
//! 2026-11-12-st-vscode-design.md, "Watching the folder"). The app's watch and the agent's `reload_sources` read them
//! the same way.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::model::Project;

/// What a change on disk asks for: an online change of the program, or a rebuild when a behaviour changed (decisions 1
/// and 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Program,
    Behaviours,
}

/// The project's sources as its folder holds them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiskSources {
    pub kind: Kind,
    pub program: Vec<(String, String)>,
    pub behaviours: BTreeMap<String, String>,
}

/// The sources `project` lists, as `folder` holds them, when they differ from the project's; `None` when they are
/// equal, or when one cannot be read as text, as while an editor replaces it: the change that ends the write reads them
/// again.
pub fn changed(project: &Project, folder: &Path) -> Option<DiskSources> {
    let read = |path: &str| fs::read_to_string(folder.join(path)).ok();
    let program = (project.program.iter())
        .map(|(path, _)| Some((path.clone(), read(path)?)))
        .collect::<Option<Vec<_>>>()?;
    let behaviours = (project.behaviours.keys())
        .map(|key| {
            Some((
                key.clone(),
                read(&project.definitions[key].behaviour_path())?,
            ))
        })
        .collect::<Option<BTreeMap<_, _>>>()?;
    let kind = if behaviours != project.behaviours {
        Kind::Behaviours
    } else if program != project.program {
        Kind::Program
    } else {
        return None;
    };
    Some(DiskSources {
        kind,
        program,
        behaviours,
    })
}

#[cfg(test)]
#[path = "sources_tests.rs"]
mod tests;
