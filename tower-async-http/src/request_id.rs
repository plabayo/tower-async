//! Set and propagate request ids.
//!
//! # Example
//!
//! ```
//! use http::{Request, Response, header::HeaderName};
//! use tower_async::{Service, ServiceExt, ServiceBuilder};
//! use tower_async_http::request_id::{
//!     SetRequestIdLayer, PropagateRequestIdLayer, MakeRequestId, RequestId,
//! };
//! use hyper::Body;
//! use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let handler = tower_async::service_fn(|request: Request<Body>| async move {
//! #     Ok::<_, std::convert::Infallible>(Response::new(request.into_body()))
//! # });
//! #
//! // A `MakeRequestId` that increments an atomic counter
//! #[derive(Clone, Default)]
//! struct MyMakeRequestId {
//!     counter: Arc<AtomicU64>,
//! }
//!
//! impl MakeRequestId for MyMakeRequestId {
//!     fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
//!         let request_id = self.counter
//!             .fetch_add(1, Ordering::SeqCst)
//!             .to_string()
//!             .parse()
//!             .unwrap();
//!
//!         Some(RequestId::new(request_id))
//!     }
//! }
//!
//! let x_request_id = HeaderName::from_static("x-request-id");
//!
//! let mut svc = ServiceBuilder::new()
//!     // set `x-request-id` header on all requests
//!     .layer(SetRequestIdLayer::new(
//!         x_request_id.clone(),
//!         MyMakeRequestId::default(),
//!     ))
//!     // propagate `x-request-id` headers from request to response
//!     .layer(PropagateRequestIdLayer::new(x_request_id))
//!     .service(handler);
//!
//! let request = Request::new(Body::empty());
//! let response = svc.call(request).await?;
//!
//! assert_eq!(response.headers()["x-request-id"], "0");
//! #
//! # Ok(())
//! # }
//! ```
//!
//! Additional convenience methods are available on [`ServiceBuilderExt`]:
//!
//! ```
//! use tower_async_http::ServiceBuilderExt;
//! # use http::{Request, Response, header::HeaderName};
//! # use tower_async::{Service, ServiceExt, ServiceBuilder};
//! # use tower_async_http::request_id::{
//! #     SetRequestIdLayer, PropagateRequestIdLayer, MakeRequestId, RequestId,
//! # };
//! # use hyper::Body;
//! # use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let handler = tower_async::service_fn(|request: Request<Body>| async move {
//! #     Ok::<_, std::convert::Infallible>(Response::new(request.into_body()))
//! # });
//! # #[derive(Clone, Default)]
//! # struct MyMakeRequestId {
//! #     counter: Arc<AtomicU64>,
//! # }
//! # impl MakeRequestId for MyMakeRequestId {
//! #     fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
//! #         let request_id = self.counter
//! #             .fetch_add(1, Ordering::SeqCst)
//! #             .to_string()
//! #             .parse()
//! #             .unwrap();
//! #         Some(RequestId::new(request_id))
//! #     }
//! # }
//!
//! let mut svc = ServiceBuilder::new()
//!     .set_x_request_id(MyMakeRequestId::default())
//!     .propagate_x_request_id()
//!     .service(handler);
//!
//! let request = Request::new(Body::empty());
//! let response = svc.call(request).await?;
//!
//! assert_eq!(response.headers()["x-request-id"], "0");
//! #
//! # Ok(())
//! # }
//! ```
//!
//! See [`SetRequestId`] and [`PropagateRequestId`] for more details.
//!
//! # Doesn't override existing headers
//!
//! [`SetRequestId`] and [`PropagateRequestId`] wont override request ids if its already present on
//! requests or responses. Among other things, this allows other middleware to conditionally set
//! request ids and use the middleware in this module as a fallback.
//!
//! [`ServiceBuilderExt`]: crate::ServiceBuilderExt
//! [`Uuid`]: https://crates.io/crates/uuid

use http::{
    header::{HeaderName, HeaderValue},
    Request, Response,
};
use tower_async_layer::Layer;
use tower_async_service::Service;
use uuid::Uuid;

pub(crate) const X_REQUEST_ID: &str = "x-request-id";

/// Trait for producing [`RequestId`]s.
///
/// Used by [`SetRequestId`].
pub trait MakeRequestId {
    /// Try and produce a [`RequestId`] from the request.
    fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId>;
}

/// An identifier for a request.
#[derive(Debug, Clone)]
pub struct RequestId(HeaderValue);

impl RequestId {
    /// Create a new `RequestId` from a [`HeaderValue`].
    pub fn new(header_value: HeaderValue) -> Self {
        Self(header_value)
    }

    /// Gets a reference to the underlying [`HeaderValue`].
    pub fn header_value(&self) -> &HeaderValue {
        &self.0
    }

    /// Consumes `self`, returning the underlying [`HeaderValue`].
    pub fn into_header_value(self) -> HeaderValue {
        self.0
    }
}

