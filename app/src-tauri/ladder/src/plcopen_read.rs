//! A PLCopen TC6 `<LD>` body read back into rungs, ported from Open Industry Project's reader (NOTICE).
//!
//! The graph is taken electrically: connection points joined by a link are one node (union-find), every output of a
//! left rail is the rail's node, and every contact, coil and box is an edge between two nodes (a function that starts
//! the rung, which nothing feeds, from the rail's node). A network is reduced as any series-parallel circuit is (edges
//! between the same two nodes are a branch; a node with one edge in and one out is a series) until one edge runs rail
//! to rail, and that edge is the rung.
//!
//! Order is the file's, not the positions': an exporter may write every position as 0,0. What cannot be a rung is
//! named and the POU left out, rather than half read: a bridge, a box with two pins on the rung, a pin with no name, a
//! connector. What TC6 lets a tool add for itself (`vendorElement`, `addData`, `documentation`) carries nothing a rung
//! runs and is skipped.

use std::collections::HashMap;

use roxmltree::Node;

/// Other names some tools give the standard blocks' pins, read back as IEC 61131-3's.
const PIN_ALIASES: [(&str, &str, &str); 8] = [
    ("CTU", "R", "RESET"),
    ("CTD", "LD", "LOAD"),
    ("CTUD", "R", "RESET"),
    ("CTUD", "LD", "LOAD"),
    ("SR", "S1", "SET1"),
    ("SR", "R", "RESET"),
    ("RS", "S", "SET"),
    ("RS", "R1", "RESET1"),
];

fn iec_pin_name(block_type: &str, pin: &str) -> String {
    (PIN_ALIASES.iter())
        .find(|(t, _, theirs)| {
            t.eq_ignore_ascii_case(block_type) && theirs.eq_ignore_ascii_case(pin)
        })
        .map_or_else(|| pin.to_string(), |(_, iec, _)| (*iec).to_string())
}

fn kids<'a, 'i>(
    node: Option<Node<'a, 'i>>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'i>> {
    (node.into_iter())
        .flat_map(|n| n.children())
        .filter(move |c| c.is_element() && c.tag_name().name() == name)
}

fn kid<'a, 'i>(node: Option<Node<'a, 'i>>, name: &'static str) -> Option<Node<'a, 'i>> {
    kids(node, name).next()
}

fn text_of(node: Option<Node>) -> String {
    let Some(node) = node else {
        return String::new();
    };
    let text: String = node
        .descendants()
        .filter(|d| d.is_text())
        .filter_map(|d| d.text())
        .collect();
    text.trim().to_string()
}

fn attr<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attribute(name)
}

/// The links a connection point lists: the element it comes from, and a box's output pin.
fn links_of(point: Option<Node>) -> Vec<(i64, String)> {
    (kids(point, "connection"))
        .map(|c| {
            let id = attr(c, "refLocalId")
                .and_then(|v| v.parse().ok())
                .unwrap_or(-1);
            (
                id,
                attr(c, "formalParameter").unwrap_or_default().to_string(),
            )
        })
        .collect()
}

/// A rung as a series-parallel tree. `at` is the place in the file of an element, which orders branches and rungs.
#[derive(Debug, Clone)]
enum Tree {
    El {
        text: String,
        at: usize,
    },
    Ser(Vec<Tree>),
    Par(Vec<Tree>),
    /// A branch the writer closed with a contact on TRUE, so that its paths keep a join of their own: beside other
    /// paths it stays one, nested, and its paths are not taken for theirs.
    Closed(Vec<Tree>),
}

impl Tree {
    fn at(&self) -> usize {
        match self {
            Tree::El { at, .. } => *at,
            Tree::Ser(items) | Tree::Par(items) | Tree::Closed(items) => {
                items.iter().map(Tree::at).min().unwrap_or(usize::MAX)
            }
        }
    }

    /// A contact on TRUE, which the writer draws an empty path as: it prints as nothing, so beside other paths it is
    /// an empty one, in its place.
    fn is_true(&self) -> bool {
        matches!(self, Tree::El { text, .. } if text.is_empty())
    }

