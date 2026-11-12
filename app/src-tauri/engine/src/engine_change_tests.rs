//! The PLC's online change (spec 2026-11-12-st-vscode-design.md, acceptance 1 to 4).
use super::*;
use std::path::Path;
use twinova_project::storage::open_folder;

const START: &str = "%IX1.0";
const MAIN: &str = "plc/Main.st";

fn fixture(name: &str) -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid")
        .join(name);
    open_folder(&dir).unwrap().opened.project
}

fn main(text: &str) -> Vec<(String, String)> {
    vec![(MAIN.into(), text.into())]
}

fn with_main(mut project: Project, text: &str) -> Project {
    project.program = main(text);
    project
}

fn steps(engine: &mut Engine, n: usize) {
    for _ in 0..n {
        engine.step().unwrap();
    }
}

fn variable(engine: &Engine, name: &str) -> Option<Value> {
    engine.line.controller().variable(name)
}

fn signal(engine: &Engine, name: &str) -> String {
    let s = engine.snapshot();
    s.signals
        .into_iter()
        .find(|x| x.name == name)
        .unwrap()
        .value
}

/// The diverter line's program, with a scan counter kept through a warm restart.
fn diverter() -> (Project, String) {
    let project = fixture("two_belts_diverter");
    let text = project.program[0].1.replace(
        "END_VAR\nbelt_b_run",
        "END_VAR\nVAR RETAIN\n    scans : DINT;\nEND_VAR\nscans := scans + 1;\nbelt_b_run",
    );
    (with_main(project, &text), text)
}

/// The scan at which the reject sink first counts a carton.
fn first_reject(engine: &mut Engine) -> usize {
    (1..=2_000)
        .find(|_| {
            engine.step().unwrap();
            engine.snapshot().sinks[1].delivered > 0
        })
        .expect("a carton reached the reject sink")
}

/// Acceptance 1: the plant, the time and the outputs are untouched by the change; the program restarts warm; the next
/// cartons follow the new rule.
#[test]
fn an_online_change_keeps_the_plant_and_restarts_the_program_warm() {
    let (project, v1) = diverter();
    let (mut engine, built_warnings) = Engine::new(&project).unwrap();
    steps(&mut engine, 800);
    let (before, fingerprint) = (engine.snapshot(), engine.fingerprint());
    assert_eq!(variable(&engine, "count"), Some(Value::DInt(1)));
    let v2 = v1.replace("count MOD 3", "count MOD 2");
    let warnings = engine.change_program(main(&v2)).unwrap();
    assert_eq!(warnings, built_warnings);
    assert_eq!(engine.snapshot(), before);
    assert_eq!(engine.fingerprint(), fingerprint);
    assert_eq!(engine.program(), main(&v2));
    assert_eq!(variable(&engine, "count"), Some(Value::DInt(0)));
    assert_eq!(variable(&engine, "scans"), Some(Value::DInt(800)));
    steps(&mut engine, 1);
    assert_eq!(variable(&engine, "scans"), Some(Value::DInt(801)));
    // Counted from 0 again, the second carton after the change is rejected under MOD 2, at about 10.5 s; without the
    // change the third carton of the run is, at about 12.5 s (measured 2026-11-12).
    let changed = 801 + first_reject(&mut engine);
    let plain = first_reject(&mut Engine::new(&project).unwrap().0);
    assert!(
        (1_000..1_100).contains(&changed),
        "changed at scan {changed}"
    );
    assert!((1_200..1_300).contains(&plain), "plain at scan {plain}");
}

/// A latched output holds through the change until the new program's first scan, then drops, as after an S7 warm
/// restart; declared RETAIN, it stays.
#[test]
fn a_latched_output_drops_at_the_first_scan_unless_retained() {
    let demo = fixture("demo_m3");
    let text = demo.program[0].1.clone();
    let retained = text.replace("    motor AT %QX0.0 : BOOL;\n", "").replace(
        "END_VAR\n\nIF start",
        "END_VAR\nVAR RETAIN\n    motor AT %QX0.0 : BOOL;\nEND_VAR\n\nIF start",
    );
    for (program, running_after) in [(&text, "FALSE"), (&retained, "TRUE")] {
        let (mut engine, _) = Engine::new(&with_main(demo.clone(), program)).unwrap();
        engine.press(START);
        steps(&mut engine, 100);
        assert_eq!(signal(&engine, "belt_a.run"), "TRUE");
        engine
            .change_program(main(&program.replace("Twinova", "Twinova's")))
            .unwrap();
        assert_eq!(signal(&engine, "belt_a.run"), "TRUE");
        steps(&mut engine, 1);
        assert_eq!(signal(&engine, "belt_a.run"), running_after);
    }
}

