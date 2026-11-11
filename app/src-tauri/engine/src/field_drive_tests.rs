//! The conveyor drive's logic, OIP's `conveyorDrive.js` in ST (spec 2026-11-10-curves-belt-options-design.md, decision 5
//! and acceptance 3): the ramp, and `speed_ref`, bound or not. Its feedback is the native model's (`drive_tests.rs`,
//! `line_tests.rs`; spec 2026-11-10-run-pace-native-feedback-design.md).

use super::*;
use std::path::Path;
use twinova_project::storage::open_folder;

fn demo(parameters: &[(&str, ParamValue)]) -> Field {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    let mut project = open_folder(&dir).unwrap().opened.project;
    let belt = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_a")
        .unwrap();
    for (name, value) in parameters {
        belt.parameters.insert((*name).into(), *value);
    }
    let mut field = Field::build(&project).unwrap().unwrap();
    field.set_command("%QX0.0", Value::Bool(true)).unwrap();
    field
}

fn real(field: &Field, name: &str) -> f32 {
    match field.native("belt_a", name) {
        Some(Value::Real(x)) => x,
        other => panic!("{name}: {other:?}"),
    }
}

/// The surface speed after each of `cycles` cycles.
fn ramp(field: &mut Field, cycles: usize) -> Vec<f32> {
    (0..cycles)
        .map(|_| {
            field.cycle().unwrap();
            real(field, "surface_speed")
        })
        .collect()
}

fn ms(n: i64) -> ParamValue {
    ParamValue::Int(n * 1_000_000)
}

/// Acceptance 3: rated speed in `accel_time`, linearly, and stopped in `decel_time`, each on its millisecond (Review
/// focus: the ramp's exact timing in field cycles).
#[test]
fn the_ramp_reaches_rated_speed_in_accel_time_and_stops_in_decel_time() {
    let mut field = demo(&[("accel_time", ms(500)), ("decel_time", ms(200))]);
    let up = ramp(&mut field, 600);
    assert!(up[498] < 0.5 && up[499] == 0.5, "{} {}", up[498], up[499]);
    assert!((up[249] - 0.25).abs() < 1e-4, "linear: {}", up[249]);
    assert!(up[500..].iter().all(|&v| v == 0.5));
    field.set_command("%QX0.0", Value::Bool(false)).unwrap();
    let down = ramp(&mut field, 300);
    assert!(
        down[198] > 0.0 && down[199] == 0.0,
        "{} {}",
        down[198],
        down[199]
    );
    // A ramp is a rate, rated speed per ramp time: half the speed takes half the time.
    let mut half = demo(&[("accel_time", ms(500)), ("speed", ParamValue::Real(0.25))]);
    let up = ramp(&mut half, 300);
    assert!(up[248] < 0.25 && up[249] == 0.25, "{} {}", up[248], up[249]);
    let mut instant = demo(&[]);
    assert_eq!(ramp(&mut instant, 1), [0.5]);
}

/// Acceptance 3: a bound `speed_ref` sets the speed, and at 0 stops the deck with run on (spec, amended in Task 4).
#[test]
fn a_bound_speed_ref_sets_the_speed() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    let mut project = open_folder(&dir).unwrap().opened.project;
    let belt = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_a")
        .unwrap();
    belt.signals.insert("speed_ref".into(), "%QD8".into());
    let mut field = Field::build(&project).unwrap().unwrap();
    assert_eq!(field.commands(), ["%QX0.0", "%QD8"]);
    field.set_command("%QX0.0", Value::Bool(true)).unwrap();
    for (reference, speed) in [(0.3_f32, 0.3), (0.0, 0.0), (-0.2, -0.2)] {
        field
            .set_command("%QD8", Value::DWord(reference.to_bits()))
            .unwrap();
        field.cycle().unwrap();
        assert_eq!(
            real(&field, "surface_speed"),
            speed,
            "speed_ref {reference}"
        );
    }
}
