use super::*;
use serde_json::json;

/// The tree as the webview receives it, and back: the contract of `src/lib/ladder` (Task 6).
#[test]
fn a_file_reads_into_the_tree_the_webview_holds_and_prints_back() {
    let text = "VAR\n    A : BOOL;\nEND_VAR\n\n(* start *)\nGo: [|A|, |/B|] T1(IN, PT := T#1s) (S Q) >>Go2;\nGo2: RETURN;\n";
    let tree = ladder_parse(text.into()).unwrap();
    assert_eq!(
        serde_json::to_value(&tree).unwrap(),
        json!({
            "declarations": "VAR\n    A : BOOL;\nEND_VAR",
            "rungs": [
                {
                    "comment": "start",
                    "label": "Go",
                    "elements": [
                        {"kind": "branch", "paths": [
                            [{"kind": "contact", "mode": "open", "operand": "A"}],
                            [{"kind": "contact", "mode": "closed", "operand": "B"}]
                        ]},
                        {"kind": "box", "head": "T1", "pins": [
                            {"name": "IN", "value": null, "out": false},
                            {"name": "PT", "value": "T#1s", "out": false}
                        ]},
                        {"kind": "coil", "mode": "set", "operand": "Q"}
                    ],
                    "exit": {"kind": "jump", "label": "Go2"}
                },
                {"comment": null, "label": "Go2", "elements": [], "exit": {"kind": "return"}}
            ]
        })
    );
    let back: Program = serde_json::from_value(serde_json::to_value(&tree).unwrap()).unwrap();
    assert_eq!(ladder_print(back), text);
    assert_eq!(
        serde_json::to_value(ladder_parse("|A|".into()).unwrap_err()).unwrap(),
        json!({"message": "expected ';' to end the rung", "offset": 3})
    );
}

#[test]
fn a_file_lowers_to_the_power_of_each_element_and_its_rungs_that_do_not() {
    let program = vec![
        (
            "plc/Main.st".to_string(),
            "PROGRAM Main\nEND_PROGRAM\n".to_string(),
        ),
        ("plc/Pump.ld".to_string(), "|A| (X);\n(?);".to_string()),
    ];
    let lowering = ladder_lowering(program.clone(), "plc/Pump.ld".into()).unwrap();
    assert_eq!(
        serde_json::to_value(&lowering).unwrap(),
        json!({
            "power": [
                {"element": {"rung": 0, "path": [0]}, "variable": "LD_P1"},
                {"element": {"rung": 0, "path": [1]}, "variable": "LD_P1"}
            ],
            "errors": [
                {"element": {"rung": 1, "path": [0]}, "message": "This coil has no variable yet: the ? on this rung"}
            ]
        })
    );
    assert_eq!(
        ladder_lowering(program, "plc/Other.ld".into()).unwrap_err(),
        "plc/Other.ld is not a Ladder file of the program"
    );
}

#[test]
fn a_ladder_file_goes_to_plcopen_and_back_through_the_commands() {
    let program = vec![(
        "plc/Pump.ld".to_string(),
        "VAR\n    A AT %IX0.0 : BOOL;\n    X : BOOL;\nEND_VAR\n\n|A| (X);\n".to_string(),
    )];
    let xml = ladder_export(
        program.clone(),
        "plc/Pump.ld".into(),
        "2026-11-14T09:00:00".into(),
    )
    .unwrap();
    assert!(
        xml.contains(r#"creationDateTime="2026-11-14T09:00:00""#),
        "{xml}"
    );
    let imported = ladder_import(xml).unwrap();
    assert_eq!(
        serde_json::to_value(&imported).unwrap(),
        json!({"files": [["plc/Pump.ld", program[0].1]], "notes": []})
    );
    assert_eq!(
        ladder_import("<x/>".into()).unwrap_err(),
        "the file is not a PLCopen project: its root is not <project>"
    );
}
