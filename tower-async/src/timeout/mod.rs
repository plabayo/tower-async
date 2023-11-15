//! Middleware that applies a timeout to requests.
//!
//! If the response does not complete within the specified timeout, the response
//! will be aborted.

pub mod error;
mod layer;

pub use self::layer::TimeoutLayer;

use error::Elapsed;

use std::time::Duration;
use tower_async_service::Service;

/// Applies a timeout to requests.
#[derive(Debug, Clone)]
pub struct Timeout<T> {
    inner: T,
    timeout: Duration,
}

// ===== impl Timeout =====

impl<T> Timeout<T> {
    /// Creates a new [`Timeout`]
    pub fn new(inner: T, timeout: Duration) -> Self {
        Timeout { inner, timeout }
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<S, Request> Service<Request> for Timeout<S>
where
    S: Service<Request>,
    S::Error: Into<crate::BoxError>,
{
    type Response = S::Response;
    type Error = crate::BoxError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        tokio::select! {
            res = self.inner.call(request) => res.map_err(Into::into),
            _ = tokio::time::sleep(self.timeout) => Err(Elapsed(()).into()),
        }
    }
}
