use super::*;
use tauri::Manager;

use crate::commands::{build, step};
use crate::monitor::{watch_range, watched, write_value};
use crate::project_commands::ProjectState;
use crate::test_fixtures::opened;

const PUMP: &str = "plc/Pump.ld";
const LD: &str = "VAR start, run : BOOL; T1 : TON; waited : TIME; END_VAR\n\
                  |start| T1(IN, PT := T#20ms, ET => waited, Q) (run);\n\
                  [|run|, |/start|] (Field_ok);\n";

/// The diverter with Pump.ld beside its Main.st, built.
fn built() -> tauri::App<tauri::test::MockRuntime> {
    let app = opened("two_belts_diverter", false);
    let project = app.state::<ProjectState>().project().unwrap();
    let mut program = project.program;
    program.push((PUMP.into(), LD.replace("Field_ok", "run")));
    build(app.state(), app.state(), program, project.behaviours).unwrap();
    app
}

fn shown(spot: &LadderSpot) -> String {
    let path: Vec<String> = spot.element.path.iter().map(ToString::to_string).collect();
    let what = match (&spot.pin, spot.power) {
        (_, true) => "power".to_string(),
        (Some(pin), false) => format!("pin {pin}"),
        (None, false) => "operand".to_string(),
    };
    format!("{}:{} {what}", spot.element.rung, path.join("."))
}

#[test]
fn a_ladder_file_shows_each_element_s_power_and_operand_as_the_program_runs() {
    let app = built();
    let found = watch_range(app.state(), PUMP.into(), 1, 60, None, vec![]).unwrap();
    assert!(found.spots.is_empty());
    let spots: Vec<String> = found.ladder.iter().map(shown).collect();
    assert_eq!(
        spots,
        [
            "0:0 power",
            "0:1 power",
            "0:2 power",
            "1:0.0.0 power",
            "1:0.1.0 power",
            "1:0 power",
            "1:1 power",
            "0:0 operand",
            "0:1 pin ET",
            "0:2 operand",
            "1:0.0.0 operand",
            "1:0.1.0 operand",
            "1:1 operand",
        ]
    );
    let start = found.ladder[7].target.clone().unwrap();
    assert_eq!(
        start,
        twinova_engine::controller::Target::variable("Pump", "start")
    );
    let read = |app: &tauri::App<tauri::test::MockRuntime>| watched(app.state()).unwrap();
    let scan = |app: &tauri::App<tauri::test::MockRuntime>| {
        tauri::async_runtime::block_on(step(app.state())).unwrap()
    };
    scan(&app);
    let values = read(&app);
    assert_eq!(values.len(), found.ladder.len() + found.rows.len());
    assert_eq!(
        &values[..3],
        [
            Some("FALSE".into()),
            Some("FALSE".into()),
            Some("FALSE".into())
        ]
    );
    // the branch's lower path, /start, has power: its join does, so the coil does
    assert_eq!(
        &values[3..7],
        [
            Some("FALSE".into()),
            Some("TRUE".into()),
            Some("TRUE".into()),
            Some("TRUE".into())
        ]
    );
    write_value(app.state(), start, "TRUE".into()).unwrap();
    for _ in 0..4 {
        scan(&app);
    }
    let values = read(&app);
    assert_eq!(values[0], Some("TRUE".into()), "start's power");
    assert_eq!(values[1], Some("TRUE".into()), "T1.Q after 20 ms");
    assert_eq!(values[7], Some("TRUE".into()), "start's value");
    assert_eq!(
        values[8].as_deref().map(|v| v.starts_with("T#")),
        Some(true),
        "{:?}",
        values[8]
    );
}
