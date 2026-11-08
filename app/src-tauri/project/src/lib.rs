//! Twinova's project format, version 1 (spec docs/superpowers/specs/2026-11-08-project-format-design.md, annex):
//! the document model, its validation, and the codec between a project and its files.

pub mod address;
pub mod codec;
pub mod diagnostic;
pub mod model;
mod validate;
mod validate_scene;

pub use codec::{Files, Opened, encode_project, parse_project};
pub use diagnostic::Diagnostic;
pub use model::Project;

/// The only format version this build reads and writes.
pub const FORMAT_VERSION: u32 = 1;
