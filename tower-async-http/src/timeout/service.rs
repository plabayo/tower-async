//! Middleware that applies a timeout to requests.
//!
//! If the request does not complete within the specified timeout it will be aborted and a `408
//! Request Timeout` response will be sent.
//!
//! # Differences from `tower_async::timeout`
//!
//! tower's [`Timeout`](tower_async::timeout::Timeout) middleware uses an error to signal timeout, i.e.
//! it changes the error type to [`BoxError`](tower_async::BoxError). For HTTP services that is rarely
//! what you want as returning errors will terminate the connection without sending a response.
//!
//! This middleware won't change the error type and instead return a `408 Request Timeout`
//! response. That means if your service's error type is [`Infallible`] it will still be
//! [`Infallible`] after applying this middleware.
//!
//! # Example
//!
//! ```
//! use http::{Request, Response};
//! use hyper::Body;
//! use std::{convert::Infallible, time::Duration};
//! use tower_async::ServiceBuilder;
//! use tower_async_http::timeout::TimeoutLayer;
//!
//! async fn handle(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     // ...
//!     # Ok(Response::new(Body::empty()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let svc = ServiceBuilder::new()
//!     // Timeout requests after 30 seconds
//!     .layer(TimeoutLayer::new(Duration::from_secs(30)))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```
//!
//! [`Infallible`]: std::convert::Infallible

use http::{Request, Response, StatusCode};
use std::time::Duration;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Layer that applies the [`Timeout`] middleware which apply a timeout to requests.
#[derive(Debug, Clone, Copy)]
pub struct TimeoutLayer {
    timeout: Duration,
}

impl TimeoutLayer {
    /// Creates a new [`TimeoutLayer`].
    pub fn new(timeout: Duration) -> Self {
        TimeoutLayer { timeout }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = Timeout<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Timeout::new(inner, self.timeout)
    }
}

/// Middleware which apply a timeout to requests.
///
/// If the request does not complete within the specified timeout it will be aborted and a `408
/// Request Timeout` response will be sent.
#[derive(Debug, Clone, Copy)]
pub struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    /// Creates a new [`Timeout`].
    pub fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `Timeout` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(timeout: Duration) -> TimeoutLayer {
        TimeoutLayer::new(timeout)
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for Timeout<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        tokio::select! {
            res = self.inner.call(req) => res,
            _ = tokio::time::sleep(self.timeout) => {
                let mut res = Response::new(ResBody::default());
                *res.status_mut() = StatusCode::REQUEST_TIMEOUT;
                Ok(res)
            }
        }
    }
}
