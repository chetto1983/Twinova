//! The 4.4c sort line (spec 2026-11-11-stopper-pusher-turntable-design.md, acceptance 4): the program singulates at
//! the stopper, turns every second carton to belt C on the turntable, and pushes every second straight one off belt B,
//! with no carton lost; the run modes reach the same states.

use super::*;
use crate::engine::{Engine, Snapshot};
use std::path::Path;
use twinova_project::storage::open_folder;

fn sort_line() -> Project {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/sort_line");
    open_folder(&dir).unwrap().opened.project
}

fn dint(line: &Line, name: &str) -> u64 {
    match line.controller().variable(name) {
        Some(Value::DInt(n)) => u64::try_from(n).unwrap(),
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn the_sort_line_routes_every_carton_as_its_program_says_and_loses_none() {
    let (mut line, warnings) = Line::build(&sort_line()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    for ms in 1..=60_000 {
        line.step_ms().unwrap();
        let w = line.world();
        let (b, c, reject) = (w.delivered(0), w.delivered(1), w.delivered(2));
        assert_eq!(
            w.created(),
            b + c + reject + w.cartons().len() as u64,
            "at {ms} ms"
        );
        if ms % 100 == 0 {
            for carton in w.cartons() {
                assert!(
                    carton.position[1] > 0.8,
                    "a carton off the line at {ms} ms: {carton:?}"
                );
            }
        }
    }
    let w = line.world();
    assert_eq!(
        (w.sink_id(0), w.sink_id(1), w.sink_id(2)),
        ("sink_b", "sink_c", "sink_reject")
    );
    // The program counts each carton at the turntable's photocell, turns every second one, and pushes every second
    // of those that run straight: what it routed is delivered, but for at most one carton still on its way.
    let routed = dint(&line, "cartons");
    let (turned, straight) = (routed / 2, routed - routed / 2);
    let pushed = straight / 2;
    let (b, c, reject) = (w.delivered(0), w.delivered(1), w.delivered(2));
    assert!(
        turned >= 3 && pushed >= 2,
        "every route taken more than once: {routed} cartons"
    );
    assert!(c + 1 >= turned && c <= turned, "{c} turned of {turned}");
    assert!(
        reject + 1 >= pushed && reject <= pushed,
        "{reject} pushed of {pushed}"
    );
    assert!(
        b + 1 >= straight - pushed && b <= straight - pushed,
        "{b} straight of {}",
        straight - pushed
    );
}

#[test]
fn every_device_needs_its_behaviour() {
    for (key, instance, family) in [
        ("twinova.stopper@1", "stopper", "twinova.stopper"),
        ("twinova.pusher@1", "pusher", "twinova.pusher"),
        ("twinova.turntable@1", "turntable", "twinova.turntable"),
    ] {
        let mut project = sort_line();
        project.definitions.get_mut(key).unwrap().behaviour = None;
        let error = Line::build(&project).err().unwrap();
        assert_eq!(error, format!("{instance}: {family} needs its behaviour"));
    }
}

/// The snapshot poses every moving device: the stopper's lift, the pusher's reach and the turntable's angle.
#[test]
fn the_snapshot_carries_each_devices_position() {
    let (mut engine, _) = Engine::new(&sort_line()).unwrap();
    let mut seen = std::collections::BTreeMap::<String, f32>::new();
    for _ in 0..3_000 {
        engine.step().unwrap();
        for a in engine.snapshot().actuators {
            let most = seen.entry(a.id).or_insert(0.0);
            *most = most.max(a.value);
        }
    }
    assert_eq!(
        seen.keys().collect::<Vec<_>>(),
        ["pusher", "stopper", "turntable"]
    );
    assert_eq!(seen["stopper"], 1.0, "raised");
    assert_eq!(seen["pusher"], 1.0, "out");
    assert_eq!(seen["turntable"], 90.0, "turned");
}

/// Spec, acceptance 4: Step, Run at ×1 against the wall clock and as fast as possible reach the same states.
#[test]
fn the_sort_line_runs_alike_under_step_x1_and_as_fast_as_possible() {
    let project = sort_line();
    let fresh = || Engine::new(&project).unwrap().0;
    let mut stepped = fresh();
    for _ in 0..2_000 {
        stepped.step().unwrap();
    }
    let mut paced = fresh();
    let mut wall = 0;
    while paced.time_ms() < 20_000 {
        wall += 500;
        paced.catch_up(0, wall, 1.0).unwrap();
    }
    let mut fast = fresh();
    while fast.time_ms() < 20_000 {
        fast.catch_up(0, 1_000_000, 1.0).unwrap();
    }
    let end = stepped.snapshot();
    assert_eq!(end.time_ms, 20_000);
    assert!(end.sinks.iter().any(|s| s.delivered > 0));
    assert_eq!(paced.snapshot(), end);
    assert_eq!(
        Snapshot {
            lag_ms: 0,
            ..fast.snapshot()
        },
        end
    );
}
