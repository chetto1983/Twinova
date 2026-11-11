//! The saved-project corpus (spec 2026-11-08-project-format-design.md, decision 6): every build keeps opening the
//! valid fixtures and rejecting the invalid ones with the same diagnostic. The fixtures were written by a separate
//! script, not by `encode_project`, so the byte comparison checks the encoder against an independent reference.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use twinova_project::{Files, encode_project, parse_project};

fn fixtures(kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(kind)
}

fn read_folder(root: &Path) -> Files {
    fn walk(root: &Path, dir: &Path, files: &mut Files) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(rel, fs::read(&path).unwrap());
            }
        }
    }
    let mut files = Files::new();
    walk(root, root, &mut files);
    files
}

#[test]
fn every_valid_fixture_opens_and_is_written_back_byte_for_byte() {
    for name in [
        "demo_m3",
        "two_belts_diverter",
        "curve_line",
        "sort_line",
        "code_sort_line",
        "merge_line",
    ] {
        let files = read_folder(&fixtures("valid").join(name));
        let opened = parse_project(&files).unwrap_or_else(|d| panic!("{name}: {d:?}"));
        assert!(
            opened.changed_outside.is_empty(),
            "{name}: {:?}",
            opened.changed_outside
        );
        let project = opened.project;
        let written = encode_project(&project);
        for (path, bytes) in &files {
            assert_eq!(
                String::from_utf8_lossy(&written[path]),
                String::from_utf8_lossy(bytes),
                "{name}/{path}"
            );
        }
        assert_eq!(written.len(), files.len(), "{name}: the same files");
        assert_eq!(
            parse_project(&written).unwrap().project,
            project,
            "{name}: parse after encode"
        );
    }
}

#[test]
fn every_invalid_fixture_is_refused_with_its_diagnostic() {
    // The same table drives the real-app E2E (e2e/project.mjs).
    let table = fs::read(fixtures("invalid.json")).unwrap();
    let expected: BTreeMap<String, String> = serde_json::from_slice(&table).unwrap();
    let on_disk = fs::read_dir(fixtures("invalid")).unwrap().count();
    assert_eq!(
        on_disk,
        expected.len(),
        "every invalid fixture has an expectation"
    );
    for (name, message) in &expected {
        let files = read_folder(&fixtures("invalid").join(name));
        let diagnostics = parse_project(&files).expect_err(name);
        let text: Vec<String> = diagnostics.iter().map(ToString::to_string).collect();
        assert!(
            text.iter().any(|d| d.contains(message)),
            "{name}: expected '{message}' in {text:#?}"
        );
    }
}

#[test]
fn files_the_manifest_does_not_list_are_ignored() {
    let mut files = read_folder(&fixtures("valid").join("demo_m3"));
    let project = parse_project(&files).unwrap().project;
    files.insert("notes.txt".into(), b"an author's note".to_vec());
    assert_eq!(parse_project(&files).unwrap().project, project);
}

/// git with autocrlf on Windows (run 42510999331), or an editor, changes bytes without an interrupted save.
#[test]
fn files_changed_outside_twinova_open_and_are_reported() {
    let files = read_folder(&fixtures("valid").join("two_belts_diverter"));
    let lf = parse_project(&files).unwrap().project;
    let crlf: Files = files
        .iter()
        .map(|(path, bytes)| {
            let text = String::from_utf8(bytes.clone()).unwrap();
            let converted = if path == "twinova.json" {
                text
            } else {
                text.replace('\n', "\r\n")
            };
            (path.clone(), converted.into_bytes())
        })
        .collect();
    let opened = parse_project(&crlf).expect("CRLF line endings are still a valid project");
    assert_eq!(
        opened.changed_outside.len(),
        files.len() - 1,
        "every file but the manifest"
    );
    assert_eq!(opened.project.scene, lf.scene);
    assert_eq!(opened.project.definitions, lf.definitions);
    assert_eq!(
        opened.project.program[0].1,
        lf.program[0].1.replace('\n', "\r\n"),
        "ST text is kept as written"
    );
}

#[test]
fn a_folder_without_a_manifest_is_not_a_project() {
    let diagnostics = parse_project(&Files::new()).unwrap_err();
    assert!(diagnostics[0].to_string().contains("not a Twinova project"));
}
