//! Service that redirects all requests.
//!
//! # Example
//!
//! Imagine that we run `example.com` and want to redirect all requests using `HTTP` to `HTTPS`.
//! That can be done like so:
//!
//! ```rust
//! use http::{Request, Uri, StatusCode};
//! use hyper::Body;
//! use tower_async::{Service, ServiceExt};
//! use tower_async_http::services::Redirect;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let uri: Uri = "https://example.com/".parse().unwrap();
//! let mut service: Redirect<Body> = Redirect::permanent(uri);
//!
//! let request = Request::builder()
//!     .uri("http://example.com")
//!     .body(Body::empty())
//!     .unwrap();
//!
//! let response = service.oneshot(request).await?;
//!
//! assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
//! assert_eq!(response.headers()["location"], "https://example.com/");
//! #
//! # Ok(())
//! # }
//! ```

use http::{header, HeaderValue, Response, StatusCode, Uri};
use std::{
    convert::{Infallible, TryFrom},
    fmt,
    marker::PhantomData,
};
use tower_async_service::Service;

/// Service that redirects all requests.
///
/// See the [module docs](crate::services::redirect) for more details.
pub struct Redirect<ResBody> {
    status_code: StatusCode,
    location: HeaderValue,
    // Covariant over ResBody, no dropping of ResBody
    _marker: PhantomData<fn() -> ResBody>,
}

impl<ResBody> Redirect<ResBody> {
    /// Create a new [`Redirect`] that uses a [`307 Temporary Redirect`][mdn] status code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307
    pub fn temporary(uri: Uri) -> Self {
        Self::with_status_code(StatusCode::TEMPORARY_REDIRECT, uri)
    }

    /// Create a new [`Redirect`] that uses a [`308 Permanent Redirect`][mdn] status code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308
    pub fn permanent(uri: Uri) -> Self {
        Self::with_status_code(StatusCode::PERMANENT_REDIRECT, uri)
    }

    /// Create a new [`Redirect`] that uses the given status code.
    ///
    /// # Panics
    ///
    /// - If `status_code` isn't a [redirection status code][mdn] (3xx).
    /// - If `uri` isn't a valid [`HeaderValue`].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status#redirection_messages
    pub fn with_status_code(status_code: StatusCode, uri: Uri) -> Self {
        assert!(
            status_code.is_redirection(),
            "not a redirection status code"
        );

        Self {
            status_code,
            location: HeaderValue::try_from(uri.to_string())
                .expect("URI isn't a valid header value"),
            _marker: PhantomData,
        }
    }
}

impl<R, ResBody> Service<R> for Redirect<ResBody>
where
    ResBody: Default,
{
    type Response = Response<ResBody>;
    type Error = Infallible;

    async fn call(&self, _req: R) -> Result<Self::Response, Self::Error> {
        let mut res = Response::default();
        *res.status_mut() = self.status_code;
        res.headers_mut()
            .insert(header::LOCATION, self.location.clone());
        Ok(res)
    }
}

impl<ResBody> fmt::Debug for Redirect<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Redirect")
            .field("status_code", &self.status_code)
            .field("location", &self.location)
            .finish()
    }
}

impl<ResBody> Clone for Redirect<ResBody> {
    fn clone(&self) -> Self {
        Self {
            status_code: self.status_code,
            location: self.location.clone(),
            _marker: PhantomData,
        }
    }
}
