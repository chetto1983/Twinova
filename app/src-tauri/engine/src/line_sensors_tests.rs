//! The 4.4d line (spec 2026-11-11-sensors-products-design.md, acceptance 7): red, green and blue cartons with codes 1,
//! 2 and 3; the colour sensor tells the turntable to turn the green ones, the barcode scanner tells the pusher to
//! reject code 3. Every carton reaches the sink of its colour, none is lost, and the run modes reach the same states.

use super::*;
use crate::engine::{Engine, Snapshot};
use std::path::Path;
use twinova_project::storage::open_folder;

fn code_sort_line() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/code_sort_line");
    open_folder(&dir).unwrap().opened.project
}

fn variable(line: &Line, name: &str) -> Value {
    line.controller()
        .variable(name)
        .unwrap_or_else(|| panic!("{name}"))
}

const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];

#[test]
fn every_carton_reaches_the_sink_of_its_colour_and_none_is_lost() {
    let (mut line, warnings) = Line::build(&code_sort_line()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let (mut measured, mut clear) = (false, false);
    for ms in 1..=90_000 {
        line.step_ms().unwrap();
        let w = line.world();
        let on_line = w.cartons().len() as u64;
        assert_eq!(
            w.created(),
            w.delivered(0) + w.delivered(1) + w.delivered(2) + on_line,
            "at {ms} ms"
        );
        match variable(&line, "distance") {
            Value::Real(d) if d < 4.0 => measured = true,
            Value::Real(_) => clear = true,
            other => panic!("distance: {other:?}"),
        }
    }
    let w = line.world();
    assert_eq!(
        (w.sink_id(0), w.sink_id(1), w.sink_id(2)),
        ("sink_b", "sink_c", "sink_reject")
    );
    // Carton n is red, green or blue as n is 0, 1 or 2 modulo 3: those of each colour are delivered to its sink, or
    // still on the line.
    let created = w.created();
    let made = |k: u64| (created + 2 - k) / 3;
    let on_line = |color: [u8; 3]| {
        let cartons = w.cartons();
        cartons.iter().filter(|c| c.color == Some(color)).count() as u64
    };
    let (b, c, reject) = (w.delivered(0), w.delivered(1), w.delivered(2));
    assert_eq!(b + on_line(RED), made(0), "red to sink B");
    assert_eq!(c + on_line(GREEN), made(1), "green turned to sink C");
    assert_eq!(reject + on_line(BLUE), made(2), "blue rejected");
    assert!(
        b >= 3 && c >= 3 && reject >= 3,
        "every route taken: {b}, {c}, {reject}"
    );
    // The scanner read each carton that reached belt B, once.
    let Value::DInt(reads) = variable(&line, "reads") else {
        panic!("reads")
    };
    assert!(reads as u64 >= b + reject, "{reads} reads");
    assert!(
        measured && clear,
        "the laser measured cartons on belt C, and read its range clear"
    );
}

/// Step, Run at ×1 against the wall clock and as fast as possible reach the same states, the products' attributes and
/// the scanner's marks included in the fingerprint.
#[test]
fn the_code_sort_line_runs_alike_under_step_x1_and_as_fast_as_possible() {
    let project = code_sort_line();
    let fresh = || Engine::new(&project).unwrap().0;
    let mut stepped = fresh();
    while stepped.time_ms() < 30_000 {
        stepped.step().unwrap();
    }
    let mut paced = fresh();
    let mut wall = 0;
    while paced.time_ms() < 30_000 {
        wall += 500;
        paced.catch_up(0, wall, 1.0).unwrap();
    }
    let mut fast = fresh();
    while fast.time_ms() < 30_000 {
        fast.catch_up(0, 1_000_000, 1.0).unwrap();
    }
    let end = stepped.snapshot();
    assert!(end.sinks.iter().all(|s| s.delivered > 0), "{:?}", end.sinks);
    assert_eq!(paced.snapshot(), end);
    assert_eq!(
        Snapshot {
            lag_ms: 0,
            ..fast.snapshot()
        },
        end
    );
    assert_eq!(paced.fingerprint(), stepped.fingerprint());
    assert_eq!(fast.fingerprint(), stepped.fingerprint());
}
