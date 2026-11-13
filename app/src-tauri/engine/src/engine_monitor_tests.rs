//! Online monitoring on the 4.4a line (spec 2026-11-13-online-monitoring-design.md, acceptance 1 to 4).
use super::*;
use std::path::Path;
use twinova_project::storage::open_folder;

fn diverter() -> Engine {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    Engine::new(&open_folder(&dir).unwrap().opened.project)
        .unwrap()
        .0
}

fn run_ms(engine: &mut Engine, ms: i64) {
    let end = engine.time_ms() + ms;
    while engine.time_ms() < end {
        engine.step().unwrap();
    }
}

fn var(engine: &Engine, name: &str) -> String {
    let value = engine.line.controller().variable(name).unwrap();
    trust_runtime::value::format_user_value(&value)
}

fn main_var(path: &str) -> Target {
    Target::variable("Main", path)
}

#[test]
fn a_forced_variable_holds_against_the_program_until_released() {
    let mut engine = diverter();
    run_ms(&mut engine, 1_000);
    assert!(
        !engine.debugged(),
        "nothing attached before the first force"
    );
    engine.force(main_var("belt_b_run"), "FALSE").unwrap();
    assert!(engine.debugged());
    for _ in 0..1_000 {
        engine.step().unwrap();
        assert_eq!(var(&engine, "belt_b_run"), "FALSE");
    }
    assert_eq!(
        engine.forces(),
        [(main_var("belt_b_run"), "FALSE".to_string())]
    );
    engine.release(main_var("belt_b_run")).unwrap();
    engine.step().unwrap();
    assert_eq!(
        var(&engine, "belt_b_run"),
        "TRUE",
        "the program writes it again"
    );
    assert!(engine.forces().is_empty());
}

/// A forced input holds against the plant, and after release reads the plant's value: the line writes every input
/// every millisecond, so nothing needs writing again (the spec's claim, measured here).
#[test]
fn a_forced_input_holds_against_the_plant_and_the_plant_comes_back() {
    let mut engine = diverter();
    run_ms(&mut engine, 1_000);
    assert_eq!(
        var(&engine, "detected_a"),
        "FALSE",
        "no carton at the transfer yet"
    );
    engine.force(Target::address("%IX0.0"), "TRUE").unwrap();
    run_ms(&mut engine, 500);
    assert_eq!(var(&engine, "detected_a"), "TRUE");
    assert_eq!(engine.forces()[0].1, "TRUE");
    engine.release(Target::address("%IX0.0")).unwrap();
    engine.step().unwrap();
    assert_eq!(var(&engine, "detected_a"), "FALSE");
}

#[test]
fn a_write_lands_once_and_the_program_goes_on_from_it() {
    let mut engine = diverter();
    run_ms(&mut engine, 1_000);
    engine.write(main_var("count"), "100").unwrap();
    engine.step().unwrap();
    assert_eq!(var(&engine, "count"), "100");
    assert!(engine.forces().is_empty(), "a write is not a force");
    run_ms(&mut engine, 8_000);
    assert_eq!(
        var(&engine, "count"),
        "102",
        "two more cartons counted from it"
    );
}

/// truST discards a force on an instance's member without a word (measured 2026-11-13: `carry.PT`, `arrived.Q`), so
/// Twinova refuses it rather than list a force that does nothing.
#[test]
fn a_member_of_an_instance_is_refused() {
    let mut engine = diverter();
    let member = "Main.carry.PT is a member of an instance: only a program's own variables can be written or forced";
    assert_eq!(
        engine.force(main_var("carry.PT"), "T#100MS").unwrap_err(),
        member
    );
    assert_eq!(
        engine.write(main_var("carry.PT"), "T#100MS").unwrap_err(),
        member
    );
    assert!(engine.forces().is_empty());
}

