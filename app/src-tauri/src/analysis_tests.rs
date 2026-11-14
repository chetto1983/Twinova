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
pub(crate) fn opened(fixture: &str) -> (tauri::App<MockRuntime>, Project) {
    let app = app();
    open_project(app.state(), copied(fixture).1).unwrap();
    let project = app.state::<ProjectState>().project().unwrap();
    (app, project)
}

pub(crate) fn analyzed(
    app: &tauri::App<MockRuntime>,
    project: &Project,
) -> BTreeMap<String, Vec<Problem>> {
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
/// 2026-11-12, spec "The frontend at upstream's v0.39.0"; again 2026-11-13 on the 4.4a line with its infeed photocell),
/// less the false E202 of the placed block variables.
#[test]
fn the_diverter_s_sources_show_what_vscode_shows_without_the_false_e202() {
    let (app, project) = opened("two_belts_diverter");
    let found = analyzed(&app, &project);
    assert_eq!(
        codes(&found, MAIN),
        ["11:9 W009", "14:5 W001", "21:5 W001", "22:5 W001"]
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
        ("warning", "unused variable 'detected_b'", 14, 15)
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
            "37:1 E203 cannot assign 'SINT' to 'BOOL'",
            "38:1 E101 undefined identifier 'not_declared'",
            "39:19 E205 expected TIME or LTIME for parameter 'PT'",
        ]
    );
}

/// A Ladder file's rungs answer for its own variables only: a name they use is still unused in an ST file after it.
#[test]
fn an_st_file_after_a_ladder_file_keeps_its_own_unused_variables() {
    let (app, mut project) = opened("two_belts_diverter");
    let ld = "VAR spare : BOOL; END_VAR\n|spare| (spare);\n";
    let st = "PROGRAM Other\nVAR\n    spare : BOOL;\nEND_VAR\nEND_PROGRAM\n";
    project.program.push(("plc/Pump.ld".into(), ld.into()));
    project.program.push(("plc/Other.st".into(), st.into()));
    let found = analyzed(&app, &project);
    let unused = |file: &str| -> Vec<String> {
        (found[file].iter())
            .filter(|p| p.code == "W001")
            .map(|p| p.message.clone())
            .collect()
    };
    assert_eq!(unused("plc/Other.st"), ["unused variable 'spare'"]);
    assert_eq!(unused("plc/Pump.ld"), Vec::<String>::new());
}

/// A name in a rung's comment is not a use: only the rungs' elements use a variable.
#[test]
fn a_variable_named_only_in_a_rung_s_comment_is_unused() {
    let (app, mut project) = opened("two_belts_diverter");
    let ld = "VAR spare, run : BOOL; END_VAR\n(* spare is wired later *)\n|run| (run);\n";
    project.program.push(("plc/Pump.ld".into(), ld.into()));
    let found = analyzed(&app, &project);
    let unused: Vec<String> = (found["plc/Pump.ld"].iter())
        .filter(|p| p.code == "W001")
        .map(|p| p.message.clone())
        .collect();
    assert_eq!(unused, ["unused variable 'spare'"]);
}

/// A Ladder file reaches the frontend as its declarations alone: their problems show at the file's own lines and
/// columns, and the ST files see its program.
#[test]
fn a_ladder_file_shows_the_problems_of_its_declarations_where_it_has_them() {
    let (app, mut project) = opened("two_belts_diverter");
    let ld =
        "VAR x : NoSuchType; END_VAR\nVAR\n    y : BOOL;\n    z : Missing;\nEND_VAR\n|y| (y);\n";
    project.program.push(("plc/Pump.ld".into(), ld.into()));
    project
        .program
        .push(("plc/Bad.ld".into(), "|y| (y)".into()));
    let found = analyzed(&app, &project);
    let errors: Vec<String> = (found["plc/Pump.ld"].iter())
        .filter(|p| p.severity == "error")
        .map(|p| {
            format!(
                "{}:{}-{}:{} {}",
                p.line, p.column, p.end_line, p.end_column, p.code
            )
        })
        .collect();
    assert_eq!(errors, ["1:9-1:19 E102", "4:9-4:16 E102"]);
    // the rungs use y: the frontend, which reads the declarations alone, would call it unused
    let unused: Vec<String> = (found["plc/Pump.ld"].iter())
        .filter(|p| p.code == "W001")
        .map(|p| p.message.clone())
        .collect();
    assert_eq!(unused, ["unused variable 'x'", "unused variable 'z'"]);
    assert_eq!(
        found["plc/Bad.ld"],
        [],
        "Build reports a file that does not read"
    );
    assert!(
        codes(&found, MAIN).iter().all(|c| !c.contains(" E")),
        "{:?}",
        codes(&found, MAIN)
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
    let text = project.program[0].1.clone();
    let (line, column) = text
        .lines()
        .enumerate()
        .find_map(|(n, l)| {
            l.find("arrived(CLK := detected_a")
                .map(|c| (n as u32 + 1, c as u32 + 16))
        })
        .unwrap();
    let shown = hover(app.state(), MAIN.into(), text.clone(), line, column)
        .unwrap()
        .unwrap();
    assert!(
        shown.contains("detected_a") && shown.contains("BOOL"),
        "{shown}"
    );
    assert_eq!(
        hover(app.state(), MAIN.into(), text.clone(), 1, 1).unwrap(),
        None,
        "a comment has no hover"
    );
    let offered = completion(app.state(), MAIN.into(), text.clone(), line, column + 3).unwrap();
    let labels: Vec<&str> = offered.iter().map(|c| c.label.as_str()).collect();
    assert!(
        labels.contains(&"detected_a") && labels.contains(&"detected_b"),
        "{labels:?}"
    );
    assert!(offered.iter().all(|c| !c.insert_text.is_empty()));
    let error = hover(app.state(), "plc/Nowhere.st".into(), String::new(), 1, 1).unwrap_err();
    assert_eq!(error, "plc/Nowhere.st is not analysed");
}

/// Completion answers on the text the editor holds now, though the debounced analysis has not run on it yet: a
/// variable declared a moment ago is offered.
#[test]
fn completion_reads_the_text_typed_since_the_last_analysis() {
    let (app, project) = opened("two_belts_diverter");
    analyzed(&app, &project);
    let typed = project.program[0]
        .1
        .replace(
            "    count : DINT;\n",
            "    count : DINT;\n    zz_fresh : BOOL;\n",
        )
        .replace("belt_b_run := TRUE;", "belt_b_run := TRUE;\nzz_");
    let line = typed.lines().position(|l| l == "zz_").unwrap() as u32 + 1;
    let offered = completion(app.state(), MAIN.into(), typed.clone(), line, 4).unwrap();
    assert!(offered.iter().any(|c| c.label == "zz_fresh"), "{offered:?}");
    let stale = completion(
        app.state(),
        MAIN.into(),
        project.program[0].1.clone(),
        line,
        4,
    )
    .unwrap();
    assert!(
        stale.iter().all(|c| c.label != "zz_fresh"),
        "the text sent back replaces it again"
    );
}
