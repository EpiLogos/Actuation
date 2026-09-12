//! The Actuation command-line product: one native executable over the public
//! application surfaces of the actuation-* libraries. The command table in
//! `dispatch` is the single source of truth for routes, help, capabilities and
//! dispatch; `verify` carries the compiled-in deterministic owner suite.
pub mod commands;
pub mod dispatch;
pub mod render;
pub mod surface;
pub mod system;
pub mod verify;

pub use dispatch::{execute, match_route, Command, Output};
