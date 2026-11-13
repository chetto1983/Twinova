//! A program lowered to the Structured Text truST compiles, with Open Industry Project's semantics (NOTICE):
//!
//! - each element is evaluated once, left to right, as the scan reaches it;
//! - a branch runs its paths holding a contact or a box first, then the paths of coils only, top to bottom;
//! - P and N elements remember their last value in a hidden BOOL;
//! - a jump is a flag that skips the rungs down to its label.
//!
//! Each element's power out is held in a hidden BOOL as the scan reaches it (spec decision 5, amended in Task 3), so
//! the online view draws the power the program ran with. Holding it there also captures every condition before a later
//! element writes what it read. The rung `|A| (/A) (B)` gives B the A the contact examined.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::lower_rung::{Part, RungLowerer};
use crate::model::{Exit, Program};

/// What a box's head names, as the project's declarations tell: a function, or a function block's instance with its
/// type's pins. A head the caller cannot resolve is lowered as an instance whose bare pins name the input, then the
/// output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Function,
    Instance {
        inputs: Vec<BlockPin>,
        outputs: Vec<BlockPin>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPin {
    pub name: String,
    pub bool: bool,
}

/// An element: its rung, then the indices down the tree, an element's then a path's in turn (`[2, 1, 0]` is the
/// first element of the second path of the rung's third element). An empty path is the rung itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId {
    pub rung: usize,
    pub path: Vec<usize>,
}

/// What part of an element some generated text copies: its operand (a box's head), or a pin's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Field {
    Operand,
    Pin(usize),
}

/// Where a place in the generated ST comes from, for a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// A byte offset in the program's declarations.
    Declarations(usize),
    Element(ElementId, Option<Field>),
}

/// The BOOL holding an element's power out, or `TRUE` where the left rail reaches it with nothing between.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Power {
    pub element: ElementId,
    pub variable: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerError {
    pub element: ElementId,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Segment {
    pub at: usize,
    pub len: usize,
    pub element: ElementId,
    pub field: Field,
}

/// The lowered program. A rung that cannot be lowered is left out and reported, so the others still run and show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lowered {
    pub st: String,
    pub power: Vec<Power>,
    pub errors: Vec<LowerError>,
    declarations: (usize, usize),
    segments: Vec<Segment>,
    /// Each generated line of the rungs: where it starts and the element it belongs to.
    lines: Vec<(usize, ElementId)>,
}

impl Lowered {
    pub fn origin(&self, offset: usize) -> Option<Origin> {
        let (start, len) = self.declarations;
        if (start..start + len).contains(&offset) {
            return Some(Origin::Declarations(offset - start));
        }
        if let Some(s) = self
            .segments
            .iter()
            .find(|s| (s.at..s.at + s.len).contains(&offset))
        {
            return Some(Origin::Element(s.element.clone(), Some(s.field)));
        }
        let line = self
            .lines
            .partition_point(|(at, _)| *at <= offset)
            .checked_sub(1)?;
        let end = self.st[self.lines[line].0..].find('\n')? + self.lines[line].0;
        (offset <= end).then(|| Origin::Element(self.lines[line].1.clone(), None))
    }
}

/// Lower `program` to `PROGRAM <name>`. `block` tells what a box's head names.
pub fn lower(program: &Program, name: &str, block: &dyn Fn(&str) -> Option<Block>) -> Lowered {
    let mut names = Names::of(program);
    let mut errors = Vec::new();
    let labels = labels(program, &mut errors);
    let mut flags: HashMap<String, String> = HashMap::new();
    let mut parts: Vec<(usize, Part, Option<String>)> = Vec::new();
    for (index, rung) in program.rungs.iter().enumerate() {
        let jump = match jump_of(index, rung.exit.as_ref(), &labels) {
            Ok(jump) => jump,
            Err(message) => {
                errors.push(rung_error(index, message));
                continue;
            }
        };
        let flag = jump.as_ref().map(|label| {
            flags
                .entry(label.clone())
                .or_insert_with(|| names.fresh("LD_J"))
                .clone()
        });
        match RungLowerer::new(index, block, &mut names).rung(rung, flag.as_deref()) {
            Ok(part) => parts.push((index, part, jump)),
            Err(error) => errors.push(error),
        }
    }
    let body = guard_jumps(program, parts, &flags);
    assemble(program, name, body, flags, errors)
}

fn rung_error(rung: usize, message: String) -> LowerError {
    LowerError {
        element: ElementId {
            rung,
            path: Vec::new(),
        },
        message,
    }
}

fn labels(program: &Program, errors: &mut Vec<LowerError>) -> HashMap<String, usize> {
    let mut labels = HashMap::new();
    for (index, rung) in program.rungs.iter().enumerate() {
        let Some(label) = &rung.label else { continue };
        if let Some(first) = labels.get(&label.to_uppercase()) {
            errors.push(rung_error(
                index,
                format!(
                    "label '{label}' is already on rung {}, so a jump to it could mean either",
                    first + 1
                ),
            ));
        } else {
            labels.insert(label.to_uppercase(), index);
        }
    }
    labels
}

