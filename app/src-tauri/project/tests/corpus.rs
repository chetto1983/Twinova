//! The saved-project corpus (spec 2026-11-08-project-format-design.md, decision 6): every build keeps opening the
//! valid fixtures and rejecting the invalid ones with the same diagnostic. The fixtures were written by a separate
//! script, not by `encode_project`, so the byte comparison checks the encoder against an independent reference.

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
    for name in ["demo_m3", "two_belts_diverter"] {
        let files = read_folder(&fixtures("valid").join(name));
        let project = parse_project(&files).unwrap_or_else(|d| panic!("{name}: {d:?}"));
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
            parse_project(&written).unwrap(),
            project,
            "{name}: parse after encode"
        );
    }
}

#[test]
fn every_invalid_fixture_is_refused_with_its_diagnostic() {
    let expected = [
        ("version_newer", "saved by a newer Twinova (format 2)"),
        ("hash_mismatch", "'scene.json' does not match its hash"),
        (
            "path_traversal",
            "'../outside.st' must not contain empty, '.' or '..' parts",
        ),
        ("case_collision", "differ only in case"),
        ("parent_cycle", "nodes.a.parent: is part of a parent cycle"),
        (
            "duplicate_id",
            "'belt_a': is used by two instances or groups",
        ),
        ("two_peers", "belt_a.out has 2 peers"),
        (
            "unknown_override",
            "instances.belt_a.parameters.colour: twinova.belt@1 has no such parameter",
        ),
        (
            "out_of_range",
            "instances.belt_a.parameters.length: 100 is outside [0.3, 50]",
        ),
        ("overlapping_addresses", "%IB0 overlaps the address of"),
        (
            "bool_on_double_word",
            "a BOOL signal needs a Bit address, not %ID0",
        ),
        (
            "input_on_output_area",
            "a PlcInput signal needs a Input address, not %QX0.5",
        ),
        ("unknown_field", "unknown field `colour`"),
    ];
    let on_disk = fs::read_dir(fixtures("invalid")).unwrap().count();
    assert_eq!(
        on_disk,
        expected.len(),
        "every invalid fixture has an expectation"
    );
    for (name, message) in expected {
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
    let project = parse_project(&files).unwrap();
    files.insert("notes.txt".into(), b"an author's note".to_vec());
    assert_eq!(parse_project(&files).unwrap(), project);
}

#[test]
fn a_folder_without_a_manifest_is_not_a_project() {
    let diagnostics = parse_project(&Files::new()).unwrap_err();
    assert!(diagnostics[0].to_string().contains("not a Twinova project"));
}
