//! The `<LD>` graph's drawing: what a tool that draws TC6 as written (Beremiz, OpenPLC) shows must read as the rung. The
//! round trip (`plcopen_read_tests.rs`) checks what a rung means; these check where it is drawn: wires that run left to
//! right, a series link straight, pins on their elements' edges, nothing over anything else, every element between its
//! rung's rails, rungs apart, and each text inside its box.

use std::collections::HashMap;

use proptest::prelude::*;
use roxmltree::{Document, Node};

use super::ld_body;
use crate::parse::parse;
use crate::test_support::{lowered, program};

/// The width of a character and the height of a line a text's box must hold, in TC6's units (pixels in Beremiz).
const CHAR_W: i64 = 6;
const LINE_H: i64 = 12;

#[derive(Debug)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Debug)]
struct In {
    at: Point,
    from: Vec<(i64, Option<String>)>,
}

#[derive(Debug)]
struct Drawn {
    id: i64,
    tag: String,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
    ins: Vec<In>,
    outs: Vec<(Option<String>, Point)>,
    /// the text drawn inside its box, line by line
    text: Vec<String>,
}

fn num(node: Node, name: &str) -> i64 {
    node.attribute(name)
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("{name} on {node:?}"))
}

fn kid<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|c| c.has_tag_name(name))
}

fn rel(point: Node, x: i64, y: i64) -> Point {
    let at = kid(point, "relPosition").expect("relPosition");
    Point {
        x: x + num(at, "x"),
        y: y + num(at, "y"),
    }
}

fn point_in(point: Node, x: i64, y: i64) -> In {
    let from = (point.children())
        .filter(|c| c.has_tag_name("connection"))
        .map(|c| {
            let param = c.attribute("formalParameter").map(str::to_string);
            (num(c, "refLocalId"), param)
        })
        .collect();
    In {
        at: rel(point, x, y),
        from,
    }
}

fn drawn(node: Node) -> Drawn {
    let position = kid(node, "position").expect("position");
    let (x, y) = (num(position, "x"), num(position, "y"));
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    if let Some(point) = kid(node, "connectionPointIn") {
        ins.push(point_in(point, x, y));
    }
    if let Some(point) = kid(node, "connectionPointOut") {
        outs.push((None, rel(point, x, y)));
    }
    for (list, input) in [("inputVariables", true), ("outputVariables", false)] {
        for variable in kid(node, list).iter().flat_map(Node::children) {
            let param = variable.attribute("formalParameter").map(str::to_string);
            if let Some(point) = kid(variable, "connectionPointIn").filter(|_| input) {
                ins.push(point_in(point, x, y));
            }
            if let Some(point) = kid(variable, "connectionPointOut").filter(|_| !input) {
                outs.push((param, rel(point, x, y)));
            }
        }
    }
    let text = match node.tag_name().name() {
        "comment" => (node.descendants().find(|d| d.has_tag_name("p")))
            .and_then(|p| p.text())
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect(),
        "label" => vec![node.attribute("label").unwrap_or_default().to_string()],
        "jump" => vec![format!(">>{}", node.attribute("label").unwrap_or_default())],
        "return" => vec!["RETURN".into()],
        "inVariable" => vec![
            kid(node, "expression")
                .and_then(|e| e.text())
                .unwrap_or_default()
                .to_string(),
        ],
        "block" => vec![node.attribute("typeName").unwrap_or_default().to_string()],
        _ => Vec::new(),
    };
    Drawn {
        id: num(node, "localId"),
        tag: node.tag_name().name().to_string(),
        x,
        y,
        w: num(node, "width"),
        h: num(node, "height"),
        ins,
        outs,
        text,
    }
}

