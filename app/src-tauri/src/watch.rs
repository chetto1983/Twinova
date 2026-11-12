//! Watching the open project's folder (spec 2026-11-12-st-vscode-design.md, "Watching the folder"). A `notify` watcher
//! reports any change in the folder but Twinova's own state; once the folder has been quiet for `QUIET`, the sources the
//! project lists are read again, and those that differ from the project's are sent to the webview as `disk-sources`,
//! which applies them (plan 2026-11-12-st-vscode.md, decision 1). Twinova's own Save writes what the project holds, so
//! it changes nothing.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use twinova_project::Project;

use crate::project_commands::ProjectState;

/// How long the folder stays quiet before it is read: truST's own debounce for the files it watches
/// (`trust-runtime/src/control.rs`, `HMI_DESCRIPTOR_WATCH_DEBOUNCE`).
pub const QUIET: Duration = Duration::from_millis(250);

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

/// Whether `event` may have changed a source: anything but a read, in `folder` but outside Twinova's state, `.twinova/`,
/// where Save keeps its copies and the language server its cache. Linux reports reads too, Twinova's own reading of
/// the sources included, which would wake the watch for ever (measured 2026-11-12).
pub fn relevant(folder: &Path, event: &notify::Event) -> bool {
    let inside = |path: &PathBuf| {
        path.strip_prefix(folder)
            .is_ok_and(|inside| !inside.starts_with(".twinova"))
    };
    !matches!(event.kind, EventKind::Access(_)) && event.paths.iter().any(inside)
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

/// Sends the current project's sources from disk to the webview when they changed; nothing when `folder` is no longer
/// the project's.
pub fn settled<R: Runtime>(app: &AppHandle<R>, folder: &Path) {
    let projects = app.state::<ProjectState>();
    let (Ok(Some(current)), Ok(project)) = (projects.folder(), projects.project()) else {
        return;
    };
    if current != folder {
        return;
    }
    if let Some(disk) = changed(&project, folder) {
        // As the Run loop's events: a webview that is gone has nothing to apply.
        let _ = app.emit("disk-sources", disk);
    }
}

/// The watcher of one folder at a time, and the thread that waits for it to be quiet.
pub struct SourceWatch {
    watching: Mutex<Option<(PathBuf, RecommendedWatcher)>>,
    events: Sender<PathBuf>,
}

impl SourceWatch {
    /// A watch that calls `quiet` with its folder once the folder's changes stop for `QUIET`.
    pub fn new(quiet: impl Fn(&Path) + Send + 'static) -> Self {
        let (events, received) = mpsc::channel();
        thread::spawn(move || debounce(&received, &quiet));
        Self {
            watching: Mutex::new(None),
            events,
        }
    }

    /// Watches `folder` instead of the previous one, or nothing.
    pub fn follow(&self, folder: Option<&Path>) -> Result<(), String> {
        let mut watching = (self.watching.lock()).map_err(|_| "watch lock poisoned".to_string())?;
        if watching.as_ref().map(|(f, _)| f.as_path()) == folder {
            return Ok(());
        }
        *watching = None;
        let Some(folder) = folder else {
            return Ok(());
        };
        let (root, events) = (folder.to_path_buf(), self.events.clone());
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                // An error of the watcher may hide a change: the sources are read again and compared all the same.
                if event.is_err() || event.is_ok_and(|e| relevant(&root, &e)) {
                    // The debounce thread ends only with the watch, so a send cannot fail while this watcher lives.
                    let _ = events.send(root.clone());
                }
            })
            .map_err(|e| format!("{}: {e}", folder.display()))?;
        (watcher.watch(folder, RecursiveMode::Recursive))
            .map_err(|e| format!("{}: {e}", folder.display()))?;
        *watching = Some((folder.to_path_buf(), watcher));
        Ok(())
    }
}

/// Calls `quiet` once per burst of events, `QUIET` after its last one.
fn debounce(received: &Receiver<PathBuf>, quiet: &dyn Fn(&Path)) {
    while let Ok(mut folder) = received.recv() {
        loop {
            match received.recv_timeout(QUIET) {
                Ok(next) => folder = next,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        quiet(&folder);
    }
}

#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
