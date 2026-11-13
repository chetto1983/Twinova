use super::*;
use crate::print::print;
use proptest::prelude::*;

fn contact(mode: ContactMode, operand: &str) -> Element {
    Element::Contact {
        mode,
        operand: Some(operand.into()),
    }
}

fn coil(mode: CoilMode, operand: &str) -> Element {
    Element::Coil {
        mode,
        operand: Some(operand.into()),
    }
}

fn rungs(text: &str) -> Vec<Rung> {
    parse(text).unwrap().rungs
}

#[test]
fn a_seal_in_reads_as_a_branch_then_a_closed_contact_then_a_coil() {
    let rung = &rungs("[|Start|, |Motor|] |/Stop| (Motor);")[0];
    assert_eq!(
        rung.elements,
        [
            Element::Branch {
                paths: vec![
                    vec![contact(ContactMode::Open, "Start")],
                    vec![contact(ContactMode::Open, "Motor")]
                ]
            },
            contact(ContactMode::Closed, "Stop"),
            coil(CoilMode::Normal, "Motor"),
        ]
    );
}

#[test]
fn modifiers_need_a_space_and_an_operand_after_them() {
    let rung =
        &rungs("|P Button| |N Line.Eye[2]| |P| (S Jam) (R Jam) (/Lamp) (P Pulse) (N Fall) (S);")[0];
    assert_eq!(
        rung.elements,
        [
            contact(ContactMode::Rising, "Button"),
            contact(ContactMode::Falling, "Line.Eye[2]"),
            contact(ContactMode::Open, "P"),
            coil(CoilMode::Set, "Jam"),
            coil(CoilMode::Reset, "Jam"),
            coil(CoilMode::Negated, "Lamp"),
            coil(CoilMode::Rising, "Pulse"),
            coil(CoilMode::Falling, "Fall"),
            coil(CoilMode::Normal, "S"),
        ]
    );
}

#[test]
fn a_box_keeps_its_pins_and_their_values_as_written() {
    let rung = &rungs(
        "|Run| T1(IN, PT := T#2s, ET => Elapsed) ?(IN1 := LIMIT(0, (a + b), 9), S := 'a,b)') (Late);",
    )[0];
    assert_eq!(
        rung.elements[1],
        Element::Box {
            head: Some("T1".into()),
            pins: vec![
                Pin {
                    name: "IN".into(),
                    value: None,
                    out: false
                },
                Pin {
                    name: "PT".into(),
                    value: Some("T#2s".into()),
                    out: false
                },
                Pin {
                    name: "ET".into(),
                    value: Some("Elapsed".into()),
                    out: true
                },
            ]
        }
    );
    assert_eq!(
        rung.elements[2],
        Element::Box {
            head: None,
            pins: vec![
                Pin {
                    name: "IN1".into(),
                    value: Some("LIMIT(0, (a + b), 9)".into()),
                    out: false
                },
                Pin {
                    name: "S".into(),
                    value: Some("'a,b)'".into()),
                    out: false
                },
            ]
        }
    );
}

#[test]
fn blanks_labels_exits_comments_and_declarations() {
    let text = "VAR\n    Motor : BOOL;\nEND_VAR\n\n(* starts the line\n   and holds it *)\nTop: |?| (?) >>Done;\n\
                [|A|, ] RETURN;\nDone: ;\n";
    let program = parse(text).unwrap();
    assert_eq!(program.declarations, "VAR\n    Motor : BOOL;\nEND_VAR");
    let [top, second, done] = &program.rungs[..] else {
        panic!("{:?}", program.rungs)
    };
    assert_eq!(
        top.comment.as_deref(),
        Some("starts the line\nand holds it")
    );
    assert_eq!(top.label.as_deref(), Some("Top"));
    assert_eq!(
        top.elements,
        [
            Element::Contact {
                mode: ContactMode::Open,
                operand: None
            },
            Element::Coil {
                mode: CoilMode::Normal,
                operand: None
            }
        ]
    );
    assert_eq!(
        top.exit,
        Some(Exit::Jump {
            label: "Done".into()
        })
    );
    assert_eq!(
        second.elements,
        [Element::Branch {
            paths: vec![vec![contact(ContactMode::Open, "A")], vec![]]
        }]
    );
    assert_eq!(second.exit, Some(Exit::Return));
    assert_eq!(
        (done.label.as_deref(), done.elements.len(), &done.exit),
        (Some("Done"), 0, &None)
    );
    assert_eq!(print(&program), text, "the canonical form prints back");
}

