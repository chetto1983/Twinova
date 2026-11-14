use super::*;
use std::path::Path;
use twinova_project::Project;
use twinova_project::storage::open_folder;

use crate::controller::Controller;

fn files(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .collect()
}

fn lowered(list: &[(&str, &str)], path: &str) -> String {
    let program = ProgramFiles::of(&files(list)).unwrap();
    let (_, text) = program.files.iter().find(|(p, _)| p == path).unwrap();
    text.clone()
}

const LATCH: &str = "FUNCTION_BLOCK Latch\nVAR_INPUT Set : BOOL; Clear : BOOL; END_VAR\nVAR_OUTPUT N : INT; On : BOOL; \
                     END_VAR\nEND_FUNCTION_BLOCK\nFUNCTION Ready : BOOL\nVAR_INPUT X : INT; END_VAR\nReady := X > 0;\n\
                     END_FUNCTION\nPROGRAM Main\nEND_PROGRAM\n";

#[test]
fn st_files_compile_as_written() {
    let st = files(&[("plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n")]);
    assert_eq!(ProgramFiles::of(&st).unwrap().files, st);
}

#[test]
fn a_ladder_file_is_a_program_named_after_it_its_boxes_read_off_the_compiled_declarations() {
    let ld = "VAR A, B, X : BOOL; T1 : TON; L : Latch; N : INT; END_VAR\n\
              |A| T1(PT := T#1s) L(Clear := B) Ready(X := N, EN) ADD(IN1 := N, IN2 := 1, OUT => N) (X);\n\
              Ready(X := N, OUT) (B);";
    let st = lowered(
        &[("plc/Main.st", LATCH), ("plc/Pump.ld", ld)],
        "plc/Pump.ld",
    );
    assert!(st.starts_with("PROGRAM Pump\n"), "{st}");
    // T1 is a TON (IN on, Q out), L the project's Latch (its first BOOL input, Set, and its first BOOL output, On),
    // Ready and ADD functions
    let body = [
        "T1(IN := LD_P1, PT := T#1s);\nLD_P2 := T1.Q;",
        "L(Set := LD_P2, Clear := B);\nLD_P3 := L.On;",
        "IF LD_P3 THEN N := ADD(IN1 := N, IN2 := 1); END_IF;\nX := LD_P3;",
        "LD_P4 := Ready(X := N);\nB := LD_P4;",
    ];
    for part in body {
        assert!(st.contains(part), "{part} in {st}");
    }
}

#[test]
fn declarations_that_do_not_compile_leave_every_box_to_its_bare_pins() {
    let ld = "VAR A, X : BOOL; T1 : Missing; END_VAR\n|A| T1(Go, Done) (X);";
    let st = lowered(&[("plc/Pump.ld", ld)], "plc/Pump.ld");
    assert!(st.contains("T1(Go := LD_P1);\nLD_P2 := T1.Done;"), "{st}");
}

#[test]
fn what_does_not_read_or_lower_is_reported_at_its_file_line_or_rung() {
    let problem = |path: &str, text: &str| ProgramFiles::of(&files(&[(path, text)])).err().unwrap();
    assert_eq!(
        problem("plc/Pump.ld", "VAR A : BOOL; END_VAR\n\n|A| (A)"),
        "plc/Pump.ld:3: expected ';' to end the rung"
    );
    assert_eq!(
        problem("plc/Pump.ld", "|A| (A);\n|?| (A);\n|A| >>Nowhere;"),
        "plc/Pump.ld, rung 2: This contact has no variable yet: the ? on this rung\n\
         plc/Pump.ld, rung 3: there is no rung labelled 'Nowhere' to jump to"
    );
    assert_eq!(
        problem("plc/2nd pump.ld", "|A| (A);"),
        "plc/2nd pump.ld: a Ladder file is a PROGRAM named after the file, so its name must be an identifier"
    );
}

#[test]
fn one_ladder_file_lowers_with_its_rungs_that_do_not_lower_reported_in_it() {
    let program = files(&[
        ("plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n"),
        (
            "plc/Pump.ld",
            "VAR A, X : BOOL; T1 : TON; END_VAR\n|A| T1(PT := T#1s) (X);\n|?| (X);",
        ),
    ]);
    let lowered = ProgramFiles::ladder(&program, "plc/Pump.ld").unwrap();
    assert!(
        lowered.st.contains("T1(IN := LD_P1, PT := T#1s);"),
        "{}",
        lowered.st
    );
    assert_eq!(lowered.errors.len(), 1);
    assert_eq!(lowered.errors[0].element.rung, 1);
    assert_eq!(
        ProgramFiles::ladder(&program, "plc/Main.st").err().unwrap(),
        "plc/Main.st is not a Ladder file of the program"
    );
}

#[test]
fn a_place_reads_as_a_file_line_a_declaration_line_or_a_rung() {
    let program = ProgramFiles::of(&files(&[
        ("plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n"),
        // four empty lines, and Q at the start of its line: an offset of the declarations that missed the leading
        // text's length would land on the line above
        ("plc/Pump.ld", "\n\n\n\nVAR\nQ : BOOL;\nEND_VAR\n|Q| (Q);"),
    ]))
    .unwrap();
    let main = program.placer("plc/Main.st").unwrap();
    assert_eq!(main(13), "plc/Main.st:2");
    let pump = program.placer("plc/Pump.ld").unwrap();
    let st = &program.files[1].1;
    assert_eq!(pump(st.find("Q : BOOL").unwrap()), "plc/Pump.ld:6");
    assert_eq!(pump(st.find("Q := LD_P1").unwrap()), "plc/Pump.ld, rung 1");
    assert_eq!(pump(0), "plc/Pump.ld", "the PROGRAM's own line");
    assert!(program.placer("plc/Other.st").is_none());
}