/// The first thing in `body`'s drawing that would not read as its rungs.
fn misdrawn(body: &str) -> Option<String> {
    let xml = format!("<LD xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">{body}</LD>");
    let document = Document::parse(&xml).expect("XML");
    let all: Vec<Drawn> = document
        .root_element()
        .children()
        .filter(Node::is_element)
        .map(drawn)
        .collect();
    let by_id: HashMap<i64, &Drawn> = all.iter().map(|d| (d.id, d)).collect();
    let mut uses: HashMap<(i64, Option<String>), usize> = HashMap::new();
    for link in all.iter().flat_map(|d| &d.ins).flat_map(|i| &i.from) {
        *uses.entry(link.clone()).or_default() += 1;
    }
    for d in &all {
        // pins on the edges: inputs on the left, outputs on the right
        for i in &d.ins {
            if i.at.x != d.x || !(d.y..=d.y + d.h).contains(&i.at.y) {
                return Some(format!(
                    "{} {}: an input off its left edge at {:?}",
                    d.tag, d.id, i.at
                ));
            }
        }
        for (_, o) in &d.outs {
            if o.x != d.x + d.w || !(d.y..=d.y + d.h).contains(&o.y) {
                return Some(format!(
                    "{} {}: an output off its right edge at {o:?}",
                    d.tag, d.id
                ));
            }
        }
        for line in &d.text {
            let (chars, lines) = (line.chars().count() as i64, d.text.len() as i64);
            if d.w < chars * CHAR_W || d.h < lines * LINE_H {
                return Some(format!(
                    "{} {}: {line:?} does not fit {}x{}",
                    d.tag, d.id, d.w, d.h
                ));
            }
        }
        // wires: left to right, and a series link straight
        for i in &d.ins {
            for (from, param) in &i.from {
                let producer = by_id.get(from)?;
                let wanted = if producer.tag == "block" {
                    param.clone()
                } else {
                    None
                };
                let Some((_, out)) = producer.outs.iter().find(|(p, _)| *p == wanted) else {
                    return Some(format!(
                        "{} {}: fed by {from} {param:?}, which has no such output",
                        d.tag, d.id
                    ));
                };
                if out.x >= i.at.x {
                    return Some(format!("{} {}: fed from its right by {from}", d.tag, d.id));
                }
                // a rail is a bar: a path below its point joins it with an elbow, as a branch's fan-out does
                let series = i.from.len() == 1
                    && uses[&(*from, param.clone())] == 1
                    && producer.tag != "leftPowerRail";
                if series && out.y != i.at.y {
                    return Some(format!(
                        "{} {}: its series link from {from} is not straight",
                        d.tag, d.id
                    ));
                }
            }
        }
    }
    // nothing over anything else
    for (k, a) in all.iter().enumerate() {
        for b in &all[k + 1..] {
            if a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h {
                return Some(format!("{} {} and {} {} overlap", a.tag, a.id, b.tag, b.id));
            }
        }
    }
    // every element between its rung's rails, a label at its rung's wire, and rungs apart
    let mut rung: Option<&Drawn> = None;
    let mut bottom = i64::MIN;
    let mut label: Option<&Drawn> = None;
    for d in &all {
        match d.tag.as_str() {
            "comment" => {}
            "label" => label = Some(d),
            "leftPowerRail" => {
                if d.y <= bottom {
                    return Some(format!("the rung at {} touches the one above it", d.y));
                }
                let wire = d.outs[0].1.y;
                if let Some(l) = label.take().filter(|l| l.y + l.h / 2 != wire) {
                    return Some(format!("label {} is not at its rung's wire {wire}", l.id));
                }
                rung = Some(d);
            }
            _ => {
                let rail = rung?;
                if d.x < rail.x + rail.w || d.y < rail.y || d.y + d.h > rail.y + rail.h {
                    return Some(format!("{} {} is outside its rung's rails", d.tag, d.id));
                }
                if d.tag == "rightPowerRail" {
                    if d.y != rail.y || d.h != rail.h {
                        return Some(format!("rung at {}: its rails differ", rail.y));
                    }
                    bottom = rail.y + rail.h;
                }
            }
        }
    }
    None
}

fn body_of(ld: &str) -> String {
    ld_body(&parse(ld).unwrap(), &lowered(ld).boxes, "").unwrap()
}

const DECLARED: &str = "VAR A, B, C, X, Y, Z, Q : BOOL; N : INT; T1 : TON; C1 : CTU; END_VAR\n";

#[test]
fn every_kind_of_rung_is_drawn_as_it_reads() {
    for rungs in [
        "[|A|, |X|] |/B| (X);",
        "|A| [|B| [|C|, ] (S Y), |/C| (R Y)] (P Z);",
        "|A| T1(IN, PT := T#1500MS) (X) (/Y);",
        "|A| C1(CU, R := B, PV := 10, CV => N) (X);",
        "|N A| ADD(IN1 := N, IN2 := 1, OUT => N) (Q);",
        "EQ(IN1 := N, IN2 := 2, OUT) |B| (X);\n[GT(IN1 := N, IN2 := 0, OUT), |C|] (Y);",
        "[|A|, |B|];\n|C|;\n;",
        "(* starts\n   the whole line *)\nTop: |A| (X) >>Done;\n(* skipped *)\n|B| (Y);\nDone: |C| RETURN;\n(Y);",
        "|A| (X) (Y) >>Far_away_label;\nFar_away_label: ;",
    ] {
        let body = body_of(&format!("{DECLARED}{rungs}"));
        assert_eq!(misdrawn(&body), None, "{rungs}\n{body}");
    }
}

#[test]
fn the_checks_catch_a_drawing_that_would_misread() {
    let body = body_of(&format!("{DECLARED}|A| (X);\n|B| (Y);"));
    let moved = |from: &str, to: &str| misdrawn(&body.replacen(from, to, 1));
    assert!(
        moved(
            "<position x=\"72\" y=\"54\"/>",
            "<position x=\"72\" y=\"60\"/>"
        )
        .is_some()
    );
    assert!(
        moved(
            "<relPosition x=\"21\" y=\"10\"/>",
            "<relPosition x=\"20\" y=\"10\"/>"
        )
        .is_some()
    );
}

/// Every kind of element, a comment, a label, a jump, a RETURN and boxes with values, drawn as the writer draws them:
/// the file is the drawing, checked once by eye in Beremiz's terms (positions, sizes, pins) and by `misdrawn`.
const GOLDEN_LD: &str = "(* starts\n   the line *)\nTop: |A| [|/B|, |P C|] T1(IN, PT := T#1500MS) (S X) >>Done;\n\
                         EQ(IN1 := N, IN2 := 2, OUT) C1(CU, R := B, PV := 10, CV => N) (Y) (N Z);\nDone: |C| RETURN;\n";

#[test]
fn the_drawing_is_the_one_the_golden_file_holds() {
    let body = body_of(&format!("{DECLARED}{GOLDEN_LD}"));
    assert_eq!(misdrawn(&body), None, "{body}");
    assert_eq!(body, include_str!("plcopen_write_golden.xml").trim_end());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn random_programs_are_drawn_as_they_read(program in program()) {
        let ld = crate::print::print(&program);
        let body = body_of(&ld);
        prop_assert_eq!(misdrawn(&body), None, "{}\n{}", ld, body);
    }
}