/// The label a rung jumps to, folded. ST has no GOTO, so only jumps down the program can be lowered.
fn jump_of(
    index: usize,
    exit: Option<&Exit>,
    labels: &HashMap<String, usize>,
) -> Result<Option<String>, String> {
    let Some(Exit::Jump { label }) = exit else {
        return Ok(None);
    };
    match labels.get(&label.to_uppercase()) {
        None => Err(format!("there is no rung labelled '{label}' to jump to")),
        Some(&to) if to <= index => Err(format!(
            "a jump back up to '{label}' (rung {}) would run rungs again in one scan, a loop: a jump goes only down",
            to + 1
        )),
        Some(_) => Ok(Some(label.to_uppercase())),
    }
}

/// The rungs in order, each skipped while a jump over it is taken. Every flag is cleared at the top of the program,
/// set by its jump to the jump rung's power, and its label ends it (that rung runs).
fn guard_jumps(
    program: &Program,
    parts: Vec<(usize, Part, Option<String>)>,
    flags: &HashMap<String, String>,
) -> Vec<Part> {
    let mut out = Vec::new();
    if !flags.is_empty() {
        let mut sorted: Vec<&String> = flags.values().collect();
        sorted.sort();
        let clear: Vec<String> = sorted.iter().map(|f| format!("{f} := FALSE;")).collect();
        out.push(Part::scaffold(&clear.join(" "), 0));
    }
    let mut active: Vec<String> = Vec::new();
    let mut parts = parts.into_iter().peekable();
    for (index, rung) in program.rungs.iter().enumerate() {
        if let Some(label) = &rung.label {
            active.retain(|l| *l != label.to_uppercase());
        }
        let Some((_, part, jump)) = parts.next_if(|(at, _, _)| *at == index) else {
            continue;
        };
        if active.is_empty() || part.text.is_empty() {
            out.push(part);
        } else {
            let taken: Vec<&str> = active.iter().map(|l| flags[l].as_str()).collect();
            out.push(Part::scaffold(
                &format!("IF NOT ({}) THEN", taken.join(" OR ")),
                index,
            ));
            out.push(part);
            out.push(Part::scaffold("END_IF;", index));
        }
        if let Some(jump) = jump.filter(|j| !active.contains(j)) {
            active.push(jump);
        }
    }
    out
}

fn assemble(
    program: &Program,
    name: &str,
    body: Vec<Part>,
    flags: HashMap<String, String>,
    errors: Vec<LowerError>,
) -> Lowered {
    let mut st = format!("PROGRAM {name}\n");
    let declarations = program.declarations.trim();
    let declarations_at = st.len();
    if !declarations.is_empty() {
        st.push_str(declarations);
        st.push('\n');
    }
    let mut temps: Vec<String> = flags.into_values().collect();
    temps.sort();
    temps.extend(body.iter().flat_map(|p| p.temps.iter().cloned()));
    if !temps.is_empty() {
        st.push_str("VAR\n");
        for temp in &temps {
            st.push_str(&format!("    {temp} : BOOL;\n"));
        }
        st.push_str("END_VAR\n");
    }
    let mut lowered = Lowered {
        st,
        power: Vec::new(),
        errors,
        declarations: (declarations_at, declarations.len()),
        segments: Vec::new(),
        lines: Vec::new(),
    };
    for part in body {
        let shift = lowered.st.len();
        lowered.st.push_str(&part.text);
        lowered
            .segments
            .extend(part.segments.into_iter().map(|s| Segment {
                at: s.at + shift,
                ..s
            }));
        lowered
            .lines
            .extend(part.lines.into_iter().map(|(at, e)| (at + shift, e)));
        lowered.power.extend(part.power);
    }
    lowered.st.push_str("END_PROGRAM\n");
    lowered
}

/// Hidden names that no identifier of the program uses, in any case: `LD_P1`, `LD_M1`, `LD_J1`…
pub(crate) struct Names {
    taken: HashSet<String>,
    next: HashMap<&'static str, usize>,
}

impl Names {
    fn of(program: &Program) -> Self {
        let text = crate::print(program).to_uppercase();
        let taken = text
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .filter(|word| !word.is_empty())
            .map(str::to_string)
            .collect();
        Names {
            taken,
            next: HashMap::new(),
        }
    }

    pub(crate) fn fresh(&mut self, stem: &'static str) -> String {
        loop {
            let n = self.next.entry(stem).or_insert(0);
            *n += 1;
            let name = format!("{stem}{n}");
            if self.taken.insert(name.clone()) {
                return name;
            }
        }
    }
}

#[cfg(test)]
#[path = "lower_tests.rs"]
mod tests;
