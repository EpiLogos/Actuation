//! General, bounded Jev invocation over the existing native HTTP adapter.
//! Source disclosure and authority are admitted by the native caller before
//! this transport; a provider answer is not a grant or a source amendment.
mod transport;
pub use actuation_core::system_one::{Answer, Determination, Question, SystemOneRequest, SystemOneResponse, SystemOneUsage};
pub use transport::*;
pub const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
pub const DOCUMENTED_INPUT_CEILING: u64 = 64_000;
