//! Navigation on the 4.4a line's program (spec 2026-11-13-completion-navigation-design.md, measured 2026-11-13).
use super::*;
use crate::analysis::tests::{analyzed, opened};
use tauri::Manager;

const MAIN: &str = "plc/Main.st";

fn on_main(f: impl FnOnce(&tauri::App<tauri::test::MockRuntime>, String)) {
    let (app, project) = opened("two_belts_diverter");
    analyzed(&app, &project);
    let text = project
        .program
        .iter()
        .find(|(f, _)| f == MAIN)
        .unwrap()
        .1
        .clone();
    f(&app, text);
}

#[test]
fn count_s_definition_is_its_declaration() {
    on_main(|app, text| {
        let found = definition(app.state(), MAIN.into(), text, 40, 5).unwrap();
        assert_eq!(
            found,
            Some(Span {
                file: MAIN.into(),
                line: 28,
                column: 5,
                end_line: 28,
                end_column: 10,
            })
        );
    });
}

#[test]
fn a_standard_fb_has_no_definition_to_go_to() {
    on_main(|app, text| {
        assert_eq!(
            definition(app.state(), MAIN.into(), text, 27, 14).unwrap(),
            None
        );
    });
}

#[test]
fn count_s_references_say_which_write() {
    on_main(|app, text| {
        let found = references(app.state(), MAIN.into(), text, 40, 5).unwrap();
        let lines: Vec<(u32, bool)> = found.iter().map(|r| (r.span.line, r.write)).collect();
        assert_eq!(lines, [(28, false), (40, true), (40, false), (42, false)]);
        assert!(found.iter().all(|r| r.span.file == MAIN));
    });
}

#[test]
fn a_place_with_nothing_named_has_no_definition_nor_reference() {
    on_main(|app, text| {
        assert_eq!(
            definition(app.state(), MAIN.into(), text.clone(), 1, 1).unwrap(),
            None
        );
        assert!(
            references(app.state(), MAIN.into(), text, 1, 1)
                .unwrap()
                .is_empty()
        );
    });
}
