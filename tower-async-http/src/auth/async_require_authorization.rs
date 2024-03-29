//! Authorize requests using the [`Authorization`] header asynchronously.
//!
//! [`Authorization`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
//!
//! # Example
//!
//! ```
//! use tower_async_http::auth::{AsyncRequireAuthorizationLayer, AsyncAuthorizeRequest};
//! use http::{Request, Response, StatusCode, header::AUTHORIZATION};
//! use http_body_util::Full;
//! use bytes::Bytes;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, service_fn, BoxError};
//! use futures_util::future::BoxFuture;
//!
//! #[derive(Clone, Copy)]
//! struct MyAuth;
//!
//! impl<B> AsyncAuthorizeRequest<B> for MyAuth
//! where
//!     B: Send + Sync + 'static,
//! {
//!     type RequestBody = B;
//!     type ResponseBody = Full<Bytes>;
//!
//!     async fn authorize(&self, mut request: Request<B>) -> Result<Request<B>, Response<Self::ResponseBody>> {
//!         if let Some(user_id) = check_auth(&request).await {
//!             // Set `user_id` as a request extension so it can be accessed by other
//!             // services down the stack.
//!             request.extensions_mut().insert(user_id);
//!
//!             Ok(request)
//!         } else {
//!             let unauthorized_response = Response::builder()
//!                 .status(StatusCode::UNAUTHORIZED)
//!                 .body(Full::<Bytes>::default())
//!                 .unwrap();
//!
//!             Err(unauthorized_response)
//!         }
//!     }
//! }
//!
//! async fn check_auth<B>(request: &Request<B>) -> Option<UserId> {
//!     // ...
//!     # None
//! }
//!
//! #[derive(Clone, Debug)]
//! struct UserId(String);
//!
//! async fn handle(request: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, BoxError> {
//!     // Access the `UserId` that was set in `on_authorized`. If `handle` gets called the
//!     // request was authorized and `UserId` will be present.
//!     let user_id = request
//!         .extensions()
//!         .get::<UserId>()
//!         .expect("UserId will be there if request was authorized");
//!
//!     println!("request from {:?}", user_id);
//!
//!     Ok(Response::new(Full::<Bytes>::default()))
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), BoxError> {
//! let service = ServiceBuilder::new()
//!     // Authorize requests using `MyAuth`
//!     .layer(AsyncRequireAuthorizationLayer::new(MyAuth))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```
//!
//! Or using a closure:
//!
//! ```
//! use tower_async_http::auth::{AsyncRequireAuthorizationLayer, AsyncAuthorizeRequest};
//! use http::{Request, Response, StatusCode};
//! use http_body_util::Full;
//! use bytes::Bytes;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, BoxError};
//! use futures_util::future::BoxFuture;
//!
//! async fn check_auth<B>(request: &Request<B>) -> Option<UserId> {
//!     // ...
//!     # None
//! }
//!
//! #[derive(Debug)]
//! struct UserId(String);
//!
//! async fn handle(request: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, BoxError> {
//!     # todo!();
//!     // ...
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), BoxError> {
//! let service = ServiceBuilder::new()
//!     .layer(AsyncRequireAuthorizationLayer::new(|request: Request<Full<Bytes>>| async move {
//!         if let Some(user_id) = check_auth(&request).await {
//!             Ok(request)
//!         } else {
//!             let unauthorized_response = Response::builder()
//!                 .status(StatusCode::UNAUTHORIZED)
//!                 .body(Full::<Bytes>::default())
//!                 .unwrap();
//!
//!             Err(unauthorized_response)
//!         }
//!     }))
//!     .service_fn(handle);
//! # Ok(())
//! # }
//! ```

use http::{Request, Response};
use std::future::Future;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Layer that applies [`AsyncRequireAuthorization`] which authorizes all requests using the
/// [`Authorization`] header.
///
/// See the [module docs](crate::auth::async_require_authorization) for an example.
///
/// [`Authorization`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
#[derive(Debug, Clone)]
pub struct AsyncRequireAuthorizationLayer<T> {
    auth: T,
}

impl<T> AsyncRequireAuthorizationLayer<T> {
    /// Authorize requests using a custom scheme.
    pub fn new(auth: T) -> AsyncRequireAuthorizationLayer<T> {
        Self { auth }
    }
}

impl<S, T> Layer<S> for AsyncRequireAuthorizationLayer<T>
where
    T: Clone,
{
    type Service = AsyncRequireAuthorization<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AsyncRequireAuthorization::new(inner, self.auth.clone())
    }
}

