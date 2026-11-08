//! A project as one archive: the same files as the folder, zipped (spec 2026-11-08-project-format-design.md,
//! decision 1). The archive is byte-reproducible: entries in path order, a fixed timestamp and fixed permissions.

use crate::codec::{Files, Opened, encode_project, parse_project};
use crate::diagnostic::Diagnostic;
use crate::model::Project;
use crate::validate::path_problem;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

const ARCHIVE: &str = "archive";
/// Far above any project of the first families, and below what a crafted archive could expand to.
pub const MAX_UNPACKED_BYTES: u64 = 512 * 1024 * 1024;

pub fn export_archive(project: &Project) -> Vec<u8> {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644);
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    for (path, bytes) in encode_project(project) {
        zip.start_file(path, options).expect("writing to memory");
        zip.write_all(&bytes).expect("writing to memory");
    }
    zip.finish().expect("writing to memory");
    buffer.into_inner()
}

/// Reads an archive through the same validation as a folder. Entry names must be project paths; directories are
/// skipped, and the archive may not unpack to more than `MAX_UNPACKED_BYTES`. An archive with a name twice reads as
/// one entry: zip 9.4.2's reader indexes entries by name (measured 2026-11-08), and the manifest's hashes then show
/// whether that entry is the one it lists.
pub fn import_archive(bytes: &[u8]) -> Result<Opened, Vec<Diagnostic>> {
    import_within(bytes, MAX_UNPACKED_BYTES)
}

fn import_within(bytes: &[u8], max_unpacked: u64) -> Result<Opened, Vec<Diagnostic>> {
    let refuse = |message: String| vec![Diagnostic::new(ARCHIVE, "", message)];
    let mut zip = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| refuse(format!("is not a zip archive: {e}")))?;
    let mut files = Files::new();
    let mut unpacked = 0u64;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| refuse(format!("entry {i}: {e}")))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry
            .name()
            .map_err(|e| refuse(format!("entry {i} has no UTF-8 name: {e}")))?
            .into_owned();
        if let Some(problem) = path_problem(&name) {
            return Err(refuse(format!("entry '{name}' {problem}")));
        }
        unpacked = unpacked.saturating_add(entry.size());
        if unpacked > max_unpacked {
            return Err(refuse(format!("unpacks to more than {max_unpacked} bytes")));
        }
        // The reader stops at the declared size and checks the CRC, and the declared sizes are bounded above.
        let mut content = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut content)
            .map_err(|e| refuse(format!("entry '{name}': {e}")))?;
        files.insert(name, content);
    }
    parse_project(&files)
}

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
