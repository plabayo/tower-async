//! Limit policies for [`super::Limit`]
//! define how requests are handled when the limit is reached
//! for a given request.

mod concurrent;
pub use concurrent::{ConcurrentPolicy, LimitReached};

/// The output of a limit policy.
#[derive(Debug)]
pub enum PolicyOutput<Guard, Error> {
    /// The request is allowed to proceed,
    /// and the guard is returned to release the limit when it is dropped,
    /// which should be done after the request is completed.
    Ready(Guard),
    /// The request is not allowed to proceed, and should be aborted.
    Abort(Error),
    /// The request is not allowed to proceed, but should be retried.
    Retry,
}

/// A limit policy is used to determine whether a request is allowed to proceed,
/// and if not, how to handle it.
pub trait Policy<Request> {
    /// The guard type that is returned when the request is allowed to proceed.
    ///
    /// See [`PolicyOutput::Ready`].
    type Guard;
    /// The error type that is returned when the request is not allowed to proceed,
    /// and should be aborted.
    ///
    /// See [`PolicyOutput::Abort`].
    type Error;

    /// Check whether the request is allowed to proceed.
    ///
    /// Optionally modify the request before it is passed to the inner service,
    /// which can be used to add metadata to the request regarding how the request
    /// was handled by this limit policy.
    fn check(
        &self,
        request: &mut Request,
    ) -> impl std::future::Future<Output = PolicyOutput<Self::Guard, Self::Error>>;
}
