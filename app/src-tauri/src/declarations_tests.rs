use super::*;

const SAMPLE: &str = "\
PROGRAM Main
VAR
    eye AT %IX0.0 : BOOL; (* the photocell *)
    count : DINT := 3; // cartons
    timer : TON;
END_VAR
VAR RETAIN
    total : LINT;
END_VAR
count := count + 1;
END_PROGRAM

FUNCTION_BLOCK Lift
VAR_INPUT
    up : BOOL;
END_VAR
VAR_OUTPUT
    level : REAL;
END_VAR
level := 1.0;
END_FUNCTION_BLOCK
";

fn row(pou: &str, name: &str, class: &str, type_name: &str) -> Declaration {
    Declaration {
        pou: pou.into(),
        name: name.into(),
        class: class.into(),
        qualifier: None,
        type_name: type_name.into(),
        address: None,
        initial: None,
        comment: None,
    }
}

#[test]
fn every_declaration_is_read_with_its_class_address_value_and_comment() {
    let rows = declarations(SAMPLE);
    let eye = Declaration {
        address: Some("%IX0.0".into()),
        comment: Some("the photocell".into()),
        ..row("Main", "eye", "VAR", "BOOL")
    };
    let count = Declaration {
        initial: Some("3".into()),
        comment: Some("cartons".into()),
        ..row("Main", "count", "VAR", "DINT")
    };
    let total = Declaration {
        qualifier: Some("RETAIN".into()),
        ..row("Main", "total", "VAR", "LINT")
    };
    assert_eq!(
        rows,
        [
            eye,
            count,
            row("Main", "timer", "VAR", "TON"),
            total,
            row("Lift", "up", "VAR_INPUT", "BOOL"),
            row("Lift", "level", "VAR_OUTPUT", "REAL"),
        ]
    );
}

#[test]
fn an_update_writes_the_declaration_s_line_again_and_nothing_else() {
    let mut count = declarations(SAMPLE)[1].clone();
    count.type_name = "INT".into();
    count.initial = None;
    count.comment = Some("per hour".into());
    let edited = edit(
        SAMPLE,
        &Change::Update {
            index: 1,
            row: count,
        },
    )
    .unwrap();
    assert_eq!(
        edited,
        SAMPLE.replace(
            "count : DINT := 3; // cartons",
            "count : INT; (* per hour *)"
        )
    );
}

#[test]
fn a_new_row_goes_before_its_block_s_end_with_its_indentation() {
    let edited = edit(
        SAMPLE,
        &Change::Add {
            row: row("Main", "speed", "VAR", "REAL"),
        },
    )
    .unwrap();
    assert_eq!(
        edited,
        SAMPLE.replace(
            "    timer : TON;\n",
            "    timer : TON;\n    speed : REAL;\n"
        )
    );
    let retained = Declaration {
        qualifier: Some("RETAIN".into()),
        initial: Some("0".into()),
        ..row("Main", "hours", "VAR", "DINT")
    };
    let edited = edit(SAMPLE, &Change::Add { row: retained }).unwrap();
    assert!(
        edited.contains("    total : LINT;\n    hours : DINT := 0;\nEND_VAR"),
        "{edited}"
    );
}

#[test]
fn a_class_without_a_block_gets_one_after_the_pou_s_last() {
    let edited = edit(
        SAMPLE,
        &Change::Add {
            row: row("Lift", "jam", "VAR_TEMP", "BOOL"),
        },
    )
    .unwrap();
    assert!(
        edited.contains(
            "    level : REAL;\nEND_VAR\nVAR_TEMP\n    jam : BOOL;\nEND_VAR\nlevel := 1.0;"
        ),
        "{edited}"
    );
    let bare = "PROGRAM Bare\nx := 1;\nEND_PROGRAM\n";
    let edited = edit(
        bare,
        &Change::Add {
            row: row("Bare", "x", "VAR", "INT"),
        },
    )
    .unwrap();
    assert_eq!(
        edited,
        "PROGRAM Bare\nVAR\n    x : INT;\nEND_VAR\nx := 1;\nEND_PROGRAM\n"
    );
}

#[test]
fn a_change_of_class_moves_the_line_and_a_delete_removes_it() {
    let mut up = declarations(SAMPLE)[4].clone();
    up.class = "VAR_OUTPUT".into();
    let moved = edit(SAMPLE, &Change::Update { index: 4, row: up }).unwrap();
    assert!(
        moved
            .contains("VAR_INPUT\nEND_VAR\nVAR_OUTPUT\n    level : REAL;\n    up : BOOL;\nEND_VAR"),
        "{moved}"
    );
    let deleted = edit(SAMPLE, &Change::Delete { index: 2 }).unwrap();
    assert_eq!(deleted, SAMPLE.replace("    timer : TON;\n", ""));
}

