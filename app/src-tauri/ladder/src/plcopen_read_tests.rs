//! Ladder through PLCopen TC6, OIP's cases (NOTICE): the `<LD>` graph a program writes, the rungs read back out of one
//! written by hand from TC6's schema alone, and what the reader refuses rather than guesses.

use super::read_ld_body;
use crate::lower::lower;
use crate::parse::parse;
use crate::plcopen_write::ld_body;
use crate::print::print;
use proptest::prelude::*;

use crate::test_support::{VARIABLES, blocks, lowered, program, trace};

#[path = "plcopen_read_box_tests.rs"]
mod boxes;

const XHTML: &str = "xmlns:xhtml=\"http://www.w3.org/1999/xhtml\"";

/// `body` read as the contents of an `<LD>`.
fn read(body: &str) -> Result<String, String> {
    let xml = format!("<LD {XHTML}>{body}</LD>");
    let document = roxmltree::Document::parse(&xml).map_err(|e| e.to_string())?;
    read_ld_body(document.root_element(), "P")
}

/// `ld` written as `<LD>` and read back.
fn round_trip(ld: &str) -> String {
    let program = parse(ld).unwrap();
    let body = ld_body(&program, &lowered(ld).boxes, "").unwrap();
    read(&body).unwrap_or_else(|e| panic!("{e}\n{body}"))
}

const BOOLS: &str = "VAR A, B, C, X, Y, Z, Q : BOOL; N : INT; T1 : TON; END_VAR\n";

/// Every kind of element, a nested branch, an edge, a timer, a function on EN, one that starts a rung, a jump, a label and
/// a RETURN, each with steps that exercise it.
const CASES: [(&str, &[&str], &[&str]); 6] = [
    ("[|A|, |X|] |/B| (X);", &["A", "!A", "B", "!B"], &["X"]),
    (
        "|A| [|B| [|C|, ] (S Y), |/C| (R Y)] (P Z);",
        &["A B", "C", "!B", "!A !C", "A"],
        &["Y", "Z"],
    ),
    (
        "|A| T1(IN, PT := T#20ms) (X) (/Y);",
        &["A", "", "", "!A"],
        &["X", "Y"],
    ),
    (
        "|N A| ADD(IN1 := N, IN2 := 1, OUT => N) (Q);",
        &["A", "!A", "", "A", "!A"],
        &["N", "Q"],
    ),
    (
        "EQ(IN1 := N, IN2 := 2, OUT) |B| (X);\n[GT(IN1 := N, IN2 := 0, OUT), |C|] (Y);",
        &["N=2 B", "N=0", "C"],
        &["X", "Y"],
    ),
    (
        "|A| >>Past;\n(* skipped while A *)\n|B| (X);\nPast: |C| RETURN;\n(Y);",
        &["B", "A !B", "!A C", "!C"],
        &["X", "Y"],
    ),
];

#[test]
fn every_case_written_as_ld_reads_back_as_a_program_that_runs_the_same_scan_for_scan() {
    for (rungs, steps, watch) in CASES {
        let ld = format!("{BOOLS}{rungs}");
        let back = format!("{BOOLS}{}", round_trip(&ld));
        let expected = trace(&[&lowered(&ld).st], steps, watch);
        assert_eq!(
            trace(&[&lowered(&back).st], steps, watch),
            expected,
            "{ld}\n----\n{back}"
        );
    }
}

#[test]
fn rungs_read_back_as_written_but_for_coils_in_series_drawn_in_parallel() {
    let ld = format!("{BOOLS}[|A|, |X|] |/B| (X);\n|A| T1(IN, PT := T#2s, ET => W) (Y) (Z);\n");
    assert_eq!(
        round_trip(&ld),
        "[|A|, |X|] |/B| (X);\n|A| T1(IN, PT := T#2s, ET => W, Q) [(Y), (Z)];\n"
    );
}

