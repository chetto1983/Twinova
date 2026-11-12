use super::*;
use tauri::Manager;
use tauri::test::{MockRuntime, mock_app};

use crate::test_fixtures::copied;

fn app() -> tauri::App<MockRuntime> {
    let app = mock_app();
    app.manage(ProjectState::default());
    app
}

fn fixture(kind: &str, name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("project/tests/fixtures")
        .join(kind)
        .join(name)
        .display()
        .to_string()
}

fn path(dir: &tempfile::TempDir, name: &str) -> String {
    dir.path().join(name).display().to_string()
}

#[test]
fn a_project_opens_into_a_view_of_its_instances() {
    let app = app();
    let (_dir, folder) = copied("two_belts_diverter");
    let view = open_project(app.state(), folder.clone()).unwrap();
    assert_eq!(view.name, "Two belts and a chain transfer");
    assert_eq!(view.instances.len(), 9);
    assert_eq!(
        view.definitions["twinova.belt@1"].parameters[0].name,
        "length"
    );
    assert!(
        view.instances
            .iter()
            .any(|i| i.id == "chain_transfer" && i.signals["popup"] == "%QX0.2")
    );
    let mount = |device: &str, host: &str, at: f64| MountView {
        device: device.into(),
        host: host.into(),
        at,
    };
    assert_eq!(
        view.mounts,
        [
            mount("sensor_a", "roller_r", 1.0),
            mount("sensor_b", "belt_b", 1.5),
            mount("chain_transfer", "roller_r", 1.0)
        ]
    );
    assert_eq!(view.folder, Some(folder));
    assert!(view.changed_outside.is_empty() && !view.recovered);
}

/// The view lists each merge, so the webview opens the host's guard where Build does.
#[test]
fn a_view_lists_the_spurs_merging_onto_a_flank() {
    let app = app();
    let view = open_project(app.state(), copied("merge_line").1).unwrap();
    assert_eq!(
        view.merges,
        [MergeView {
            spur: "belt_s".into(),
            host: "belt_a".into(),
            at: 2.0,
            side: Side::Right,
        }]
    );
    assert_eq!(
        view.flows.len(),
        2,
        "the spur's exit is a merge, not a flow"
    );
    let view = open_project(app.state(), copied("two_belts_diverter").1).unwrap();
    assert!(view.merges.is_empty());
}

#[test]
fn a_project_that_fails_to_open_keeps_the_current_one() {
    let app = app();
    open_project(app.state(), copied("demo_m3").1).unwrap();
    let error = open_project(app.state(), fixture("invalid", "overlapping_addresses")).unwrap_err();
    assert!(error.contains("overlaps the address of"), "{error}");
    assert!(open_project(app.state(), fixture("valid", "no_such_project")).is_err());
    let dir = tempfile::tempdir().unwrap();
    let saved = save_project(app.state(), path(&dir, "copy")).unwrap();
    assert_eq!(
        saved.name, "M3 demo",
        "the demo is still the current project"
    );
}

#[test]
fn save_as_then_open_gives_the_same_project() {
    let app = app();
    let first = open_project(app.state(), copied("two_belts_diverter").1).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let saved = save_project(app.state(), path(&dir, "line")).unwrap();
    assert_eq!(saved.folder, Some(path(&dir, "line")));
    let reopened = open_project(app.state(), path(&dir, "line")).unwrap();
    assert_eq!(reopened.instances, first.instances);
    assert_eq!(reopened.name, first.name);
}

#[test]
fn an_exported_archive_imports_back_without_a_folder() {
    let app = app();
    let first = open_project(app.state(), copied("two_belts_diverter").1).unwrap();
    let dir = tempfile::tempdir().unwrap();
    export_project(app.state(), path(&dir, "line.zip")).unwrap();
    let imported = import_project(app.state(), path(&dir, "line.zip")).unwrap();
    assert_eq!(
        (imported.instances, imported.folder),
        (first.instances, None)
    );
    assert!(
        import_project(app.state(), path(&dir, "missing.zip"))
            .unwrap_err()
            .contains("missing.zip")
    );
    std::fs::write(dir.path().join("bad.zip"), b"not a zip").unwrap();
    assert!(
        import_project(app.state(), path(&dir, "bad.zip"))
            .unwrap_err()
            .contains("is not a zip archive")
    );
}

#[test]
fn the_app_starts_on_the_bundled_demo_with_its_sources_and_panel() {
    let app = app();
    let view = current_project(app.state()).unwrap();
    assert_eq!((view.name.as_str(), view.folder.clone()), ("M3 demo", None));
    assert_eq!(view.program[0].0, "plc/Main.st");
    assert!(view.behaviours["twinova.belt@1"].contains("FUNCTION_BLOCK BeltBehaviour"));
    let panel: Vec<(&str, &str)> = view
        .panel
        .iter()
        .map(|b| (b.name.as_str(), b.address.as_str()))
        .collect();
    assert_eq!(panel, [("Start", "%IX1.0"), ("Stop", "%IX1.1")]);
    let dir = tempfile::tempdir().unwrap();
    let saved = save_project(app.state(), path(&dir, "demo")).unwrap();
    assert_eq!(
        saved.folder,
        Some(path(&dir, "demo")),
        "the demo saves like any project"
    );
}

#[test]
fn an_export_to_an_unwritable_path_says_where() {
    let app = app();
    open_project(app.state(), copied("demo_m3").1).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target = path(&dir, "no/such/dir/x.zip");
    assert!(
        export_project(app.state(), target.clone())
            .unwrap_err()
            .starts_with(&target)
    );
}

