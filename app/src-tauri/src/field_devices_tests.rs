//! The stopper's, the pusher's and the turntable's blocks (spec 2026-11-11-stopper-pusher-turntable-design.md,
//! acceptance 1–3), in the sort line's field: each stroke takes its time to the millisecond with feedback at its ends
//! only, and the turntable follows OIP's slew.

use super::*;
use std::path::Path;
use twinova_project::storage::open_folder;

/// The sort line, with `parameters` and extra `signals` on its turntable.
fn sort_line(parameters: &[(&str, ParamValue)], signals: &[(&str, &str)]) -> Field {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("project/tests/fixtures/valid/sort_line");
    let mut project = open_folder(&dir).unwrap().opened.project;
    let turntable = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "turntable")
        .unwrap();
    for (name, value) in parameters {
        turntable.parameters.insert((*name).into(), *value);
    }
    for (name, address) in signals {
        turntable.signals.insert((*name).into(), (*address).into());
    }
    Field::build(&project).unwrap().unwrap()
}

fn real(field: &Field, instance: &str, name: &str) -> f32 {
    match field.native(instance, name) {
        Some(Value::Real(x)) => x,
        other => panic!("{instance}.{name}: {other:?}"),
    }
}

fn bit(field: &Field, address: &str) -> bool {
    field.feedback_value(address).unwrap() == Value::Bool(true)
}

/// A stroke of `ms` driven by the command at `command`: `position` rises one millisecond of stroke per cycle and
/// falls the same way, and each `(address, out)` of `feedback` reads true only at the end of the out stroke (`out`) or
/// of the return.
fn stroke(
    field: &mut Field,
    command: &str,
    (instance, position): (&str, &str),
    ms: usize,
    feedback: &[(&str, bool)],
) {
    for out in [true, false] {
        field.set_command(command, Value::Bool(out)).unwrap();
        for k in 1..=ms {
            field.cycle().unwrap();
            let travel = if out { k } else { ms - k };
            let expected = travel as f32 / ms as f32;
            assert!(
                (real(field, instance, position) - expected).abs() < 1e-6,
                "{instance} {out} at {k} ms"
            );
            for &(address, end) in feedback {
                assert_eq!(
                    bit(field, address),
                    k == ms && out == end,
                    "{address}, {out} at {k} ms"
                );
            }
        }
    }
}

#[test]
fn the_stopper_raises_over_its_raise_time_and_reports_the_top() {
    let mut field = sort_line(&[], &[]);
    field.cycle().unwrap();
    assert_eq!(real(&field, "stopper", "lift"), 0.0);
    assert!(!bit(&field, "%IX0.4"));
    stroke(
        &mut field,
        "%QX0.5",
        ("stopper", "lift"),
        150,
        &[("%IX0.4", true)],
    );
}

#[test]
fn the_pusher_sweeps_over_its_push_time_and_reports_both_ends() {
    let mut field = sort_line(&[], &[]);
    field.cycle().unwrap();
    assert!(bit(&field, "%IX0.7") && !bit(&field, "%IX0.6"), "home");
    stroke(
        &mut field,
        "%QX0.6",
        ("pusher", "reach"),
        250,
        &[("%IX0.6", true), ("%IX0.7", false)],
    );
}

fn target(field: &mut Field, degrees: f32) {
    field
        .set_command("%QD8", Value::DWord(degrees.to_bits()))
        .unwrap();
}

/// Cycles until the turntable is in position, at most `limit`; the cycles taken.
fn turn(field: &mut Field, limit: usize) -> usize {
    (1..=limit)
        .find(|_| {
            field.cycle().unwrap();
            bit(field, "%IX0.5")
        })
        .unwrap_or_else(|| panic!("not in position after {limit} ms"))
}

/// Spec, acceptance 3 (amended in Task 2): 90° at 20°/s in 4 983 ms, in position only once landed, the lane held at
/// rest while the deck turns and running once it is in position.
#[test]
fn the_turntable_turns_with_oips_profile_and_runs_its_lane_once_in_position() {
    let mut field = sort_line(&[("rotation_speed", ParamValue::Real(20.0))], &[]);
    field.set_command("%QX0.2", Value::Bool(true)).unwrap();
    field.cycle().unwrap();
    assert!(bit(&field, "%IX0.5"), "in position at its start angle");
    assert_eq!(real(&field, "turntable", "surface_speed"), 0.5);
    target(&mut field, 90.0);
    let mut peak: f32 = 0.0;
    let ms = (1..=6000)
        .find(|_| {
            field.cycle().unwrap();
            peak = peak.max(real(&field, "turntable", "deck_rate"));
            if !bit(&field, "%IX0.5") {
                assert_eq!(
                    real(&field, "turntable", "surface_speed"),
                    0.0,
                    "the lane rests while it turns"
                );
            }
            bit(&field, "%IX0.5")
        })
        .unwrap();
    assert_eq!(ms, 4983);
    assert!((peak - 20.0).abs() < 1e-4, "{peak}");
    assert_eq!(real(&field, "turntable", "deck_angle"), 90.0);
    assert_eq!(real(&field, "turntable", "deck_rate"), 0.0);
    assert_eq!(real(&field, "turntable", "surface_speed"), 0.5);
}

