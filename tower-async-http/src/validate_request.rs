//! Middleware that validates requests.
//!
//! # Example
//!
//! ```
//! use tower_async_http::validate_request::ValidateRequestHeaderLayer;
//! use http::{Request, Response, StatusCode, header::ACCEPT};
//! use http_body_util::Full;
//! use bytes::Bytes;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, service_fn, BoxError};
//!
//! async fn handle(request: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, BoxError> {
//!     Ok(Response::new(Full::default()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), BoxError> {
//! let mut service = ServiceBuilder::new()
//!     // Require the `Accept` header to be `application/json`, `*/*` or `application/*`
//!     .layer(ValidateRequestHeaderLayer::accept("application/json"))
//!     .service_fn(handle);
//!
//! // Requests with the correct value are allowed through
//! let request = Request::builder()
//!     .header(ACCEPT, "application/json")
//!     .body(Full::default())
//!     .unwrap();
//!
//! let response = service
//!     .call(request)
//!     .await?;
//!
//! assert_eq!(StatusCode::OK, response.status());
//!
//! // Requests with an invalid value get a `406 Not Acceptable` response
//! let request = Request::builder()
//!     .header(ACCEPT, "text/strings")
//!     .body(Full::default())
//!     .unwrap();
//!
//! let response = service
//!     .call(request)
//!     .await?;
//!
//! assert_eq!(StatusCode::NOT_ACCEPTABLE, response.status());
//! # Ok(())
//! # }
//! ```
//!
//! Custom validation can be made by implementing [`ValidateRequest`]:
//!
//! ```
//! use tower_async_http::validate_request::{ValidateRequestHeaderLayer, ValidateRequest};
//! use http::{Request, Response, StatusCode, header::ACCEPT};
//! use http_body_util::Full;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, service_fn, BoxError};
//! use bytes::Bytes;
//!
//! #[derive(Clone, Copy)]
//! pub struct MyHeader { /* ...  */ }
//!
//! impl<B> ValidateRequest<B> for MyHeader {
//!     type ResponseBody = Full<Bytes>;
//!
//!     fn validate(
//!         &self,
//!         request: &mut Request<B>,
//!     ) -> Result<(), Response<Self::ResponseBody>> {
//!         // validate the request...
//!         # unimplemented!()
//!     }
//! }
//!
//! async fn handle(request: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, BoxError> {
//!     Ok(Response::new(Full::default()))
//! }
//!
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = ServiceBuilder::new()
//!     // Validate requests using `MyHeader`
//!     .layer(ValidateRequestHeaderLayer::custom(MyHeader { /* ... */ }))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```
//!
//! Or using a closure:
//!
//! ```
//! use tower_async_http::validate_request::{ValidateRequestHeaderLayer, ValidateRequest};
//! use http::{Request, Response, StatusCode, header::ACCEPT};
//! use bytes::Bytes;
//! use http_body_util::Full;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, service_fn, BoxError};
//!
//! async fn handle(request: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, BoxError> {
//!     # todo!();
//!     // ...
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service = ServiceBuilder::new()
//!     .layer(ValidateRequestHeaderLayer::custom(|request: &mut Request<Full<Bytes>>| {
//!         // Validate the request
//!         # Ok::<_, Response<Full<Bytes>>>(())
//!     }))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```

use http::{header, Request, Response, StatusCode};
use http_body::Body;
use mime::{Mime, MimeIter};
use std::{fmt, marker::PhantomData, sync::Arc};
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Layer that applies [`ValidateRequestHeader`] which validates all requests.
///
/// See the [module docs](crate::validate_request) for an example.
#[derive(Debug, Clone)]
pub struct ValidateRequestHeaderLayer<T> {
    validate: T,
}

impl<ResBody> ValidateRequestHeaderLayer<AcceptHeader<ResBody>> {
    /// Validate requests have the required Accept header.
    ///
    /// The `Accept` header is required to be `*/*`, `type/*` or `type/subtype`,
    /// as configured.
    ///
    /// # Panics
    ///
    /// Panics if `header_value` is not in the form: `type/subtype`, such as `application/json`
    /// See `AcceptHeader::new` for when this method panics.
    ///
    /// # Example
    ///
    /// ```
    /// use http_body_util::Full;
    /// use bytes::Bytes;
    /// use tower_async_http::validate_request::{AcceptHeader, ValidateRequestHeaderLayer};
    ///
    /// let layer = ValidateRequestHeaderLayer::<AcceptHeader<Full<Bytes>>>::accept("application/json");
    /// ```
    ///
    /// [`Accept`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept
    pub fn accept(value: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self::custom(AcceptHeader::new(value))
    }
}

impl<T> ValidateRequestHeaderLayer<T> {
    /// Validate requests using a custom method.
    pub fn custom(validate: T) -> ValidateRequestHeaderLayer<T> {
        Self { validate }
    }
}

impl<S, T> Layer<S> for ValidateRequestHeaderLayer<T>
where
    T: Clone,
{
    type Service = ValidateRequestHeader<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        ValidateRequestHeader::new(inner, self.validate.clone())
    }
}

/// Middleware that validates requests.
///
/// See the [module docs](crate::validate_request) for an example.
#[derive(Clone, Debug)]
pub struct ValidateRequestHeader<S, T> {
    inner: S,
    validate: T,
}

