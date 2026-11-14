//! A program's rungs as the `<LD>` body of a PLCopen TC6 POU, ported from Open Industry Project's writer (NOTICE).
//!
//! TC6 draws LD as a graph, not as rungs: every element has a `localId`, and its `connectionPointIn` lists the
//! elements whose outputs feed it. A series is a chain of such links; a branch is a fan-out (one output feeding the
//! first element of each path) and a wired OR (one input listing the last element of every path). A box's pins are
//! `inputVariables` and `outputVariables` named by `formalParameter`. Each rung is drawn between its own pair of power
//! rails, so a network is one connected graph.
//!
//! OIP's three choices keep the graph plain for any reader: a value on an input comes from an `inVariable`; an output
//! bound to a place names it on its own pin; and coils in series at the end of a rung are drawn in parallel, which means
//! the same, since a coil passes its power on unchanged. Positions are a plain grid: a column per element, a row per
//! branch path. The schema requires them, and the editors that draw TC6 as written (Beremiz, OpenPLC) use them.

use crate::lower::{BoxPins, ElementId};
use crate::model::{CoilMode, ContactMode, Element, Exit, Pin, Program, Rung};

const COL: usize = 96;
const ROW: usize = 64;
const LEFT: usize = 32;
const RAIL_GAP: usize = 40;
const SYMBOL_W: usize = 21;
const SYMBOL_H: usize = 20;
const BOX_W: usize = 64;
const PIN_STEP: usize = 20;

pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Where a link comes from: an element, and for a box its output pin.
#[derive(Debug, Clone, PartialEq)]
struct Producer {
    id: usize,
    param: Option<String>,
}

/// The `<LD>` body's contents for `program`, one line per tag, each line led by `indent`. `boxes` are the lowering's
/// (`Lowered::boxes`): every box of the program must have lowered.
pub fn ld_body(
    program: &Program,
    boxes: &[(ElementId, BoxPins)],
    indent: &str,
) -> Result<String, String> {
    let mut writer = Writer {
        lines: Vec::new(),
        next_id: 1,
        indent,
        boxes,
        rung: 0,
    };
    let mut top = 32;
    for (index, rung) in program.rungs.iter().enumerate() {
        writer.rung = index;
        top = writer.write_rung(rung, top)?;
    }
    Ok(writer.lines.join("\n"))
}

/// The variable of the contact an empty path is drawn as.
pub(crate) const WIRE: &str = "TRUE";

fn wire_contact() -> Element {
    Element::Contact {
        mode: ContactMode::Open,
        operand: Some(WIRE.into()),
    }
}

/// Columns and rows an element takes on the grid.
fn size_of(elements: &[Element]) -> (usize, usize) {
    (elements.iter().map(element_size)).fold((0, 1), |(cols, rows), (c, r)| (cols + c, rows.max(r)))
}

fn element_size(element: &Element) -> (usize, usize) {
    match element {
        Element::Branch { paths } => {
            let sizes: Vec<(usize, usize)> = paths.iter().map(|p| size_of(p)).collect();
            // a column more for the contact on TRUE that may close or open it
            let cols = sizes.iter().map(|s| s.0).max().unwrap_or(0).max(1) + 1;
            (cols, sizes.iter().map(|s| s.1).sum())
        }
        Element::Box { pins, .. } => {
            // the rung's pin counted for every box, a function's that starts the rung too: a row more at most
            let inputs = 1 + pins.iter().filter(|p| !p.out && p.value.is_some()).count();
            (2, ((inputs * PIN_STEP + PIN_STEP).div_ceil(ROW)).max(1))
        }
        _ => (1, 1),
    }
}

/// A box's pins as TC6 lists them: the one on the rung first, then those with a value, as written. A function that
/// starts the rung has no pin the rung feeds.
fn box_pins(pins: &[Pin], resolved: &BoxPins) -> (Vec<String>, Vec<String>) {
    let with_value = |out: bool| {
        (pins.iter())
            .filter(move |p| p.out == out && p.value.is_some())
            .map(|p| p.name.clone())
    };
    let inputs = resolved
        .input
        .iter()
        .cloned()
        .chain(with_value(false))
        .collect();
    let outputs = std::iter::once(resolved.output.clone())
        .chain(with_value(true))
        .collect();
    (inputs, outputs)
}

struct Writer<'a> {
    lines: Vec<String>,
    next_id: usize,
    indent: &'a str,
    boxes: &'a [(ElementId, BoxPins)],
    rung: usize,
}

