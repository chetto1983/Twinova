//! A box on a rung, as OIP lowers one (NOTICE). truST checks its pins' names and types: the lowering only places the
//! rung's power on the box and the box's output on the rung.

use super::{Lowering, OPERATORS, RungLowerer};
use crate::lower::{BlockPin, Field};
use crate::model::Pin;

fn named(pin: &Pin, name: &str) -> bool {
    pin.name.eq_ignore_ascii_case(name)
}

impl RungLowerer<'_> {
    /// A function sits on the rung by EN and ENO, its result going where `OUT =>` says; or it starts the rung, its OUT
    /// bare, and its BOOL result is the rung's power.
    pub(super) fn function(
        &mut self,
        head: &str,
        pins: &[Pin],
        path: &[usize],
        power: &str,
        rail: bool,
    ) -> Lowering<String> {
        let starts = pins.iter().any(|p| p.value.is_none() && named(p, "OUT"));
        for p in pins.iter().filter(|p| p.value.is_none()) {
            if !["EN", "ENO", "OUT"].iter().any(|n| named(p, n)) {
                return Err(self.error(
                    path,
                    format!(
                        "a function sits on the rung by EN and ENO, or starts it by OUT: '{0}' needs a value ({0} := ...)",
                        p.name
                    ),
                ));
            }
            if starts && !named(p, "OUT") {
                return Err(self.error(
                    path,
                    format!(
                        "{head} starts the rung by OUT, so nothing feeds it and it has no {}",
                        p.name
                    ),
                ));
            }
        }
        if let Some(p) = pins.iter().find(|p| p.out && !named(p, "OUT")) {
            return Err(self.error(
                path,
                format!("a function has one output, OUT: '{}' is not it", p.name),
            ));
        }
        let target = pins.iter().position(|p| p.out);
        if starts {
            if let Some(t) = target {
                let bound = pins[t].value.as_deref().unwrap_or_default();
                return Err(self.error(
                    path,
                    format!(
                        "{head}'s OUT carries the rung, so it cannot also be written to {bound}"
                    ),
                ));
            }
            if !rail {
                return Err(self.error(
                    path,
                    format!(
                        "{head}(..., OUT) starts a rung: nothing feeds it, so it stands at the left rail, first on the \
                         rung or first on a path of a branch that is"
                    ),
                ));
            }
            let out = self.temp("LD_P");
            self.put(&format!("{out} := "));
            self.call(head, pins, path);
            self.put(";");
            self.end_line(path);
            return Ok(out);
        }
        // a function whose result goes nowhere does nothing: IEC functions have no side effects
        if let Some(t) = target {
            self.guarded(power, |me| {
                me.operand(
                    pins[t].value.as_deref().unwrap_or_default(),
                    path,
                    Field::Pin(t),
                );
                me.put(" := ");
                me.call(head, pins, path);
                me.put(";");
            });
            self.end_line(path);
        }
        Ok(power.into())
    }

    /// The call, its inputs named: truST takes named arguments to standard functions (measured, spec). AND, OR, XOR,
    /// MOD and NOT are written as the operators they are, their inputs in the order of their numbers.
    fn call(&mut self, head: &str, pins: &[Pin], path: &[usize]) {
        let mut inputs: Vec<usize> = (0..pins.len())
            .filter(|&i| !pins[i].out && pins[i].value.is_some())
            .collect();
        let Some(operator) = OPERATORS.iter().find(|o| o.eq_ignore_ascii_case(head)) else {
            self.operand(head, path, Field::Operand);
            self.put("(");
            for (n, &i) in inputs.iter().enumerate() {
                self.put(if n == 0 { "" } else { ", " });
                self.put(&format!("{} := ", pins[i].name));
                self.operand(
                    pins[i].value.as_deref().unwrap_or_default(),
                    path,
                    Field::Pin(i),
                );
            }
            self.put(")");
            return;
        };
        inputs.sort_by_key(|&i| pin_number(&pins[i].name));
        self.put(if *operator == "NOT" { "(NOT " } else { "(" });
        let between = format!(" {operator} ");
        for (n, &i) in inputs.iter().enumerate() {
            self.put(if n == 0 { "" } else { &between });
            self.operand(
                pins[i].value.as_deref().unwrap_or_default(),
                path,
                Field::Pin(i),
            );
        }
        self.put(")");
    }

    /// A function block's instance: the rung feeds one BOOL input and takes one BOOL output on. Its type tells a bare
    /// output from a bare input; a block whose type is not known names them in that order.
    pub(super) fn instance(
        &mut self,
        head: &str,
        pins: &[Pin],
        path: &[usize],
        power: &str,
        pins_of: Option<(&[BlockPin], &[BlockPin])>,
    ) -> Lowering<String> {
        let bare: Vec<&Pin> = pins.iter().filter(|p| p.value.is_none()).collect();
        let (ins, outs): (Vec<&Pin>, Vec<&Pin>) = match pins_of {
            Some((_, outputs)) => bare.iter().partition(|p| {
                !(named(p, "ENO") || outputs.iter().any(|o| o.name.eq_ignore_ascii_case(&p.name)))
            }),
            None => (
                bare.iter().take(1).copied().collect(),
                bare.iter().skip(1).copied().collect(),
            ),
        };
        if ins.len() > 1 || outs.len() > 1 {
            let names: Vec<&str> = bare.iter().map(|p| p.name.as_str()).collect();
            return Err(self.error(
                path,
                format!(
                    "a box sits on the rung by one input and one output: '{head}' names {}",
                    names.join(", ")
                ),
            ));
        }
        let first_bool = |list: Option<&[BlockPin]>| {
            list.and_then(|l| l.iter().find(|p| p.bool))
                .map(|p| p.name.clone())
        };
        let input = ins
            .first()
            .map(|p| p.name.clone())
            .or_else(|| first_bool(pins_of.map(|p| p.0)));
        let enabled = input
            .as_deref()
            .is_some_and(|i| i.eq_ignore_ascii_case("EN"));
        let output = outs.first().map(|p| p.name.clone()).or_else(|| {
            if enabled {
                Some("ENO".into())
            } else {
                first_bool(pins_of.map(|p| p.1))
            }
        });
        let (Some(input), Some(output)) = (input, output) else {
            return Err(self.error(
                path,
                format!("cannot tell which pins of '{head}' sit on the rung: name them bare, as in {head}(IN, ..., Q)"),
            ));
        };
        let fed = (!enabled).then_some(input.as_str());
        if enabled {
            self.guarded(power, |me| me.invoke(head, pins, path, None, power));
        } else {
            self.invoke(head, pins, path, fed, power);
        }
        self.end_line(path);
        if output.eq_ignore_ascii_case("ENO") {
            return Ok(power.into());
        }
        let out = self.temp("LD_P");
        self.put(&format!("{out} := "));
        self.operand(head, path, Field::Operand);
        self.put(&format!(".{output};"));
        self.end_line(path);
        Ok(out)
    }

    /// `T1(IN := power, PT := T#2s, ET => elapsed);`, the rung's power on `fed` when it feeds one.
    fn invoke(&mut self, head: &str, pins: &[Pin], path: &[usize], fed: Option<&str>, power: &str) {
        self.operand(head, path, Field::Operand);
        self.put("(");
        let mut first = true;
        if let Some(input) = fed {
            self.put(&format!("{input} := {power}"));
            first = false;
        }
        for (i, pin) in pins.iter().enumerate() {
            let Some(value) = &pin.value else { continue };
            self.put(if first { "" } else { ", " });
            first = false;
            self.put(&format!(
                "{} {} ",
                pin.name,
                if pin.out { "=>" } else { ":=" }
            ));
            self.operand(value, path, Field::Pin(i));
        }
        self.put(");");
    }
}

/// The number a function's input ends in: `IN2` is 2, `IN` none.
fn pin_number(name: &str) -> usize {
    let digits = name.trim_start_matches(|c: char| !c.is_ascii_digit());
    digits.parse().unwrap_or(0)
}
