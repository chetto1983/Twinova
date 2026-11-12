use super::*;
use std::path::PathBuf;

use crate::storage::{open_folder, save_folder};

/// Valid fixture `name` saved into a new temporary folder, opened from there.
fn copied(name: &str) -> (tempfile::TempDir, Project) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/valid")
        .join(name);
    let project = open_folder(&source).unwrap().opened.project;
    let dir = tempfile::tempdir().unwrap();
    save_folder(dir.path(), &project).unwrap();
    (dir, project)
}

fn main_st(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("plc/Main.st")
}

#[test]
fn sources_equal_to_the_project_s_are_no_change() {
    let (dir, mut project) = copied("two_belts_diverter");
    assert_eq!(changed(&project, dir.path()), None);
    save_folder(dir.path(), &project).unwrap();
    assert_eq!(changed(&project, dir.path()), None, "Twinova's own Save");
    project.program[0].1.push_str("(* edited in Twinova *)\n");
    let disk = changed(&project, dir.path()).unwrap();
    assert_eq!(
        disk.kind,
        Kind::Program,
        "the project changed, the disk did not"
    );
}

#[test]
fn a_program_that_changed_on_disk_is_an_online_change() {
    let (dir, project) = copied("two_belts_diverter");
    let text = project.program[0].1.replace("count MOD 3", "count MOD 2");
    fs::write(main_st(&dir), &text).unwrap();
    let disk = changed(&project, dir.path()).unwrap();
    assert_eq!(disk.kind, Kind::Program);
    assert_eq!(disk.program, [("plc/Main.st".to_string(), text)]);
    assert_eq!(disk.behaviours, project.behaviours);
}

#[test]
fn a_behaviour_that_changed_on_disk_is_a_rebuild_with_every_source() {
    let (dir, project) = copied("two_belts_diverter");
    let program = project.program[0].1.replace("count MOD 3", "count MOD 2");
    fs::write(main_st(&dir), &program).unwrap();
    let block = project.behaviours["twinova.belt@1"].replace("(* Twinova", "(* Our");
    fs::write(dir.path().join("components/twinova.belt@1.st"), &block).unwrap();
    let disk = changed(&project, dir.path()).unwrap();
    assert_eq!(disk.kind, Kind::Behaviours);
    assert_eq!(disk.behaviours["twinova.belt@1"], block);
    assert_eq!(disk.program[0].1, program);
}

#[test]
fn a_source_that_cannot_be_read_as_text_yet_waits_for_the_next_change() {
    let (dir, project) = copied("two_belts_diverter");
    fs::remove_file(main_st(&dir)).unwrap();
    assert_eq!(
        changed(&project, dir.path()),
        None,
        "replaced, not there yet"
    );
    fs::write(main_st(&dir), [0x50, 0xC3]).unwrap();
    assert_eq!(changed(&project, dir.path()), None, "half a character");
}

#[test]
fn a_change_serialises_its_kind_as_the_webview_reads_it() {
    let disk = |kind| DiskSources {
        kind,
        program: Vec::new(),
        behaviours: BTreeMap::new(),
    };
    let json = |kind| serde_json::to_value(disk(kind)).unwrap()["kind"].clone();
    assert_eq!(json(Kind::Program), "program");
    assert_eq!(json(Kind::Behaviours), "behaviours");
}