impl Writer<'_> {
    fn id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id - 1
    }

    fn put(&mut self, depth: usize, line: &str) {
        self.lines
            .push(format!("{}{}{line}", self.indent, "  ".repeat(depth)));
    }

    fn point_in(&mut self, depth: usize, from: &[Producer], x: usize, y: usize) {
        self.put(depth, "<connectionPointIn>");
        self.put(depth + 1, &format!("<relPosition x=\"{x}\" y=\"{y}\"/>"));
        for p in from {
            let param = (p.param.as_ref()).map_or(String::new(), |p| {
                format!(" formalParameter=\"{}\"", esc(p))
            });
            self.put(
                depth + 1,
                &format!("<connection refLocalId=\"{}\"{param}/>", p.id),
            );
        }
        self.put(depth, "</connectionPointIn>");
    }

    fn point_out(&mut self, depth: usize, x: usize, y: usize) {
        self.put(depth, "<connectionPointOut>");
        self.put(depth + 1, &format!("<relPosition x=\"{x}\" y=\"{y}\"/>"));
        self.put(depth, "</connectionPointOut>");
    }

    /// One rung between its own rails, from `top`; where the next may start.
    fn write_rung(&mut self, rung: &Rung, top: usize) -> Result<usize, String> {
        let (cols, rows) = size_of(&rung.elements);
        let height = rows * ROW;
        // a rung's comment and its label stand ahead of it in the file: a reader gives each to the network after it
        if let Some(comment) = &rung.comment {
            let lines: Vec<&str> = comment.lines().collect();
            let width = lines.iter().map(|l| l.len()).max().unwrap_or(0) * 7 + 16;
            let id = self.id();
            self.put(
                0,
                &format!(
                    "<comment localId=\"{id}\" height=\"{}\" width=\"{width}\">",
                    lines.len() * 16
                ),
            );
            self.put(1, &format!("<position x=\"{LEFT}\" y=\"{top}\"/>"));
            self.put(
                1,
                &format!("<content><xhtml:p>{}</xhtml:p></content>", esc(comment)),
            );
            self.put(0, "</comment>");
        }
        if let Some(label) = &rung.label {
            let id = self.id();
            let width = label.len() * 8 + 16;
            self.put(
                0,
                &format!(
                    "<label localId=\"{id}\" height=\"20\" width=\"{width}\" label=\"{}\">",
                    esc(label)
                ),
            );
            self.put(
                1,
                &format!("<position x=\"0\" y=\"{}\"/>", top + ROW / 2 - 10),
            );
            self.put(0, "</label>");
        }
        let left = self.id();
        self.put(
            0,
            &format!("<leftPowerRail localId=\"{left}\" width=\"3\" height=\"{height}\">"),
        );
        self.put(1, &format!("<position x=\"{LEFT}\" y=\"{top}\"/>"));
        self.put(1, "<connectionPointOut formalParameter=\"\">");
        self.put(2, &format!("<relPosition x=\"3\" y=\"{}\"/>", ROW / 2));
        self.put(1, "</connectionPointOut>");
        self.put(0, "</leftPowerRail>");
        // Coils that end a rung with a jump or a RETURN after them are drawn beside it, fed from what feeds them: a
        // coil passes its power on unchanged, and editors that draw every output at the right end can take it.
        let coils = match rung.exit {
            Some(_) => (rung.elements.iter().rev())
                .take_while(|e| matches!(e, Element::Coil { .. }))
                .count(),
            None => 0,
        };
        let (head, tail) = rung.elements.split_at(rung.elements.len() - coils);
        let rail = vec![Producer {
            id: left,
            param: None,
        }];
        let mut x = LEFT + RAIL_GAP;
        // a rung that is one branch runs rail to rail on each path, which reads back as separate rungs: a contact on
        // TRUE at the rail gives the paths a node of their own, and leaves the coils at the right end
        let mut rail = rail;
        if rung.exit.is_none() && matches!(rung.elements[..], [Element::Branch { .. }]) {
            rail = self.element(&wire_contact(), &[], x, top, rail)?;
            x += COL;
        }
        let feed = self.series(head, &[], 0, x, top, rail)?;
        let out = if tail.is_empty() {
            feed.clone()
        } else {
            self.series(
                tail,
                &[],
                head.len(),
                x + size_of(head).0 * COL,
                top,
                feed.clone(),
            )?
        };
        let rail_x = x + cols * COL + RAIL_GAP;
        if let Some(exit) = &rung.exit {
            let from = if coils > 0 { feed } else { out.clone() };
            let (tag, label, width) = match exit {
                Exit::Jump { label } => (
                    "jump",
                    format!(" label=\"{}\"", esc(label)),
                    label.len() * 8 + 24,
                ),
                Exit::Return => ("return", String::new(), 56),
            };
            let id = self.id();
            self.put(
                0,
                &format!("<{tag} localId=\"{id}\" height=\"20\" width=\"{width}\"{label}>"),
            );
            self.put(
                1,
                &format!(
                    "<position x=\"{}\" y=\"{}\"/>",
                    rail_x - RAIL_GAP / 2,
                    top + ROW / 2 - 10
                ),
            );
            self.point_in(1, &from, 0, 10);
            self.put(0, &format!("</{tag}>"));
        }
        let right = self.id();
        self.put(
            0,
            &format!("<rightPowerRail localId=\"{right}\" width=\"3\" height=\"{height}\">"),
        );
        self.put(1, &format!("<position x=\"{rail_x}\" y=\"{top}\"/>"));
        if !out.is_empty() && (rung.exit.is_none() || coils > 0) {
            self.point_in(1, &out, 0, ROW / 2);
        }
        self.put(0, "</rightPowerRail>");
        Ok(top + height + ROW / 2)
    }

    /// `elements` from column `x`, the first at index `first` of the series at `prefix`.
    fn series(
        &mut self,
        elements: &[Element],
        prefix: &[usize],
        first: usize,
        x: usize,
        y: usize,
        mut feed: Vec<Producer>,
    ) -> Result<Vec<Producer>, String> {
        let mut at = x;
        for (i, element) in elements.iter().enumerate() {
            // coils in series at the end of a rung are drawn in parallel
            let rest = &elements[i..];
            if rest.len() > 1 && rest.iter().all(|e| matches!(e, Element::Coil { .. })) {
                let paths: Vec<Vec<Element>> = rest.iter().map(|e| vec![e.clone()]).collect();
                return self.branch(&paths, prefix, at, y, feed);
            }
            let path = [prefix, &[first + i]].concat();
            feed = self.element(element, &path, at, y, feed)?;
            at += element_size(element).0 * COL;
            // a branch ending a path of another would close on that branch's join, its paths read back as the
            // other's, and their order with them: a contact on TRUE after it keeps its join its own
            if !prefix.is_empty()
                && i + 1 == elements.len()
                && matches!(element, Element::Branch { .. })
            {
                feed = self.element(&wire_contact(), &path, at, y, feed)?;
            }
        }
        Ok(feed)
    }

    fn element(
        &mut self,
        element: &Element,
        path: &[usize],
        x: usize,
        y: usize,
        from: Vec<Producer>,
    ) -> Result<Vec<Producer>, String> {
        let (tag, operand, negated, edge, storage) = match element {
            Element::Branch { paths } => return self.branch(paths, path, x, y, from),
            Element::Box { head, pins } => {
                return self.block(head.as_deref().unwrap_or_default(), pins, path, x, y, from);
            }
            Element::Contact { mode, operand } => {
                let edge = match mode {
                    ContactMode::Rising => "rising",
                    ContactMode::Falling => "falling",
                    _ => "none",
                };
                (
                    "contact",
                    operand,
                    *mode == ContactMode::Closed,
                    edge,
                    "none",
                )
            }
            Element::Coil { mode, operand } => {
                let (edge, storage) = match mode {
                    CoilMode::Rising => ("rising", "none"),
                    CoilMode::Falling => ("falling", "none"),
                    CoilMode::Set => ("none", "set"),
                    CoilMode::Reset => ("none", "reset"),
                    _ => ("none", "none"),
                };
                ("coil", operand, *mode == CoilMode::Negated, edge, storage)
            }
        };
        let id = self.id();
        self.put(
            0,
            &format!(
                "<{tag} localId=\"{id}\" negated=\"{negated}\" edge=\"{edge}\" storage=\"{storage}\" \
                 width=\"{SYMBOL_W}\" height=\"{SYMBOL_H}\">"
            ),
        );
        self.put(
            1,
            &format!("<position x=\"{x}\" y=\"{}\"/>", y + ROW / 2 - SYMBOL_H / 2),
        );
        self.point_in(1, &from, 0, SYMBOL_H / 2);
        self.point_out(1, SYMBOL_W, SYMBOL_H / 2);
        self.put(
            1,
            &format!(
                "<variable>{}</variable>",
                esc(operand.as_deref().unwrap_or_default())
            ),
        );
        self.put(0, &format!("</{tag}>"));
        Ok(vec![Producer { id, param: None }])
    }

    fn branch(
        &mut self,
        paths: &[Vec<Element>],
        path: &[usize],
        x: usize,
        y: usize,
        from: Vec<Producer>,
    ) -> Result<Vec<Producer>, String> {
        let mut outs: Vec<Producer> = Vec::new();
        let mut row = y;
        for (j, elements) in paths.iter().enumerate() {
            // TC6 has no wire: one drawn as a link would put a path's two ends on one node, and the order the paths
            // run in could not be read back. An empty path is a contact on TRUE, which passes its power on unchanged,
            // and reads back as an empty path (`plcopen_read.rs`).
            let reached = if elements.is_empty() {
                self.element(&wire_contact(), path, x, row, from.clone())?
            } else {
                self.series(elements, &[path, &[j]].concat(), 0, x, row, from.clone())?
            };
            // one producer reached by two paths is still one connection
            for p in reached {
                if !outs.contains(&p) {
                    outs.push(p);
                }
            }
            row += size_of(elements).1 * ROW;
        }
        Ok(outs)
    }

    fn block(
        &mut self,
        head: &str,
        pins: &[Pin],
        path: &[usize],
        x: usize,
        y: usize,
        from: Vec<Producer>,
    ) -> Result<Vec<Producer>, String> {
        let element = ElementId {
            rung: self.rung,
            path: path.to_vec(),
        };
        let Some((_, resolved)) = self.boxes.iter().find(|(e, _)| *e == element) else {
            return Err(format!(
                "the box '{head}' did not lower, so its pins are not known"
            ));
        };
        let (inputs, outputs) = box_pins(pins, resolved);
        let bx = x + COL / 2;
        let by = y + ROW / 2 - PIN_STEP;
        let height = inputs.len().max(outputs.len()) * PIN_STEP + PIN_STEP;
        // a value on a pin is an element of its own, drawn beside the box; they follow the pin the rung feeds, if any,
        // in the order written
        let valued: Vec<(&str, &str)> = (pins.iter())
            .filter(|p| !p.out)
            .filter_map(|p| Some((p.name.as_str(), p.value.as_deref()?)))
            .collect();
        let fed = usize::from(resolved.input.is_some());
        let mut sources = Vec::new();
        for (i, (_, value)) in valued.iter().enumerate() {
            let v = self.id();
            sources.push(v);
            let width = (value.len() * 8).max(24);
            self.put(
                0,
                &format!("<inVariable localId=\"{v}\" width=\"{width}\" height=\"20\">"),
            );
            self.put(
                1,
                &format!(
                    "<position x=\"{}\" y=\"{}\"/>",
                    bx - COL / 2,
                    by + (fed + i + 1) * PIN_STEP - 10
                ),
            );
            self.point_out(1, width, 10);
            self.put(1, &format!("<expression>{}</expression>", esc(value)));
            self.put(0, "</inVariable>");
        }
        let id = self.id();
        let type_name = if resolved.type_name.is_empty() {
            head
        } else {
            &resolved.type_name
        };
        let instance = if resolved.function {
            String::new()
        } else {
            format!(" instanceName=\"{}\"", esc(head))
        };
        self.put(
            0,
            &format!(
                "<block localId=\"{id}\" width=\"{BOX_W}\" height=\"{height}\" typeName=\"{}\"{instance}>",
                esc(type_name)
            ),
        );
        self.put(1, &format!("<position x=\"{bx}\" y=\"{by}\"/>"));
        self.put(1, "<inputVariables>");
        for (i, name) in inputs.iter().enumerate() {
            let rel = (i + 1) * PIN_STEP;
            self.put(2, &format!("<variable formalParameter=\"{}\">", esc(name)));
            if i < fed {
                self.point_in(3, &from, 0, rel);
            } else {
                let source = Producer {
                    id: sources[i - fed],
                    param: None,
                };
                self.point_in(3, &[source], 0, rel);
            }
            self.put(2, "</variable>");
        }
        self.put(1, "</inputVariables>");
        self.put(1, "<inOutVariables/>");
        self.put(1, "<outputVariables>");
        for (i, name) in outputs.iter().enumerate() {
            // an output bound to a place names it on its own pin
            let bound = (pins.iter())
                .find(|p| p.out && p.name == *name)
                .and_then(|p| p.value.as_deref());
            self.put(2, &format!("<variable formalParameter=\"{}\">", esc(name)));
            self.put(3, "<connectionPointOut>");
            self.put(
                4,
                &format!("<relPosition x=\"{BOX_W}\" y=\"{}\"/>", (i + 1) * PIN_STEP),
            );
            if let Some(place) = bound {
                self.put(4, &format!("<expression>{}</expression>", esc(place)));
            }
            self.put(3, "</connectionPointOut>");
            self.put(2, "</variable>");
        }
        self.put(1, "</outputVariables>");
        self.put(0, "</block>");
        Ok(vec![Producer {
            id,
            param: Some(resolved.output.clone()),
        }])
    }
}
