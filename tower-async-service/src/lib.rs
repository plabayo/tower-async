#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]
// `rustdoc::broken_intra_doc_links` is checked on CI

//! Definition of the core `Service` trait to Tower
//!
//! The [`Service`] trait provides the necessary abstractions for defining
//! request / response clients and servers. It is simple but powerful and is
//! used as the foundation for the rest of Tower.

/// An asynchronous function from a `Request` to a `Response`.
///
/// The `Service` trait is a simplified interface making it easy to write
/// network applications in a modular and reusable way, decoupled from the
/// underlying protocol. It is one of Tower's fundamental abstractions.
///
/// # Functional
///
/// A `Service` is a function of a `Request`. It immediately returns a
/// `Future` representing the eventual completion of processing the
/// request. The actual request processing may happen at any time in the
/// future, on any thread or executor. The processing may depend on calling
/// other services. At some point in the future, the processing will complete,
/// and the `Future` will resolve to a response or error.
///
/// At a high level, the `Service::call` function represents an RPC request. The
/// `Service` value can be a server or a client.
///
/// # Server
///
/// An RPC server *implements* the `Service` trait. Requests received by the
/// server over the network are deserialized and then passed as an argument to the
/// server value. The returned response is sent back over the network.
///
/// As an example, here is how an HTTP request is processed by a server:
///
/// ```rust
/// # use tower_async_service::Service;
/// use http::{Request, Response, StatusCode};
///
/// struct HelloWorld;
///
/// impl Service<Request<Vec<u8>>> for HelloWorld {
///     type Response = Response<Vec<u8>>;
///     type Error = http::Error;
///
///     async fn call(&self, req: Request<Vec<u8>>) -> Result<Self::Response, Self::Error> {
///         // create the body
///         let body: Vec<u8> = "hello, world!\n"
///             .as_bytes()
///             .to_owned();
///         // Create the HTTP response
///         let resp = Response::builder()
///             .status(StatusCode::OK)
///             .body(body)
///             .expect("Unable to create `http::Response`");
///         // Return the response
///         Ok(resp)
///     }
/// }
/// ```
///
/// # Client
///
/// A client consumes a service by using a `Service` value. The client may
/// issue requests by invoking `call` and passing the request as an argument.
/// It then receives the response by waiting for the returned future.
///
/// As an example, here is how a Redis request would be issued:
///
/// ```rust,ignore
/// let client = redis::Client::new()
///     .connect("127.0.0.1:6379".parse().unwrap())
///     .unwrap();
///
/// let resp = client.call(Cmd::set("foo", "this is the value of foo")).await?;
///
/// // Wait for the future to resolve
/// println!("Redis response: {:?}", resp);
/// ```
///
/// # Middleware / Layer
///
/// More often than not, all the pieces needed for writing robust, scalable
/// network applications are the same no matter the underlying protocol. By
/// unifying the API for both clients and servers in a protocol agnostic way,
/// it is possible to write middleware that provide these pieces in a
/// reusable way.
///
/// Take timeouts as an example:
///
/// ```rust
/// use tower_async_service::Service;
/// use tower_async_layer::Layer;
/// use futures::FutureExt;
/// use std::future::Future;
/// use std::task::{Context, Poll};
/// use std::time::Duration;
/// use std::pin::Pin;
/// use std::fmt;
/// use std::error::Error;
///
/// // Our timeout service, which wraps another service and
/// // adds a timeout to its response future.
/// pub struct Timeout<T> {
///     inner: T,
///     timeout: Duration,
/// }
///
/// impl<T> Timeout<T> {
///     pub fn new(inner: T, timeout: Duration) -> Timeout<T> {
///         Timeout {
///             inner,
///             timeout
///         }
///     }
/// }
///
/// // The error returned if processing a request timed out
/// #[derive(Debug)]
/// pub struct Expired;
///
/// impl fmt::Display for Expired {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "expired")
///     }
/// }
///
/// impl Error for Expired {}
///
/// // We can implement `Service` for `Timeout<T>` if `T` is a `Service`
/// impl<T, Request> Service<Request> for Timeout<T>
/// where
///     T: Service<Request>,
///     T::Error: Into<Box<dyn Error + Send + Sync>> + 'static,
///     T::Response: 'static,
/// {
///     // `Timeout` doesn't modify the response type, so we use `T`'s response type
///     type Response = T::Response;
///     // Errors may be either `Expired` if the timeout expired, or the inner service's
///     // `Error` type. Therefore, we return a boxed `dyn Error + Send + Sync` trait object to erase
///     // the error's type.
///     type Error = Box<dyn Error + Send + Sync>;
///
///     async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
///         tokio::select! {
///             res = self.inner.call(req) => {
///                 res.map_err(|err| err.into())
///             },
///             _ = tokio::time::sleep(self.timeout) => {
///                 Err(Box::new(Expired) as Box<dyn Error + Send + Sync>)
///             },
///         }
///     }
/// }
///
/// // A layer for wrapping services in `Timeout`
/// pub struct TimeoutLayer(Duration);
///
/// impl TimeoutLayer {
///     pub fn new(delay: Duration) -> Self {
///         TimeoutLayer(delay)
///     }
/// }
///
/// impl<S> Layer<S> for TimeoutLayer {
///     type Service = Timeout<S>;
///
///     fn layer(&self, service: S) -> Timeout<S> {
///         Timeout::new(service, self.0)
///     }
/// }
/// ```
///
/// The above timeout implementation is decoupled from the underlying protocol
/// and is also decoupled from client or server concerns. In other words, the
/// same timeout middleware could be used in either a client or a server.
///
/// # Backpressure
///
/// There is no explicit support for Backpressure in the service.
/// This can be achieved by having middleware at the front which
/// implements backpressure and propagates it through the service
///
/// Or one can also implement it in the location where this
/// service gets created.
///
/// The original tower library had a `poll_ready` method which
/// was used to implement backpressure. This was removed in this fork in
/// favor of the above approach.
pub trait Service<Request> {
    /// Responses given by the service.
    type Response;

    /// Errors produced by the service.
    type Error;

    /// Process the request and return the response asynchronously.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    fn call(
        &self,
        req: Request,
    ) -> impl std::future::Future<Output = Result<Self::Response, Self::Error>>;
}

impl<'a, S, Request> Service<Request> for &'a mut S
where
    S: Service<Request> + 'a,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(
        &self,
        request: Request,
    ) -> impl std::future::Future<
        Output = Result<
            <&'a mut S as Service<Request>>::Response,
            <&'a mut S as Service<Request>>::Error,
        >,
    > {
        (**self).call(request)
    }
}

impl<S, Request> Service<Request> for Box<S>
where
    S: Service<Request> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(
        &self,
        request: Request,
    ) -> impl std::future::Future<Output = Result<Self::Response, Self::Error>> {
        (**self).call(request)
    }
}
