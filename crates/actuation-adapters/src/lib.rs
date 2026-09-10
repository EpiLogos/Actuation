//! Observation at the boundary of actual agency, not model/body selection.
//!
//! The catalogue declares a target. A probe observes that target in a supplied
//! environment. Neither a declaration, a provider inventory nor a process fact
//! manufactures Agent identity, authority, a model choice or executed work.
//! Native body and secret effects are bounded and replaceable; production calls
//! use the same application surfaces as deterministic conformance.
mod admission;
mod catalog;
pub mod effects;
mod instantiation;
mod observation;
pub mod secrets;
pub mod usage;

pub use actuation_core::{Error, Result};
pub use admission::*;
pub use catalog::*;
pub use instantiation::*;
pub use observation::*;
