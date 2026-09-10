//! Actuation's constitutional domain: what agency is situated here, under
//! which bounds, and how its attributable difference can return.
//!
//! This crate is deliberately independent of I/O, async executors, harnesses,
//! model providers, QL and the suite's other product implementations. Opaque
//! cross-owner references do not import their owner's behaviour.
//!
//! [`Record`] admits typed fields only after their invariants hold. Its fields
//! are immutable thereafter; changing a draft requires a fresh admission.
//! Missing, explicit null and present optional wire values remain distinct.
//!
//! Identity roles cannot be accidentally interchanged:
//! ```compile_fail
//! use actuation_core::{AgentRef, AgencyRef};
//! let agent = AgentRef::new("agent:independent").unwrap();
//! let agency: AgencyRef = agent;
//! ```

mod agency;
mod composition;
mod refs;
mod wire;

pub use agency::*;
pub use composition::*;
pub use refs::*;
pub use wire::{Error, Extensions, Invariant, NonEmpty, Record, Result, Slot};

pub const AGENCY_CONTRACT_VERSION: &str = "actuation.agency/v1";
