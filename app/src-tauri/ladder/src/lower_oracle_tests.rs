//! Random rungs of contacts, coils and nested branches, lowered and run in truST, against an interpreter of the tree
//! written here from the rules alone: each element evaluated once, left to right, as the scan reaches it; a branch's
//! paths holding a contact first, then its coil-only paths; a P or N element remembering its last value.

use std::collections::HashMap;

use proptest::prelude::*;

use crate::lower::lower;
use crate::model::{CoilMode, ContactMode, Element, Program};
use crate::test_support::{VARIABLES, program, trace};

#[derive(Default)]
struct Oracle {
    values: HashMap<String, bool>,
    memory: HashMap<Vec<usize>, bool>,
}

impl Oracle {
    fn scan(&mut self, program: &Program) {
        for (rung, r) in program.rungs.iter().enumerate() {
            self.series(&r.elements, &[rung], true);
        }
    }

    fn remember(&mut self, path: &[usize], now: bool) -> bool {
        self.memory.insert(path.to_vec(), now).unwrap_or(false)
    }

    fn series(&mut self, elements: &[Element], prefix: &[usize], mut power: bool) -> bool {
        for (i, element) in elements.iter().enumerate() {
            let path = [prefix, &[i]].concat();
            match element {
                Element::Contact { mode, operand } => {
                    let value = self.values[operand.as_deref().unwrap()];
                    // the memory follows the variable on every evaluation
                    let was = self.remember(&path, value);
                    power &= match mode {
                        ContactMode::Open => value,
                        ContactMode::Closed => !value,
                        ContactMode::Rising => value && !was,
                        ContactMode::Falling => !value && was,
                    };
                }
                Element::Coil { mode, operand } => {
                    let name = operand.clone().unwrap();
                    let old = self.values[&name];
                    let was = self.remember(&path, power);
                    let new = match mode {
                        CoilMode::Normal => power,
                        CoilMode::Negated => !power,
                        CoilMode::Set => old || power,
                        CoilMode::Reset => old && !power,
                        CoilMode::Rising => power && !was,
                        CoilMode::Falling => !power && was,
                    };
                    self.values.insert(name, new);
                }
                Element::Branch { paths } => {
                    let coils_only = |p: &Vec<Element>| {
                        !p.is_empty() && p.iter().all(|e| matches!(e, Element::Coil { .. }))
                    };
                    let mut any = false;
                    for pass in [false, true] {
                        for (j, p) in paths
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| coils_only(p) == pass)
                        {
                            any |= self.series(p, &[&path[..], &[j]].concat(), power);
                        }
                    }
                    power = any;
                }
                Element::Box { .. } => unreachable!("no boxes generated"),
            }
        }
        power
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn random_rungs_run_as_the_rules_say(program in program(), inputs in proptest::collection::vec(0u8..4, 1..16)) {
        let lowered = lower(&program, "Ld", &|_| None);
        prop_assert_eq!(&lowered.errors, &[]);
        // V0 and V1 are inputs, set before each scan; the coils write any of them
        let steps: Vec<String> = inputs
            .iter()
            .map(|bits| format!("{}V0 {}V1", if bits & 1 == 0 { "!" } else { "" }, if bits & 2 == 0 { "!" } else { "" }))
            .collect();
        let steps: Vec<&str> = steps.iter().map(String::as_str).collect();
        let ran = trace(&[&lowered.st], &steps, &VARIABLES);
        let mut oracle = Oracle {
            values: VARIABLES.iter().map(|v| (v.to_string(), false)).collect(),
            ..Oracle::default()
        };
        let expected: Vec<String> = inputs
            .iter()
            .map(|bits| {
                oracle.values.insert("V0".into(), bits & 1 != 0);
                oracle.values.insert("V1".into(), bits & 2 != 0);
                oracle.scan(&program);
                let values: Vec<String> = VARIABLES.iter().map(|v| format!("{v}={}", u8::from(oracle.values[*v]))).collect();
                values.join(" ")
            })
            .collect();
        prop_assert_eq!(ran, expected, "{}", lowered.st);
    }
}