impl From<HeaderValue> for RequestId {
    fn from(value: HeaderValue) -> Self {
        Self::new(value)
    }
}

/// Set request id headers and extensions on requests.
///
/// This layer applies the [`SetRequestId`] middleware.
///
/// See the [module docs](self) and [`SetRequestId`] for more details.
#[derive(Debug, Clone)]
pub struct SetRequestIdLayer<M> {
    header_name: HeaderName,
    make_request_id: M,
}

impl<M> SetRequestIdLayer<M> {
    /// Create a new `SetRequestIdLayer`.
    pub fn new(header_name: HeaderName, make_request_id: M) -> Self
    where
        M: MakeRequestId,
    {
        SetRequestIdLayer {
            header_name,
            make_request_id,
        }
    }

    /// Create a new `SetRequestIdLayer` that uses `x-request-id` as the header name.
    pub fn x_request_id(make_request_id: M) -> Self
    where
        M: MakeRequestId,
    {
        SetRequestIdLayer::new(HeaderName::from_static(X_REQUEST_ID), make_request_id)
    }
}

impl<S, M> Layer<S> for SetRequestIdLayer<M>
where
    M: Clone + MakeRequestId,
{
    type Service = SetRequestId<S, M>;

    fn layer(&self, inner: S) -> Self::Service {
        SetRequestId::new(
            inner,
            self.header_name.clone(),
            self.make_request_id.clone(),
        )
    }
}

/// Set request id headers and extensions on requests.
///
/// See the [module docs](self) for an example.
///
/// If [`MakeRequestId::make_request_id`] returns `Some(_)` and the request doesn't already have a
/// header with the same name, then the header will be inserted.
///
/// Additionally [`RequestId`] will be inserted into [`Request::extensions`] so other
/// services can access it.
#[derive(Debug, Clone)]
pub struct SetRequestId<S, M> {
    inner: S,
    header_name: HeaderName,
    make_request_id: M,
}

impl<S, M> SetRequestId<S, M> {
    /// Create a new `SetRequestId`.
    pub fn new(inner: S, header_name: HeaderName, make_request_id: M) -> Self
    where
        M: MakeRequestId,
    {
        Self {
            inner,
            header_name,
            make_request_id,
        }
    }

    /// Create a new `SetRequestId` that uses `x-request-id` as the header name.
    pub fn x_request_id(inner: S, make_request_id: M) -> Self
    where
        M: MakeRequestId,
    {
        Self::new(
            inner,
            HeaderName::from_static(X_REQUEST_ID),
            make_request_id,
        )
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `SetRequestId` middleware.
    pub fn layer(header_name: HeaderName, make_request_id: M) -> SetRequestIdLayer<M>
    where
        M: MakeRequestId,
    {
        SetRequestIdLayer::new(header_name, make_request_id)
    }
}

impl<S, M, ReqBody, ResBody> Service<Request<ReqBody>> for SetRequestId<S, M>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    M: MakeRequestId,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, mut req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        if let Some(request_id) = req.headers().get(&self.header_name) {
            if req.extensions().get::<RequestId>().is_none() {
                let request_id = request_id.clone();
                req.extensions_mut().insert(RequestId::new(request_id));
            }
        } else if let Some(request_id) = self.make_request_id.make_request_id(&req) {
            req.extensions_mut().insert(request_id.clone());
            req.headers_mut()
                .insert(self.header_name.clone(), request_id.0);
        }

        self.inner.call(req).await
    }
}

/// Propagate request ids from requests to responses.
///
/// This layer applies the [`PropagateRequestId`] middleware.
///
/// See the [module docs](self) and [`PropagateRequestId`] for more details.
#[derive(Debug, Clone)]
pub struct PropagateRequestIdLayer {
    header_name: HeaderName,
}

impl PropagateRequestIdLayer {
    /// Create a new `PropagateRequestIdLayer`.
    pub fn new(header_name: HeaderName) -> Self {
        PropagateRequestIdLayer { header_name }
    }

    /// Create a new `PropagateRequestIdLayer` that uses `x-request-id` as the header name.
    pub fn x_request_id() -> Self {
        Self::new(HeaderName::from_static(X_REQUEST_ID))
    }
}

impl<S> Layer<S> for PropagateRequestIdLayer {
    type Service = PropagateRequestId<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PropagateRequestId::new(inner, self.header_name.clone())
    }
}

/// Propagate request ids from requests to responses.
///
/// See the [module docs](self) for an example.
///
/// If the request contains a matching header that header will be applied to responses. If a
/// [`RequestId`] extension is also present it will be propagated as well.
#[derive(Debug, Clone)]
pub struct PropagateRequestId<S> {
    inner: S,
    header_name: HeaderName,
}

impl<S> PropagateRequestId<S> {
    /// Create a new `PropagateRequestId`.
    pub fn new(inner: S, header_name: HeaderName) -> Self {
        Self { inner, header_name }
    }