#[test]
fn an_edit_that_would_not_parse_or_names_nothing_is_refused() {
    let broken = Declaration {
        type_name: "".into(),
        ..declarations(SAMPLE)[2].clone()
    };
    assert_eq!(
        edit(
            SAMPLE,
            &Change::Update {
                index: 2,
                row: broken
            }
        )
        .unwrap_err(),
        "the edit would not parse"
    );
    assert_eq!(
        edit(SAMPLE, &Change::Delete { index: 9 }).unwrap_err(),
        "no declaration 9"
    );
    assert_eq!(
        edit(
            SAMPLE,
            &Change::Add {
                row: row("Nowhere", "x", "VAR", "INT")
            }
        )
        .unwrap_err(),
        "Nowhere is not a program or a function block of this file"
    );
}

#[test]
fn a_rename_reaches_every_reference_through_trust() {
    let files = vec![("plc/Main.st".to_string(), SAMPLE.to_string())];
    let renamed = rename(&files, "plc/Main.st", 1, "cartons").unwrap();
    assert_eq!(renamed.len(), 1);
    let text = &renamed[0].1;
    assert!(
        text.contains("cartons : DINT := 3;") && text.contains("cartons := cartons + 1;"),
        "{text}"
    );
    assert!(
        rename(&files, "plc/Main.st", 1, "IF").is_err(),
        "a keyword is refused"
    );
    assert_eq!(
        rename(&files, "plc/Other.st", 0, "x").unwrap_err(),
        "no such file"
    );
}

#[test]
fn the_diverter_s_program_reads_whole() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/project/tests/fixtures/valid/two_belts_diverter/plc/Main.st"
    );
    let text = std::fs::read_to_string(path).unwrap();
    let rows = declarations_of("plc/Main.st".into(), text.clone());
    let detected_a = Declaration {
        address: Some("%IX0.0".into()),
        ..row("Main", "detected_a", "VAR", "BOOL")
    };
    // Its comment is on the line above, so it stays in the text and out of the row.
    let in_zone = Declaration {
        qualifier: Some("RETAIN".into()),
        ..row("Main", "in_zone", "VAR", "DINT")
    };
    assert_eq!(
        (rows.len(), &rows[0], rows.last().unwrap()),
        (20, &detected_a, &in_zone)
    );
    let renamed = rename_declaration(
        vec![("plc/Main.st".into(), text)],
        "plc/Main.st".into(),
        0,
        "eye_a".into(),
    );
    let text = &renamed.unwrap()[0].1;
    assert!(text.contains("eye_a AT %IX0.0 : BOOL;") && text.contains("arrived(CLK := eye_a);"));
    let edited = edit_declaration(
        "plc/Main.st".into(),
        text.clone(),
        Change::Delete { index: 0 },
    )
    .unwrap();
    assert!(!edited.contains("eye_a AT"));
}

#[test]
fn f2_renames_the_symbol_at_its_place_but_never_inside_a_block() {
    let files = vec![("plc/Main.st".to_string(), SAMPLE.to_string())];
    let line = SAMPLE
        .lines()
        .position(|l| l.contains("count := count + 1"))
        .unwrap() as u32
        + 1;
    let column = SAMPLE
        .lines()
        .nth(line as usize - 1)
        .unwrap()
        .find("count")
        .unwrap() as u32
        + 1;
    let renamed = rename_at(&files, "plc/Main.st", line, column, "cartons").unwrap();
    assert!(
        renamed[0].1.contains("cartons := cartons + 1;"),
        "{}",
        renamed[0].1
    );
    assert_eq!(
        rename_at(&files, "plc/Main.st", 900, 1, "x").unwrap_err(),
        "no such place"
    );
    let block = "FUNCTION_BLOCK B\nVAR_INPUT\n    speed : REAL;\nEND_VAR\nspeed := speed;\nEND_FUNCTION_BLOCK\n";
    let files = vec![("components/twinova.b@1.st".to_string(), block.to_string())];
    assert_eq!(
        rename_at(&files, "components/twinova.b@1.st", 5, 1, "v").unwrap_err(),
        "components/twinova.b@1.st is read-only: the rename would change it"
    );
}
