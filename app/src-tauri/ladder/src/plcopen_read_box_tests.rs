//! A box read from a TC6 `<LD>` as another tool may draw it: each pin a rung can take, and each one it cannot, named.

use super::{RAIL, in_var, pin_in, read, round_trip, symbol};

/// A block between contact `a` (id 2, on the rail) and whatever reads its outputs; `inputs` and `outputs` as given.
fn boxed(type_name: &str, instance: &str, inputs: &str, outputs: &str, after: &str) -> String {
    let instance = if instance.is_empty() {
        String::new()
    } else {
        format!(" instanceName=\"{instance}\"")
    };
    format!(
        "{RAIL}{}<block localId=\"3\" typeName=\"{type_name}\"{instance}><position x=\"0\" y=\"0\"/>\
         <inputVariables>{inputs}</inputVariables><inOutVariables/><outputVariables>{outputs}</outputVariables></block>{after}",
        symbol("contact", 2, "a", "", &[0])
    )
}

fn out_pin(name: &str, bound: &str) -> String {
    let bound = if bound.is_empty() {
        String::new()
    } else {
        format!("<expression>{bound}</expression>")
    };
    format!(
        "<variable formalParameter=\"{name}\"><connectionPointOut>{bound}</connectionPointOut></variable>"
    )
}

/// A coil `variable` (id `id`) fed by the block's output `pin`.
fn coil_on(id: usize, pin: &str, variable: &str) -> String {
    format!(
        "<coil localId=\"{id}\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"3\" \
         formalParameter=\"{pin}\"/></connectionPointIn><connectionPointOut/><variable>{variable}</variable></coil>"
    )
}

fn place(id: usize, pin: &str, variable: &str) -> String {
    format!(
        "<outVariable localId=\"{id}\"><position x=\"0\" y=\"0\"/><connectionPointIn><connection refLocalId=\"3\" \
         formalParameter=\"{pin}\"/></connectionPointIn><expression>{variable}</expression></outVariable>"
    )
}

#[test]
fn a_value_on_its_pin_an_unlinked_pin_and_an_output_bound_by_an_out_variable_read_as_pins() {
    let inputs = format!(
        "{}<variable formalParameter=\"PT\"><connectionPointIn><expression>T#1s</expression></connectionPointIn></variable>\
         <variable formalParameter=\"X\"><connectionPointIn/></variable>",
        pin_in("IN", 2)
    );
    let outputs = format!("{}{}", out_pin("Q", ""), out_pin("ET", ""));
    let after = format!(
        "{}{}<comment localId=\"9\"/>",
        coil_on(4, "Q", "q"),
        place(5, "ET", "Waited")
    );
    assert_eq!(
        read(&boxed("TON", "T1", &inputs, &outputs, &after)).unwrap(),
        "|a| T1(IN, PT := T#1s, ET => Waited, Q) (q);\n"
    );
}

#[test]
fn an_input_marked_plain_is_plain_and_one_sensing_an_edge_is_refused() {
    let input = |attrs: &str| {
        format!(
            "<variable formalParameter=\"IN\"{attrs}><connectionPointIn><connection refLocalId=\"2\"/>\
             </connectionPointIn></variable>"
        )
    };
    let body = |attrs: &str| {
        boxed(
            "TON",
            "T1",
            &input(attrs),
            &out_pin("Q", ""),
            &coil_on(4, "Q", "q"),
        )
    };
    assert_eq!(
        read(&body(" negated=\"false\" edge=\"none\"")).unwrap(),
        "|a| T1(IN, Q) (q);\n"
    );
    assert!(
        (read(&body(" edge=\"rising\"")).unwrap_err())
            .contains("'T1' with its input IN negated or edge-sensing")
    );
}

#[test]
fn a_contact_whose_input_has_no_link_stands_on_the_rail() {
    let body = format!(
        "{RAIL}<contact localId=\"2\"><position x=\"0\" y=\"0\"/><connectionPointIn/><connectionPointOut/><variable>a</variable></contact>{}",
        symbol("coil", 3, "q", "", &[2])
    );
    assert_eq!(read(&body).unwrap(), "|a| (q);\n");
}

#[test]
fn coils_drawn_beside_a_jump_are_the_coils_then_the_jump() {
    let ld = "VAR A, X, Y : BOOL; END_VAR\n|A| (X) (Y) >>Done;\nDone: ;\n";
    assert_eq!(round_trip(ld), "|A| [(X), (Y)] >>Done;\nDone: ;\n");
}

#[test]
fn what_a_box_cannot_be_on_a_rung_is_named() {
    let element = |what: &str| {
        format!(
            "POU 'P': its ladder has {what}, which a rung here cannot express, so the POU was left out"
        )
    };
    let fed = pin_in("IN", 2);
    let negated = "<variable formalParameter=\"IN\" negated=\"true\"><connectionPointIn><connection refLocalId=\"2\"/>\
                   </connectionPointIn></variable>";
    let cases = [
        (
            boxed(
                "TON",
                "T1",
                negated,
                &out_pin("Q", ""),
                &coil_on(4, "Q", "q"),
            ),
            "'T1' with its input IN negated or edge-sensing",
        ),
        (
            boxed(
                "TON",
                "T1",
                &fed,
                &out_pin("ET", "Waited"),
                &place(4, "ET", "Other"),
            ),
            "'T1''s ET bound to two places",
        ),
        (
            boxed(
                "TON",
                "T1",
                &fed,
                &out_pin("ET", "Waited"),
                &coil_on(4, "ET", "q"),
            ),
            "'T1''s ET both bound to a place and carried on along the rung",
        ),
        (
            boxed(
                "TON",
                "T1",
                &fed,
                &format!("{}{}", out_pin("Q", ""), out_pin("ET", "")),
                &format!("{}{}", coil_on(4, "Q", "q"), coil_on(5, "ET", "r")),
            ),
            "'T1' on the rung by two outputs, Q and ET",
        ),
        (
            boxed(
                "TON",
                "T1",
                &pin_in("IN", 8),
                &out_pin("Q", ""),
                &format!("{}{}", in_var(8, "TRUE"), coil_on(4, "Q", "q")),
            ),
            "'T1' that nothing on the rung feeds",
        ),
        (
            boxed(
                "ADD",
                "",
                &pin_in("IN1", 2),
                &out_pin("ENO", ""),
                &coil_on(4, "ENO", "q"),
            ),
            "a ADD function fed by the rung on IN1 rather than EN",
        ),
        (
            boxed(
                "ADD",
                "",
                &pin_in("EN", 2),
                &out_pin("OUT", ""),
                &coil_on(4, "OUT", "q"),
            ),
            "a ADD function carrying the rung on from OUT rather than ENO",
        ),
    ];
    for (body, what) in cases {
        assert_eq!(read(&body).unwrap_err(), element(what), "{body}");
    }
}

#[test]
fn a_label_after_the_last_network_opens_an_empty_rung_of_its_own() {
    let body = format!(
        "{RAIL}{}{}<label localId=\"4\" label=\"End\"><position x=\"0\" y=\"0\"/></label>",
        symbol("contact", 2, "a", "", &[0]),
        symbol("coil", 3, "q", "", &[2])
    );
    assert_eq!(read(&body).unwrap(), "|a| (q);\nEnd: ;\n");
}
