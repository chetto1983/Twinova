use super::*;
use crate::validate;
use std::collections::BTreeSet;
use std::path::Path;

fn library_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../library/components")
}

#[test]
fn every_family_parses_and_validates_as_a_definition() {
    for f in families() {
        let key = f.definition.key();
        let problems = validate::definition(
            &format!("components/{key}.json"),
            &f.definition,
            &BTreeSet::new(),
        );
        assert!(problems.is_empty(), "{key}: {problems:?}");
        assert_eq!(
            f.behaviour.is_some(),
            f.definition.behaviour.is_some(),
            "{key}: its block, if its definition names one"
        );
    }
}

#[test]
fn the_list_names_every_file_of_the_library_folder_once() {
    let listed: BTreeSet<String> = families()
        .iter()
        .flat_map(|f| {
            let key = f.definition.key();
            let block = f.behaviour.map(|_| format!("{key}.st"));
            std::iter::once(format!("{key}.json")).chain(block)
        })
        .collect();
    let stored: BTreeSet<String> = std::fs::read_dir(library_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(listed, stored);
    assert_eq!(families().len(), 15);
}

#[test]
fn every_fixtures_copy_of_a_family_equals_the_librarys() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid");
    let mut compared = 0;
    let mut used = BTreeSet::new();
    for project in std::fs::read_dir(&fixtures).unwrap() {
        let components = project.unwrap().path().join("components");
        for file in std::fs::read_dir(&components).unwrap() {
            let file = file.unwrap();
            let name = file.file_name().into_string().unwrap();
            let library = std::fs::read(library_dir().join(&name))
                .unwrap_or_else(|_| panic!("{name} is in the library"));
            assert_eq!(
                std::fs::read(file.path()).unwrap(),
                library,
                "{}",
                file.path().display()
            );
            compared += 1;
            used.insert(name);
        }
    }
    assert!(compared >= 40, "{compared} copies");
    // Every family is in a fixture that the engine's tests build and run, so every block compiles in the field.
    let stored: BTreeSet<String> = std::fs::read_dir(library_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(used, stored, "every family is used by a fixture");
}

#[test]
fn a_family_is_found_by_its_key_and_listed_in_its_group() {
    let belt = family("twinova.belt@1").unwrap();
    assert_eq!(
        (belt.group, belt.definition.id.as_str()),
        (Group::Conveyors, "twinova.belt")
    );
    assert!(belt.behaviour.unwrap().contains("FUNCTION_BLOCK"));
    assert_eq!(
        family("twinova.scanner_barcode@1").unwrap().group,
        Group::Sensors
    );
    assert_eq!(family("twinova.sink@1").unwrap().group, Group::Flow);
    assert_eq!(family("twinova.pusher@1").unwrap().group, Group::Devices);
    assert!(family("twinova.belt@2").is_none());
}
