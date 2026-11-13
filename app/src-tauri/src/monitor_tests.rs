use super::*;
use tauri::Manager;
use tauri::test::{MockRuntime, mock_app};

use crate::commands::{build, step};
use crate::project_commands::{ProjectState, open_project};
use crate::test_fixtures::copied;

const MAIN: &str = "plc/Main.st";
const TRANSFER: &str = "components/twinova.chain_transfer@1.st";

fn diverter() -> Vec<(String, String)> {
    let project = twinova_project::storage::open_folder(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/project/tests/fixtures/valid/two_belts_diverter"
    )))
    .unwrap()
    .opened
    .project;
    sources(&project).unwrap()
}

/// Each spot of `file`'s lines `first..=last` as `line:column kind watch`.
fn shown(file: &str, first: u32, last: u32, device: Option<&str>) -> Vec<String> {
    (spots(&diverter(), file, first, last, device).into_iter())
        .map(|(s, w)| {
            let what = match w {
                Watch::Path(scope, p) => format!("{scope:?} {p}"),
                Watch::Condition(scope, c) => format!("{scope:?} if {c}"),
            };
            format!("{}:{} {what}", s.line, s.column)
        })
        .collect()
}

#[test]
fn a_line_shows_each_reference_once_qualified_by_its_program() {
    assert_eq!(shown(MAIN, 40, 40, None), ["40:10 Plc Main.count"]);
    assert_eq!(
        shown(MAIN, 42, 42, None),
        ["42:16 Plc Main.reject_next", "42:25 Plc Main.count"]
    );
}

#[test]
fn a_member_shows_whole_and_a_condition_shows_qualified() {
    assert_eq!(
        shown(MAIN, 39, 39, None),
        ["39:13 Plc Main.entered.Q", "39:13 Plc if Main.entered.Q"]
    );
    assert_eq!(
        shown(MAIN, 45, 45, None),
        [
            "45:14 Plc Main.released.Q",
            "45:30 Plc Main.reject_next",
            "45:30 Plc if Main.released.Q AND Main.reject_next"
        ]
    );
}

#[test]
fn a_program_s_variables_and_members_are_targets() {
    let found = spots(&diverter(), MAIN, 39, 40, None);
    let targets: Vec<Option<String>> = (found.iter())
        .map(|(s, _)| s.target.as_ref().map(ToString::to_string))
        .collect();
    assert_eq!(
        targets,
        [
            Some("Main.entered.Q".into()),
            None,
            Some("Main.count".into())
        ]
    );
}

#[test]
fn a_block_s_file_shows_the_selected_device_s_instance() {
    let lines = shown(TRANSFER, 22, 22, Some("chain_transfer"));
    assert_eq!(
        lines,
        [
            "22:10 Field F.chain_transfer.raised",
            "22:10 Field if F.chain_transfer.raised"
        ]
    );
    assert!(
        shown(TRANSFER, 22, 22, None).is_empty(),
        "no device, no instance to read"
    );
}

#[test]
fn comments_an_unknown_file_or_lines_past_the_end_show_nothing() {
    assert!(shown(MAIN, 1, 10, None).is_empty(), "the header comment");
    assert!(shown("plc/Other.st", 1, 99, None).is_empty());
    assert!(shown(MAIN, 900, 999, None).is_empty());
}

#[test]
fn a_position_counts_utf16_columns_and_lines_cover_whole_lines() {
    let text = "a\nè x\n";
    assert_eq!(position(text, 0), (1, 1));
    assert_eq!(position(text, 2), (2, 1));
    assert_eq!(
        position(text, 5),
        (2, 3),
        "è is one UTF-16 unit and two bytes"
    );
    let at = |n: u32| TextSize::from(n);
    assert_eq!(lines(text, 2, 2), TextRange::new(at(2), at(6)));
    assert_eq!(lines(text, 1, 3), TextRange::new(at(0), at(7)));
    assert_eq!(lines(text, 5, 6), TextRange::new(at(7), at(7)));
}

fn app() -> tauri::App<MockRuntime> {
    let app = mock_app();
    app.manage(AppState::default());
    app.manage(ProjectState::default());
    open_project(app.state(), copied("two_belts_diverter").1).unwrap();
    app
}

fn built(app: &tauri::App<MockRuntime>) {
    let project = app.state::<ProjectState>().project().unwrap();
    build(
        app.state(),
        app.state(),
        project.program,
        project.behaviours,
    )
    .unwrap();
}

#[test]
fn the_commands_watch_read_force_and_release() {
    let app = app();
    assert_eq!(
        watch_range(app.state(), MAIN.into(), 37, 40, None).unwrap_err(),
        "build the project first"
    );
    built(&app);
    let spots = watch_range(app.state(), MAIN.into(), 37, 37, None).unwrap();
    assert_eq!(spots.len(), 1);
    step(app.state()).unwrap();
    assert_eq!(watched(app.state()).unwrap(), [Some("TRUE".to_string())]);
    let target = spots[0].target.clone().unwrap();
    let forced = force(app.state(), target.clone(), "FALSE".into()).unwrap();
    assert_eq!(
        forced,
        [Force {
            target: target.clone(),
            label: "Main.belt_b_run".into(),
            value: "FALSE".into()
        }]
    );
    step(app.state()).unwrap();
    assert_eq!(watched(app.state()).unwrap(), [Some("FALSE".to_string())]);
    assert_eq!(forces(app.state()).unwrap(), forced);
    assert!(release(app.state(), target.clone()).unwrap().is_empty());
    write_value(app.state(), target, "FALSE".into()).unwrap();
    unwatch(app.state()).unwrap();
    assert!(watched(app.state()).unwrap().is_empty());
}
