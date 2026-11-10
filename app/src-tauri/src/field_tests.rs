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

fn instance<'a>(project: &'a mut Project, id: &str) -> &'a mut Instance {
    project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == id)
        .unwrap()
}

fn bool_at(field: &Field, address: &str) -> bool {
    field.feedback_value(address).unwrap() == Value::Bool(true)
}

/// Cycles until the controller input at `address` reads true; the number of cycles, in ms.
fn ms_until(field: &mut Field, address: &str, limit: i64) -> Option<i64> {
    (1..=limit).find(|_| {
        field.cycle().unwrap();
        bool_at(field, address)
    })
}

#[test]
fn the_mirror_swaps_inputs_and_outputs() {
    assert_eq!(mirror("%QX0.0").unwrap(), "%IX0.0");
    assert_eq!(mirror("%IX1.7").unwrap(), "%QX1.7");
    assert_eq!(mirror("%ID4").unwrap(), "%QD4");
    assert!(
        mirror("%MX0.0")
            .unwrap_err()
            .contains("not an input or an output")
    );
    assert!(mirror("QX0.0").is_err());
}

#[test]
fn the_demo_field_holds_the_belt_and_its_mirrored_signals() {
    let (text, _) = generate(&fixture("demo_m3")).unwrap();
    let generated = &text[text.find("PROGRAM Field").unwrap()..];
    assert_eq!(
        generated,
        "PROGRAM Field\nVAR\n    belt_a : BeltBehaviour;\nEND_VAR\nbelt_a();\nEND_PROGRAM\n\n\
         CONFIGURATION Twinova\nRESOURCE Devices ON PLC\n    TASK Cycle (INTERVAL := T#1ms, PRIORITY := 1);\n    \
         PROGRAM F WITH Cycle : Field;\nEND_RESOURCE\nVAR_CONFIG\n    F.belt_a.run AT %IX0.0 : BOOL;\n    \
         F.belt_a.running AT %QX0.1 : BOOL;\nEND_VAR\nEND_CONFIGURATION\n"
    );
    assert!(text.starts_with("(* Twinova belt conveyor"));
}

#[test]
fn a_line_without_behaviours_has_no_field() {
    let mut project = fixture("demo_m3");
    for def in project.definitions.values_mut() {
        def.behaviour = None;
    }
    assert!(Field::build(&project).unwrap().is_none());
}

#[test]
fn the_belt_runs_at_once_and_reports_running_after_its_start_delay() {
    let mut project = fixture("demo_m3");
    instance(&mut project, "belt_a")
        .parameters
        .insert("start_delay".into(), ParamValue::Int(500_000_000));
    let mut field = Field::build(&project).unwrap().unwrap();
    assert_eq!(field.commands(), ["%QX0.0"]);
    assert_eq!(field.feedback(), ["%IX0.1"]);
    field.cycle().unwrap();
    assert_eq!(field.native("belt_a", "motor"), Some(Value::Bool(false)));
    field.set_command("%QX0.0", Value::Bool(true)).unwrap();
    field.cycle().unwrap();
    assert_eq!(field.native("belt_a", "motor"), Some(Value::Bool(true)));
    // truST's TON reaches Q one cycle before PT (spec 2026-11-09, "Measured"): 500 ms after the cycle that saw run,
    // less one.
    assert_eq!(ms_until(&mut field, "%IX0.1", 1000), Some(499));
}

#[test]
fn the_belt_without_a_delay_reports_running_in_the_cycle_that_sees_run() {
    let mut field = Field::build(&fixture("demo_m3")).unwrap().unwrap();
    field.set_command("%QX0.0", Value::Bool(true)).unwrap();
    assert_eq!(ms_until(&mut field, "%IX0.1", 10), Some(1));
}

