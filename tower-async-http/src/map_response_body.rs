//! Apply a transformation to the response body.
//!
//! # Example
//!
//! ```
//! use bytes::Bytes;
//! use http::{Request, Response};
//! use http_body_util::Empty;
//! use hyper::body::Body;
//! use std::convert::Infallible;
//! use std::{pin::Pin, task::{Context, Poll}};
//! use tower_async::{ServiceBuilder, service_fn, ServiceExt, Service};
//! use tower_async_http::map_response_body::MapResponseBodyLayer;
//! use futures::ready;
//!
//! // A wrapper for a `hyper::body::Body` that prints the size of data chunks
//! struct PrintChunkSizesBody {
//!     inner: Body,
//! }
//!
//! impl PrintChunkSizesBody {
//!     fn new(inner: Body) -> Self {
//!         Self { inner }
//!     }
//! }
//!
//! impl http_body::Body for PrintChunkSizesBody {
//!     type Data = Bytes;
//!     type Error = hyper::Error;
//!
//!     fn poll_data(
//!         mut self: Pin<&mut Self>,
//!         cx: &mut Context<'_>,
//!     ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
//!         if let Some(chunk) = ready!(Pin::new(&mut self.inner).poll_data(cx)?) {
//!             println!("chunk size = {}", chunk.len());
//!             Poll::Ready(Some(Ok(chunk)))
//!         } else {
//!             Poll::Ready(None)
//!         }
//!     }
//!
//!     fn poll_trailers(
//!         mut self: Pin<&mut Self>,
//!         cx: &mut Context<'_>,
//!     ) -> Poll<Result<Option<hyper::HeaderMap>, Self::Error>> {
//!         Pin::new(&mut self.inner).poll_trailers(cx)
//!     }
//!
//!     fn is_end_stream(&self) -> bool {
//!         self.inner.is_end_stream()
//!     }
//!
//!     fn size_hint(&self) -> http_body::SizeHint {
//!         self.inner.size_hint()
//!     }
//! }
//!
//! async fn handle<B>(_: Request<B>) -> Result<Response<Empty>, Infallible> {
//!     // ...
//!     # Ok(Response::new(Empty::new()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut svc = ServiceBuilder::new()
//!     // Wrap response bodies in `PrintChunkSizesBody`
//!     .layer(MapResponseBodyLayer::new(PrintChunkSizesBody::new))
//!     .service_fn(handle);
//!
//! // Call the service
//! let request = Request::new(Body::from("foobar"));
//!
//! svc.call(request).await?;
//! # Ok(())
//! # }
//! ```

use http::{Request, Response};
use std::fmt;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Apply a transformation to the response body.
///
/// See the [module docs](crate::map_response_body) for an example.
#[derive(Clone)]
pub struct MapResponseBodyLayer<F> {
    f: F,
}

impl<F> MapResponseBodyLayer<F> {
    /// Create a new [`MapResponseBodyLayer`].
    ///
    /// `F` is expected to be a function that takes a body and returns another body.
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<S, F> Layer<S> for MapResponseBodyLayer<F>
where
    F: Clone,
{
    type Service = MapResponseBody<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        MapResponseBody::new(inner, self.f.clone())
    }
}

impl<F> fmt::Debug for MapResponseBodyLayer<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResponseBodyLayer")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// Apply a transformation to the response body.
///
/// See the [module docs](crate::map_response_body) for an example.
#[derive(Clone)]
pub struct MapResponseBody<S, F> {
    inner: S,
    f: F,
}

impl<S, F> MapResponseBody<S, F> {
    /// Create a new [`MapResponseBody`].
    ///
    /// `F` is expected to be a function that takes a body and returns another body.
    pub fn new(service: S, f: F) -> Self {
        Self { inner: service, f }
    }

    /// Returns a new [`Layer`] that wraps services with a `MapResponseBodyLayer` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(f: F) -> MapResponseBodyLayer<F> {
        MapResponseBodyLayer::new(f)
    }

    define_inner_service_accessors!();
}

impl<F, S, ReqBody, ResBody, NewResBody> Service<Request<ReqBody>> for MapResponseBody<S, F>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    F: Fn(ResBody) -> NewResBody + Clone,
{
    type Response = Response<NewResBody>;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let res = self.inner.call(req).await?;
        Ok(res.map(self.f.clone()))
    }
}

impl<S, F> fmt::Debug for MapResponseBody<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResponseBody")
            .field("inner", &self.inner)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
