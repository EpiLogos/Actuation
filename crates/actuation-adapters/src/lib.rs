//! The observed body/provider boundary of Actuation.
//!
//! A catalogue declares a target. Probe results establish only the facts they
//! actually measured. An environment marker may identify a native context but
//! cannot grant authority or manufacture enduring Agent identity. Neither a
//! provider inventory nor a model-access receipt selects a model or a body.
mod capability;
mod catalog;
mod detection;
mod instantiation;
mod observation;
mod probes;
mod process;
mod secret;
pub mod usage;
mod wire;
pub use actuation_core::{Error, Result};
pub use capability::*;
pub use catalog::*;
pub use detection::*;
pub use instantiation::*;
pub use observation::*;
pub use probes::*;
pub use process::*;
pub use secret::*;
