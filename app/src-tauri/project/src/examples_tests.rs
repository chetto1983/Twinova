use super::*;
use crate::storage::open_folder;
use std::collections::BTreeSet;
use std::path::Path;

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid")
}

#[test]
fn every_example_is_its_fixture_as_opened_from_its_folder() {
    for e in examples() {
        let folder = open_folder(&fixtures().join(e.id)).unwrap().opened;
        assert!(folder.changed_outside.is_empty(), "{}", e.id);
        assert_eq!(example(e.id).unwrap(), folder.project, "{}", e.id);
        assert_eq!(e.name, folder.project.name);
    }
}

#[test]
fn the_examples_are_every_valid_fixture_once_each_with_its_own_name() {
    let listed: BTreeSet<&str> = examples().iter().map(|e| e.id).collect();
    let stored: BTreeSet<String> = (std::fs::read_dir(fixtures()).unwrap())
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(
        listed,
        stored.iter().map(String::as_str).collect(),
        "a new valid fixture is an example too"
    );
    let names: BTreeSet<String> = examples().into_iter().map(|e| e.name).collect();
    assert_eq!(names.len(), EXAMPLES.len(), "the menu tells them apart");
    assert_eq!(examples()[0].id, DEMO);
    assert_eq!(example(DEMO).unwrap().name, "M3 demo");
}

#[test]
fn there_is_no_example_by_an_unknown_id() {
    assert_eq!(example("nope"), None);
}

#[test]
fn the_library_s_files_are_its_families_definitions_and_blocks() {
    let files = library::files();
    assert_eq!(
        files
            .iter()
            .filter(|(path, _)| path.ends_with(".st"))
            .count(),
        (library::families().iter())
            .filter(|f| f.behaviour.is_some())
            .count()
    );
    let (path, text) = &files[0];
    assert_eq!(path, "components/twinova.belt@1.json");
    assert_eq!(
        serde_json::from_str::<crate::model::Definition>(text).unwrap(),
        library::family("twinova.belt@1").unwrap().definition
    );
    assert_eq!(files[1].0, "components/twinova.belt@1.st");
}
