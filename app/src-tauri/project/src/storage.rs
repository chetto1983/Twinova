//! A project folder on disk, with the save protocol of decision 2 (spec 2026-11-08-project-format-design.md):
//! a marker `.twinova/saving` exists from before the first write until after the manifest, so its presence at open
//! means an interrupted save, and `.twinova/previous/` holds what that save replaced.

use crate::codec::{Files, MANIFEST, Opened, encode_project, parse_project};
use crate::diagnostic::Diagnostic;
use crate::model::Project;
use crate::validate;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const STATE: &str = ".twinova";
const MARKER: &str = ".twinova/saving";
const PREVIOUS: &str = ".twinova/previous";

#[derive(Debug)]
pub enum StorageError {
    Io {
        path: PathBuf,
        error: io::Error,
    },
    Invalid(Vec<Diagnostic>),
    /// A save was interrupted and its recovery has not run: open the folder first.
    Interrupted,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io { path, error } => write!(f, "{}: {error}", path.display()),
            StorageError::Invalid(diagnostics) => {
                let lines: Vec<String> = diagnostics.iter().map(ToString::to_string).collect();
                write!(f, "{}", lines.join("\n"))
            }
            StorageError::Interrupted => {
                write!(f, "a save was interrupted; open the project to recover it")
            }
        }
    }
}

fn io_at(path: &Path) -> impl FnOnce(io::Error) -> StorageError + '_ {
    move |error| StorageError::Io {
        path: path.to_path_buf(),
        error,
    }
}

/// An opened folder: the project, and whether an interrupted save was rolled back first.
#[derive(Debug)]
pub struct OpenedFolder {
    pub opened: Opened,
    pub recovered: bool,
}

/// Every file under `dir` except Twinova's own state, by relative path with `/` separators.
pub fn read_folder(dir: &Path) -> Result<Files, StorageError> {
    fn walk(root: &Path, dir: &Path, files: &mut Files) -> Result<(), StorageError> {
        for entry in fs::read_dir(dir).map_err(io_at(dir))? {
            let path = entry.map_err(io_at(dir))?.path();
            let rel = path
                .strip_prefix(root)
                .expect("under the root")
                .to_string_lossy()
                .replace('\\', "/");
            if rel == STATE {
                continue;
            }
            if path.is_dir() {
                walk(root, &path, files)?;
            } else {
                files.insert(rel, fs::read(&path).map_err(io_at(&path))?);
            }
        }
        Ok(())
    }
    let mut files = Files::new();
    walk(dir, dir, &mut files)?;
    Ok(files)
}

/// Opens a project folder, first rolling back a save that was interrupted.
pub fn open_folder(dir: &Path) -> Result<OpenedFolder, StorageError> {
    let recovered = dir.join(MARKER).exists();
    if recovered {
        restore_previous(dir)?;
    }
    let opened = parse_project(&read_folder(dir)?).map_err(StorageError::Invalid)?;
    Ok(OpenedFolder { opened, recovered })
}

/// Saves `project` into `dir`. A failure rolls the folder back to its previous save before returning the error.
pub fn save_folder(dir: &Path, project: &Project) -> Result<(), StorageError> {
    if dir.join(MARKER).exists() {
        return Err(StorageError::Interrupted);
    }
    let files = encode_project(project);
    let old = fs::read(dir.join(MANIFEST))
        .ok()
        .and_then(|bytes| parse_project_paths(&bytes));
    let previous = dir.join(PREVIOUS);
    if previous.exists() {
        fs::remove_dir_all(&previous).map_err(io_at(&previous))?;
    }
    fs::create_dir_all(&previous).map_err(io_at(&previous))?;
    write_atomic(&dir.join(MARKER), b"")?;
    let result = write_files(dir, &files);
    match result {
        Ok(()) => {
            let stale = old
                .unwrap_or_default()
                .into_iter()
                .filter(|p| validate::path_problem(p).is_none());
            for stale in stale.filter(|p| !files.contains_key(p)) {
                let _ = fs::remove_file(dir.join(stale));
            }
            let marker = dir.join(MARKER);
            fs::remove_file(&marker).map_err(io_at(&marker))
        }
        Err(error) => {
            restore_previous(dir)?;
            Err(error)
        }
    }
}

/// Copies what each file replaces into `previous`, then writes the files and the manifest last.
fn write_files(dir: &Path, files: &Files) -> Result<(), StorageError> {
    let previous = dir.join(PREVIOUS);
    for path in files.keys() {
        let current = dir.join(path);
        if current.is_file() {
            let keep = previous.join(path);
            fs::create_dir_all(keep.parent().expect("a file has a parent"))
                .map_err(io_at(&keep))?;
            fs::copy(&current, &keep).map_err(io_at(&current))?;
        }
    }
    let manifest = files
        .get(MANIFEST)
        .expect("encode_project writes the manifest");
    for (path, bytes) in files.iter().filter(|(path, _)| *path != MANIFEST) {
        write_atomic(&dir.join(path), bytes)?;
    }
    write_atomic(&dir.join(MANIFEST), manifest)
}

/// Puts back every file `previous` holds, then removes the marker. Running it again after an interruption gives the
/// same result, because `previous` is only read.
fn restore_previous(dir: &Path) -> Result<(), StorageError> {
    let previous = dir.join(PREVIOUS);
    if previous.is_dir() {
        for (path, bytes) in read_folder(&previous)? {
            write_atomic(&dir.join(&path), &bytes)?;
        }
    }
    let marker = dir.join(MARKER);
    fs::remove_file(&marker)
        .or_else(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e)
            }
        })
        .map_err(io_at(&marker))
}

/// A temporary file in the same directory, flushed to disk, then renamed over `path`: a reader never sees half a file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let parent = path.parent().expect("a project file has a parent");
    fs::create_dir_all(parent).map_err(io_at(parent))?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(io_at(parent))?;
    file.write_all(bytes).map_err(io_at(path))?;
    file.as_file().sync_all().map_err(io_at(path))?;
    persist(file, path).map_err(io_at(path))
}

/// Waits between renames refused because another process holds the file, about 2 s in all.
#[cfg(windows)]
const HELD_RETRY_MS: [u64; 8] = [10, 20, 40, 80, 160, 320, 640, 730];

/// On Windows a rename over a file another process holds open is refused with "Access is denied", even when Rust
/// opened it (measured on windows-2022, run 42514839589): an antivirus, an indexer or OneDrive holds files for a
/// moment, so a refusal is retried before the save fails and rolls back.
#[cfg(windows)]
fn persist(mut file: tempfile::NamedTempFile, path: &Path) -> io::Result<()> {
    let mut waits = HELD_RETRY_MS.iter();
    loop {
        match file.persist(path) {
            Ok(_) => return Ok(()),
            Err(e) => match waits.next() {
                Some(ms) if e.error.kind() == io::ErrorKind::PermissionDenied => {
                    file = e.file;
                    std::thread::sleep(std::time::Duration::from_millis(*ms));
                }
                _ => return Err(e.error),
            },
        }
    }
}

/// Elsewhere a refused rename is a permission, which waiting does not change.
#[cfg(not(windows))]
fn persist(file: tempfile::NamedTempFile, path: &Path) -> io::Result<()> {
    file.persist(path).map(drop).map_err(|e| e.error)
}

/// The files an existing manifest lists, to remove the ones a save no longer writes.
fn parse_project_paths(manifest: &[u8]) -> Option<Vec<String>> {
    let value: serde_json::Value = serde_json::from_slice(manifest).ok()?;
    Some(value.get("files")?.as_object()?.keys().cloned().collect())
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
