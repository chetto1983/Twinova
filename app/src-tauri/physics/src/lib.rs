//! Twinova's line physics (spec 2026-11-09-component-behaviour-design.md, "Transport with physics"): a project's line
//! as a Rapier world stepped every millisecond. A crate of its own, without Tauri, so its tests run on every platform
//! and the state fingerprint can be compared between Linux and Windows.

pub mod interfaces;
pub mod layout;
pub mod path;
pub mod snap;
mod surfaces;
pub mod world;

pub use layout::{Layout, layout};
pub use world::{Carton, Reading, STEP_NS, Scan, World};