/// The chain transfer's block (spec 2026-11-10-diverter-line-design.md): `popup` raises and carries sideways at once,
/// the lift slews over `raise_time` one millisecond per cycle, and `raised_fb` reports the top of the stroke.
#[test]
fn each_chain_transfer_lifts_over_its_own_raise_time_and_reports_the_top() {
    let mut project = fixture("two_belts_diverter");
    project.scene.instances.push(Instance {
        id: "chain_transfer_2".into(),
        name: "Second chain transfer".into(),
        definition: "twinova.chain_transfer@1".into(),
        position: [8.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        parameters: [("raise_time".to_string(), ParamValue::Int(100_000_000))].into(),
        signals: [
            ("popup".to_string(), "%QX2.0".to_string()),
            ("raised_fb".to_string(), "%IX2.0".to_string()),
        ]
        .into(),
    });
    let t = instance(&mut project, "chain_transfer").signals.clone();
    let mut field = Field::build(&project).unwrap().unwrap();
    field.cycle().unwrap();
    assert_eq!(
        field.native("chain_transfer", "lift"),
        Some(Value::Real(0.0))
    );
    assert_eq!(
        field.native("chain_transfer", "raised"),
        Some(Value::Bool(false))
    );
    assert!(!bool_at(&field, &t["raised_fb"]));
    field.set_command(&t["popup"], Value::Bool(true)).unwrap();
    field.set_command("%QX2.0", Value::Bool(true)).unwrap();
    field.cycle().unwrap();
    assert_eq!(
        field.native("chain_transfer", "raised"),
        Some(Value::Bool(true))
    );
    assert_eq!(
        field.native("chain_transfer_2", "lift"),
        Some(Value::Real(0.01))
    );
    // The first millisecond of stroke ran above, in the cycle that saw popup.
    assert_eq!(ms_until(&mut field, "%IX2.0", 1000), Some(99), "100 ms");
    assert_eq!(
        ms_until(&mut field, &t["raised_fb"], 1000),
        Some(50),
        "the default 150 ms, 100 of them already run"
    );
    assert_eq!(
        field.native("chain_transfer", "lift"),
        Some(Value::Real(1.0))
    );
    // Down: the chains stop carrying sideways at once, and the stroke takes raise_time again.
    field.set_command(&t["popup"], Value::Bool(false)).unwrap();
    field.cycle().unwrap();
    assert_eq!(
        field.native("chain_transfer", "raised"),
        Some(Value::Bool(false))
    );
    assert!(!bool_at(&field, &t["raised_fb"]));
    for _ in 1..149 {
        field.cycle().unwrap();
    }
    assert!(matches!(field.native("chain_transfer", "lift"), Some(Value::Real(l)) if l > 0.0));
    field.cycle().unwrap();
    assert_eq!(
        field.native("chain_transfer", "lift"),
        Some(Value::Real(0.0))
    );
}

fn with_belt_source(edit: impl FnOnce(&str) -> String) -> Result<Option<Field>, String> {
    let mut project = fixture("demo_m3");
    let source = project.behaviours.get_mut("twinova.belt@1").unwrap();
    *source = edit(source);
    Field::build(&project)
}

#[test]
fn a_block_without_its_native_variable_is_refused() {
    let error = with_belt_source(|s| {
        s.replace("motor : BOOL;", "engine : BOOL;")
            .replace("motor :=", "engine :=")
    })
    .err()
    .unwrap();
    assert_eq!(
        error,
        "components/twinova.belt@1.st: BeltBehaviour must declare motor : BOOL, which the native twinova.belt/1 uses"
    );
    let error = with_belt_source(|s| {
        s.replace("motor : BOOL;", "motor : DINT;")
            .replace("motor := run;", "motor := 1;")
    })
    .err()
    .unwrap();
    assert!(
        error.ends_with("which the native twinova.belt/1 uses (it is DINT)"),
        "{error}"
    );
}

#[test]
fn a_parameter_of_another_type_is_refused() {
    let error = with_belt_source(|s| {
        s.replace("start_delay : TIME;", "start_delay : DINT;")
            .replace("PT := start_delay", "PT := T#0ms")
    })
    .err()
    .unwrap();
    assert_eq!(
        error,
        "components/twinova.belt@1.st: BeltBehaviour.start_delay must be TIME like the parameter, not DINT"
    );
}

#[test]
fn a_mistake_in_a_block_is_reported_against_its_file_and_line() {
    let error = with_belt_source(|s| s.replace("motor := run;", "motor := ;"))
        .err()
        .unwrap();
    assert_eq!(
        error,
        "field runtime: expected expression at components/twinova.belt@1.st:15\n\
         expected ';' at components/twinova.belt@1.st:16"
    );
}

#[test]
fn a_location_outside_the_blocks_is_the_generated_program() {
    let spans = vec![(0, 5, "components/a.st".to_string())];
    assert_eq!(
        locate("e (at 2..3) f at 9..10 g", "ab\ncd\nefghijk", &spans),
        "e (at components/a.st:1) f at the generated field program g"
    );
    assert_eq!(
        locate("x at 4..5", "ab\ncdefg", &spans),
        "x at components/a.st:2"
    );
    for unchanged in ["no place at what", "at 12", "at 1..x", "seat ..2"] {
        assert_eq!(locate(unchanged, "", &spans), unchanged);
    }
}

#[test]
fn a_chain_transfer_without_its_lift_is_refused() {
    let mut project = fixture("two_belts_diverter");
    let source = project
        .behaviours
        .get_mut("twinova.chain_transfer@1")
        .unwrap();
    *source = source.replace("lift", "height");
    let error = Field::build(&project).err().unwrap();
    assert!(
        error.contains(
            "ChainTransferBehaviour must declare lift : REAL, which the native twinova.chain_transfer/1 uses"
        ),
        "{error}"
    );
}

#[test]
fn a_roller_without_its_motor_is_refused() {
    let mut project = fixture("two_belts_diverter");
    let source = project.behaviours.get_mut("twinova.roller@1").unwrap();
    *source = source.replace("motor", "drive");
    let error = Field::build(&project).err().unwrap();
    assert!(
        error.contains(
            "RollerBehaviour must declare motor : BOOL, which the native twinova.roller/1 uses"
        ),
        "{error}"
    );
}

#[test]
fn a_real_parameter_reaches_the_block() {
    let mut project = fixture("demo_m3");
    let source = project.behaviours.get_mut("twinova.belt@1").unwrap();
    *source = source.replace(
        "    start_delay : TIME;",
        "    start_delay : TIME;\n    speed : REAL;",
    );
    let field = Field::build(&project).unwrap().unwrap();
    assert_eq!(field.native("belt_a", "speed"), Some(Value::Real(0.5)));
}

/// The chain transfer's behaviour driven by a line program (spec 2026-11-09, acceptance 2): each millisecond the field
/// reads the controller's outputs and writes its inputs; every 10 ms the controller scans.
#[test]
fn a_line_program_commands_the_chain_transfer_and_reads_raised_fb() {
    use crate::controller::Controller;
    let mut project = fixture("two_belts_diverter");
    let t = instance(&mut project, "chain_transfer").signals.clone();
    project.program = vec![(
        "plc/Main.st".into(),
        format!(
            "PROGRAM Main\nVAR\n  popup AT {} : BOOL;\n  raised_fb AT {} : BOOL;\n  n : DINT;\nEND_VAR\n\
             n := n + 1;\npopup := n >= 5 AND n < 50;\nEND_PROGRAM\n",
            t["popup"], t["raised_fb"]
        ),
    )];
    let (mut controller, _) = Controller::build(&project).unwrap();
    let mut field = Field::build(&project).unwrap().unwrap();
    let mut changes = Vec::new();
    let mut up = false;
    for ms in 1..=1000 {
        for address in field.commands().to_vec() {
            field
                .set_command(&address, controller.read(&address).unwrap())
                .unwrap();
        }
        field.cycle().unwrap();
        for address in field.feedback().to_vec() {
            controller
                .write(&address, field.feedback_value(&address).unwrap())
                .unwrap();
        }
        if ms % 10 == 0 {
            controller.scan().unwrap();
            let now = controller.read(&t["raised_fb"]).unwrap() == Value::Bool(true);
            if now != up {
                changes.push((ms, now));
            }
            up = now;
        }
    }
    // popup rises at the 50 ms scan; the field sees it at 51 ms and tops out 150 ms of stroke later, at 200 ms, read
    // at that scan. popup falls at the 500 ms scan; raised_fb drops at 501 and is read at 510.
    assert_eq!(changes, [(200, true), (510, false)]);
}
