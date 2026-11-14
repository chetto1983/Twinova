use super::*;
use crate::declarations::{declarations_of, edit_declaration, rename_declaration, rename_symbol};

const LD: &str = "VAR\n    Start AT %IX0.0 : BOOL;\n    T1 : TON;\nEND_VAR\nVAR RETAIN\n    count : DINT := 3; (* cartons *)\nEND_VAR\n\n|Start| T1(PT := T#1s) (S Start);\n";

fn row(name: &str, type_name: &str) -> Declaration {
    Declaration {
        pou: "Pump".into(),
        name: name.into(),
        class: "VAR".into(),
        qualifier: None,
        type_name: type_name.into(),
        address: None,
        initial: None,
        comment: None,
    }
}

#[test]
fn a_ladder_file_s_variables_are_its_declarations_rows() {
    let rows = declarations_of("plc/Pump.ld".into(), LD.into());
    let names: Vec<(&str, &str, Option<&str>)> = (rows.iter())
        .map(|r| (r.name.as_str(), r.class.as_str(), r.qualifier.as_deref()))
        .collect();
    assert_eq!(
        names,
        [
            ("Start", "VAR", None),
            ("T1", "VAR", None),
            ("count", "VAR", Some("RETAIN"))
        ]
    );
    assert_eq!(rows[0].address.as_deref(), Some("%IX0.0"));
    assert_eq!(rows[2].comment.as_deref(), Some("cartons"));
    assert_eq!(declarations_of("plc/Pump.ld".into(), "|A| (A)".into()), []);
}

#[test]
fn an_edit_of_the_table_changes_the_declarations_and_leaves_the_rungs() {
    let retyped = Change::Update {
        index: 1,
        row: row("T1", "TOF"),
    };
    let text = edit_declaration("plc/Pump.ld".into(), LD.into(), retyped).unwrap();
    assert_eq!(text, LD.replace("T1 : TON;", "T1 : TOF;"));
    // a file with no declarations gets its first VAR block before the rungs
    let added = edit_declaration(
        "plc/Pump.ld".into(),
        "|A| (A);\n".into(),
        Change::Add {
            row: row("A", "BOOL"),
        },
    )
    .unwrap();
    assert!(added.ends_with("\n|A| (A);\n"), "{added}");
    assert_eq!(
        declarations_of("plc/Pump.ld".into(), added.clone())[0].name,
        "A",
        "{added}"
    );
    assert_eq!(
        edit_declaration(
            "plc/Pump.ld".into(),
            "|A| (A)".into(),
            Change::Delete { index: 0 }
        )
        .unwrap_err(),
        "expected ';' to end the rung"
    );
}

#[test]
fn a_rename_from_a_ladder_file_is_refused_until_the_ladder_editor_renames() {
    let files = vec![("plc/Pump.ld".to_string(), LD.to_string())];
    let refusal = "plc/Pump.ld: a variable of a Ladder file is renamed in the Ladder editor, which renames its rungs too";
    let from_table = rename_declaration(files.clone(), "plc/Pump.ld".into(), 0, "Go".into());
    assert_eq!(from_table.unwrap_err(), refusal);
    assert_eq!(
        rename_symbol(files, "plc/Pump.ld".into(), 2, 5, "Go".into()).unwrap_err(),
        refusal
    );
    assert!(is_ladder("plc/Pump.ld") && !is_ladder("plc/Main.st"));
}
