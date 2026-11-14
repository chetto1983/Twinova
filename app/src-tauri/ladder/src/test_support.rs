//! What the crate's tests share: the fixtures' blocks, as a project would resolve them, and truST running a program scan
//! by scan.

use trust_runtime::harness::TestHarness;
use trust_runtime::value::{Duration, Value};

use proptest::prelude::*;

use crate::lower::{Block, BlockPin, Lowered, lower};
use crate::model::{CoilMode, ContactMode, Element, Program, Rung};
use crate::parse::parse;

fn pins(list: &[(&str, bool)]) -> Vec<BlockPin> {
    list.iter()
        .map(|&(name, bool)| BlockPin {
            name: name.into(),
            bool,
        })
        .collect()
}

/// The fixtures' declarations, as the project's would resolve them.
pub(crate) fn blocks(head: &str) -> Option<Block> {
    let instance =
        |type_name: &str, inputs: &[(&str, bool)], outputs: &[(&str, bool)]| Block::Instance {
            type_name: type_name.into(),
            inputs: pins(inputs),
            outputs: pins(outputs),
        };
    match head.to_ascii_uppercase().as_str() {
        "T1" => Some(instance(
            "TON",
            &[("IN", true), ("PT", false)],
            &[("Q", true), ("ET", false)],
        )),
        "C1" => Some(instance(
            "CTU",
            &[("CU", true), ("R", true), ("PV", false)],
            &[("Q", true), ("CV", false)],
        )),
        "R1" => Some(instance("R_TRIG", &[("CLK", true)], &[("Q", true)])),
        "L" => Some(instance(
            "Latch",
            &[("Set", true), ("Clear", true)],
            &[("On", true)],
        )),
        "ADD" | "GT" | "EQ" | "LT" | "MOVE" | "INRANGE" => Some(Block::Function),
        _ => None,
    }
}

pub(crate) fn lowered(text: &str) -> Lowered {
    let lowered = lower(&parse(text).unwrap(), "Ld", &blocks);
    assert_eq!(lowered.errors, [], "{}", lowered.st);
    lowered
}

/// Run `sources` one 10 ms scan per step, each step setting inputs (`Start` TRUE, `!Start` FALSE, `N=3` an INT), and
/// give `watch` after each scan, BOOLs as 1 and 0: `Motor=1 Count=2`.
pub(crate) fn trace(sources: &[&str], steps: &[&str], watch: &[&str]) -> Vec<String> {
    let mut harness = TestHarness::from_sources(sources)
        .unwrap_or_else(|e| panic!("{e:?}\n{}", sources.join("\n")));
    steps
        .iter()
        .map(|step| {
            for input in step.split_whitespace() {
                match input.split_once('=') {
                    Some((name, n)) => harness.set_input(name, Value::Int(n.parse().unwrap())),
                    None => match input.strip_prefix('!') {
                        Some(name) => harness.set_input(name, false),
                        None => harness.set_input(input, true),
                    },
                }
            }
            harness.cycle();
            harness.advance_time(Duration::from_millis(10));
            let values: Vec<String> = watch
                .iter()
                .map(|name| match harness.get_output(name) {
                    Some(Value::Bool(b)) => format!("{name}={}", u8::from(b)),
                    Some(Value::Int(n)) => format!("{name}={n}"),
                    other => format!("{name}={other:?}"),
                })
                .collect();
            values.join(" ")
        })
        .collect()
}

/// The BOOLs random programs read and write.
pub(crate) const VARIABLES: [&str; 5] = ["V0", "V1", "V2", "V3", "V4"];

fn variable() -> impl Strategy<Value = Option<String>> {
    proptest::sample::select(&VARIABLES[..]).prop_map(|v| Some(v.to_string()))
}

fn element() -> impl Strategy<Value = Element> {
    let contact = (
        prop_oneof![
            Just(ContactMode::Open),
            Just(ContactMode::Closed),
            Just(ContactMode::Rising),
            Just(ContactMode::Falling)
        ],
        variable(),
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
        variable(),
    )
        .prop_map(|(mode, operand)| Element::Coil { mode, operand });
    prop_oneof![2 => contact, 1 => coil].prop_recursive(3, 24, 4, |inner| {
        proptest::collection::vec(proptest::collection::vec(inner, 0..4), 2..4)
            .prop_map(|paths| Element::Branch { paths })
    })
}

pub(crate) fn program() -> impl Strategy<Value = Program> {
    proptest::collection::vec(proptest::collection::vec(element(), 1..5), 1..4).prop_map(|rungs| {
        Program {
            declarations: format!("VAR {} : BOOL; END_VAR", VARIABLES.join(", ")),
            rungs: rungs
                .into_iter()
                .map(|elements| Rung {
                    comment: None,
                    label: None,
                    elements,
                    exit: None,
                })
                .collect(),
        }
    })
}
