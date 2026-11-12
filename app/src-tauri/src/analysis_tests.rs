use super::*;
use tauri::Manager;
use tauri::test::{MockRuntime, mock_app};

use crate::project_commands::open_project;
use crate::test_fixtures::copied;

const MAIN: &str = "plc/Main.st";

fn app() -> tauri::App<MockRuntime> {
    let app = mock_app();
    app.manage(ProjectState::default());
    app.manage(AnalysisState::default());
    app
}

/// The app on a copy of `fixture`, and that project.
fn opened(fixture: &str) -> (tauri::App<MockRuntime>, Project) {
    let app = app();
    open_project(app.state(), copied(fixture).1).unwrap();
    let project = app.state::<ProjectState>().project().unwrap();
    (app, project)
}

fn analyzed(app: &tauri::App<MockRuntime>, project: &Project) -> BTreeMap<String, Vec<Problem>> {
    let (program, behaviours) = (project.program.clone(), project.behaviours.clone());
    analyze(app.state(), app.state(), program, behaviours).unwrap()
}

/// Each problem of `file` as `line:column code`.
fn codes(found: &BTreeMap<String, Vec<Problem>>, file: &str) -> Vec<String> {
    (found[file].iter())
        .map(|p| format!("{}:{} {}", p.line, p.column, p.code))
        .collect()
}

/// The diverter's program and blocks as trust-lsp 0.39.0 shows them with Twinova's trust-lsp.toml (measured
/// 2026-11-12, spec "The frontend at upstream's v0.39.0"), less the false E202 of the placed block variables.
#[test]
fn the_diverter_s_sources_show_what_vscode_shows_without_the_false_e202() {
    let (app, project) = opened("two_belts_diverter");
    let found = analyzed(&app, &project);
    assert_eq!(
        codes(&found, MAIN),
        ["6:9 W009", "9:5 W001", "15:5 W001", "16:5 W001"]
    );
    assert_eq!(codes(&found, "components/twinova.belt@1.st"), ["34:3 W013"]);
    assert_eq!(
        codes(&found, "components/twinova.chain_transfer@1.st"),
        ["21:56 W005", "25:30 W005"]
    );
    assert_eq!(
        codes(&found, "components/twinova.roller@1.st"),
        ["34:3 W013"]
    );
    assert!(
        !found.contains_key(FIELD),
        "the generated configuration is not shown"
    );
    let first = &found[MAIN][1];
    assert_eq!(
        (
            first.severity.as_str(),
            first.message.as_str(),
            first.end_line,
            first.end_column
        ),
        ("warning", "unused variable 'detected_b'", 9, 15)
    );
}

#[test]
fn a_broken_program_shows_its_errors_before_build() {
    let (app, mut project) = opened("two_belts_diverter");
    project.program[0].1 = project.program[0].1.replace(
        "belt_b_run := TRUE;",
        "belt_b_run := 5;\nnot_declared := 1;\ncarry(IN := popup, PT := 15);",
    );
    let found = analyzed(&app, &project);
    let errors: Vec<&Problem> = found[MAIN]
        .iter()
        .filter(|p| p.severity == "error")
        .collect();
    let at = |p: &&Problem| format!("{}:{} {} {}", p.line, p.column, p.code, p.message);
    assert_eq!(
        errors.iter().map(at).collect::<Vec<_>>(),
        [
            "21:1 E203 cannot assign 'SINT' to 'BOOL'",
            "22:1 E101 undefined identifier 'not_declared'",
            "23:19 E205 expected TIME or LTIME for parameter 'PT'",
        ]
    );
}

/// An `AT %I*` the configuration does not place is a real E202: Build's field would refuse it.
#[test]
fn an_unplaced_wildcard_keeps_its_e202() {
    let (app, mut project) = opened("two_belts_diverter");
    let belt = project.behaviours.get_mut("twinova.belt@1").unwrap();
    *belt = belt.replace(
        "    run AT %I* : BOOL;\n",
        "    run AT %I* : BOOL;\n    extra AT %I* : BOOL;\n",
    );
    let found = analyzed(&app, &project);
    let belt = &found["components/twinova.belt@1.st"];
    let unmapped: Vec<&str> = (belt.iter())
        .filter(|p| p.code == "E202")
        .map(|p| p.message.as_str())
        .collect();
    assert_eq!(
        unmapped,
        ["direct address '%I*' requires VAR_CONFIG mapping"]
    );
    assert_eq!(belt.iter().find(|p| p.code == "E202").unwrap().line, 22);
}

/// Columns count UTF-16 units, as Monaco's do: after "(* ° è *) " the identifier starts at column 11, where bytes would
/// put it at 13. The engine's range of a name it cannot resolve takes the space after it, so it ends at 19 (measured).
#[test]
fn a_column_after_accented_letters_counts_utf16_units() {
    let (app, mut project) = opened("demo_m3");
    project.program[0].1 = project.program[0]
        .1
        .replace("END_PROGRAM", "(* ° è *) nowhere := 1;\nEND_PROGRAM");
    let found = analyzed(&app, &project);
    let undefined = found[MAIN].iter().find(|p| p.code == "E101").unwrap();
    assert_eq!((undefined.column, undefined.end_column), (11, 19));
}

#[test]
fn hover_tells_a_variable_s_type_and_completion_offers_the_declared_ones() {
    let (app, project) = opened("two_belts_diverter");
    analyzed(&app, &project);
    let text = &project.program[0].1;
    let (line, column) = text
        .lines()
        .enumerate()
        .find_map(|(n, l)| {
            l.find("arrived(CLK := detected_a")
                .map(|c| (n as u32 + 1, c as u32 + 16))
        })
        .unwrap();
    let shown = hover(app.state(), MAIN.into(), line, column)
        .unwrap()
        .unwrap();
    assert!(
        shown.contains("detected_a") && shown.contains("BOOL"),
        "{shown}"
    );
    assert_eq!(
        hover(app.state(), MAIN.into(), 1, 1).unwrap(),
        None,
        "a comment has no hover"
    );
    let offered = completion(app.state(), MAIN.into(), line, column + 3).unwrap();
    let labels: Vec<&str> = offered.iter().map(|c| c.label.as_str()).collect();
    assert!(
        labels.contains(&"detected_a") && labels.contains(&"detected_b"),
        "{labels:?}"
    );
    assert!(offered.iter().all(|c| !c.insert_text.is_empty()));
    assert!(hover(app.state(), "plc/Nowhere.st".into(), 1, 1).is_err());
}
