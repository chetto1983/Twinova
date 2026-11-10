use super::*;
use std::path::Path;
use twinova_project::model::ParamValue;
use twinova_project::storage::open_folder;

fn fixture(name: &str) -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("project/tests/fixtures/valid")
        .join(name);
    open_folder(&dir).unwrap().opened.project
}

fn demo() -> Line {
    Line::build(&fixture("demo_m3")).unwrap().0
}

fn run_ms(line: &mut Line, ms: i64) {
    for _ in 0..ms {
        line.step_ms().unwrap();
    }
}

fn bit(line: &Line, address: &str) -> bool {
    is_true(&line.controller().read(address).unwrap())
}

#[test]
fn the_demo_builds_and_warns_about_the_unused_counter() {
    let (line, warnings) = Line::build(&fixture("demo_m3")).unwrap();
    assert_eq!(
        warnings,
        ["sink.count at %ID4: the program does not declare it"]
    );
    assert_eq!(line.scan_ns(), 10_000_000);
}

#[test]
fn the_roller_conveyor_runs_on_its_behaviour_and_carries_onto_belt_b() {
    let (mut line, _) = Line::build(&fixture("two_belts_diverter")).unwrap();
    // From the entry to past belt B's start: 5 m at 0.5 m/s; before the third carton holds the infeed at 11.4 s.
    run_ms(&mut line, 11_000);
    assert!(bit(&line, "%QX0.3"), "the program's roller_run");
    // The roller's motor is its block's: a carton past the roller shows the block turned it on.
    let on_b = line
        .world()
        .cartons()
        .iter()
        .any(|c| c.position[0] > 5.0 + 0.15);
    assert!(on_b, "the first carton crossed the roller onto belt B");
}

#[test]
fn start_runs_the_belt_and_cartons_are_delivered() {
    let mut line = demo();
    run_ms(&mut line, 100);
    assert!(!bit(&line, "%QX0.0"), "the motor waits for Start");
    line.press("%IX1.0");
    run_ms(&mut line, 20_000);
    assert!(bit(&line, "%QX0.0"));
    // A carton every 2 s, each needing about (3.0 - 0.3) / 0.5 = 5.4 s to reach the sink.
    let delivered = line.world().delivered(0);
    assert!((6..=9).contains(&delivered), "{delivered}");
}

#[test]
fn stop_stops_the_belt_and_cartons_stay() {
    let mut line = demo();
    line.press("%IX1.0");
    run_ms(&mut line, 3000);
    line.press("%IX1.1");
    run_ms(&mut line, 50);
    assert!(!bit(&line, "%QX0.0"));
    let before = line.world().cartons();
    run_ms(&mut line, 2000);
    let after = line.world().cartons();
    let moved = (after[0].position[0] - before[0].position[0]).abs();
    assert!(moved < 0.01, "the first carton stays put: {moved} m");
}

#[test]
fn a_blocked_sink_covers_the_photocell_and_trips_the_jam() {
    let mut line = demo();
    line.set_sink_blocked(0, true);
    line.press("%IX1.0");
    let mut jam_at = None;
    for ms in 1..=15_000 {
        line.step_ms().unwrap();
        if jam_at.is_none() && line.controller().variable("jam") == Some(Value::Bool(true)) {
            jam_at = Some(ms);
        }
    }
    // The first carton reaches the wall at about 5.4 s and covers the photocell at 2.75 m from about 4.9 s, and keeps
    // covering it held there; the jam latches 2 s of cover with the motor on later.
    let jam_at = jam_at.expect("a jam");
    assert!((6_700..7_000).contains(&jam_at), "jam at {jam_at} ms");
    assert!(!bit(&line, "%QX0.0"), "the jam stops the motor");
    assert_eq!(line.world().delivered(0), 0);
}

/// The step order: the field writes the belt's running in the millisecond after the scan that commanded run, and the
/// controller sees it at its next scan.
#[test]
fn feedback_reaches_the_controller_at_the_scan_after_the_command() {
    let mut project = fixture("demo_m3");
    let belt = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_a")
        .unwrap();
    belt.parameters
        .insert("start_delay".into(), ParamValue::Int(0));
    let (mut line, _) = Line::build(&project).unwrap();
    line.press("%IX1.0");
    let (mut commanded, mut fed_back) = (None, None);
    for ms in 1..=100 {
        line.step_ms().unwrap();
        if commanded.is_none() && bit(&line, "%QX0.0") {
            commanded = Some(ms);
        }
        if fed_back.is_none() && bit(&line, "%IX0.1") {
            fed_back = Some(ms);
        }
    }
    // Start is scanned at 10 ms; the field sees run at 11 ms and, with no delay, writes running then.
    assert_eq!((commanded, fed_back), (Some(10), Some(11)));
}

