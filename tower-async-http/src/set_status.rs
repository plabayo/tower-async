//! Middleware to override status codes.
//!
//! # Example
//!
//! ```
//! use tower_async_http::set_status::SetStatusLayer;
//! use http::{Request, Response, StatusCode};
//! use http_body_util::Full;
//! use bytes::Bytes;
//! use std::{iter::once, convert::Infallible};
//! use tower_async::{ServiceBuilder, Service, ServiceExt};
//!
//! async fn handle(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     // ...
//!     # Ok(Response::new(Full::default()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut service = ServiceBuilder::new()
//!     // change the status to `404 Not Found` regardless what the inner service returns
//!     .layer(SetStatusLayer::new(StatusCode::NOT_FOUND))
//!     .service_fn(handle);
//!
//! // Call the service.
//! let request = Request::builder().body(Full::default())?;
//!
//! let response = service.call(request).await?;
//!
//! assert_eq!(response.status(), StatusCode::NOT_FOUND);
//! #
//! # Ok(())
//! # }
//! ```

use http::{Request, Response, StatusCode};

use tower_async_layer::Layer;
use tower_async_service::Service;

/// Layer that applies [`SetStatus`] which overrides the status codes.
#[derive(Debug, Clone, Copy)]
pub struct SetStatusLayer {
    status: StatusCode,
}

impl SetStatusLayer {
    /// Create a new [`SetStatusLayer`].
    ///
    /// The response status code will be `status` regardless of what the inner service returns.
    pub fn new(status: StatusCode) -> Self {
        SetStatusLayer { status }
    }
}

impl<S> Layer<S> for SetStatusLayer {
    type Service = SetStatus<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SetStatus::new(inner, self.status)
    }
}

/// Middleware to override status codes.
///
/// See the [module docs](self) for more details.
#[derive(Debug, Clone, Copy)]
pub struct SetStatus<S> {
    inner: S,
    status: StatusCode,
}

impl<S> SetStatus<S> {
    /// Create a new [`SetStatus`].
    ///
    /// The response status code will be `status` regardless of what the inner service returns.
    pub fn new(inner: S, status: StatusCode) -> Self {
        Self { status, inner }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `SetStatus` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(status: StatusCode) -> SetStatusLayer {
        SetStatusLayer::new(status)
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SetStatus<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let mut response = self.inner.call(req).await?;
        *response.status_mut() = self.status;
        Ok(response)
    }
}