impl<S, T> ValidateRequestHeader<S, T> {
    fn new(inner: S, validate: T) -> Self {
        Self::custom(inner, validate)
    }

    define_inner_service_accessors!();
}

impl<S, ResBody> ValidateRequestHeader<S, AcceptHeader<ResBody>> {
    /// Validate requests have the required Accept header.
    ///
    /// The `Accept` header is required to be `*/*`, `type/*` or `type/subtype`,
    /// as configured.
    ///
    /// # Panics
    ///
    /// See `AcceptHeader::new` for when this method panics.
    pub fn accept(inner: S, value: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self::custom(inner, AcceptHeader::new(value))
    }
}

impl<S, T> ValidateRequestHeader<S, T> {
    /// Validate requests using a custom method.
    pub fn custom(inner: S, validate: T) -> ValidateRequestHeader<S, T> {
        Self { inner, validate }
    }
}

impl<ReqBody, ResBody, S, V> Service<Request<ReqBody>> for ValidateRequestHeader<S, V>
where
    V: ValidateRequest<ReqBody, ResponseBody = ResBody>,
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = Response<ResBody>;
    type Error = S::Error;

    async fn call(&self, mut req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        match self.validate.validate(&mut req) {
            Ok(_) => self.inner.call(req).await,
            Err(res) => Ok(res),
        }
    }
}

/// Trait for validating requests.
pub trait ValidateRequest<B> {
    /// The body type used for responses to unvalidated requests.
    type ResponseBody;

    /// Validate the request.
    ///
    /// If `Ok(())` is returned then the request is allowed through, otherwise not.
    fn validate(&self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>>;
}

impl<B, F, ResBody> ValidateRequest<B> for F
where
    F: Fn(&mut Request<B>) -> Result<(), Response<ResBody>>,
{
    type ResponseBody = ResBody;

    fn validate(&self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        self(request)
    }
}

/// Type that performs validation of the Accept header.
pub struct AcceptHeader<ResBody> {
    header_value: Arc<Mime>,
    _ty: PhantomData<fn() -> ResBody>,
}

impl<ResBody> AcceptHeader<ResBody> {
    /// Create a new `AcceptHeader`.
    ///
    /// # Panics
    ///
    /// Panics if `header_value` is not in the form: `type/subtype`, such as `application/json`
    fn new(header_value: &str) -> Self
    where
        ResBody: Body + Default,
    {
        Self {
            header_value: Arc::new(
                header_value
                    .parse::<Mime>()
                    .expect("value is not a valid header value"),
            ),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> Clone for AcceptHeader<ResBody> {
    fn clone(&self) -> Self {
        Self {
            header_value: self.header_value.clone(),
            _ty: PhantomData,
        }
    }
}

impl<ResBody> fmt::Debug for AcceptHeader<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcceptHeader")
            .field("header_value", &self.header_value)
            .finish()
    }
}

impl<B, ResBody> ValidateRequest<B> for AcceptHeader<ResBody>
where
    ResBody: Body + Default,
{
    type ResponseBody = ResBody;

    fn validate(&self, req: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        if !req.headers().contains_key(header::ACCEPT) {
            return Ok(());
        }
        if req
            .headers()
            .get_all(header::ACCEPT)
            .into_iter()
            .filter_map(|header| header.to_str().ok())
            .any(|h| {
                MimeIter::new(h)
                    .map(|mim| {
                        if let Ok(mim) = mim {
                            let typ = self.header_value.type_();
                            let subtype = self.header_value.subtype();
                            match (mim.type_(), mim.subtype()) {
                                (t, s) if t == typ && s == subtype => true,
                                (t, mime::STAR) if t == typ => true,
                                (mime::STAR, mime::STAR) => true,
                                _ => false,
                            }
                        } else {
                            false
                        }
                    })
                    .reduce(|acc, mim| acc || mim)
                    .unwrap_or(false)
            })
        {
            return Ok(());
        }
        let mut res = Response::new(ResBody::default());
        *res.status_mut() = StatusCode::NOT_ACCEPTABLE;
        Err(res)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    use crate::test_helpers::Body;

    use http::{header, StatusCode};
    use tower_async::{BoxError, ServiceBuilder};

    #[tokio::test]
    async fn valid_accept_header() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "application/json")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_accept_header_accept_all_json() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "application/*")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_accept_header_accept_all() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "*/*")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_accept_header() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "invalid")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::NOT_ACCEPTABLE);
    }
    #[tokio::test]
    async fn not_accepted_accept_header_subtype() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "application/strings")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[tokio::test]
    async fn not_accepted_accept_header() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "text/strings")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[tokio::test]
    async fn accepted_multiple_header_value() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "text/strings")
            .header(header::ACCEPT, "invalid, application/json")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepted_inner_header_value() {
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/json"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, "text/strings, invalid, application/json")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepted_header_with_quotes_valid() {
        let value = "foo/bar; parisien=\"baguette, text/html, jambon, fromage\", application/*";
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("application/xml"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, value)
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn accepted_header_with_quotes_invalid() {
        let value = "foo/bar; parisien=\"baguette, text/html, jambon, fromage\"";
        let service = ServiceBuilder::new()
            .layer(ValidateRequestHeaderLayer::accept("text/html"))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::ACCEPT, value)
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::NOT_ACCEPTABLE);
    }

    async fn echo<B>(req: Request<B>) -> Result<Response<B>, BoxError> {
        Ok(Response::new(req.into_body()))
    }
}