#[test]
fn a_start_delay_holds_running_back() {
    let mut project = fixture("demo_m3");
    let belt = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_a")
        .unwrap();
    belt.parameters
        .insert("start_delay".into(), ParamValue::Int(500_000_000));
    let (mut line, _) = Line::build(&project).unwrap();
    line.press("%IX1.0");
    let first = (1..=1000).find(|_| {
        line.step_ms().unwrap();
        bit(&line, "%IX0.1")
    });
    // run at 10 ms, seen by the field at 11 ms; truST's TON reaches Q one cycle before PT (Task 2): 11 + 499.
    assert_eq!(first, Some(510));
}

#[test]
fn equal_inputs_give_equal_lines() {
    let script = || {
        let mut line = demo();
        line.press("%IX1.0");
        run_ms(&mut line, 4000);
        line.set_sink_blocked(0, true);
        run_ms(&mut line, 4000);
        line.press("%IX1.1");
        run_ms(&mut line, 1000);
        (line.world().fingerprint(), line.time_ns())
    };
    assert_eq!(script(), script());
}

#[test]
fn a_belt_without_a_behaviour_follows_the_controller_and_reports_running_itself() {
    let mut project = fixture("demo_m3");
    project
        .definitions
        .get_mut("twinova.belt@1")
        .unwrap()
        .behaviour = None;
    let (mut line, _) = Line::build(&project).unwrap();
    run_ms(&mut line, 8000);
    assert_eq!(line.world().delivered(0), 0, "the belt waits for Start");
    assert!(!bit(&line, "%IX0.1"));
    line.press("%IX1.0");
    run_ms(&mut line, 20);
    assert!(bit(&line, "%QX0.0") && bit(&line, "%IX0.1"));
    run_ms(&mut line, 8000);
    assert!(line.world().delivered(0) > 0);
}

#[test]
fn a_behaviour_drives_its_belt_motor_whatever_the_controller_commands() {
    let mut project = fixture("demo_m3");
    let block = project.behaviours.get_mut("twinova.belt@1").unwrap();
    *block = block.replace("motor := run;", "motor := FALSE;");
    let (mut line, _) = Line::build(&project).unwrap();
    line.press("%IX1.0");
    run_ms(&mut line, 8000);
    assert!(bit(&line, "%QX0.0"), "the controller commands run");
    assert_eq!(
        line.world().delivered(0),
        0,
        "the block holds its motor off"
    );
}

/// The 4.4a line (spec 2026-11-10-diverter-line-design.md): the source made `count` cartons, two seconds apart.
fn diverter_line(count: i64) -> Line {
    let mut project = fixture("two_belts_diverter");
    let source = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "source")
        .unwrap();
    source
        .parameters
        .insert("max_count".into(), ParamValue::Int(count));
    Line::build(&project).unwrap().0
}

#[test]
fn every_third_carton_is_raised_and_carried_to_the_reject_sink() {
    let mut line = diverter_line(9);
    // The ninth carton reaches the transfer at about 23.7 s and belt B's end 6 s later.
    run_ms(&mut line, 40_000);
    let w = line.world();
    assert_eq!(
        (w.delivered(0), w.delivered(1), w.cartons().len()),
        (6, 3, 0)
    );
}

#[test]
fn a_chain_transfer_needs_its_behaviour() {
    let mut project = fixture("two_belts_diverter");
    project
        .definitions
        .get_mut("twinova.chain_transfer@1")
        .unwrap()
        .behaviour = None;
    let error = Line::build(&project).err().unwrap();
    assert_eq!(
        error,
        "chain_transfer: twinova.chain_transfer needs its behaviour"
    );
}

#[test]
fn no_carton_is_lost_or_duplicated_and_a_blocked_normal_sink_holds_a_finite_queue() {
    let mut line = diverter_line(0);
    line.set_sink_blocked(0, true);
    let mut most = 0;
    // The queue backs up over the transfer within about 20 s.
    for _ in 0..60_000 {
        line.step_ms().unwrap();
        let w = line.world();
        assert_eq!(
            w.created(),
            w.delivered(0) + w.delivered(1) + w.cartons().len() as u64
        );
        most = most.max(w.cartons().len());
    }
    let w = line.world();
    assert_eq!(w.delivered(0), 0);
    assert!(w.delivered(1) > 0, "the transfer keeps diverting");
    // Belt A, the roller and belt B hold 7 m of 0.3 m cartons at most; the source waits for a free entry.
    assert!(most <= 24, "{most} cartons on the line");
}
