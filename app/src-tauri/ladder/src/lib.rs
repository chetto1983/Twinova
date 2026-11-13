//! Twinova's Ladder Diagram (spec 2026-11-13-ladder-editor-design.md): the model, its text grammar and printer, and
//! its lowering to Structured Text. They are ported from Open Industry Project (NOTICE); truST compiles only the
//! Structured Text a program lowers to.

pub mod lower;
mod lower_rung;
pub mod model;
pub mod parse;
pub mod print;
mod program;

pub use lower::{Block, BlockPin, ElementId, Field, LowerError, Lowered, Origin, Power, lower};
pub use model::{CoilMode, ContactMode, Element, Exit, Pin, Program, Rung};
pub use parse::{ParseError, parse};
pub use print::print;
pub use program::{declarations_only, declarations_prefix, program_name};