#[test]
fn an_empty_path_is_drawn_as_a_contact_on_true_and_reads_back_in_its_place() {
    let rungs = [
        "|A| [|B|, ] (Q);\n",
        "[|A|, ] (Q);\n",
        "|A| [, (X), |C|] |B| (Y);\n",
        "[, ];\n",
    ];
    for rung in rungs {
        assert_eq!(round_trip(&format!("{BOOLS}{rung}")), rung);
    }
    let program = parse("[|A|, ] (Q);").unwrap();
    let body = ld_body(&program, &[], "").unwrap();
    assert_eq!(
        body.matches("<variable>TRUE</variable>").count(),
        1,
        "{body}"
    );
    // and a contact on TRUE written by another tool in a series is nothing more than the wire it is
    let series = format!(
        "{RAIL}{}{}",
        symbol("contact", 2, "TRUE", "", &[0]),
        symbol("coil", 3, "q", "", &[2])
    );
    assert_eq!(read(&series).unwrap(), "(q);\n");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]
    /// The lowering's random programs, written and read back, run as they did.
    #[test]
    fn random_programs_written_and_read_back_run_the_same(program in program(), inputs in proptest::collection::vec(0u8..4, 1..12)) {
        let ld = crate::print::print(&program);
        let back = format!("{}\n{}", program.declarations, round_trip(&ld));
        let steps: Vec<String> = inputs
            .iter()
            .map(|bits| format!("{}V0 {}V1", if bits & 1 == 0 { "!" } else { "" }, if bits & 2 == 0 { "!" } else { "" }))
            .collect();
        let steps: Vec<&str> = steps.iter().map(String::as_str).collect();
        let run = |text: &str| trace(&[&lowered(text).st], &steps, &VARIABLES);
        prop_assert_eq!(run(&back), run(&ld), "{}\n----\n{}", ld, back);
    }
}

#[test]
fn a_function_starting_a_rung_is_drawn_with_nothing_linked_into_it_and_reads_back() {
    let ld = format!(
        "{BOOLS}EQ(IN1 := N, IN2 := 3, OUT) |A| (X);\n[GT(IN1 := N, IN2 := 5, OUT), |B|] (Y);\n"
    );
    let program = parse(&ld).unwrap();
    let body = ld_body(&program, &lowered(&ld).boxes, "").unwrap();
    let eq = &body[body.find("typeName=\"EQ\"").unwrap()..];
    let eq = &eq[..eq.find("</block>").unwrap()];
    // the rail is localId 1: the comparison's pins take only its inVariables
    assert!(!eq.contains("refLocalId=\"1\""), "{eq}");
    assert!(
        !eq.contains("instanceName"),
        "a function names no instance: {eq}"
    );
    assert!(body.contains("typeName=\"TON\"") || !ld.contains("T1("));
    assert_eq!(
        round_trip(&ld),
        "EQ(IN1 := N, IN2 := 3, OUT) |A| (X);\n[GT(IN1 := N, IN2 := 5, OUT), |B|] (Y);\n"
    );
}

#[test]
fn comments_labels_and_jumps_keep_their_rungs() {
    let ld = format!("{BOOLS}(* starts\n   the line *)\nTop: |A| (X) >>Done;\n|B| (Y);\nDone: ;\n");
    assert_eq!(
        round_trip(&ld),
        "(* starts *)\n(* the line *)\nTop: |A| (X) >>Done;\n|B| (Y);\nDone: ;\n"
    );
    assert_eq!(
        print(&parse(&round_trip(&ld)).unwrap()).lines().next(),
        Some("(* starts")
    );
}

#[test]
fn a_box_that_did_not_lower_cannot_be_drawn() {
    let program = parse("|A| T9(IN, Q);").unwrap();
    let lowered = lower(&program, "Ld", &blocks);
    assert_eq!(
        ld_body(&program, &[], "").unwrap_err(),
        "the box 'T9' did not lower, so its pins are not known"
    );
    assert_eq!(
        lowered.errors.len(),
        0,
        "T9 lowers by its bare pins, its type unknown"
    );
    // a block of a type nothing knows is written by its head, an instance
    let body = ld_body(&program, &lowered.boxes, "").unwrap();
    assert!(
        body.contains("typeName=\"T9\" instanceName=\"T9\""),
        "{body}"
    );
}

