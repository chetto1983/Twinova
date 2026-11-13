use super::*;
use trust_runtime::value::Duration;

const PROGRAM: &str = "\
PROGRAM Main
VAR
    run : BOOL := TRUE;
    count : DINT := 7;
    level : REAL := 1.5;
    delay : TIME := T#2S;
    timer : TON;
END_VAR
timer(IN := run, PT := delay);
END_PROGRAM
";

fn harness() -> TestHarness {
    let mut h = TestHarness::from_source(PROGRAM).unwrap();
    h.cycle();
    h
}

fn read_path(path: &str) -> Option<String> {
    read(
        &mut harness(),
        &compile(&harness(), &Watch::Path(Scope::Plc, path.into())),
    )
}

fn condition(text: &str) -> Option<String> {
    let mut h = harness();
    let compiled = compile(&h, &Watch::Condition(Scope::Plc, text.into()));
    read(&mut h, &compiled)
}

#[test]
fn a_path_reads_a_variable_or_a_member_as_st_writes_it() {
    assert_eq!(read_path("Main.count").as_deref(), Some("7"));
    assert_eq!(read_path("Main.run").as_deref(), Some("TRUE"));
    assert_eq!(read_path("Main.delay").as_deref(), Some("T#2s"));
    assert_eq!(read_path("Main.timer.IN").as_deref(), Some("TRUE"));
    assert_eq!(
        read_path("Main.timer").as_deref(),
        None,
        "an instance has no single value"
    );
    assert_eq!(read_path("Main.nowhere"), None);
}

#[test]
fn a_condition_reads_true_or_false_once_qualified() {
    assert_eq!(condition("Main.count MOD 7 = 0").as_deref(), Some("TRUE"));
    assert_eq!(
        condition("Main.run AND Main.count > 9").as_deref(),
        Some("FALSE")
    );
    // Unqualified, the evaluation has no program scope (measured 2026-11-13).
    assert_eq!(condition("count > 1"), None);
    assert_eq!(
        condition("Main.count >"),
        None,
        "a condition that does not parse shows nothing"
    );
}

fn value(text: &str, current: Value) -> Result<Value, String> {
    value_for(&mut harness(), text, &current)
}

#[test]
fn a_text_becomes_a_value_of_the_variable_s_type() {
    assert_eq!(value("FALSE", Value::Bool(true)), Ok(Value::Bool(false)));
    assert_eq!(value(" 42 ", Value::DInt(0)), Ok(Value::DInt(42)));
    assert_eq!(value("16#FF", Value::Int(0)), Ok(Value::Int(255)));
    assert_eq!(value("5", Value::Real(0.0)), Ok(Value::Real(5.0)));
    assert!(matches!(
        value("T#250MS", Value::Time(Duration::from_millis(0))),
        Ok(Value::Time(_))
    ));
}

#[test]
fn a_text_of_another_type_is_refused_with_the_type_wanted() {
    assert_eq!(
        value("abc", Value::DInt(0)),
        Err("'abc' is not DINT".into())
    );
    assert_eq!(
        value("T#2S", Value::Bool(false)),
        Err("'T#2S' is not BOOL".into())
    );
    assert_eq!(
        value("5", Value::Time(Duration::from_millis(0))),
        Err("'5' is not a value of its type".into())
    );
}

#[test]
fn an_instance_lists_its_members_and_a_variable_none() {
    let members = super::members(&harness(), "Main.timer");
    for name in ["IN", "PT", "Q", "ET"] {
        assert!(members.iter().any(|m| m == name), "{members:?}");
    }
    assert!(super::members(&harness(), "Main.count").is_empty());
    assert!(super::members(&harness(), "Main.nowhere").is_empty());
}

/// Each kind names its type in the refusal, as a typed literal would (`DINT#5`).
#[test]
fn a_refusal_names_each_kind_s_type() {
    let kinds = [
        (Value::Bool(false), "BOOL"),
        (Value::SInt(0), "SINT"),
        (Value::Int(0), "INT"),
        (Value::DInt(0), "DINT"),
        (Value::LInt(0), "LINT"),
        (Value::USInt(0), "USINT"),
        (Value::UInt(0), "UINT"),
        (Value::UDInt(0), "UDINT"),
        (Value::ULInt(0), "ULINT"),
        (Value::Real(0.0), "REAL"),
        (Value::LReal(0.0), "LREAL"),
        (Value::Byte(0), "BYTE"),
        (Value::Word(0), "WORD"),
        (Value::DWord(0), "DWORD"),
        (Value::LWord(0), "LWORD"),
    ];
    for (current, name) in kinds {
        assert_eq!(value("abc", current), Err(format!("'abc' is not {name}")));
    }
}
