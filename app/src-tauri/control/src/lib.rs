//! Twinova's control channel (spec 2026-11-23-app-mcp-design.md, decision 1): how an agent reaches the open app. The
//! app listens on the loopback, on the first free port of a small range, one JSON message per line; at each launch it
//! makes a token and writes the port and the token to a file only its user can read. A connection's first line carries
//! the token, then each line is a request and each answer a line. The design follows SolidCraft's control channel
//! (its `docs/control-protocol.md`, read 2026-11-23), in Twinova's own terms.

pub mod client;
pub mod file;
pub mod protocol;
pub mod server;

pub use client::Client;
pub use file::ControlFile;
pub use protocol::{Hello, Reply, Request};
pub use server::{Handler, PORTS, bind, serve, token};
