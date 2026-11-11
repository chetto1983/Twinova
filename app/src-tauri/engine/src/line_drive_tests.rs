//! A conveyor's drive feedback through the line (spec 2026-11-10-run-pace-native-feedback-design.md, acceptance 1): the
//! native model writes the velocity, the encoder and the pulse to the controller, with or without a behaviour block.

use super::*;
use std::path::Path;
use twinova_project::model::ParamValue;
use twinova_project::storage::open_folder;

/// Curve A in the world's conveyors, which are in id order: belt_a, belt_b, belt_c, curve_a, curve_b.
const CURVE_A: usize = 3;

fn curve_line() -> Project {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/curve_line");
    open_folder(&dir).unwrap().opened.project
}

fn real(line: &Line, address: &str) -> f32 {
    match line.controller().read(address).unwrap() {
        Value::DWord(bits) => f32::from_bits(bits),
        other => panic!("{address}: {other:?}"),
    }
}

/// The encoder is the world's own travel of the surface, to the bit, and the pulse comes once per pitch of it.
#[test]
fn the_encoder_is_the_surfaces_travel_and_the_pulse_counts_its_pitches() {
    let mut project = curve_line();
    let program = &mut project.program[0].1;
    *program = program.replace(
        "    delivered AT %ID4 : DINT;",
        "    delivered AT %ID4 : DINT;\n    curve_a_pulse AT %IX0.7 : BOOL;",
    );
    let curve = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "curve_a")
        .unwrap();
    curve.signals.insert("pulse".into(), "%IX0.7".into());
    curve
        .parameters
        .insert("accel_time".into(), ParamValue::Int(400_000_000));
    let (mut line, warnings) = Line::build(&project).unwrap();
    assert!(
        warnings.iter().all(|w| !w.contains("curve_a.pulse")),
        "{warnings:?}"
    );
    let mut pulses = 0;
    for _ in 0..10_000 {
        line.step_ms().unwrap();
        pulses += usize::from(is_true(&line.controller().read("%IX0.7").unwrap()));
    }
    let travel = line.world().belt_travel(CURVE_A);
    // 10 s less the 400 ms ramp's half and the first scans before run: a little under 4.9 m.
    assert!((4.8..4.9).contains(&travel), "{travel}");
    assert_eq!(real(&line, "%ID12"), travel as f32);
    assert_eq!(real(&line, "%ID8"), 30.0);
    assert_eq!(
        pulses,
        (travel / 0.0254).floor() as usize,
        "one pulse per inch of {travel} m"
    );
}

/// The velocity follows the ramp: half way up it reads 15.
#[test]
fn the_velocity_follows_the_ramp() {
    let mut project = curve_line();
    let curve = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "curve_a")
        .unwrap();
    curve
        .parameters
        .insert("accel_time".into(), ParamValue::Int(1_000_000_000));
    let (mut line, _) = Line::build(&project).unwrap();
    assert_eq!(real(&line, "%ID8"), 0.0);
    // The first scan, at 10 ms, commands run; the block ramps from the cycle after it, and the world runs each speed
    // the millisecond after it is set.
    for _ in 0..512 {
        line.step_ms().unwrap();
    }
    assert!(
        (real(&line, "%ID8") - 15.0).abs() < 0.1,
        "{}",
        real(&line, "%ID8")
    );
}

/// A conveyor without a behaviour block runs on the controller's command, and its native model still reports.
#[test]
fn a_conveyor_without_a_behaviour_reports_its_drive_feedback() {
    let mut project = curve_line();
    project
        .definitions
        .get_mut("twinova.belt_curve@1")
        .unwrap()
        .behaviour = None;
    let (mut line, _) = Line::build(&project).unwrap();
    for _ in 0..2000 {
        line.step_ms().unwrap();
    }
    assert_eq!(real(&line, "%ID8"), 30.0);
    assert_eq!(
        real(&line, "%ID12"),
        line.world().belt_travel(CURVE_A) as f32
    );
    assert!(real(&line, "%ID12") > 0.9);
}

/// Review focus: a bound native signal is the line's to write. Were it also among the field's feedback, the field's
/// image, which has nothing there, would overwrite it with zero every millisecond.
#[test]
fn a_bound_native_signal_is_not_the_fields() {
    let field = Field::build(&curve_line()).unwrap().unwrap();
    assert!(
        field.feedback().contains(&"%IX0.2".to_string()),
        "curve A's running"
    );
    for address in ["%ID8", "%ID12"] {
        assert!(
            !field.feedback().contains(&address.to_string()),
            "{address}"
        );
        assert!(
            !field.commands().contains(&address.to_string()),
            "{address}"
        );
    }
}