fn args(list: &[&str]) -> Vec<std::ffi::OsString> {
    list.iter().map(Into::into).collect()
}

#[test]
fn the_launch_folder_is_the_first_argument_that_is_not_a_switch() {
    assert_eq!(launch_folder(args(&["twinova"])), None);
    assert_eq!(
        launch_folder(args(&[
            "twinova",
            "--remote-debugging-port=0",
            "/p/line",
            "/p/other"
        ])),
        Some(PathBuf::from("/p/line"))
    );
    assert_eq!(
        launch_folder(args(&["/p/twinova", "-v"])),
        None,
        "the program itself is never the folder"
    );
    assert_eq!(
        launch_folder(args(&["twinova", "/p/line", "--project=/p/named"])),
        Some(PathBuf::from("/p/named")),
        "the switch wins, as msedgedriver can pass only switches"
    );
    assert_eq!(
        launch_folder(args(&["twinova", "--projects=/p/x", "--project"])),
        None
    );
}

#[test]
fn the_launch_folder_opens_once() {
    let app = mock_app();
    let (_dir, folder) = copied("demo_m3");
    app.manage(ProjectState::launching(Some(folder.into())));
    let view = open_launch_project(app.state()).unwrap().unwrap();
    assert_eq!(view.name, "M3 demo");
    assert_eq!(open_launch_project(app.state()), Ok(None));
}

#[test]
fn a_launch_folder_that_fails_reports_and_leaves_the_demo() {
    let app = mock_app();
    app.manage(ProjectState::launching(Some(
        fixture("invalid", "overlapping_addresses").into(),
    )));
    let error = open_launch_project(app.state()).unwrap_err();
    assert!(error.contains("overlaps the address of"), "{error}");
    assert_eq!(current_project(app.state()).unwrap().name, "M3 demo");
    assert_eq!(open_launch_project(app.state()), Ok(None));
}

#[test]
fn the_demo_comes_back_as_shipped() {
    let app = app();
    open_project(app.state(), copied("two_belts_diverter").1).unwrap();
    let view = open_demo(app.state()).unwrap();
    assert_eq!((view.name.as_str(), view.folder), ("M3 demo", None));
    assert_eq!(
        app.state::<ProjectState>().project().unwrap(),
        crate::demo::demo()
    );
}

/// Opening or saving a folder gives it VS Code's files (`lsp_files.rs`); a failure to write them is a notice, and the
/// project opens all the same.
#[test]
fn a_folder_opened_or_saved_gets_the_files_for_vscode() {
    let app = app();
    let (dir, folder) = copied("two_belts_diverter");
    let opened = open_project(app.state(), folder).unwrap();
    assert_eq!(opened.notice, None);
    for file in [crate::lsp_files::CONFIG, crate::lsp_files::FIELD] {
        assert!(dir.path().join(file).is_file(), "{file}");
    }
    let saved_to = tempfile::tempdir().unwrap();
    let saved = save_project(app.state(), path(&saved_to, "line")).unwrap();
    assert_eq!(saved.notice, None);
    assert!(
        saved_to
            .path()
            .join("line")
            .join(crate::lsp_files::FIELD)
            .is_file()
    );
    let (blocked, folder) = copied("two_belts_diverter");
    std::fs::create_dir_all(blocked.path().join(crate::lsp_files::FIELD)).unwrap();
    let opened = open_project(app.state(), folder).unwrap();
    assert_eq!(opened.name, "Two belts and a chain transfer");
    let notice = opened.notice.unwrap();
    assert!(
        notice.starts_with("The files for VS Code were not written: .twinova/lsp/field.st: "),
        "{notice}"
    );
}

/// The watch follows the project's folder: the one opened or saved to, none for the demo; a folder it cannot watch is a
/// notice.
#[test]
fn the_watch_follows_the_project_s_folder() {
    let app = app();
    let (tx, quiet) = std::sync::mpsc::channel();
    let state = app.state::<ProjectState>();
    state.set_watch(SourceWatch::new(move |folder| {
        tx.send(folder.to_path_buf()).unwrap()
    }));
    let wait = std::time::Duration::from_secs(10);
    let nothing = std::time::Duration::from_millis(1_500);
    let (dir, folder) = copied("two_belts_diverter");
    open_project(app.state(), folder).unwrap();
    std::fs::write(dir.path().join("plc/Main.st"), "(* edited *)\n").unwrap();
    assert_eq!(quiet.recv_timeout(wait).unwrap(), dir.path());
    let saved = tempfile::tempdir().unwrap();
    save_project(app.state(), path(&saved, "line")).unwrap();
    std::fs::write(saved.path().join("line/plc/Main.st"), "(* edited *)\n").unwrap();
    assert_eq!(quiet.recv_timeout(wait).unwrap(), saved.path().join("line"));
    open_demo(app.state()).unwrap();
    std::fs::write(saved.path().join("line/plc/Main.st"), "(* again *)\n").unwrap();
    assert!(
        quiet.recv_timeout(nothing).is_err(),
        "the demo has no folder"
    );
    let notice = state.follow(Some(Path::new("/no/such/folder"))).unwrap();
    assert!(
        notice.starts_with("Changes made outside Twinova are not followed: /no/such/folder: "),
        "{notice}"
    );
}

#[test]
fn a_command_s_notices_are_one_a_line() {
    assert_eq!(notices([None, None]), None);
    assert_eq!(notices([Some("a".into()), None]).as_deref(), Some("a"));
    assert_eq!(
        notices([Some("a".into()), Some("b".into())]).as_deref(),
        Some("a\nb")
    );
}