// A ladder written from TC6's schema alone, in the plainest shape it allows: every position 0,0 (order comes from the
// file), the rails shared by every network, and what TC6 lets a tool add for itself under a made-up namespace.
const EXTENSION: &str = "http://example.org/made-up-tool/decoration";

fn network(id: usize, title: &str) -> String {
    format!(
        "<comment localId=\"{id}\" height=\"0\" width=\"0\"><position x=\"0\" y=\"0\"/><content><xhtml:p>{title}</xhtml:p></content></comment>\
         <vendorElement localId=\"{}\"><position x=\"0\" y=\"0\"/><alternativeText><xhtml:p>banner</xhtml:p></alternativeText>\
         <addData><data name=\"{EXTENSION}/banner\" handleUnknown=\"discard\"><Banner style=\"wide\"/></data></addData></vendorElement>",
        id + 1
    )
}

fn pin_in(name: &str, from: usize) -> String {
    format!(
        "<variable formalParameter=\"{name}\"><connectionPointIn><connection refLocalId=\"{from}\"/></connectionPointIn></variable>"
    )
}

fn in_var(id: usize, expression: &str) -> String {
    format!(
        "<inVariable localId=\"{id}\"><position x=\"0\" y=\"0\"/><connectionPointOut/><expression>{expression}</expression></inVariable>"
    )
}

fn symbol(tag: &str, id: usize, variable: &str, attrs: &str, from: &[usize]) -> String {
    let links: String = from
        .iter()
        .map(|f| format!("<connection refLocalId=\"{f}\"/>"))
        .collect();
    format!(
        "<{tag} localId=\"{id}\"{attrs}><position x=\"0\" y=\"0\"/><connectionPointIn>{links}</connectionPointIn><connectionPointOut/><variable>{variable}</variable></{tag}>"
    )
}

const RAIL: &str = "<leftPowerRail localId=\"0\"><position x=\"0\" y=\"0\"/><connectionPointOut formalParameter=\"\"/></leftPowerRail>";

#[test]
fn a_ladder_from_the_schema_alone_reads_in_file_order_with_its_comments_and_pins() {
    let body = [
        RAIL.to_string(),
        network(1, "seal-in"),
        symbol("contact", 3, "Start", "", &[0]),
        symbol("contact", 4, "Motor", "", &[0]),
        symbol("contact", 5, "Stop", " negated=\"true\"", &[3, 4]),
        symbol("coil", 6, "Motor", "", &[5]),
        format!("<addData><data name=\"{EXTENSION}/layout\" handleUnknown=\"discard\"><Layout grid=\"8\"/></data></addData>"),
        network(8, ""),
        symbol("contact", 10, "Eye", "", &[0]),
        in_var(11, "T#50MS"),
        format!(
            "<block localId=\"12\" typeName=\"TON\" instanceName=\"T1\"><position x=\"0\" y=\"0\"/><inputVariables>{}{}</inputVariables><inOutVariables/>\
             <outputVariables><variable formalParameter=\"Q\"><connectionPointOut/></variable>\
             <variable formalParameter=\"ET\"><connectionPointOut><expression>Waited</expression></connectionPointOut></variable></outputVariables></block>",
            pin_in("IN", 10),
            pin_in("PT", 11)
        ),
        "<coil localId=\"13\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"12\" formalParameter=\"Q\"/></connectionPointIn><connectionPointOut/><variable>Late</variable></coil>".into(),
        network(14, ""),
        symbol("contact", 16, "Eye", " edge=\"rising\"", &[0]),
        in_var(17, "Count"),
        in_var(18, "1"),
        format!(
            "<block localId=\"19\" typeName=\"ADD\"><position x=\"0\" y=\"0\"/><inputVariables>{}{}{}</inputVariables><inOutVariables/>\
             <outputVariables><variable formalParameter=\"ENO\"><connectionPointOut/></variable>\
             <variable formalParameter=\"OUT\"><connectionPointOut><expression>Count</expression></connectionPointOut></variable></outputVariables></block>",
            pin_in("EN", 16),
            pin_in("IN1", 17),
            pin_in("IN2", 18)
        ),
        "<coil localId=\"20\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"19\" formalParameter=\"ENO\"/></connectionPointIn><connectionPointOut/><variable>Counted</variable></coil>".into(),
        "<rightPowerRail localId=\"21\"><position x=\"0\" y=\"0\"/><connectionPointIn/></rightPowerRail>".into(),
    ]
    .concat();
    assert_eq!(
        read(&body).unwrap(),
        "(* seal-in *)\n[|Start|, |Motor|] |/Stop| (Motor);\n|Eye| T1(IN, PT := T#50MS, ET => Waited, Q) (Late);\n\
         |P Eye| ADD(IN1 := Count, IN2 := 1, OUT => Count) (Counted);\n"
    );
}

