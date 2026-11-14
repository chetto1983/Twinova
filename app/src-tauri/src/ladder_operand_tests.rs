use super::*;
use tauri::Manager;

use crate::analysis::tests::{analyzed, opened};

const PUMP: &str = "plc/Pump.ld";
const LD: &str = "VAR\n    Start AT %IX0.0 : BOOL;\n    Stop : BOOL;\n    Pump_T : TON;\n    Level : INT;\nEND_VAR\n|Start| (Stop);\n";

fn offered(
    app: &tauri::App<tauri::test::MockRuntime>,
    text: &str,
    typed: &str,
) -> Vec<(String, String, Option<String>)> {
    (ladder_completion(app.state(), PUMP.into(), text.into(), typed.into())
        .unwrap()
        .into_iter())
    .map(|c| (c.label, c.kind, c.detail))
    .collect()
}

#[test]
fn an_operand_is_offered_the_file_s_variables_with_their_types_and_the_globals() {
    let (app, mut project) = opened("two_belts_diverter");
    project.program.push((PUMP.into(), LD.into()));
    analyzed(&app, &project);
    let st = offered(&app, LD, "St");
    let start = (
        "Start".to_string(),
        "variable".to_string(),
        Some("BOOL".to_string()),
    );
    assert!(st.contains(&start), "{st:?}");
    assert!(st.iter().any(|c| c.0 == "Stop"), "{st:?}");
    assert!(st.iter().all(|c| c.0 != "Level"), "{st:?}");
    // a global of the project, the field's
    assert!(
        offered(&app, LD, "Fi")
            .iter()
            .any(|c| c.0 == "Field" && c.1 == "variable")
    );
    // a variable declared a moment ago, not yet analysed
    let typed = LD.replace(
        "    Level : INT;\n",
        "    Level : INT;\n    Stage_2 : BOOL;\n",
    );
    assert!(
        offered(&app, &typed, "Sta")
            .iter()
            .any(|c| c.0 == "Stage_2")
    );
}

#[test]
fn a_file_that_does_not_read_or_is_not_ladder_offers_nothing() {
    let (app, project) = opened("two_belts_diverter");
    analyzed(&app, &project);
    let state = || app.state();
    let unread = ladder_completion(state(), PUMP.into(), "|A| (B)".into(), "A".into()).unwrap_err();
    assert_eq!(unread, "expected ';' to end the rung");
    let st =
        ladder_completion(state(), "plc/Main.st".into(), String::new(), "A".into()).unwrap_err();
    assert!(st.contains("plc/Main.st"), "{st}");
}