/// OIP's safe stop: with enable off the deck stops within 0.3 s, so within 20² / (2 · 20 / 0.3) = 3°, and resumes to
/// its target when enable returns.
#[test]
fn the_turntable_stops_within_three_degrees_while_disabled_and_resumes() {
    let mut field = sort_line(
        &[("rotation_speed", ParamValue::Real(20.0))],
        &[("enable", "%QX3.0")],
    );
    field.set_command("%QX3.0", Value::Bool(true)).unwrap();
    target(&mut field, 90.0);
    for _ in 0..1000 {
        field.cycle().unwrap();
    }
    let at = real(&field, "turntable", "deck_angle");
    field.set_command("%QX3.0", Value::Bool(false)).unwrap();
    for _ in 0..1000 {
        field.cycle().unwrap();
    }
    let stopped = real(&field, "turntable", "deck_angle");
    assert!(
        stopped > at && stopped - at <= 3.0 + 1e-3,
        "{at} → {stopped}"
    );
    assert_eq!(real(&field, "turntable", "deck_rate"), 0.0);
    assert!(!bit(&field, "%IX0.5"));
    field.set_command("%QX3.0", Value::Bool(true)).unwrap();
    turn(&mut field, 6000);
    assert_eq!(real(&field, "turntable", "deck_angle"), 90.0);
}

/// A new target the other way mid-turn ramps the turn through 0 at OIP's acceleration, never reversing within a step,
/// and the deck lands on it, at the rate OIP's step lands it (spec, amended in Task 2); the shortest way to −170° from
/// 170° is through 180°.
#[test]
fn a_reversal_ramps_through_zero_and_the_shortest_way_wraps() {
    let mut field = sort_line(&[("rotation_speed", ParamValue::Real(20.0))], &[]);
    target(&mut field, 90.0);
    for _ in 0..1000 {
        field.cycle().unwrap();
    }
    target(&mut field, -90.0);
    let mut rate = real(&field, "turntable", "deck_rate");
    let mut through_zero = false;
    for _ in 0..12_000 {
        field.cycle().unwrap();
        let next = real(&field, "turntable", "deck_rate");
        if bit(&field, "%IX0.5") {
            assert!(next == 0.0 && rate > -1.0, "landed from {rate}");
            break;
        }
        assert!((next - rate).abs() <= 0.04 + 1e-4, "{rate} → {next}");
        through_zero |= rate > 0.0 && next <= 0.0;
        rate = next;
    }
    assert!(through_zero);
    assert_eq!(real(&field, "turntable", "deck_angle"), -90.0);
    let mut field = sort_line(&[("angle", ParamValue::Real(170.0))], &[]);
    target(&mut field, -170.0);
    field.cycle().unwrap();
    assert!(
        real(&field, "turntable", "deck_rate") > 0.0,
        "the short way, through 180°"
    );
    turn(&mut field, 2000);
    assert_eq!(real(&field, "turntable", "deck_angle"), -170.0);
}

/// The deck starts at its angle parameter, which OIP ignores; with no PLC writing a target, it stays there.
#[test]
fn the_deck_starts_at_its_angle_parameter() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("project/tests/fixtures/valid/sort_line");
    let mut project = open_folder(&dir).unwrap().opened.project;
    let turntable = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "turntable")
        .unwrap();
    turntable
        .parameters
        .insert("angle".into(), ParamValue::Real(45.0));
    turntable.signals.remove("target_angle");
    let mut field = Field::build(&project).unwrap().unwrap();
    for _ in 0..10 {
        field.cycle().unwrap();
        assert_eq!(real(&field, "turntable", "deck_angle"), 45.0);
        assert!(bit(&field, "%IX0.5"));
    }
}

/// Build refuses a device's block without the outputs its native model poses it from.
#[test]
fn each_device_block_must_declare_what_its_native_model_reads() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("project/tests/fixtures/valid/sort_line");
    for (key, block, name, generator) in [
        (
            "twinova.stopper@1",
            "StopperBehaviour",
            "lift",
            "twinova.stopper/1",
        ),
        (
            "twinova.pusher@1",
            "PusherBehaviour",
            "reach",
            "twinova.pusher/1",
        ),
        (
            "twinova.turntable@1",
            "TurntableBehaviour",
            "deck_rate",
            "twinova.turntable/1",
        ),
    ] {
        let mut project = open_folder(&dir).unwrap().opened.project;
        let source = project.behaviours.get_mut(key).unwrap();
        *source = source.replace(name, "renamed");
        let error = Field::build(&project).err().unwrap();
        assert_eq!(
            error,
            format!(
                "components/{key}.st: {block} must declare {name} : REAL, which the native {generator} uses"
            )
        );
    }
}