/// Middleware that authorizes all requests using the [`Authorization`] header.
///
/// See the [module docs](crate::auth::async_require_authorization) for an example.
///
/// [`Authorization`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization
#[derive(Clone, Debug)]
pub struct AsyncRequireAuthorization<S, T> {
    inner: S,
    auth: T,
}

impl<S, T> AsyncRequireAuthorization<S, T> {
    define_inner_service_accessors!();
}

impl<S, T> AsyncRequireAuthorization<S, T> {
    /// Authorize requests using a custom scheme.
    ///
    /// The `Authorization` header is required to have the value provided.
    pub fn new(inner: S, auth: T) -> AsyncRequireAuthorization<S, T> {
        Self { inner, auth }
    }

    /// Returns a new [`Layer`] that wraps services with an [`AsyncRequireAuthorizationLayer`]
    /// middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(auth: T) -> AsyncRequireAuthorizationLayer<T> {
        AsyncRequireAuthorizationLayer::new(auth)
    }
}

impl<ReqBody, ResBody, S, Auth> Service<Request<ReqBody>> for AsyncRequireAuthorization<S, Auth>
where
    Auth: AsyncAuthorizeRequest<ReqBody, ResponseBody = ResBody>,
    S: Service<Request<Auth::RequestBody>, Response = Response<ResBody>> + Clone,
{
    type Response = Response<ResBody>;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let req = match self.auth.authorize(req).await {
            Ok(req) => req,
            Err(res) => return Ok(res),
        };
        self.inner.call(req).await
    }
}

/// Trait for authorizing requests.
pub trait AsyncAuthorizeRequest<B> {
    /// The type of request body returned by `authorize`.
    ///
    /// Set this to `B` unless you need to change the request body type.
    type RequestBody;

    /// The body type used for responses to unauthorized requests.
    type ResponseBody;

    /// Authorize the request.
    ///
    /// If the future resolves to `Ok(request)` then the request is allowed through, otherwise not.
    fn authorize(
        &self,
        request: Request<B>,
    ) -> impl std::future::Future<
        Output = Result<Request<Self::RequestBody>, Response<Self::ResponseBody>>,
    >;
}

impl<B, F, Fut, ReqBody, ResBody> AsyncAuthorizeRequest<B> for F
where
    F: Fn(Request<B>) -> Fut,
    Fut: Future<Output = Result<Request<ReqBody>, Response<ResBody>>>,
{
    type RequestBody = ReqBody;
    type ResponseBody = ResBody;

    async fn authorize(
        &self,
        request: Request<B>,
    ) -> Result<Request<Self::RequestBody>, Response<Self::ResponseBody>> {
        self(request).await
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    use crate::test_helpers::Body;

    use http::{header, StatusCode};
    use tower_async::{BoxError, ServiceBuilder};

    #[derive(Clone, Copy)]
    struct MyAuth;

    impl<B> AsyncAuthorizeRequest<B> for MyAuth
    where
        B: Send + 'static,
    {
        type RequestBody = B;
        type ResponseBody = Body;

        async fn authorize(
            &self,
            mut request: Request<B>,
        ) -> Result<Request<Self::RequestBody>, Response<Self::ResponseBody>> {
            let authorized = request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|it: &http::HeaderValue| it.to_str().ok())
                .and_then(|it| it.strip_prefix("Bearer "))
                .map(|it| it == "69420")
                .unwrap_or(false);

            if authorized {
                let user_id = UserId("6969".to_owned());
                request.extensions_mut().insert(user_id);
                Ok(request)
            } else {
                Err(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap())
            }
        }
    }

    #[derive(Debug, Clone)]
    struct UserId(String);

    #[tokio::test]
    async fn require_async_auth_works() {
        let service = ServiceBuilder::new()
            .layer(AsyncRequireAuthorizationLayer::new(MyAuth))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer 69420")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn require_async_auth_401() {
        let service = ServiceBuilder::new()
            .layer(AsyncRequireAuthorizationLayer::new(MyAuth))
            .service_fn(echo);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer deez")
            .body(Body::empty())
            .unwrap();

        let res = service.call(request).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    async fn echo<Body>(req: Request<Body>) -> Result<Response<Body>, BoxError> {
        Ok(Response::new(req.into_body()))
    }
}
