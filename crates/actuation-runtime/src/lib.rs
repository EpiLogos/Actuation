//! Actuality of situated agency, not body resolution or process hosting.
//!
//! Constitutional admission makes a bounded relation actual; it does not
//! materialise it. A driver can report an acting locus only from observations.
//! The same observer/driver grammar serves managed and already-existing bodies.
//! The ordinary loop depends on an acting host, not AIKit, Factory or QL.
mod actualisation;
mod locus;
mod loop_runtime;
mod realised;
pub use actualisation::*;
pub use actuation_core::{Error, Result};
pub use locus::*;
pub use loop_runtime::*;
pub use realised::*;

pub type PortFuture<'a, T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>>;

macro_rules! invariant {
    ($ty:ty, [$($field:ident),*], $check:expr) => {
        impl actuation_core::Invariant for $ty {
            const FIELDS: &'static [&'static str] = &[$(stringify!($field)),*];
            fn extensions(&self) -> &actuation_core::Extensions { &self.extensions }
            fn check(&self) -> actuation_core::Result<()> { ($check)(self) }
        }
    };
}
pub(crate) use invariant;
