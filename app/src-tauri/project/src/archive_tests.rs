use super::*;
use crate::storage::read_folder;
use std::path::Path;

fn reference() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/two_belts_diverter");
    parse_project(&read_folder(&dir).unwrap()).unwrap().project
}

fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    for (name, bytes) in entries {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    buffer.into_inner()
}

fn refused(bytes: &[u8]) -> String {
    import_archive(bytes)
        .unwrap_err()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn an_archive_is_reproducible_and_imports_back() {
    let project = reference();
    let archive = export_archive(&project);
    assert_eq!(
        archive,
        export_archive(&project),
        "same project, same bytes"
    );
    let opened = import_archive(&archive).unwrap();
    assert_eq!(opened.project, project);
    assert!(opened.changed_outside.is_empty());
}

#[test]
fn directories_in_an_archive_are_skipped() {
    let files = encode_project(&reference());
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    zip.add_directory("plc/", SimpleFileOptions::default())
        .unwrap();
    for (name, bytes) in &files {
        zip.start_file(name.as_str(), SimpleFileOptions::default())
            .unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    assert_eq!(
        import_archive(&buffer.into_inner()).unwrap().project,
        reference()
    );
}

#[test]
fn a_bad_archive_is_refused_before_any_validation() {
    assert!(refused(b"not a zip").contains("is not a zip archive"));
    assert!(
        refused(&zip_of(&[("../evil.st", b"x")])).contains("entry '../evil.st' must not contain")
    );
    assert!(refused(&zip_of(&[("/etc/passwd", b"x")])).contains("must be relative"));
    assert!(refused(&zip_of(&[("notes.txt", b"x")])).contains("not a Twinova project"));
}

#[test]
fn an_archive_that_unpacks_too_large_is_refused() {
    let archive = export_archive(&reference());
    let total: u64 = encode_project(&reference())
        .values()
        .map(|b| b.len() as u64)
        .sum();
    assert!(
        import_within(&archive, total).is_ok(),
        "exactly at the limit"
    );
    let text = import_within(&archive, total - 1).unwrap_err()[0].to_string();
    assert!(
        text.contains(&format!("unpacks to more than {} bytes", total - 1)),
        "{text}"
    );
}

#[test]
fn the_unpack_limit_is_512_mib() {
    assert_eq!(MAX_UNPACKED_BYTES, 536_870_912);
}
