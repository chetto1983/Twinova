use super::*;
use crate::program_files::tests::{MAIN_4_4A, diverter, fifteen_seconds};

const CREATED: &str = "2026-11-14T09:00:00";

fn files(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(path, text)| (path.to_string(), text.to_string()))
        .collect()
}

fn export(path: &str, text: &str) -> Result<String, String> {
    export_ladder(
        &files(&[("plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n"), (path, text)]),
        path,
        CREATED,
    )
}

/// The spec's semantic round trip (decision 7): the 4.4a line's program in Ladder, exported to PLCopen, imported back
/// and run, gives the ST program's fingerprints over 15 s; exported again, the file is the same.
#[test]
fn the_4_4a_ladder_through_plcopen_and_back_runs_as_the_st_program() {
    let program = files(&[("plc/Main.ld", MAIN_4_4A)]);
    let xml = export_ladder(&program, "plc/Main.ld", CREATED).unwrap();
    for part in [
        r#"<pou name="Main" pouType="program">"#,
        r#"<variable name="detected_a" address="%IX0.0">"#,
        r#"<type><derived name="R_TRIG"/></type>"#,
        r#"<localVars retain="true">"#,
        r#"<block localId="#,
        r#"typeName="TON" instanceName="deck_clear""#,
        r#"<fileHeader companyName="Twinova" productName="Twinova""#,
    ] {
        assert!(xml.contains(part), "{part}\n{xml}");
    }
    let imported = import_ladder(&xml).unwrap();
    assert_eq!(imported.notes, Vec::<String>::new());
    let [(path, text)] = &imported.files[..] else {
        panic!("{:?}", imported.files)
    };
    assert_eq!(path, "plc/Main.ld");
    let mut back = diverter();
    back.program = vec![(path.clone(), text.clone())];
    let (st_marks, st_end) = fifteen_seconds(&diverter());
    let (back_marks, back_end) = fifteen_seconds(&back);
    assert_eq!(back_marks, st_marks, "{text}");
    assert_eq!(back_end, st_end);
    assert_eq!(
        export_ladder(&imported.files, "plc/Main.ld", CREATED).unwrap(),
        xml
    );
}

const DECLARATIONS: &str = "VAR_INPUT CONSTANT
    k : REAL := 1.5; (* the gain *)
END_VAR
VAR
    a, b : INT := 5;
    Arr : ARRAY[1..3] OF INT := [1, 2, 3];
    Grid : ARRAY[0..1, 0..2] OF BOOL;
    Name : STRING;
    Stamp : DATE_AND_TIME;
    Shift : TIME_OF_DAY;
    Lamp AT %QX1.0 : BOOL;
    T1 : TON;
END_VAR
VAR_OUTPUT NON_RETAIN
    q : BOOL;
END_VAR
VAR_IN_OUT
    io : DINT;
END_VAR
VAR_TEMP
    t : LREAL;
END_VAR
VAR_EXTERNAL PERSISTENT
    shared : WORD;
END_VAR
";

