//! Valid fixtures for the app's tests, copied: opening a folder writes VS Code's files into it (`lsp_files.rs`), and the
//! corpus (`project/tests/corpus.rs`) checks that a fixture holds only its project's files.

use std::path::Path;

use twinova_project::storage::{open_folder, save_folder};

/// Valid fixture `name` saved into a new temporary folder, with that folder's path; the folder lives as long as the
/// returned guard.
pub fn copied(name: &str) -> (tempfile::TempDir, String) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("project/tests/fixtures/valid")
        .join(name);
    let project = open_folder(&source).unwrap().opened.project;
    let dir = tempfile::tempdir().unwrap();
    save_folder(dir.path(), &project).unwrap();
    let folder = dir.path().display().to_string();
    (dir, folder)
}
