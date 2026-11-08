use super::*;
use crate::codec::sha256;

fn reference() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/two_belts_diverter");
    parse_project(&read_folder(&dir).unwrap()).unwrap().project
}

fn renamed(project: &Project, name: &str) -> Project {
    let mut p = project.clone();
    p.scene.instances[1].name = name.into();
    p
}

fn with_asset(project: &Project, bytes: &[u8]) -> (Project, String) {
    let mut p = project.clone();
    let path = format!("assets/{}.glb", sha256(bytes));
    p.assets.insert(path.clone(), bytes.to_vec());
    (p, path)
}

fn open(dir: &Path) -> OpenedFolder {
    open_folder(dir).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn a_save_opens_back_and_leaves_no_marker() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let opened = open(dir.path());
    assert_eq!((opened.opened.project, opened.recovered), (p1, false));
    assert!(opened.opened.changed_outside.is_empty());
    assert!(!dir.path().join(MARKER).exists());
}

#[test]
fn a_second_save_keeps_what_it_replaced_in_previous() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let old_scene = fs::read(dir.path().join("scene.json")).unwrap();
    let p2 = renamed(&p1, "Renamed belt");
    save_folder(dir.path(), &p2).unwrap();
    assert_eq!(open(dir.path()).opened.project, p2);
    assert_eq!(
        fs::read(dir.path().join(PREVIOUS).join("scene.json")).unwrap(),
        old_scene
    );
}

/// The state a save leaves when it is killed after replacing some files: the marker, `previous`, and new bytes.
fn interrupted(dir: &Path, p1: &Project, p2: &Project) {
    save_folder(dir, p1).unwrap();
    for (path, bytes) in read_folder(dir).unwrap() {
        write_atomic(&dir.join(PREVIOUS).join(&path), &bytes).unwrap();
    }
    write_atomic(&dir.join(MARKER), b"").unwrap();
    write_atomic(&dir.join("scene.json"), &encode_project(p2)["scene.json"]).unwrap();
}

#[test]
fn an_interrupted_save_is_rolled_back_at_open() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    interrupted(dir.path(), &p1, &renamed(&p1, "Half saved"));
    let opened = open(dir.path());
    assert!(opened.recovered);
    assert_eq!(opened.opened.project, p1);
    assert!(
        opened.opened.changed_outside.is_empty(),
        "the rollback restores the exact bytes"
    );
    assert!(!dir.path().join(MARKER).exists());
    assert!(!open(dir.path()).recovered, "recovered once");
}

#[test]
fn a_recovery_that_is_itself_interrupted_completes_at_the_next_open() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    interrupted(dir.path(), &p1, &renamed(&p1, "Half saved"));
    // The recovery was killed after restoring only the program: the marker is still there.
    let previous = dir.path().join(PREVIOUS);
    write_atomic(
        &dir.path().join("plc/Main.st"),
        &fs::read(previous.join("plc/Main.st")).unwrap(),
    )
    .unwrap();
    let opened = open(dir.path());
    assert!(opened.recovered);
    assert_eq!(opened.opened.project, p1);
}

#[test]
fn a_change_made_outside_is_opened_and_reported_not_rolled_back() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let edited = format!("{}(* edited in VS Code *)\n", p1.program[0].1);
    fs::write(dir.path().join("plc/Main.st"), &edited).unwrap();
    let opened = open(dir.path());
    assert!(!opened.recovered);
    assert_eq!(opened.opened.changed_outside, ["plc/Main.st"]);
    assert_eq!(opened.opened.project.program[0].1, edited);
}

#[test]
fn a_failed_save_rolls_back_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let (p2, asset) = with_asset(&renamed(&p1, "Never saved"), b"glTF");
    // A directory where the new asset must go makes its rename fail, for any user and on every platform.
    fs::create_dir_all(dir.path().join(&asset)).unwrap();
    let error = save_folder(dir.path(), &p2).unwrap_err();
    assert!(matches!(error, StorageError::Io { .. }), "{error}");
    assert!(!dir.path().join(MARKER).exists(), "the rollback ran");
    let opened = open(dir.path());
    assert_eq!((opened.opened.project, opened.recovered), (p1, false));
}

#[test]
fn a_save_refuses_to_run_over_an_unrecovered_interruption() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    interrupted(dir.path(), &p1, &renamed(&p1, "Half saved"));
    assert!(matches!(
        save_folder(dir.path(), &p1),
        Err(StorageError::Interrupted)
    ));
}

#[test]
fn files_a_save_no_longer_writes_are_removed() {
    let dir = tempfile::tempdir().unwrap();
    let (p1, asset) = with_asset(&reference(), b"glTF");
    save_folder(dir.path(), &p1).unwrap();
    assert!(dir.path().join(&asset).is_file());
    save_folder(dir.path(), &reference()).unwrap();
    assert!(!dir.path().join(&asset).exists());
    assert_eq!(open(dir.path()).opened.project, reference());
}

#[test]
fn a_folder_that_is_not_a_valid_project_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        open_folder(dir.path()),
        Err(StorageError::Invalid(_))
    ));
    assert!(matches!(
        open_folder(&dir.path().join("missing")),
        Err(StorageError::Io { .. })
    ));
}

/// Windows refuses to rename over a read-only file (MoveFileExW); Linux replaces it, because only the directory's
/// permissions matter there. Measured by CI on windows-2022.
#[test]
fn a_read_only_file_in_the_way() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let scene = dir.path().join("scene.json");
    let mut permissions = fs::metadata(&scene).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&scene, permissions).unwrap();
    let p2 = renamed(&p1, "Over a read-only file");
    let result = save_folder(dir.path(), &p2);
    if cfg!(windows) {
        assert!(result.is_err(), "Windows refuses the rename");
        let mut permissions = fs::metadata(&scene).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        fs::set_permissions(&scene, permissions).unwrap();
        assert_eq!(
            open(dir.path()).opened.project,
            p1,
            "the previous save survives"
        );
    } else {
        result.unwrap();
        assert_eq!(open(dir.path()).opened.project, p2);
    }
}

/// Another process holding the file open for good. Windows refuses the rename even after the retries (run 42514839589),
/// so the save fails; its rollback writes the same file and fails too, which leaves the marker, and the next open
/// recovers once the file is released. Linux renames over an open file.
#[test]
fn a_file_held_open_by_another_reader() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let held = fs::File::open(dir.path().join("plc/Main.st")).unwrap();
    let mut p2 = p1.clone();
    p2.program[0].1.push_str("(* saved while open *)\n");
    let result = save_folder(dir.path(), &p2);
    drop(held);
    if cfg!(windows) {
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("Main.st"),
            "the error names the held file: {error}"
        );
        let opened = open(dir.path());
        assert_eq!(opened.opened.project, p1, "the previous save survives");
    } else {
        result.unwrap();
        assert_eq!(open(dir.path()).opened.project, p2);
    }
}

/// A file held for a moment, as an antivirus scan does: on Windows the retry outlasts it. The thread's sleep is the
/// length of the hold being modelled, not a synchronisation.
#[test]
fn a_file_held_for_a_moment_is_saved() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = reference();
    save_folder(dir.path(), &p1).unwrap();
    let held = fs::File::open(dir.path().join("plc/Main.st")).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(held);
    });
    let mut p2 = p1.clone();
    p2.program[0].1.push_str("(* saved after the scan *)\n");
    save_folder(dir.path(), &p2).unwrap();
    release.join().unwrap();
    assert_eq!(open(dir.path()).opened.project, p2);
}
