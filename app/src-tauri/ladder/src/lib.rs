//! Twinova's Ladder Diagram (spec 2026-11-13-ladder-editor-design.md): the model, its text grammar and printer, and
//! its lowering to Structured Text. They are ported from Open Industry Project (NOTICE); truST compiles only the
//! Structured Text a program lowers to.

pub mod lower;
mod lower_rung;
pub mod model;
pub mod parse;
mod plcopen_read;
mod plcopen_write;
pub mod print;
mod program;
#[cfg(test)]
mod test_support;

pub use lower::{
    Block, BlockPin, BoxPins, ElementId, Field, LowerError, Lowered, Origin, Power, lower,
};
pub use model::{CoilMode, ContactMode, Element, Exit, Pin, Program, Rung};
pub use parse::{ParseError, parse};
pub use plcopen_read::read_ld_body;
pub use plcopen_write::ld_body;
pub use print::print;
pub use program::{declarations_only, declarations_prefix, program_name};
