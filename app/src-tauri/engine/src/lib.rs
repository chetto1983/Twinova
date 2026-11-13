//! Twinova's run engine (spec 2026-11-11-agent-server-design.md, decision 1): a project's line in lockstep on virtual
//! time, the physics, the field runtime and the line controller, with no UI dependency. The app and the agent server
//! both drive it, and both build a project with `build`.

pub mod build;
pub mod controller;
pub mod debugger;
mod debugger_kinds;
mod drive;
pub mod engine;
pub mod field;
mod field_image;
pub mod line;
mod line_bind;
mod locate;
pub mod timing;
pub mod watch;

pub use build::{build, diagnostics};
