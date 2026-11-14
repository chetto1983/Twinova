//! Online monitoring of a Ladder file (spec 2026-11-13-ladder-editor-design.md, decisions 5 and 6): what the drawing
//! shows after each scan, in the order the engine reads it. Each element's power out is the BOOL the lowering holds it
//! in (`Main.LD_P3`), so a wire is drawn energised exactly as the program ran; each contact's and coil's operand, and
//! each box pin bound to a reference, shows its value. The file is read as it was built, as an ST file is: an edited
//! tab is not monitored until it is built (`monitor.rs`).

use serde::Serialize;
use twinova_engine::controller::Target;
use twinova_engine::program_files::ProgramFiles;
use twinova_engine::watch::{Scope, Watch};
use twinova_ladder::{Element, ElementId, parse, program_name};

use crate::declarations::declarations;

/// A value the drawing shows: an element's power out, its operand's value, or a box pin's.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LadderSpot {
    pub element: ElementId,
    /// None for the element's power or operand, else the box pin the value is drawn at
    pub pin: Option<String>,
    pub power: bool,
    /// What a click writes or forces: a variable of this file's program; a global is not written from here.
    pub target: Option<Target>,
}

fn reference(text: &str) -> bool {
    let mut parts = text.split('.');
    parts.all(|p| {
        let mut chars = p.chars();
        chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }) && !text.eq_ignore_ascii_case("TRUE")
        && !text.eq_ignore_ascii_case("FALSE")
}

/// Every element of `elements` with its path, branches' paths in turn.
fn walk<'a>(elements: &'a [Element], path: &[usize], out: &mut Vec<(Vec<usize>, &'a Element)>) {
    for (i, element) in elements.iter().enumerate() {
        let at = [path, &[i]].concat();
        if let Element::Branch { paths } = element {
            for (j, p) in paths.iter().enumerate() {
                walk(p, &[at.as_slice(), &[j]].concat(), out);
            }
        }
        out.push((at, element));
    }
}

/// What Ladder file `file` of the built `program` shows, `only` being its declarations as the frontend reads them:
/// the powers first, in the lowering's order, then the operands and pins, rung by rung.
pub(crate) fn ladder_spots(
    program: &[(String, String)],
    file: &str,
    only: &str,
) -> Result<Vec<(LadderSpot, Watch)>, String> {
    let pou = program_name(file)?;
    let text = (program.iter())
        .find(|(f, _)| f == file)
        .map(|(_, t)| t)
        .ok_or_else(|| format!("{file} is not built"))?;
    let rungs = parse(text).map_err(|e| e.message)?.rungs;
    let lowered = ProgramFiles::ladder(program, file)?;
    // a declaration's row names all it declares, `start, run`
    let local: Vec<String> = (declarations(only).iter())
        .flat_map(|d| {
            d.name
                .split(',')
                .map(|n| n.trim().to_ascii_uppercase())
                .collect::<Vec<_>>()
        })
        .collect();
    // a name the file declares is its program's; any other is a global
    let watched = |text: &str| -> (Watch, Option<Target>) {
        let base = text
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        if local.contains(&base) {
            (
                Watch::Path(Scope::Plc, format!("{pou}.{text}")),
                Some(Target::variable(pou, text)),
            )
        } else {
            (Watch::Path(Scope::Plc, text.to_string()), None)
        }
    };
    let mut out: Vec<(LadderSpot, Watch)> = (lowered.power.into_iter())
        .map(|p| {
            let watch = if p.variable == "TRUE" {
                Watch::Condition(Scope::Plc, "TRUE".into())
            } else {
                Watch::Path(Scope::Plc, format!("{pou}.{}", p.variable))
            };
            let spot = LadderSpot {
                element: p.element,
                pin: None,
                power: true,
                target: None,
            };
            (spot, watch)
        })
        .collect();
    for (r, rung) in rungs.iter().enumerate() {
        let mut elements = Vec::new();
        walk(&rung.elements, &[], &mut elements);
        for (path, element) in elements {
            let element_id = |pin: Option<String>| {
                (
                    ElementId {
                        rung: r,
                        path: path.clone(),
                    },
                    pin,
                )
            };
            let shown: Vec<(Option<String>, &str)> = match element {
                Element::Contact {
                    operand: Some(o), ..
                }
                | Element::Coil {
                    operand: Some(o), ..
                } => {
                    vec![(None, o.as_str())]
                }
                Element::Box { pins, .. } => (pins.iter())
                    .filter_map(|p| Some((Some(p.name.clone()), p.value.as_deref()?)))
                    .collect(),
                _ => Vec::new(),
            };
            for (pin, value) in shown.into_iter().filter(|(_, v)| reference(v)) {
                let (element, pin) = element_id(pin);
                let (watch, target) = watched(value);
                out.push((
                    LadderSpot {
                        element,
                        pin,
                        power: false,
                        target,
                    },
                    watch,
                ));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "monitor_ladder_tests.rs"]
mod tests;
