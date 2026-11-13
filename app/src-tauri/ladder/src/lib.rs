//! Twinova's Ladder Diagram (spec 2026-11-13-ladder-editor-design.md): the model, its text grammar and printer. The
//! model and grammar are ported from Open Industry Project (NOTICE); truST compiles only the Structured Text a program
//! lowers to.

pub mod model;
pub mod parse;
pub mod print;

pub use model::{CoilMode, ContactMode, Element, Exit, Pin, Program, Rung};
pub use parse::{ParseError, parse};
pub use print::print;
