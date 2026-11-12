use super::*;
use std::path::Path;
use twinova_project::storage::open_folder;

fn demo() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    open_folder(&dir).unwrap().opened.project
}

fn with_program(text: &str) -> Project {
    let mut project = demo();
    project.program = vec![("plc/Main.st".into(), text.into())];
    project
}

const BELT: &str = "PROGRAM Main\nVAR\n  sensor AT %IX0.0 : BOOL;\n  motor AT %QX0.0 : BOOL;\nEND_VAR\n\
                    motor := sensor;\nEND_PROGRAM\n";

#[test]
fn the_demo_program_builds_and_warns_about_what_it_leaves_undeclared() {
    let (controller, warnings) = Controller::build(&demo()).unwrap();
    assert_eq!(
        warnings,
        ["sink.count at %ID4: the program does not declare it"]
    );
    assert_eq!(controller.scan_ns(), DEFAULT_SCAN_NS);
    assert!(controller.declares("%IX1.0"));
    assert!(!controller.declares("%ID4"));
    assert!(!controller.declares("not an address"));
}

#[test]
fn a_scan_reads_its_inputs_and_writes_its_outputs_by_address() {
    let (mut c, _) = Controller::build(&with_program(BELT)).unwrap();
    c.write("%IX0.0", Value::Bool(true)).unwrap();
    assert_eq!(
        c.read("%QX0.0").unwrap(),
        Value::Bool(false),
        "not before the scan"
    );
    c.scan().unwrap();
    assert_eq!(c.read("%QX0.0").unwrap(), Value::Bool(true));
    assert_eq!(c.read("%IX0.0").unwrap(), Value::Bool(true));
    assert!(
        c.write("%IX9.0", Value::Bool(true))
            .unwrap_err()
            .starts_with("%IX9.0: ")
    );
    assert_eq!(
        c.read("%QX99.0").unwrap(),
        Value::Bool(false),
        "outside the image reads as zero"
    );
    assert!(c.read("QX0.0").unwrap_err().starts_with("QX0.0: "));
}

#[test]
fn execution_timing_counts_real_scans_and_resets_on_program_change() {
    let project = with_program(BELT);
    let (mut controller, _) = Controller::build(&project).unwrap();
    assert_eq!(controller.timing().scans, 0);
    assert_eq!(controller.timing().interval_ms, 10.0);
    for _ in 0..3 {
        controller.scan().unwrap();
    }
    let timing = controller.timing();
    assert_eq!(timing.scans, 3);
    assert!(timing.maximum_us >= timing.last_us);
    assert!(timing.maximum_us >= timing.average_us);
    controller.reload(&project).unwrap();
    assert_eq!(controller.timing().scans, 0);
}

#[test]
fn a_panel_press_holds_its_bit_for_one_scan() {
    let (mut c, _) = Controller::build(&demo()).unwrap();
    c.press("%IX1.0");
    c.press("%IX7.7");
    c.scan().unwrap();
    assert_eq!(
        c.read("%QX0.0").unwrap(),
        Value::Bool(true),
        "Start latched the motor"
    );
    assert_eq!(
        c.read("%IX1.0").unwrap(),
        Value::Bool(false),
        "released after the scan"
    );
    assert_eq!(c.variable("jam"), Some(Value::Bool(false)));
    c.press("%IX1.1");
    c.scan().unwrap();
    assert_eq!(c.read("%QX0.0").unwrap(), Value::Bool(false), "Stop");
}

