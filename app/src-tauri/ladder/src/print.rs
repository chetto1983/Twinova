//! A program as `.ld` text, in its one canonical form: what `parse` reads back to the same program.

use crate::model::{CoilMode, ContactMode, Element, Exit, Pin, Program, Rung};

pub fn print(program: &Program) -> String {
    let mut out = String::new();
    let declarations = program.declarations.trim();
    if !declarations.is_empty() {
        out.push_str(declarations);
        out.push_str("\n\n");
    }
    for rung in &program.rungs {
        print_rung(rung, &mut out);
    }
    out
}

fn print_rung(rung: &Rung, out: &mut String) {
    if let Some(comment) = &rung.comment {
        out.push_str("(* ");
        out.push_str(&comment.replace('\n', "\n   "));
        out.push_str(" *)\n");
    }
    if let Some(label) = &rung.label {
        out.push_str(label);
        out.push_str(": ");
    }
    out.push_str(&series(&rung.elements));
    match &rung.exit {
        Some(Exit::Jump { label }) => {
            out.push_str(if rung.elements.is_empty() {
                ">>"
            } else {
                " >>"
            });
            out.push_str(label);
        }
        Some(Exit::Return) => out.push_str(if rung.elements.is_empty() {
            "RETURN"
        } else {
            " RETURN"
        }),
        None => {}
    }
    out.push_str(";\n");
}

fn series(elements: &[Element]) -> String {
    elements.iter().map(element).collect::<Vec<_>>().join(" ")
}

fn operand(text: &Option<String>) -> &str {
    text.as_deref().unwrap_or("?")
}

fn element(e: &Element) -> String {
    match e {
        Element::Contact { mode, operand: o } => {
            let modifier = match mode {
                ContactMode::Open => "",
                ContactMode::Closed => "/",
                ContactMode::Rising => "P ",
                ContactMode::Falling => "N ",
            };
            format!("|{modifier}{}|", operand(o))
        }
        Element::Coil { mode, operand: o } => {
            let modifier = match mode {
                CoilMode::Normal => "",
                CoilMode::Negated => "/",
                CoilMode::Set => "S ",
                CoilMode::Reset => "R ",
                CoilMode::Rising => "P ",
                CoilMode::Falling => "N ",
            };
            format!("({modifier}{})", operand(o))
        }
        Element::Box { head, pins } => {
            let pins: Vec<String> = pins.iter().map(pin).collect();
            format!("{}({})", operand(head), pins.join(", "))
        }
        Element::Branch { paths } => {
            let paths: Vec<String> = paths.iter().map(|p| series(p)).collect();
            format!("[{}]", paths.join(", "))
        }
    }
}

fn pin(p: &Pin) -> String {
    match (&p.value, p.out) {
        (None, _) => p.name.clone(),
        (Some(value), false) => format!("{} := {value}", p.name),
        (Some(value), true) => format!("{} => {value}", p.name),
    }
}
