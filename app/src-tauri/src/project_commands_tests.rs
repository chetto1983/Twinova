use super::*;
use tauri::Manager;
use tauri::test::{MockRuntime, mock_app};

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
    let view = open_project(app.state(), fixture("valid", "two_belts_diverter")).unwrap();
    assert_eq!(view.name, "Two belts and a diverter");
    assert_eq!(view.instances.len(), 8);
    assert_eq!(
        view.definitions["twinova.belt@1"].parameters[0].name,
        "length"
    );
    assert!(
        view.instances
            .iter()
            .any(|i| i.id == "diverter" && i.signals["divert"] == "%QX0.2")
    );
    assert_eq!(view.folder, Some(fixture("valid", "two_belts_diverter")));
    assert!(view.changed_outside.is_empty() && !view.recovered);
}

#[test]
fn a_project_that_fails_to_open_keeps_the_current_one() {
    let app = app();
    open_project(app.state(), fixture("valid", "demo_m3")).unwrap();
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
    let first = open_project(app.state(), fixture("valid", "two_belts_diverter")).unwrap();
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
    let first = open_project(app.state(), fixture("valid", "two_belts_diverter")).unwrap();
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
    open_project(app.state(), fixture("valid", "demo_m3")).unwrap();
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
    app.manage(ProjectState::launching(Some(
        fixture("valid", "demo_m3").into(),
    )));
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