/// An adder on the rung by EN, its inputs and its result as given.
fn adder(inputs: &str, result: &str) -> String {
    format!(
        "{RAIL}{}{}{}<block localId=\"4\" typeName=\"ADD\"><position x=\"0\" y=\"0\"/><inputVariables>{}{inputs}</inputVariables>\
         <inOutVariables/><outputVariables><variable formalParameter=\"ENO\"><connectionPointOut/></variable>\
         <variable formalParameter=\"{result}\"><connectionPointOut><expression>Count</expression></connectionPointOut></variable></outputVariables></block>\
         <coil localId=\"5\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"4\" formalParameter=\"ENO\"/></connectionPointIn><connectionPointOut/><variable>Counted</variable></coil>",
        symbol("contact", 1, "Eye", "", &[0]),
        in_var(2, "Count"),
        in_var(3, "1"),
        pin_in("EN", 1)
    )
}

#[test]
fn a_box_pin_with_no_name_or_a_box_reached_twice_is_refused_by_name() {
    assert_eq!(
        read(&adder(
            &format!("{}{}", pin_in("IN1", 2), pin_in("IN2", 3)),
            "OUT"
        ))
        .unwrap(),
        "|Eye| ADD(IN1 := Count, IN2 := 1, OUT => Count) (Counted);\n"
    );
    let refused = |body: String| read(&body).unwrap_err();
    let cases = [
        (
            adder(&format!("{}{}", pin_in("", 2), pin_in("", 3)), "OUT"),
            "a ADD box with an input pin that has no name (an empty formalParameter)",
        ),
        (
            adder(&format!("{}{}", pin_in("IN1", 2), pin_in("IN2", 3)), ""),
            "a ADD box with an output pin that has no name (an empty formalParameter)",
        ),
        (
            adder(&format!("{}{}", pin_in("IN1", 2), pin_in("IN2", 1)), "OUT"),
            "a ADD box on the rung by two inputs, EN and IN2",
        ),
    ];
    for (body, what) in cases {
        assert_eq!(
            refused(body),
            format!(
                "POU 'P': its ladder has {what}, which a rung here cannot express, so the POU was left out"
            )
        );
    }
}

#[test]
fn a_jump_its_label_and_a_return_read_one_network_each_in_file_order() {
    let body = [
        RAIL.to_string(),
        symbol("contact", 2, "a", "", &[0]),
        "<jump localId=\"3\" label=\"Past\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"2\"/></connectionPointIn></jump>".into(),
        symbol("contact", 4, "b", "", &[0]),
        symbol("coil", 5, "c", "", &[4]),
        "<label localId=\"6\" label=\"Past\"><position x=\"0\" y=\"0\"/></label>".into(),
        symbol("contact", 7, "d", "", &[0]),
        "<return localId=\"8\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"7\"/></connectionPointIn></return>".into(),
        symbol("contact", 9, "b", "", &[0]),
        symbol("coil", 10, "q", "", &[9]),
    ]
    .concat();
    assert_eq!(
        read(&body).unwrap(),
        "|a| >>Past;\n|b| (c);\nPast: |d| RETURN;\n|b| (q);\n"
    );
}

