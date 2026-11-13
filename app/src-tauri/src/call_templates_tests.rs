//! Call templates and Auto Declare's proposal (spec 2026-11-13-completion-navigation-design.md, decisions 3 and 4).
use super::*;

const PROGRAM: &str = "PROGRAM Main
VAR
    carry : TON;
    count : DINT;
    ok : BOOL;
END_VAR
@
END_PROGRAM
";
const SCALE: &str = "FUNCTION Scale : REAL
VAR_INPUT
    x : REAL;
    k : REAL;
END_VAR
VAR_IN_OUT
    acc : REAL;
END_VAR
VAR_OUTPUT
    clipped : BOOL;
END_VAR
Scale := x * k;
END_FUNCTION
";

/// The program with `line` in place of `@`, and Monaco's place of the `@`-th character's end in `cursor`'s mark `|`.
fn at(line: &str) -> (Vec<(String, String)>, u32, u32) {
    let column = line.find('|').expect("a cursor") as u32 + 1;
    let text = PROGRAM.replace('@', &line.replace('|', ""));
    let row = PROGRAM.lines().position(|l| l == "@").unwrap() as u32 + 1;
    let files = vec![
        ("plc/Main.st".to_string(), text),
        ("plc/Scale.st".to_string(), SCALE.to_string()),
    ];
    (files, row, column)
}

fn template(line: &str) -> Option<Template> {
    let (files, row, column) = at(line);
    call_template(files, "plc/Main.st".into(), row, column).unwrap()
}

#[test]
fn an_fb_call_takes_its_parameters_in_declaration_order_and_has_none_mandatory() {
    assert_eq!(
        template("carry(|"),
        Some(Template {
            all: "IN := ${1}, PT := ${2}, Q => ${3}, ET => ${4}".into(),
            required: String::new(),
        })
    );
}

#[test]
fn a_function_s_inputs_and_in_outs_are_mandatory_its_outputs_not() {
    assert_eq!(
        template("ok := Scale(|"),
        Some(Template {
            all: "x := ${1}, k := ${2}, acc := ${3}, clipped => ${4}".into(),
            required: "x := ${1}, k := ${2}, acc := ${3}".into(),
        })
    );
}

#[test]
fn a_standard_function_is_a_function() {
    let limit = template("count := LIMIT(|").expect("LIMIT's signature");
    assert_eq!(limit.all, limit.required, "{limit:?}");
    assert!(limit.all.starts_with("MN := ${1}"), "{limit:?}");
}

#[test]
fn no_call_no_template() {
    assert_eq!(template("count := 1|;"), None);
}

fn proposal(line: &str) -> Option<Proposal> {
    let (files, row, column) = at(line);
    declare_proposal(files, "plc/Main.st".into(), row, column).unwrap()
}

fn proposed(name: &str, type_name: Option<&str>) -> Option<Proposal> {
    Some(Proposal {
        name: name.into(),
        pou: "Main".into(),
        type_name: type_name.map(String::from),
    })
}

#[test]
fn a_literal_proposes_its_own_type() {
    assert_eq!(proposal("|speed := 3;"), proposed("speed", Some("INT")));
    assert_eq!(proposal("|flag := TRUE;"), proposed("flag", Some("BOOL")));
    assert_eq!(proposal("|delay := T#1S;"), proposed("delay", Some("TIME")));
    assert_eq!(proposal("|gain := 1.5;"), proposed("gain", Some("REAL")));
    assert_eq!(proposal("|big := DINT#7;"), proposed("big", Some("DINT")));
    assert_eq!(proposal("|mask := 16#FF;"), proposed("mask", Some("INT")));
}

#[test]
fn an_identifier_proposes_its_declared_type_and_anything_else_none() {
    assert_eq!(proposal("|copy := count;"), proposed("copy", Some("DINT")));
    assert_eq!(proposal("|next := count + 1;"), proposed("next", None));
    assert_eq!(proposal("ok := |unknown;"), proposed("unknown", None));
    assert_eq!(proposal("ok := TRUE;|"), None);
}