#[test]
fn the_program_variable_at_a_scene_signal_must_have_its_type() {
    let program = |decls: &str| {
        with_program(&format!(
            "PROGRAM Main\nVAR\n{decls}  start AT %IX1.0 : BOOL;\n  stop AT %IX1.1 : BOOL;\nEND_VAR\nEND_PROGRAM\n"
        ))
    };
    let error = Controller::build(&program(
        "  sensor AT %IX0.0 : INT;\n  count AT %ID4 : REAL;\n  motor AT %QX0.0 : BOOL;\n",
    ))
    .err()
    .unwrap();
    assert_eq!(
        error,
        "sensor_a.detected at %IX0.0 is BOOL, but the program declares sensor : INT there\n\
         sink.count at %ID4 is DINT, but the program declares count : REAL there"
    );
    let error = Controller::build(&with_program(
        "PROGRAM Main\nVAR\n  b AT %IX1.0 : DINT;\nEND_VAR\nEND_PROGRAM\n",
    ))
    .err()
    .unwrap();
    assert!(
        error.contains(
            "the button 'Start' at %IX1.0 is BOOL, but the program declares b : DINT there"
        ),
        "{error}"
    );
    let (_, warnings) = Controller::build(&program(
        "  motor AT %QX0.0 : BOOL;\n  sensor AT %IX0.0 : BOOL;\n  running AT %IX0.1 : BOOL;\n  count AT %ID4 : DINT;\n",
    ))
    .unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn a_bit_is_declared_only_at_its_own_bit() {
    let (c, warnings) = Controller::build(&with_program(BELT)).unwrap();
    assert!(c.declares("%IX0.0"));
    assert!(!c.declares("%IX0.1"));
    assert!(!c.declares("%QX0.1"));
    assert!(
        c.declares("%IB0"),
        "a byte address starting at a declared bit's byte"
    );
    assert!(!c.declares("%QB1"));
    assert!(
        warnings.contains(&"belt_a.running at %IX0.1: the program does not declare it".to_string())
    );
}

#[test]
fn the_scan_period_comes_from_the_program_tasks() {
    let tasks = |attrs: &[&str]| {
        let declarations: String = attrs
            .iter()
            .enumerate()
            .map(|(i, a)| format!("TASK T{i}({a});\n"))
            .collect();
        with_program(&format!(
            "{BELT}CONFIGURATION Cell\nRESOURCE CPU ON PLC\n{declarations}PROGRAM P WITH T0 : Main;\nEND_RESOURCE\nEND_CONFIGURATION\n"
        ))
    };
    let period = |attrs: &[&str]| Controller::build(&tasks(attrs)).map(|(c, _)| c.scan_ns());
    assert_eq!(period(&["INTERVAL := T#5ms, PRIORITY := 1"]), Ok(5_000_000));
    assert_eq!(
        period(&[
            "INTERVAL := T#6ms, PRIORITY := 1",
            "INTERVAL := T#4ms, PRIORITY := 2"
        ]),
        Ok(2_000_000)
    );
    assert_eq!(
        period(&["PRIORITY := 1"]),
        Ok(DEFAULT_SCAN_NS),
        "a task without an interval leaves the default"
    );
    let error = period(&["SINGLE := start, PRIORITY := 1"]).unwrap_err();
    assert!(error.contains("SINGLE"), "{error}");
}

#[test]
fn a_runtime_fault_is_an_error_of_the_scan() {
    let (mut c, _) = Controller::build(&with_program(
        "PROGRAM Main\nVAR\n  x AT %IX0.0 : BOOL;\n  n : DINT;\n  z : DINT;\nEND_VAR\nn := 1 / z;\nEND_PROGRAM\n",
    ))
    .unwrap();
    assert!(!c.scan().unwrap_err().is_empty());
}

#[test]
fn a_syntax_error_is_refused() {
    assert!(Controller::build(&with_program("PROGRAM Main\nx := ;\nEND_PROGRAM\n")).is_err());
}

/// A compile error names the program file and line it is in, as the field's name a behaviour file's: truST locates
/// it in bytes of its own source, and labels it with the source's path once asked to.
#[test]
fn a_compile_error_names_its_file_and_line() {
    let error = |files: &[(&str, &str)]| {
        let mut project = demo();
        project.program = (files.iter())
            .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
            .collect();
        Controller::build(&project).err().expect("a compile error")
    };
    assert_eq!(
        error(&[(
            "plc/Main.st",
            "PROGRAM Main\nVAR x : ; END_VAR\nEND_PROGRAM\n"
        )]),
        "expected type at plc/Main.st:2"
    );
    let two = error(&[
        ("plc/Types.st", "TYPE Speed : REAL; END_TYPE\n"),
        (
            "plc/Main.st",
            "PROGRAM Main\nVAR\n  y : BOOL;\n  x : ;\nEND_VAR\nEND_PROGRAM\n",
        ),
    ]);
    assert_eq!(two, "expected type at plc/Main.st:4");
}

/// What the line writes every millisecond it resolves once at Build (spec
/// 2026-11-10-solver-substeps-io-links-design.md, decision 2): an address the program declares, parsed; reads and
/// writes through it are the string's.
#[test]
fn a_declared_address_resolves_once_and_reads_and_writes_as_its_text() {
    let (mut c, _) = Controller::build(&with_program(BELT)).unwrap();
    assert_eq!(
        c.declared_at("%ID4"),
        None,
        "the program does not declare it"
    );
    assert_eq!(c.declared_at("not an address"), None);
    let sensor = c.declared_at("%IX0.0").expect("declared");
    let motor = c.declared_at("%QX0.0").expect("declared");
    c.write_at(&sensor, Value::Bool(true)).unwrap();
    assert_eq!(c.read("%IX0.0").unwrap(), Value::Bool(true));
    c.scan().unwrap();
    assert_eq!(c.read_at(&motor).unwrap(), Value::Bool(true));
    assert_eq!(c.read_at(&motor).unwrap(), c.read("%QX0.0").unwrap());
    let wrong = c.write_at(&sensor, Value::DWord(1)).unwrap_err();
    assert!(wrong.starts_with("%IX0.0: "), "{wrong}");
}
