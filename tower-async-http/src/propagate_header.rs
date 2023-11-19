//! Propagate a header from the request to the response.
//!
//! # Example
//!
//! ```rust
//! use http::{Request, Response, header::HeaderName};
//! use http_body_util::Full;
//! use bytes::Bytes;
//! use std::convert::Infallible;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, service_fn};
//! use tower_async_http::propagate_header::PropagateHeaderLayer;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! async fn handle(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     // ...
//!     # Ok(Response::new(Full::default()))
//! }
//!
//! let mut svc = ServiceBuilder::new()
//!     // This will copy `x-request-id` headers from requests onto responses.
//!     .layer(PropagateHeaderLayer::new(HeaderName::from_static("x-request-id")))
//!     .service_fn(handle);
//!
//! // Call the service.
//! let request = Request::builder()
//!     .header("x-request-id", "1337")
//!     .body(Full::default())?;
//!
//! let response = svc.call(request).await?;
//!
//! assert_eq!(response.headers()["x-request-id"], "1337");
//! #
//! # Ok(())
//! # }
//! ```

use http::{header::HeaderName, Request, Response};
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Layer that applies [`PropagateHeader`] which propagates headers from requests to responses.
///
/// If the header is present on the request it'll be applied to the response as well. This could
/// for example be used to propagate headers such as `X-Request-Id`.
///
/// See the [module docs](crate::propagate_header) for more details.
#[derive(Clone, Debug)]
pub struct PropagateHeaderLayer {
    header: HeaderName,
}

impl PropagateHeaderLayer {
    /// Create a new [`PropagateHeaderLayer`].
    pub fn new(header: HeaderName) -> Self {
        Self { header }
    }
}

impl<S> Layer<S> for PropagateHeaderLayer {
    type Service = PropagateHeader<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PropagateHeader {
            inner,
            header: self.header.clone(),
        }
    }
}

/// Middleware that propagates headers from requests to responses.
///
/// If the header is present on the request it'll be applied to the response as well. This could
/// for example be used to propagate headers such as `X-Request-Id`.
///
/// See the [module docs](crate::propagate_header) for more details.
#[derive(Clone, Debug)]
pub struct PropagateHeader<S> {
    inner: S,
    header: HeaderName,
}

impl<S> PropagateHeader<S> {
    /// Create a new [`PropagateHeader`] that propagates the given header.
    pub fn new(inner: S, header: HeaderName) -> Self {
        Self { inner, header }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `PropagateHeader` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(header: HeaderName) -> PropagateHeaderLayer {
        PropagateHeaderLayer::new(header)
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for PropagateHeader<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let value = req.headers().get(&self.header).cloned();

        let mut res = self.inner.call(req).await?;

        if let Some(value) = value {
            res.headers_mut().insert(self.header.clone(), value);
        }

        Ok(res)
    }
}