/// Acceptance 2: a snapshot taken after changes replays them, at their times and in their order, to the same states.
/// The built program trips the jam before the changes, so a replay that started from the changed one would differ.
#[test]
fn a_restored_state_replays_the_changes() {
    let demo = fixture("demo_m3");
    let text = demo.program[0].1.clone();
    let quick = text.replace("PT := T#2S", "PT := T#1S");
    let slow = text.replace("PT := T#2S", "PT := T#3S");
    let script = |engine: &mut Engine, from: usize, to: usize| -> Vec<Snapshot> {
        (from..to)
            .map(|scan| {
                match scan {
                    0 => engine.press(START),
                    100 => engine.set_sink_blocked(0, true),
                    950 => {
                        engine.change_program(main(&quick)).unwrap();
                        engine.press(START);
                        engine.change_program(main(&slow)).unwrap();
                    }
                    _ => {}
                }
                engine.step().unwrap();
                engine.snapshot()
            })
            .collect()
    };
    let fresh = || Engine::new(&demo).unwrap().0;
    let uninterrupted = script(&mut fresh(), 0, 1_600);
    // With PT 2 s the jam trips at about scan 900 (`engine_tests.rs`); with PT 3 s it would not have by scan 949.
    assert_eq!(uninterrupted[949].jam, Some(true));
    let mut first = fresh();
    script(&mut first, 0, 1_000);
    // The app restores onto the project as it is now, its program the changed one: the replay starts from the built one.
    let now = with_main(demo.clone(), &slow);
    let mut restored = Engine::restore(&now, &first.save()).unwrap();
    assert_eq!(restored.program(), main(&slow));
    assert_eq!(restored.snapshot(), uninterrupted[999]);
    assert_eq!(restored.fingerprint(), first.fingerprint());
    assert_eq!(script(&mut restored, 1_000, 1_600), uninterrupted[1_000..]);
}

/// Acceptance 3: a change Build would refuse leaves the running program as it was.
#[test]
fn a_refused_change_leaves_the_running_program() {
    let (project, v1) = diverter();
    let broken = v1.replace("count := count + 1;", "count := count + ;");
    let mistyped = v1.replace("ok_count AT %ID4 : DINT;", "ok_count AT %ID4 : REAL;");
    let (mut twin, _) = Engine::new(&project).unwrap();
    let (mut engine, _) = Engine::new(&project).unwrap();
    steps(&mut twin, 300);
    steps(&mut engine, 300);
    let error = engine.change_program(main(&broken)).unwrap_err();
    assert!(error.contains("plc/Main.st:"), "{error}");
    let error = engine.change_program(main(&mistyped)).unwrap_err();
    assert!(
        error.contains("sink_ok.count at %ID4 is DINT, but the program declares ok_count : REAL"),
        "{error}"
    );
    assert_eq!(engine.program(), main(&v1));
    assert_eq!(engine.save(), twin.save());
    steps(&mut twin, 1_000);
    steps(&mut engine, 1_000);
    assert_eq!(engine.snapshot(), twin.snapshot());
}

/// Acceptance 4: the line writes an input the new program declares, and stops writing one it no longer declares.
#[test]
fn a_change_binds_the_line_to_the_new_declarations() {
    let demo = fixture("demo_m3");
    let text = demo.program[0].1.clone();
    let blind = text.replace("    sensor AT %IX0.0 : BOOL;\n", "    sensor : BOOL;\n");
    let renamed = text.replace("sensor AT %IX0.0", "seen AT %IX0.0").replace(
        "jamTimer(IN := sensor AND motor",
        "sensor := seen;\njamTimer(IN := sensor AND motor",
    );
    let renamed = renamed.replace("    jam : BOOL;\n", "    jam : BOOL;\n    sensor : BOOL;\n");
    let (mut engine, built) = Engine::new(&with_main(demo.clone(), &blind)).unwrap();
    assert!(built.contains(&"sensor_a.detected at %IX0.0: the program does not declare it".into()));
    // The first carton reaches the photocell at scan 494, about 4.9 s after Start (measured 2026-11-12).
    let seen = |engine: &mut Engine, name: &str| {
        (0..700).any(|_| {
            engine.step().unwrap();
            variable(engine, name) == Some(Value::Bool(true))
        })
    };
    let warnings = engine.change_program(main(&renamed)).unwrap();
    assert_eq!(
        warnings,
        ["sink.count at %ID4: the program does not declare it"]
    );
    // After the change: the warm restart drops the latched motor (`a_latched_output_drops_...`).
    engine.press(START);
    assert!(
        seen(&mut engine, "seen"),
        "the photocell reaches the new declaration"
    );
    let warnings = engine.change_program(main(&blind)).unwrap();
    assert_eq!(warnings, built);
    assert_eq!(signal(&engine, "sensor_a.detected"), "FALSE");
    assert!(
        !seen(&mut engine, "sensor"),
        "an undeclared photocell is not written"
    );
}

/// A sink's count the new program no longer declares reads zero, inside the new image (`%ID4`, with `%ID8` still
/// declared) and past it (`%ID8`, the image's last double word), as it would after Build.
#[test]
fn an_input_the_line_no_longer_writes_reads_zero() {
    let (project, v1) = diverter();
    let (mut engine, _) = Engine::new(&project).unwrap();
    // By scan 1,500 both sinks have counted a carton (`an_online_change_keeps_the_plant_...`, measured).
    steps(&mut engine, 1_500);
    for name in ["sink_ok.count", "sink_reject.count"] {
        assert_ne!(signal(&engine, name), "0", "{name}");
    }
    let unbound = |text: &str, from: &str, to: &str| text.replace(from, to);
    let ok_dropped = unbound(&v1, "ok_count AT %ID4 : DINT;", "ok_count : DINT;");
    engine.change_program(main(&ok_dropped)).unwrap();
    assert_eq!(signal(&engine, "sink_ok.count"), "0");
    assert_ne!(signal(&engine, "sink_reject.count"), "0");
    let both = unbound(
        &ok_dropped,
        "reject_count AT %ID8 : DINT;",
        "reject_count : DINT;",
    );
    engine.change_program(main(&both)).unwrap();
    assert_eq!(signal(&engine, "sink_reject.count"), "0");
    steps(&mut engine, 100);
    assert_eq!(signal(&engine, "sink_ok.count"), "0");
}
