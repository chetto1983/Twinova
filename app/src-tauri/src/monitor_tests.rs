use super::*;
use tauri::Manager;

use crate::commands::step;
use crate::test_fixtures::opened;

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
    assert_eq!(shown(MAIN, 39, 39, None), ["39:13 Plc Main.entered.Q"]);
    assert_eq!(
        shown(MAIN, 45, 45, None),
        [
            "45:14 Plc Main.released.Q",
            "45:30 Plc Main.reject_next",
            "45:30 Plc if Main.released.Q AND Main.reject_next"
        ]
    );
    let merged = &spots(&diverter(), MAIN, 39, 39, None)[0].0;
    assert!(
        merged.condition,
        "a one-reference condition colours its reference's value"
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
        [Some("Main.entered.Q".into()), Some("Main.count".into())]
    );
}

#[test]
fn a_repeat_s_condition_is_after_until() {
    let text = "PROGRAM Main\nVAR\n    n : INT;\nEND_VAR\nREPEAT\n    n := n + 1;\nUNTIL n > 3\nEND_REPEAT;\nEND_PROGRAM\n";
    let files = [("plc/Main.st".to_string(), text.to_string())];
    let shown: Vec<String> = (spots(&files, "plc/Main.st", 7, 7, None).into_iter())
        .map(|(s, w)| format!("{}:{} {w:?}", s.line, s.column))
        .collect();
    assert_eq!(
        shown,
        [
            "7:8 Path(Plc, \"Main.n\")",
            "7:12 Condition(Plc, \"Main.n > 3\")"
        ]
    );
}

#[test]
fn a_block_s_file_shows_the_selected_device_s_instance() {
    let found = spots(&diverter(), TRANSFER, 22, 22, Some("chain_transfer"));
    assert!(
        found.iter().all(|(s, _)| s.target.is_none()),
        "a block is read-only"
    );
    let lines = shown(TRANSFER, 22, 22, Some("chain_transfer"));
    assert_eq!(lines, ["22:10 Field F.chain_transfer.raised"]);
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

#[test]
fn the_commands_watch_read_force_and_release() {
    let app = opened("two_belts_diverter", false);
    assert_eq!(
        watch_range(app.state(), MAIN.into(), 37, 40, None, vec![]).unwrap_err(),
        "build the project first"
    );
    let app = opened("two_belts_diverter", true);
    let watched_now =
        watch_range(app.state(), MAIN.into(), 37, 37, None, vec!["carry".into()]).unwrap();
    let spots = watched_now.spots;
    assert_eq!(spots.len(), 1);
    let rows = &watched_now.rows;
    assert_eq!(
        (rows.len(), rows[0].name.as_str()),
        (20 + 5, "detected_a"),
        "20 declarations, carry's 5 members"
    );
    let carry = rows.iter().position(|r| r.name == "carry").unwrap();
    assert!(rows[carry].instance && rows[carry].target.is_none());
    let pt = &rows[carry + 1..carry + 6];
    assert!(
        pt.iter()
            .all(|r| r.name.starts_with("carry.") && r.declaration.is_none() && r.target.is_some()),
        "{pt:?}"
    );
    tauri::async_runtime::block_on(step(app.state())).unwrap();
    let values = watched(app.state()).unwrap();
    assert_eq!(values[0], Some("TRUE".to_string()));
    assert_eq!(values.len(), 1 + rows.len());
    assert_eq!(
        values[1 + rows.iter().position(|r| r.name == "carry.PT").unwrap()],
        Some("T#1500ms".into())
    );
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
    tauri::async_runtime::block_on(step(app.state())).unwrap();
    assert_eq!(watched(app.state()).unwrap()[0], Some("FALSE".to_string()));
    assert_eq!(forces(app.state()).unwrap(), forced);
    let eye = Target::address("%IX0.6");
    force(app.state(), eye.clone(), "TRUE".into()).unwrap();
    let left = release(app.state(), target.clone()).unwrap();
    let labels: Vec<&str> = left.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(labels, ["%IX0.6"], "only the released force goes");
    assert!(release(app.state(), eye.clone()).unwrap().is_empty());
    write_value(app.state(), target, "FALSE".into()).unwrap();
    assert_eq!(
        write_value(app.state(), eye, "TRUE".into()).unwrap_err(),
        "%IX0.6 is an address: force it instead"
    );
    unwatch(app.state()).unwrap();
    assert!(watched(app.state()).unwrap().is_empty());
}
