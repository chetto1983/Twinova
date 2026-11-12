//! The files that let truST's language server read a project folder in VS Code (spec 2026-11-12-st-vscode-design.md,
//! decision 4, "Files for the LSP"). Neither is in the manifest, so the format ignores them, and Save never removes
//! them (`storage.rs` removes only files the previous manifest listed).

use std::fs;
use std::path::Path;

use twinova_engine::field::config_source;
use twinova_project::Project;

/// truST's project file, at the folder's root (`trust-lsp/src/config.rs`, `CONFIG_FILES`).
pub const CONFIG: &str = "trust-lsp.toml";
/// The field's generated program and configuration, so the blocks' instances and `VAR_CONFIG` are visible.
pub const FIELD: &str = ".twinova/lsp/field.st";

/// Written once: the user may tune it afterwards. Each setting was measured on trust-lsp 0.24.75 and 0.38.0
/// (spec, "trust-lsp on a Twinova folder").
const CONFIG_TEXT: &str = "\
# Written by Twinova for truST's language server (spec 2026-11-12-st-vscode-design.md); Twinova does not read it.
# Index only the project's sources: .twinova/previous keeps copies from the last save, which would declare everything
# twice. The generated field configuration is in .twinova/lsp.
[project]
include_paths = [\"plc\", \"components\", \".twinova/lsp\"]

# Keep the index cache out of the project's files.
[indexing]
cache_dir = \".twinova/lsp-cache\"

# The nondeterminism warnings flag every AT variable and every TIME of a simulated line.
[diagnostics]
warn_nondeterminism = false
";

fn io(path: &'static str) -> impl Fn(std::io::Error) -> String {
    move |e| format!("{path}: {e}")
}

/// Writes `trust-lsp.toml` when `folder` has none, and the field's configuration when its text changed; removes the
/// latter when no device has a behaviour.
pub fn write(folder: &Path, project: &Project) -> Result<(), String> {
    let at = |path: &str| folder.join(path);
    if !at(CONFIG).exists() {
        fs::write(at(CONFIG), CONFIG_TEXT).map_err(io(CONFIG))?;
    }
    let field = at(FIELD);
    match config_source(project)? {
        Some(text) if fs::read_to_string(&field).ok().as_deref() != Some(&text) => {
            fs::create_dir_all(field.parent().expect("a parent")).map_err(io(FIELD))?;
            fs::write(&field, text).map_err(io(FIELD))
        }
        Some(_) => Ok(()),
        None if field.exists() => fs::remove_file(&field).map_err(io(FIELD)),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "lsp_files_tests.rs"]
mod tests;