#[test]
fn what_cannot_be_written_or_forced_is_refused_with_a_reason() {
    let mut engine = diverter();
    let refused = |r: Result<(), String>| r.unwrap_err();
    assert_eq!(
        refused(engine.force(main_var("nowhere"), "1")),
        "Main.nowhere is not a variable of the PLC"
    );
    assert_eq!(
        refused(engine.force(main_var("carry"), "1")),
        "Main.carry has no single value to write"
    );
    assert_eq!(
        refused(engine.force(Target::variable("Other", "x"), "1")),
        "Other.x is not a variable of the PLC"
    );
    assert_eq!(
        refused(engine.force(main_var("count"), "abc")),
        "'abc' is not DINT"
    );
    assert_eq!(
        refused(engine.write(Target::address("%QX0.1"), "FALSE")),
        "%QX0.1 is an address: force it instead"
    );
    assert_eq!(
        refused(engine.release(main_var("count"))),
        "Main.count is not forced"
    );
    assert!(
        engine.forces().is_empty() && engine.save().inputs.is_empty(),
        "nothing refused is logged"
    );
}

#[test]
fn the_watched_values_equal_the_program_s() {
    let mut engine = diverter();
    engine.watch(&[
        Watch::Path(Scope::Plc, "Main.count".into()),
        Watch::Condition(Scope::Plc, "Main.count MOD 3 = 0".into()),
        Watch::Path(Scope::Plc, "Main.carry.Q".into()),
        Watch::Path(Scope::Field, "F.chain_transfer.travel".into()),
        Watch::Path(Scope::Plc, "Main.nowhere".into()),
        Watch::Condition(Scope::Plc, "Main.count >".into()),
    ]);
    run_ms(&mut engine, 12_200);
    let count = var(&engine, "count");
    let third = (count.parse::<i64>().unwrap() % 3 == 0)
        .to_string()
        .to_uppercase();
    let values = engine.watched();
    assert_eq!(
        values[..3],
        [Some(count), Some(third), Some(var(&engine, "carry.Q"))]
    );
    let travel = values[3].as_deref().unwrap().parse::<i64>().unwrap();
    assert!(
        (1..=150).contains(&travel),
        "the chains are rising or up: {travel}"
    );
    assert_eq!(values[4..], [None, None]);
    assert!(!engine.debugged(), "monitoring alone attaches nothing");
}

/// Monitoring changes nothing: a run read at every scan has the fingerprint of a run never read.
#[test]
fn monitoring_leaves_the_line_s_trajectory() {
    let mut plain = diverter();
    let mut watched = diverter();
    watched.watch(&[Watch::Condition(
        Scope::Plc,
        "Main.popup AND Main.raised_fb".into(),
    )]);
    for _ in 0..2_000 {
        plain.step().unwrap();
        watched.step().unwrap();
        watched.watched();
    }
    assert_eq!(plain.fingerprint(), watched.fingerprint());
    assert_eq!(plain.snapshot(), watched.snapshot());
}

/// Acceptance 4: writes, forces and releases are inputs, so a restored state replays them snapshot by snapshot.
#[test]
fn a_restored_state_replays_the_forces() {
    let mut engine = diverter();
    run_ms(&mut engine, 2_000);
    engine.force(main_var("belt_b_run"), "FALSE").unwrap();
    run_ms(&mut engine, 3_000);
    engine.write(main_var("count"), "1").unwrap();
    engine.force(Target::address("%IX0.6"), "TRUE").unwrap();
    run_ms(&mut engine, 1_000);
    engine.release(main_var("belt_b_run")).unwrap();
    run_ms(&mut engine, 1_000);
    let saved = engine.save();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    let mut restored = Engine::restore(&open_folder(&dir).unwrap().opened.project, &saved).unwrap();
    assert_eq!(restored.forces(), engine.forces());
    for _ in 0..500 {
        engine.step().unwrap();
        restored.step().unwrap();
        assert_eq!(restored.snapshot(), engine.snapshot());
    }
    assert_eq!(restored.fingerprint(), engine.fingerprint());
}