#[test]
fn every_declaration_a_ladder_program_has_goes_to_the_interface_and_back() {
    let ld = format!("{DECLARATIONS}\n|Lamp| (q);\n");
    let xml = export("plc/Pump.ld", &ld).unwrap();
    for part in [
        r#"<inputVars constant="true">"#,
        r#"<documentation><xhtml:p>the gain</xhtml:p></documentation>"#,
        r#"<variable name="a"><type><INT/></type><initialValue><simpleValue value="5"/></initialValue>"#,
        r#"<variable name="b">"#,
        r#"<array><dimension lower="1" upper="3"/><baseType><INT/></baseType></array>"#,
        r#"<arrayValue><value><simpleValue value="1"/></value><value><simpleValue value="2"/></value>"#,
        r#"<dimension lower="0" upper="1"/><dimension lower="0" upper="2"/><baseType><BOOL/></baseType>"#,
        "<type><string/></type>",
        "<type><DT/></type>",
        "<type><TOD/></type>",
        r#"<variable name="Lamp" address="%QX1.0">"#,
        r#"<outputVars nonretain="true">"#,
        "<inOutVars>",
        "<tempVars>",
        r#"<externalVars persistent="true">"#,
    ] {
        assert!(
            xml.contains(
                &part
                    .replace("><", ">\n")
                    .lines()
                    .next()
                    .unwrap()
                    .to_string()
            ),
            "{part}\n{xml}"
        );
    }
    let imported = import_ladder(&xml).unwrap();
    let text = &imported.files[0].1;
    for line in [
        "VAR_INPUT CONSTANT\n    k : REAL := 1.5; (* the gain *)\nEND_VAR",
        "    a : INT := 5;\n    b : INT := 5;",
        "    Arr : ARRAY[1..3] OF INT := [1, 2, 3];",
        "    Grid : ARRAY[0..1, 0..2] OF BOOL;",
        "    Name : STRING;\n    Stamp : DT;\n    Shift : TOD;\n    Lamp AT %QX1.0 : BOOL;\n    T1 : TON;",
        "VAR_OUTPUT NON_RETAIN",
        "VAR_EXTERNAL PERSISTENT\n    shared : WORD;",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    let again = export("plc/Pump.ld", text).unwrap();
    assert_eq!(again, xml, "a file read back exports the same");
}

#[test]
fn what_the_interface_has_no_place_for_is_named() {
    let refused = |declarations: &str| {
        export("plc/Pump.ld", &format!("{declarations}\n|a| (a);\n")).unwrap_err()
    };
    assert_eq!(
        refused("VAR\n    a : BOOL;\n    p : POINTER TO INT;\nEND_VAR"),
        "plc/Pump.ld: p's type POINTER TO INT is not one PLCopen is written with here"
    );
    // a structure declared in an ST file, given a value field by field
    assert_eq!(
        refused("VAR\n    a : BOOL;\n    s : Pair := (x := 1);\nEND_VAR"),
        "plc/Pump.ld: s's initial value (x := 1) is a structure, which PLCopen is not written with here"
    );
}

#[test]
fn declarations_truss_parser_does_not_read_are_named_at_their_line() {
    let refused = export(
        "plc/Pump.ld",
        "VAR\n    a : BOOL;\n    b : ;\nEND_VAR\n|a| (a);\n",
    )
    .unwrap_err();
    assert!(refused.starts_with("plc/Pump.ld: line 3: "), "{refused}");
    // an array of a size left open has no bounds TC6 can write
    assert_eq!(
        export(
            "plc/Pump.ld",
            "VAR\n    a : BOOL;\n    g : ARRAY[*] OF INT;\nEND_VAR\n|a| (a);\n"
        )
        .unwrap_err(),
        "plc/Pump.ld: g's array bounds * are not a range, lower..upper"
    );
}

#[test]
fn a_pou_with_no_interface_has_no_declarations_and_one_that_reads_back_wrong_is_named() {
    let pou = |label: &str| {
        format!(
            r#"<project><types><pous><pou name="P" pouType="program"><body><LD><leftPowerRail localId="1"><connectionPointOut formalParameter=""/></leftPowerRail><label localId="2" label="{label}"/><contact localId="3"><connectionPointIn><connection refLocalId="1"/></connectionPointIn><variable>a</variable></contact><coil localId="4"><connectionPointIn><connection refLocalId="3"/></connectionPointIn><variable>q</variable></coil></LD></body></pou></pous></types></project>"#
        )
    };
    assert_eq!(
        import_ladder(&pou("Top")).unwrap().files,
        [("plc/P.ld".to_string(), "\nTop: |a| (q);\n".to_string())]
    );
    assert_eq!(
        import_ladder(&pou("1st")).unwrap().notes,
        [
            "POU 'P' reads back as a Ladder file that does not: expected a contact |x|, a coil (x), a branch [..., ...] or a block"
        ]
    );
}

#[test]
fn a_ladder_file_that_does_not_lower_or_is_not_there_is_not_exported() {
    assert_eq!(
        export("plc/Pump.ld", "VAR a : BOOL; END_VAR\n|a| (a);\n|?| (a);").unwrap_err(),
        "plc/Pump.ld, rung 2: This contact has no variable yet: the ? on this rung"
    );
    assert_eq!(
        export_ladder(&[], "plc/Pump.ld", CREATED).unwrap_err(),
        "plc/Pump.ld is not a file of the program"
    );
}

/// A TC6 project of four POUs: the Ladder program is taken, the rest named and left out.
#[test]
fn only_ladder_programs_are_imported_and_the_rest_is_named() {
    let ld = |name: &str, kind: &str, body: &str| {
        format!(
            r#"<pou name="{name}" pouType="{kind}"><interface><localVars><variable name="a"><type><BOOL/></type></variable></localVars></interface><body>{body}</body></pou>"#
        )
    };
    let rung = r#"<LD><leftPowerRail localId="1"><connectionPointOut formalParameter=""/></leftPowerRail><contact localId="2"><connectionPointIn><connection refLocalId="1"/></connectionPointIn><variable>a</variable></contact><coil localId="3"><connectionPointIn><connection refLocalId="2"/></connectionPointIn><variable>a</variable></coil></LD>"#;
    let bridge = r#"<LD><connector localId="2" name="C"/></LD>"#;
    let xml = format!(
        r#"<project xmlns="http://www.plcopen.org/xml/tc6_0201"><types><pous>{}{}{}{}{}</pous></types></project>"#,
        ld("Cell", "program", rung),
        ld("Text", "program", "<ST><xhtml>a := TRUE;</xhtml></ST>"),
        ld("Latch", "functionBlock", rung),
        ld("Joined", "program", bridge),
        ld("2nd", "program", rung),
    );
    let imported = import_ladder(&xml).unwrap();
    assert_eq!(
        imported.files,
        [(
            "plc/Cell.ld".to_string(),
            "VAR\n    a : BOOL;\nEND_VAR\n\n|a| (a);\n".to_string()
        )]
    );
    assert_eq!(
        imported.notes,
        [
            "POU 'Text' is not Ladder, so it was left out",
            "POU 'Latch' is a functionBlock: a Ladder file here is a program, so it was left out",
            "POU 'Joined': it draws a connector in its ladder, which a rung here does not have yet, so the POU was left out",
            "POU '2nd': its name is not an identifier",
        ]
    );
    assert_eq!(
        import_ladder("<LD/>").unwrap_err(),
        "the file is not a PLCopen project: its root is not <project>"
    );
    assert!(
        import_ladder("not xml")
            .unwrap_err()
            .starts_with("the file is not XML")
    );
}

#[test]
fn an_interface_read_back_takes_types_and_values_other_tools_write() {
    let xml = r#"<project><types><pous><pou name="P" pouType="program"><interface>
        <localVars>
          <variable name="p"><type><struct/></type></variable>
        </localVars></interface><body><LD/></body></pou>
        <pou name="Q" pouType="program"><interface><localVars>
          <variable name="v"><type><derived name="Pair"/></type><initialValue><structValue><value member="x"><simpleValue value="1"/></value></structValue></initialValue></variable>
          <variable name="w"><type><INT/></type><initialValue><enumValue/></initialValue></variable>
        </localVars></interface><body><LD/></body></pou>
        <pou name="R" pouType="program"><interface><accessVars/></interface><body><LD/></body></pou>
        <pou name="S" pouType="program"><interface><localVars><variable name="n"/></localVars></interface><body><LD/></body></pou>
        </pous></types></project>"#;
    assert_eq!(
        import_ladder(xml).unwrap().notes,
        [
            "POU 'P': variable 'p' is of a <struct> type, which a Ladder program here is not read with",
            "POU 'Q': variable 'w''s initial value is a <enumValue>, which is not read here",
            "POU 'R': an interface's <accessVars>, which a Ladder program here has no section for",
            "POU 'S': variable 'n' has no type",
        ]
    );
    let struct_init = r#"<project><types><pous><pou name="Q" pouType="program"><interface><localVars>
          <variable name="v"><type><derived name="Pair"/></type><initialValue><structValue><value member="x"><simpleValue value="1"/></value></structValue></initialValue></variable>
        </localVars></interface><body><LD/></body></pou></pous></types></project>"#;
    assert_eq!(
        import_ladder(struct_init).unwrap().files[0].1,
        "VAR\n    v : Pair := (x := 1);\nEND_VAR\n\n"
    );
}
