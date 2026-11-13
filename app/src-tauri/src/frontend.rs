//! truST's language frontend on the editor's sources, as the rename, the monitoring and the call templates read them.

use trust_hir::db::FileId;
use trust_hir::project::{Project as Frontend, SourceKey};

/// A frontend holding `files`, and each file's id in the same order.
pub(crate) fn frontend_of(files: &[(String, String)]) -> (Frontend, Vec<FileId>) {
    let mut frontend = Frontend::new();
    let ids = (files.iter())
        .map(|(path, source)| {
            frontend.set_source_text(SourceKey::from_virtual(path.clone()), source.clone())
        })
        .collect();
    (frontend, ids)
}

/// `file`'s id among `files`.
pub(crate) fn id_of(files: &[(String, String)], ids: &[FileId], file: &str) -> Option<FileId> {
    files.iter().position(|(f, _)| f == file).map(|at| ids[at])
}

/// `file`'s text among `files`.
pub(crate) fn text_of<'a>(files: &'a [(String, String)], file: &str) -> Result<&'a str, String> {
    (files.iter().find(|(f, _)| f == file))
        .map(|(_, text)| text.as_str())
        .ok_or_else(|| "no such file".to_string())
}
