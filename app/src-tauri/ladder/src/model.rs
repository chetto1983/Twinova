//! A Ladder program: rungs of elements in series, a branch holding paths in parallel, nested without limit, as Open
//! Industry Project's model is (NOTICE). Positions are not stored: the editor lays a rung out from its tree.

use serde::{Deserialize, Serialize};

/// How a contact passes power: while its operand is TRUE (open), FALSE (closed), or on its rising or falling edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactMode {
    Open,
    Closed,
    Rising,
    Falling,
}

/// How a coil writes its operand: the power (normal), its negation, set or reset while powered, or TRUE for one scan
/// on the power's rising or falling edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoilMode {
    Normal,
    Negated,
    Set,
    Reset,
    Rising,
    Falling,
}

/// One parameter of a box as written: an input (`PT := T#2s`), an output bound to a place (`ET => elapsed`), or a bare
/// name with no value, which puts that pin on the rung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    pub name: String,
    pub value: Option<String>,
    pub out: bool,
}

/// An element of a rung. An operand is its text as written, `None` while blank (`?`), which a rung reads but does not
/// lower.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Element {
    Contact {
        mode: ContactMode,
        operand: Option<String>,
    },
    Coil {
        mode: CoilMode,
        operand: Option<String>,
    },
    /// A function block's instance (`T1(IN, PT := T#2s)`) or a function (`ADD(...)`), its head still blank or named.
    Box {
        head: Option<String>,
        pins: Vec<Pin>,
    },
    Branch {
        paths: Vec<Vec<Element>>,
    },
}

/// How a rung may end: a jump to a labelled rung below it, or a RETURN from the program, acting while powered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Exit {
    Jump { label: String },
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rung {
    /// The comment written above the rung, as plain lines.
    pub comment: Option<String>,
    /// The label a jump lands on.
    pub label: Option<String>,
    pub elements: Vec<Element>,
    pub exit: Option<Exit>,
}

/// An `.ld` file: its declarations, Structured Text `VAR … END_VAR` blocks as written, then its rungs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub declarations: String,
    pub rungs: Vec<Rung>,
}