#[test]
fn what_does_not_read_says_where() {
    let at = |text: &str| parse(text).map(|_| ()).unwrap_err();
    assert_eq!(at("|A| (B)").message, "expected ';' to end the rung");
    assert_eq!(
        at("[|A|] (B);").message,
        "a branch has two paths or more: [|A|, |B|]"
    );
    assert_eq!(at("[|A|] (B);").offset, 0);
    assert_eq!(
        at("|| (B);").message,
        "a contact needs a variable, or ? while blank"
    );
    assert_eq!(at("|A| (B) >>;").message, "a jump names a label: >>Next");
    assert_eq!(
        at("|A| 3(B);").message,
        "expected a contact |x|, a coil (x), a branch [..., ...] or a block"
    );
    assert_eq!(at("T1(IN := );").message, "a pin's value is missing");
    assert_eq!(
        at("T1(IN PT);").message,
        "expected ',' between the block's pins"
    );
    assert_eq!(
        at("T1(:= 1);").message,
        "a pin is named: IN, PT := T#2s or Q => done"
    );
    assert_eq!(at("(* open").message, "a comment with no '*)'");
    assert_eq!(
        at("|A| (B);\n(* dangling *)").message,
        "a comment with no rung after it"
    );
    assert_eq!(
        at("|A| (B) >>X |C|;").message,
        "expected ';' to end the rung"
    );
}

// The grammar's round trip on generated programs (CLAUDE.md: property tests for codecs).

fn name() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,6}".prop_filter("not a keyword the grammar reads", |s| {
        !s.eq_ignore_ascii_case("RETURN")
    })
}

fn operand() -> impl Strategy<Value = Option<String>> {
    prop_oneof![1 => Just(None), 4 => name().prop_map(Some), 1 => (name(), name()).prop_map(|(a, b)| Some(format!("{a}.{b}[1]")))]
}

fn value() -> impl Strategy<Value = String> {
    prop_oneof![
        name(),
        Just("T#2s".to_string()),
        (name(), name()).prop_map(|(a, b)| format!("LIMIT(0, ({a} + {b}), 9)"))
    ]
}

fn leaf() -> impl Strategy<Value = Element> {
    let contact = (
        prop_oneof![
            Just(ContactMode::Open),
            Just(ContactMode::Closed),
            Just(ContactMode::Rising),
            Just(ContactMode::Falling)
        ],
        operand(),
    )
        .prop_map(|(mode, operand)| Element::Contact { mode, operand });
    let coil = (
        prop_oneof![
            Just(CoilMode::Normal),
            Just(CoilMode::Negated),
            Just(CoilMode::Set),
            Just(CoilMode::Reset),
            Just(CoilMode::Rising),
            Just(CoilMode::Falling)
        ],
        operand(),
    )
        .prop_map(|(mode, operand)| Element::Coil { mode, operand });
    let pin =
        (name(), proptest::option::of(value()), any::<bool>()).prop_map(|(name, value, out)| Pin {
            out: out && value.is_some(),
            name,
            value,
        });
    let block = (operand(), proptest::collection::vec(pin, 0..4))
        .prop_map(|(head, pins)| Element::Box { head, pins });
    prop_oneof![contact, coil, block]
}

fn element() -> impl Strategy<Value = Element> {
    leaf().prop_recursive(3, 24, 3, |inner| {
        proptest::collection::vec(proptest::collection::vec(inner, 0..3), 2..4)
            .prop_map(|paths| Element::Branch { paths })
    })
}

fn rung() -> impl Strategy<Value = Rung> {
    (
        proptest::option::of(
            proptest::collection::vec("[a-z][a-z ]{0,8}[a-z]", 1..3).prop_map(|l| l.join("\n")),
        ),
        proptest::option::of(name()),
        proptest::collection::vec(element(), 0..4),
        prop_oneof![
            Just(None),
            Just(Some(Exit::Return)),
            name().prop_map(|label| Some(Exit::Jump { label }))
        ],
    )
        .prop_map(|(comment, label, elements, exit)| Rung {
            comment,
            label,
            elements,
            exit,
        })
}

proptest! {
    #[test]
    fn a_printed_program_reads_back_the_same(
        declared in proptest::bool::ANY,
        rungs in proptest::collection::vec(rung(), 0..5),
    ) {
        let declarations = if declared { "VAR\n    a : BOOL;\nEND_VAR".to_string() } else { String::new() };
        let program = Program { declarations, rungs };
        let text = print(&program);
        prop_assert_eq!(parse(&text), Ok(program), "{}", text);
    }
}