#[test]
fn what_a_rung_cannot_express_is_named() {
    let whole = |what: &str| format!("POU 'P': {what}, so the POU was left out");
    let element = |what: &str| {
        format!(
            "POU 'P': its ladder has {what}, which a rung here cannot express, so the POU was left out"
        )
    };
    let coil_q = |from: &[usize]| symbol("coil", 7, "q", "", from);
    // the Wheatstone shape: c across from a's output to b's
    let bridge = [
        RAIL.to_string(),
        symbol("contact", 2, "a", "", &[0]),
        symbol("contact", 3, "b", "", &[0]),
        symbol("contact", 4, "c", "", &[2]),
        symbol("contact", 5, "d", "", &[2]),
        symbol("contact", 6, "c", "", &[3, 4]),
        coil_q(&[5, 6]),
    ]
    .concat();
    let cases = [
        (
            format!(
                "{RAIL}<connector localId=\"2\" name=\"C1\"><position x=\"0\" y=\"0\"/></connector>"
            ),
            whole("it draws a connector in its ladder, which a rung here does not have yet"),
        ),
        (
            bridge,
            whole(
                "a network of its ladder is not series-parallel (a bridge), which a rung here cannot express",
            ),
        ),
        (
            format!("{RAIL}{}", coil_q(&[9])),
            whole("an element of its ladder is not connected to the left rail"),
        ),
        (
            format!("{RAIL}{}", symbol("coil", 2, "", "", &[0])),
            element("a coil with no variable"),
        ),
        (
            format!(
                "{RAIL}{}",
                symbol("contact", 2, "a", " negated=\"true\" edge=\"rising\"", &[0])
            ),
            element("a contact on 'a' that is negated and rising-edge"),
        ),
        (
            format!(
                "{RAIL}{}",
                symbol("contact", 2, "a", " storage=\"set\"", &[0])
            ),
            element("a contact on 'a' that is set"),
        ),
        (
            format!(
                "{RAIL}{}<outVariable localId=\"3\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"2\"/></connectionPointIn><expression>q</expression></outVariable>",
                symbol("contact", 2, "a", "", &[0])
            ),
            element("the rung wired straight into a variable, 'q' (draw a coil)"),
        ),
        (
            format!("{RAIL}<step localId=\"2\"/>"),
            element("a <step> element"),
        ),
        (
            // a bare wire beside contact b: b's input and output on one node
            format!(
                "{RAIL}{}{}{}",
                symbol("contact", 2, "a", "", &[0]),
                symbol("contact", 3, "b", "", &[2]),
                coil_q(&[2, 3])
            ),
            whole(
                "a network of its ladder joins a path's two ends with a bare wire, which this reader takes only drawn \
                 as a contact on TRUE",
            ),
        ),
    ];
    for (body, why) in cases {
        assert_eq!(read(&body).unwrap_err(), why, "{body}");
    }
}

#[test]
fn a_body_with_no_rungs_is_an_empty_program_and_other_tools_pin_names_read_as_iec() {
    assert_eq!(read("").unwrap(), "");
    let counter = format!(
        "{RAIL}{}<block localId=\"3\" typeName=\"CTU\" instanceName=\"C1\"><position x=\"0\" y=\"0\"/><inputVariables>{}</inputVariables>\
         <inOutVariables/><outputVariables><variable formalParameter=\"Q\"><connectionPointOut/></variable></outputVariables></block>\
         <coil localId=\"4\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"3\" formalParameter=\"Q\"/></connectionPointIn><connectionPointOut/><variable>q</variable></coil>",
        symbol("contact", 2, "a", "", &[0]),
        pin_in("RESET", 2)
    );
    assert_eq!(read(&counter).unwrap(), "|a| C1(R, Q) (q);\n");
}
