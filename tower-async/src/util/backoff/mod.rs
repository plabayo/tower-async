//! This module contains generic [backoff] utlities to be used with the retry
//! and limit layers.
//!
//! The [`Backoff`] trait is a generic way to represent backoffs that can use
//! any timer type.
//!
//! [`ExponentialBackoffMaker`] implements the maker type for  
//! [`ExponentialBackoff`] which implements the [`Backoff`] trait and provides
//! a batteries included exponential backoff and jitter strategy.
//!
//! [backoff]: https://en.wikipedia.org/wiki/Exponential_backoff

use std::future::Future;

/// Trait used to construct [`Backoff`] trait implementors.
pub trait MakeBackoff {
    /// The backoff type produced by this maker.
    type Backoff: Backoff;

    /// Constructs a new backoff type.
    fn make_backoff(&mut self) -> Self::Backoff;
}

/// A backoff trait where a single mutable reference represents a single
/// backoff session. Implementors must also implement [`Clone`] which will
/// reset the backoff back to the default state for the next session.
pub trait Backoff {
    /// The future associated with each backoff. This usually will be some sort
    /// of timer.
    type Future: Future<Output = ()>;

    /// Initiate the next backoff in the sequence.
    fn next_backoff(&mut self) -> Self::Future;
}

#[cfg(feature = "util-tokio")]
mod exponential;
#[cfg(feature = "util-tokio")]
pub use exponential::{ExponentialBackoff, ExponentialBackoffMaker};
