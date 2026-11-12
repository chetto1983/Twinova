use super::*;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use twinova_project::storage::open_folder;

fn fixture(name: &str) -> Project {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("project/tests/fixtures/valid")
        .join(name);
    open_folder(&dir).unwrap().opened.project
}

fn read(dir: &Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).unwrap()
}

/// The modification time pushed back, so a rewrite shows whatever the clock's resolution.
fn aged(dir: &Path, path: &str) -> SystemTime {
    let old = SystemTime::now() - Duration::from_secs(3_600);
    let file = fs::File::options()
        .write(true)
        .open(dir.join(path))
        .unwrap();
    file.set_modified(old).unwrap();
    old
}

fn modified(dir: &Path, path: &str) -> SystemTime {
    fs::metadata(dir.join(path)).unwrap().modified().unwrap()
}

#[test]
fn a_folder_gets_trust_s_config_and_the_field_s_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let project = fixture("two_belts_diverter");
    write(dir.path(), &project).unwrap();
    let config = read(dir.path(), CONFIG);
    for line in [
        "include_paths = [\"plc\", \"components\", \".twinova/lsp\"]",
        "cache_dir = \".twinova/lsp-cache\"",
        "warn_nondeterminism = false",
    ] {
        assert!(config.contains(line), "{line}");
    }
    assert_eq!(
        read(dir.path(), FIELD),
        config_source(&project).unwrap().unwrap()
    );
}

#[test]
fn the_user_s_config_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(CONFIG), "# mine\n").unwrap();
    write(dir.path(), &fixture("two_belts_diverter")).unwrap();
    assert_eq!(read(dir.path(), CONFIG), "# mine\n");
}

#[test]
fn the_field_s_configuration_is_rewritten_only_when_it_changed() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = fixture("two_belts_diverter");
    write(dir.path(), &project).unwrap();
    let old = aged(dir.path(), FIELD);
    write(dir.path(), &project).unwrap();
    assert_eq!(modified(dir.path(), FIELD), old);
    let belt = (project.scene.instances.iter_mut())
        .find(|i| i.id == "belt_b")
        .unwrap();
    belt.id = "belt_c".into();
    write(dir.path(), &project).unwrap();
    assert!(read(dir.path(), FIELD).contains("    belt_c : BeltBehaviour;\n"));
}

#[test]
fn a_line_without_behaviours_leaves_no_field_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = fixture("two_belts_diverter");
    write(dir.path(), &project).unwrap();
    for def in project.definitions.values_mut() {
        def.behaviour = None;
    }
    write(dir.path(), &project).unwrap();
    assert!(!dir.path().join(FIELD).exists());
    write(dir.path(), &project).unwrap();
    assert!(dir.path().join(CONFIG).exists());
}

#[test]
fn a_folder_that_cannot_be_written_is_an_error_naming_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("gone");
    let error = write(&missing, &fixture("two_belts_diverter")).unwrap_err();
    assert!(error.starts_with("trust-lsp.toml: "), "{error}");
    fs::create_dir_all(missing.join(".twinova/lsp/field.st")).unwrap();
    let error = write(&missing, &fixture("two_belts_diverter")).unwrap_err();
    assert!(error.starts_with(".twinova/lsp/field.st: "), "{error}");
}