fn demo_with(program: &[(&str, &str)]) -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    let mut project = open_folder(&dir).unwrap().opened.project;
    project.program = files(program);
    project
}

#[test]
fn truss_errors_in_a_ladder_file_point_at_its_rung_or_declaration_line() {
    let build = |ld: &str| {
        let project = demo_with(&[
            ("plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n"),
            ("plc/Pump.ld", ld),
        ]);
        Controller::build(&project).err().unwrap()
    };
    let error = build("VAR Count : INT; Q : BOOL; END_VAR\n|Q| (Q);\n|Count| (Q);");
    assert!(error.contains("at plc/Pump.ld, rung 2"), "{error}");
    let error = build("\n\nVAR\n    Q : BOOL;\n    Count : NoSuchType;\nEND_VAR\n|Q| (Q);");
    assert!(error.contains("at plc/Pump.ld:5"), "{error}");
}

#[test]
fn a_ladder_program_scans_after_main_on_the_process_image() {
    let ld = "VAR\n    sensor AT %IX0.0 : BOOL;\n    motor AT %QX0.0 : BOOL;\nEND_VAR\n|sensor| (motor);";
    let project = demo_with(&[
        ("plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n"),
        ("plc/Belt.ld", ld),
    ]);
    let (mut controller, _) = Controller::build(&project).unwrap();
    controller
        .write("%IX0.0", trust_runtime::value::Value::Bool(true))
        .unwrap();
    controller.scan().unwrap();
    assert_eq!(
        controller.read("%QX0.0").unwrap(),
        trust_runtime::value::Value::Bool(true)
    );
    let (path, text) = &controller.sources()[1];
    assert_eq!(path, "plc/Belt.ld");
    assert!(
        text.starts_with("PROGRAM Belt\n"),
        "the debugger sees what truST compiled"
    );
}

/// The 4.4a line's program as Ladder, rung for statement (`two_belts_diverter/plc/Main.st`).
pub(crate) const MAIN_4_4A: &str = "VAR
    detected_a AT %IX0.0 : BOOL;
    detected_b AT %IX0.1 : BOOL;
    detected_in AT %IX0.6 : BOOL;
    raised_fb AT %IX0.4 : BOOL;
    belt_a_run AT %QX0.0 : BOOL;
    belt_b_run AT %QX0.1 : BOOL;
    popup AT %QX0.2 : BOOL;
    roller_run AT %QX0.3 : BOOL;
    ok_count AT %ID4 : DINT;
    reject_count AT %ID8 : DINT;
    entered : R_TRIG;
    released : F_TRIG;
    arrived : R_TRIG;
    deck_clear : TON;
    carry : TON;
    count : DINT;
    reject_next : BOOL;
    reject_go : BOOL;
    hold : BOOL;
END_VAR
VAR RETAIN
    in_zone : DINT;
END_VAR

(belt_b_run);
|detected_in| entered(CLK);
|entered.Q| ADD(IN1 := count, IN2 := 1, OUT => count) ADD(IN1 := in_zone, IN2 := 1, OUT => in_zone) \
MOVE(IN := count MOD 3 = 0, OUT => reject_next);
|detected_in| released(CLK);
|released.Q| |reject_next| (R reject_next) (S reject_go);
|detected_a| arrived(CLK);
|arrived.Q| SUB(IN1 := in_zone, IN2 := 1, OUT => in_zone) |reject_go| (S popup) (R reject_go);
|/detected_a| deck_clear(IN, PT := T#800MS);
[|/reject_next|, EQ(IN1 := in_zone, IN2 := 1, OUT) |deck_clear.Q|] (/hold);
|popup| |raised_fb| carry(IN, PT := T#1500MS);
|carry.Q| (R popup);
|/popup| |/hold| (belt_a_run);
|/popup| (roller_run);
";

/// The 4.4a line's project, its program as the fixture writes it.
pub(crate) fn diverter() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    open_folder(&dir).unwrap().opened.project
}

/// `project` run 15 s through Build (the format writes its files and reads them back): the line's fingerprint every
/// second, and where it ends.
pub(crate) fn fifteen_seconds(project: &Project) -> (Vec<u64>, crate::engine::Snapshot) {
    let (_, mut engine, _) = crate::build(project).unwrap();
    let mut marks = Vec::new();
    for step in 1..=1_500 {
        engine.step().unwrap();
        if step % 100 == 0 {
            marks.push(engine.fingerprint());
        }
    }
    (marks, engine.snapshot())
}

/// The plan's Global Constraints: the Ladder program and its ST twin give the same fingerprint on the 4.4a line.
#[test]
fn the_4_4a_line_runs_alike_from_its_st_program_and_from_the_same_program_in_ladder() {
    let st = diverter();
    let mut ld = st.clone();
    ld.program = files(&[("plc/Main.ld", MAIN_4_4A)]);
    let (st_marks, st_end) = fifteen_seconds(&st);
    let (ld_marks, ld_end) = fifteen_seconds(&ld);
    assert_eq!(
        st_end.sinks[1].delivered, 1,
        "the third carton went to reject"
    );
    assert_eq!(ld_marks, st_marks);
    assert_eq!(ld_end, st_end);
}