    fn is_exit(&self) -> bool {
        matches!(self, Tree::El { text, .. } if text.starts_with(">>") || text == "RETURN")
    }

    fn print(&self) -> String {
        match self {
            Tree::El { text, .. } => text.clone(),
            Tree::Ser(items) => items.iter().map(Tree::print).collect::<Vec<_>>().join(" "),
            Tree::Par(items) | Tree::Closed(items) => format!(
                "[{}]",
                items.iter().map(Tree::print).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

fn wire() -> Tree {
    Tree::Ser(Vec::new())
}

fn ser(a: Tree, b: Tree) -> Tree {
    let flat = |t: Tree| match t {
        Tree::Ser(items) => items,
        other => vec![other],
    };
    let mut items = flat(a);
    items.extend(flat(b));
    // a contact on TRUE passes its power on unchanged: in a series it is nothing, but beside a branch it closes it
    if items.len() > 1 && items.iter().any(Tree::is_true) {
        items = (items.into_iter())
            .filter(|t| !t.is_true())
            .map(|t| match t {
                Tree::Par(paths) => Tree::Closed(paths),
                other => other,
            })
            .collect();
    }
    if items.len() == 1 {
        items.remove(0)
    } else {
        Tree::Ser(items)
    }
}

/// Paths in parallel, in the file's order. Edges between two nodes meet as the network reduces, so a parallel among
/// them is the same branch, taken apart; a closed one is a branch inside it, which the order its paths run in depends on
/// (`lower_rung::branch_order`).
fn par(all: Vec<Tree>) -> Tree {
    let mut items: Vec<Tree> = (all.into_iter())
        .flat_map(|t| match t {
            Tree::Par(items) => items,
            other => vec![other],
        })
        .collect();
    items.sort_by_key(Tree::at);
    // a jump or a RETURN drawn beside the coils that end its rung (the writer's shape) is the rung of the coils
    // followed by it: a coil passes its power on unchanged
    let (exits, rest): (Vec<Tree>, Vec<Tree>) = items.into_iter().partition(Tree::is_exit);
    let coils = rest
        .iter()
        .all(|t| matches!(t, Tree::El { text, .. } if text.starts_with('(')));
    if exits.len() == 1 && !rest.is_empty() && coils {
        let exit = exits.into_iter().next().expect("one exit");
        let before = if rest.len() == 1 {
            rest.into_iter().next().expect("one")
        } else {
            Tree::Par(rest)
        };
        return ser(before, exit);
    }
    let mut all = rest;
    all.extend(exits);
    all.sort_by_key(Tree::at);
    Tree::Par(all)
}

#[derive(Debug, Clone)]
struct Edge {
    from: String,
    to: String,
    tree: Tree,
}

const LEFT: &str = "rail:left";
const RIGHT: &str = "rail:right";

/// Connection points joined by links, as one node each.
#[derive(Default)]
struct Nodes(HashMap<String, String>);

impl Nodes {
    fn find(&mut self, key: &str) -> String {
        let mut root = key.to_string();
        while let Some(up) = self.0.get(&root).filter(|up| **up != root) {
            root = up.clone();
        }
        self.0.insert(key.to_string(), root.clone());
        root
    }

    fn join(&mut self, a: &str, b: &str) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.0.insert(ra, rb);
        }
    }
}

#[path = "plcopen_read_graph.rs"]
mod graph;

/// The rungs an `<LD>` body draws, as `.ld` text, or why POU `pou` cannot be read as rungs.
pub fn read_ld_body(ld: Node, pou: &str) -> Result<String, String> {
    graph::Reader::new(ld).rungs().map_err(|why| match why {
        graph::Refused::Element(what) => format!(
            "POU '{pou}': its ladder has {what}, which a rung here cannot express, so the POU was left out"
        ),
        graph::Refused::Whole(what) => format!("POU '{pou}': {what}, so the POU was left out"),
    })
}

#[cfg(test)]
#[path = "plcopen_read_tests.rs"]
mod tests;