    /// Create a new `PropagateRequestId` that uses `x-request-id` as the header name.
    pub fn x_request_id(inner: S) -> Self {
        Self::new(inner, HeaderName::from_static(X_REQUEST_ID))
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `PropagateRequestId` middleware.
    pub fn layer(header_name: HeaderName) -> PropagateRequestIdLayer {
        PropagateRequestIdLayer::new(header_name)
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for PropagateRequestId<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let request_id = req
            .headers()
            .get(&self.header_name)
            .cloned()
            .map(RequestId::new);

        let mut response = self.inner.call(req).await?;

        if let Some(current_id) = response.headers().get(&self.header_name) {
            if response.extensions().get::<RequestId>().is_none() {
                let current_id = current_id.clone();
                response.extensions_mut().insert(RequestId::new(current_id));
            }
        } else if let Some(request_id) = request_id {
            response
                .headers_mut()
                .insert(self.header_name.clone(), request_id.0.clone());
            response.extensions_mut().insert(request_id);
        }

        Ok(response)
    }
}

/// A [`MakeRequestId`] that generates `UUID`s.
#[derive(Clone, Copy, Default)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string().parse().unwrap();
        Some(RequestId::new(request_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::ServiceBuilderExt as _;
    use hyper::{Body, Response};
    use std::{
        convert::Infallible,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
    };
    use tower_async::{ServiceBuilder, ServiceExt};

    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn basic() {
        let svc = ServiceBuilder::new()
            .set_x_request_id(Counter::default())
            .propagate_x_request_id()
            .service_fn(handler);

        // header on response
        let req = Request::builder().body(Body::empty()).unwrap();
        let res = svc.clone().oneshot(req).await.unwrap();
        assert_eq!(res.headers()["x-request-id"], "0");

        let req = Request::builder().body(Body::empty()).unwrap();
        let res = svc.clone().oneshot(req).await.unwrap();
        assert_eq!(res.headers()["x-request-id"], "1");

        // doesn't override if header is already there
        let req = Request::builder()
            .header("x-request-id", "foo")
            .body(Body::empty())
            .unwrap();
        let res = svc.clone().oneshot(req).await.unwrap();
        assert_eq!(res.headers()["x-request-id"], "foo");

        // extension propagated
        let req = Request::builder().body(Body::empty()).unwrap();
        let res = svc.clone().oneshot(req).await.unwrap();
        assert_eq!(res.extensions().get::<RequestId>().unwrap().0, "2");
    }

    #[tokio::test]
    async fn other_middleware_setting_request_id() {
        let svc = ServiceBuilder::new()
            .override_request_header(
                HeaderName::from_static("x-request-id"),
                HeaderValue::from_str("foo").unwrap(),
            )
            .set_x_request_id(Counter::default())
            .map_request(|request: Request<_>| {
                // `set_x_request_id` should set the extension if its missing
                assert_eq!(request.extensions().get::<RequestId>().unwrap().0, "foo");
                request
            })
            .propagate_x_request_id()
            .service_fn(handler);

        let req = Request::builder()
            .header(
                "x-request-id",
                "this-will-be-overriden-by-override_request_header-middleware",
            )
            .body(Body::empty())
            .unwrap();
        let res = svc.clone().oneshot(req).await.unwrap();
        assert_eq!(res.headers()["x-request-id"], "foo");
        assert_eq!(res.extensions().get::<RequestId>().unwrap().0, "foo");
    }

    #[tokio::test]
    async fn other_middleware_setting_request_id_on_response() {
        let svc = ServiceBuilder::new()
            .set_x_request_id(Counter::default())
            .propagate_x_request_id()
            .override_response_header(
                HeaderName::from_static("x-request-id"),
                HeaderValue::from_str("foo").unwrap(),
            )
            .service_fn(handler);

        let req = Request::builder()
            .header("x-request-id", "foo")
            .body(Body::empty())
            .unwrap();
        let res = svc.clone().oneshot(req).await.unwrap();
        assert_eq!(res.headers()["x-request-id"], "foo");
        assert_eq!(res.extensions().get::<RequestId>().unwrap().0, "foo");
    }

    #[derive(Clone, Default)]
    struct Counter(Arc<AtomicU64>);

    impl MakeRequestId for Counter {
        fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
            let id =
                HeaderValue::from_str(&self.0.fetch_add(1, Ordering::SeqCst).to_string()).unwrap();
            Some(RequestId::new(id))
        }
    }

    async fn handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new(Body::empty()))
    }

    #[tokio::test]
    async fn uuid() {
        let svc = ServiceBuilder::new()
            .set_x_request_id(MakeRequestUuid)
            .propagate_x_request_id()
            .service_fn(handler);

        // header on response
        let req = Request::builder().body(Body::empty()).unwrap();
        let mut res = svc.clone().oneshot(req).await.unwrap();
        let id = res.headers_mut().remove("x-request-id").unwrap();
        id.to_str().unwrap().parse::<Uuid>().unwrap();
    }
}