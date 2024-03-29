//! Builder types to compose layers and services

use tower_async_layer::{Identity, Layer, Stack};
use tower_async_service::Service;

use std::fmt;

/// Declaratively construct [`Service`] values.
///
/// [`ServiceBuilder`] provides a [builder-like interface][builder] for composing
/// layers to be applied to a [`Service`].
///
/// # Service
///
/// A [`Service`] is a trait representing an asynchronous function of a request
/// to a response. It is similar to `async fn(Request) -> Result<Response, Error>`.
///
/// A [`Service`] is typically bound to a single transport, such as a TCP
/// connection.  It defines how _all_ inbound or outbound requests are handled
/// by that connection.
///
/// [builder]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
///
/// # Order
///
/// The order in which layers are added impacts how requests are handled. Layers
/// that are added first will be called with the request first. The argument to
/// `service` will be last to see the request.
///
/// [`Service`]: crate::Service
#[derive(Clone)]
pub struct ServiceBuilder<L> {
    layer: L,
}

impl Default for ServiceBuilder<Identity> {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceBuilder<Identity> {
    /// Create a new [`ServiceBuilder`].
    pub fn new() -> Self {
        ServiceBuilder {
            layer: Identity::new(),
        }
    }
}

impl<L> ServiceBuilder<L> {
    /// Add a new layer `T` into the [`ServiceBuilder`].
    ///
    /// This wraps the inner service with the service provided by a user-defined
    /// [`Layer`]. The provided layer must implement the [`Layer`] trait.
    ///
    /// [`Layer`]: crate::Layer
    pub fn layer<T>(self, layer: T) -> ServiceBuilder<Stack<T, L>> {
        ServiceBuilder {
            layer: Stack::new(layer, self.layer),
        }
    }

    /// Optionally add a new layer `T` into the [`ServiceBuilder`].
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use tower_async::Service;
    /// # use tower_async::builder::ServiceBuilder;
    /// # use tower_async::timeout::TimeoutLayer;
    /// # async fn wrap<S>(svc: S) where S: Service<(), Error = &'static str> + 'static + Send {
    /// # let timeout = Some(Duration::new(10, 0));
    /// // Apply a timeout if configured
    /// ServiceBuilder::new()
    ///     .option_layer(timeout.map(TimeoutLayer::new))
    ///     .service(svc)
    /// # ;
    /// # }
    /// ```
    #[cfg(feature = "util")]
    pub fn option_layer<T>(
        self,
        layer: Option<T>,
    ) -> ServiceBuilder<Stack<crate::util::Either<T, Identity>, L>> {
        self.layer(crate::util::option_layer(layer))
    }

    /// Add a [`Layer`] built from a function that accepts a service and returns another service.
    ///
    /// See the documentation for [`layer_fn`] for more details.
    ///
    /// [`layer_fn`]: crate::layer::layer_fn
    pub fn layer_fn<F>(self, f: F) -> ServiceBuilder<Stack<crate::layer::LayerFn<F>, L>> {
        self.layer(crate::layer::layer_fn(f))
    }

    /// Retry failed requests according to the given [retry policy][policy].
    ///
    /// `policy` determines which failed requests will be retried. It must
    /// implement the [`retry::Policy`][policy] trait.
    ///
    /// This wraps the inner service with an instance of the [`Retry`]
    /// middleware.
    ///
    /// [`Retry`]: crate::retry
    /// [policy]: crate::retry::Policy
    #[cfg(feature = "retry")]
    pub fn retry<P>(self, policy: P) -> ServiceBuilder<Stack<crate::retry::RetryLayer<P>, L>> {
        self.layer(crate::retry::RetryLayer::new(policy))
    }

    /// Fail requests that take longer than `timeout`.
    ///
    /// If the next layer takes more than `timeout` to respond to a request,
    /// processing is terminated and an error is returned.
    ///
    /// This wraps the inner service with an instance of the [`timeout`]
    /// middleware.
    ///
    /// [`timeout`]: crate::timeout
    #[cfg(feature = "timeout")]
    pub fn timeout(
        self,
        timeout: std::time::Duration,
    ) -> ServiceBuilder<Stack<crate::timeout::TimeoutLayer, L>> {
        self.layer(crate::timeout::TimeoutLayer::new(timeout))
    }

    /// Conditionally reject requests based on `predicate`.
    ///
    /// `predicate` must implement the [`Predicate`] trait.
    ///
    /// This wraps the inner service with an instance of the [`Filter`]
    /// middleware.
    ///
    /// [`Filter`]: crate::filter
    /// [`Predicate`]: crate::filter::Predicate
    #[cfg(feature = "filter")]
    pub fn filter<P>(
        self,
        predicate: P,
    ) -> ServiceBuilder<Stack<crate::filter::FilterLayer<P>, L>> {
        self.layer(crate::filter::FilterLayer::new(predicate))
    }

    /// Conditionally reject requests based on an asynchronous `predicate`.
    ///
    /// `predicate` must implement the [`AsyncPredicate`] trait.
    ///
    /// This wraps the inner service with an instance of the [`AsyncFilter`]
    /// middleware.
    ///
    /// [`AsyncFilter`]: crate::filter::AsyncFilter
    /// [`AsyncPredicate`]: crate::filter::AsyncPredicate
    #[cfg(feature = "filter")]
    pub fn filter_async<P>(
        self,
        predicate: P,
    ) -> ServiceBuilder<Stack<crate::filter::AsyncFilterLayer<P>, L>> {
        self.layer(crate::filter::AsyncFilterLayer::new(predicate))
    }

    /// Limit the number of in-flight requests.
    ///
    /// This wraps the inner service with an instance of the [`Limit`]
    /// middleware. The `policy` determines how to handle requests sent
    /// to the inner service when the limit has been reached.
    ///
    /// [`Limit`]: crate::limit::Limit
    #[cfg(feature = "limit")]
    pub fn limit<P>(self, policy: P) -> ServiceBuilder<Stack<crate::limit::LimitLayer<P>, L>> {
        self.layer(crate::limit::LimitLayer::new(policy))
    }

    /// Map one request type to another.
    ///
    /// This wraps the inner service with an instance of the [`MapRequest`]
    /// middleware.
    ///
    /// # Examples
    ///
    /// Changing the type of a request:
    ///
    /// ```rust
    /// use tower_async::ServiceBuilder;
    /// use tower_async::ServiceExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ()> {
    /// // Suppose we have some `Service` whose request type is `String`:
    /// let string_svc = tower_async::service_fn(|request: String| async move {
    ///     println!("request: {}", request);
    ///     Ok(())
    /// });
    ///
    /// // ...but we want to call that service with a `usize`. What do we do?
    ///
    /// let usize_svc = ServiceBuilder::new()
    ///      // Add a middleware that converts the request type to a `String`:
    ///     .map_request(|request: usize| format!("{}", request))
    ///     // ...and wrap the string service with that middleware:
    ///     .service(string_svc);
    ///
    /// // Now, we can call that service with a `usize`:
    /// usize_svc.oneshot(42).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Modifying the request value:
    ///
    /// ```rust
    /// use tower_async::ServiceBuilder;
    /// use tower_async::ServiceExt;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), ()> {
    /// // A service that takes a number and returns it:
    /// let svc = tower_async::service_fn(|request: usize| async move {
    ///    Ok(request)
    /// });
    ///
    /// let svc = ServiceBuilder::new()
    ///      // Add a middleware that adds 1 to each request
    ///     .map_request(|request: usize| request + 1)
    ///     .service(svc);
    ///
    /// let response = svc.oneshot(1).await?;
    /// assert_eq!(response, 2);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`MapRequest`]: crate::util::MapRequest
    #[cfg(feature = "util")]
    pub fn map_request<F, R1, R2>(
        self,
        f: F,
    ) -> ServiceBuilder<Stack<crate::util::MapRequestLayer<F>, L>>
    where
        F: Fn(R1) -> R2 + Clone,
    {
        self.layer(crate::util::MapRequestLayer::new(f))
    }

    /// Map one response type to another.
    ///
    /// This wraps the inner service with an instance of the [`MapResponse`]
    /// middleware.
    ///
    /// See the documentation for the [`map_response` combinator] for details.
    ///
    /// [`MapResponse`]: crate::util::MapResponse
    /// [`map_response` combinator]: crate::util::ServiceExt::map_response
    #[cfg(feature = "util")]
    pub fn map_response<F>(
        self,
        f: F,
    ) -> ServiceBuilder<Stack<crate::util::MapResponseLayer<F>, L>> {
        self.layer(crate::util::MapResponseLayer::new(f))
    }

    /// Map one error type to another.
    ///
    /// This wraps the inner service with an instance of the [`MapErr`]
    /// middleware.
    ///
    /// See the documentation for the [`map_err` combinator] for details.
    ///
    /// [`MapErr`]: crate::util::MapErr
    /// [`map_err` combinator]: crate::util::ServiceExt::map_err
    #[cfg(feature = "util")]
    pub fn map_err<F>(self, f: F) -> ServiceBuilder<Stack<crate::util::MapErrLayer<F>, L>> {
        self.layer(crate::util::MapErrLayer::new(f))
    }

    /// Apply an asynchronous function after the service, regardless of whether the future
    /// succeeds or fails.
    ///
    /// This wraps the inner service with an instance of the [`Then`]
    /// middleware.
    ///
    /// This is similar to the [`map_response`] and [`map_err`] functions,
    /// except that the *same* function is invoked when the service's future
    /// completes, whether it completes successfully or fails. This function
    /// takes the [`Result`] returned by the service's future, and returns a
    /// [`Result`].
    ///
    /// See the documentation for the [`then` combinator] for details.
    ///
    /// [`Then`]: crate::util::Then
    /// [`then` combinator]: crate::util::ServiceExt::then
    /// [`map_response`]: ServiceBuilder::map_response
    /// [`map_err`]: ServiceBuilder::map_err
    #[cfg(feature = "util")]
    pub fn then<F>(self, f: F) -> ServiceBuilder<Stack<crate::util::ThenLayer<F>, L>> {
        self.layer(crate::util::ThenLayer::new(f))
    }

    /// Executes a new future after this service's future resolves.
    ///
    /// This method can be used to change the [`Response`] type of the service
    /// into a different type. You can use this method to chain along a computation once the
    /// service's response has been resolved.
    ///
    /// This wraps the inner service with an instance of the [`AndThen`]
    /// middleware.
    ///
    /// See the documentation for the [`and_then` combinator] for details.
    ///
    /// [`Response`]: crate::Service::Response
    /// [`and_then` combinator]: crate::util::ServiceExt::and_then
    /// [`AndThen`]: crate::util::AndThen
    #[cfg(feature = "util")]
    pub fn and_then<F>(self, f: F) -> ServiceBuilder<Stack<crate::util::AndThenLayer<F>, L>> {
        self.layer(crate::util::AndThenLayer::new(f))
    }

    /// Maps this service's result type (`Result<Self::Response, Self::Error>`)
    /// to a different value, regardless of whether the future succeeds or
    /// fails.
    ///
    /// This wraps the inner service with an instance of the [`MapResult`]
    /// middleware.
    ///
    /// See the documentation for the [`map_result` combinator] for details.
    ///
    /// [`map_result` combinator]: crate::util::ServiceExt::map_result
    /// [`MapResult`]: crate::util::MapResult
    #[cfg(feature = "util")]
    pub fn map_result<F>(self, f: F) -> ServiceBuilder<Stack<crate::util::MapResultLayer<F>, L>> {
        self.layer(crate::util::MapResultLayer::new(f))
    }

    /// Returns the underlying `Layer` implementation.
    pub fn into_inner(self) -> L {
        self.layer
    }

    /// Wrap the service `S` with the middleware provided by this
    /// [`ServiceBuilder`]'s [`Layer`]'s, returning a new [`Service`].
    ///
    /// [`Layer`]: crate::Layer
    /// [`Service`]: crate::Service
    pub fn service<S>(&self, service: S) -> L::Service
    where
        L: Layer<S>,
    {
        self.layer.layer(service)
    }

    /// Wrap the async function `F` with the middleware provided by this [`ServiceBuilder`]'s
    /// [`Layer`]s, returning a new [`Service`].
    ///
    /// This is a convenience method which is equivalent to calling
    /// [`ServiceBuilder::service`] with a [`service_fn`], like this:
    ///
    /// ```rust
    /// # use tower_async::{ServiceBuilder, service_fn};
    /// # async fn handler_fn(_: ()) -> Result<(), ()> { Ok(()) }
    /// # let _ = {
    /// ServiceBuilder::new()
    ///     // ...
    ///     .service(service_fn(handler_fn))
    /// # };
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use tower_async::{ServiceBuilder, ServiceExt, BoxError, service_fn};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BoxError> {
    /// async fn handle(request: &'static str) -> Result<&'static str, BoxError> {
    ///    Ok(request)
    /// }
    ///
    /// let svc = ServiceBuilder::new()
    ///     .timeout(Duration::from_secs(10))
    ///     .service_fn(handle);
    ///
    /// let response = svc.oneshot("foo").await?;
    ///
    /// assert_eq!(response, "foo");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Layer`]: crate::Layer
    /// [`Service`]: crate::Service
    /// [`service_fn`]: crate::service_fn
    #[cfg(feature = "util")]
    pub fn service_fn<F>(self, f: F) -> L::Service
    where
        L: Layer<crate::util::ServiceFn<F>>,
    {
        self.service(crate::util::service_fn(f))
    }

    /// Check that the builder implements `Clone`.
    ///
    /// This can be useful when debugging type errors in `ServiceBuilder`s with lots of layers.
    ///
    /// Doesn't actually change the builder but serves as a type check.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tower_async::ServiceBuilder;
    ///
    /// let builder = ServiceBuilder::new()
    ///     // Do something before processing the request
    ///     .map_request(|request: String| {
    ///         println!("got request!");
    ///         request
    ///     })
    ///     // Ensure our `ServiceBuilder` can be cloned
    ///     .check_clone()
    ///     // Do something after processing the request
    ///     .map_response(|response: String| {
    ///         println!("got response!");
    ///         response
    ///     });
    /// ```
    #[inline]
    pub fn check_clone(self) -> Self
    where
        Self: Clone,
    {
        self
    }

    /// Check that the builder when given a service of type `S` produces a service that implements
    /// `Clone`.
    ///
    /// This can be useful when debugging type errors in `ServiceBuilder`s with lots of layers.
    ///
    /// Doesn't actually change the builder but serves as a type check.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tower_async::ServiceBuilder;
    ///
    /// # #[derive(Clone)]
    /// # struct MyService;
    /// #
    /// let builder = ServiceBuilder::new()
    ///     // Do something before processing the request
    ///     .map_request(|request: String| {
    ///         println!("got request!");
    ///         request
    ///     })
    ///     // Ensure that the service produced when given a `MyService` implements
    ///     .check_service_clone::<MyService>()
    ///     // Do something after processing the request
    ///     .map_response(|response: String| {
    ///         println!("got response!");
    ///         response
    ///     });
    /// ```
    #[inline]
    pub fn check_service_clone<S>(self) -> Self
    where
        L: Layer<S>,
        L::Service: Clone,
    {
        self
    }

    /// Check that the builder when given a service of type `S` produces a service with the given
    /// request, response, and error types.
    ///
    /// This can be useful when debugging type errors in `ServiceBuilder`s with lots of layers.
    ///
    /// Doesn't actually change the builder but serves as a type check.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tower_async::ServiceBuilder;
    /// use std::task::{Poll, Context};
    /// use tower_async::{Service, ServiceExt};
    ///
    /// // An example service
    /// struct MyService;
    ///
    /// impl Service<Request> for MyService {
    ///   type Response = Response;
    ///   type Error = Error;
    ///
    ///   async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
    ///       // ...
    ///       # todo!()
    ///   }
    /// }
    ///
    /// struct Request;
    /// struct Response;
    /// struct Error;
    ///
    /// struct WrappedResponse(Response);
    ///
    /// let builder = ServiceBuilder::new()
    ///     // At this point in the builder if given a `MyService` it produces a service that
    ///     // accepts `Request`s, produces `Response`s, and fails with `Error`s
    ///     .check_service::<MyService, Request, Response, Error>()
    ///     // Wrap responses in `WrappedResponse`
    ///     .map_response(|response: Response| WrappedResponse(response))
    ///     // Now the response type will be `WrappedResponse`
    ///     .check_service::<MyService, _, WrappedResponse, _>();
    /// ```
    #[inline]
    pub fn check_service<S, T, U, E>(self) -> Self
    where
        L: Layer<S>,
        L::Service: Service<T, Response = U, Error = E>,
    {
        self
    }
}

impl<L: fmt::Debug> fmt::Debug for ServiceBuilder<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ServiceBuilder").field(&self.layer).finish()
    }
}

impl<S, L> Layer<S> for ServiceBuilder<L>
where
    L: Layer<S>,
{
    type Service = L::Service;

    fn layer(&self, inner: S) -> Self::Service {
        self.layer.layer(inner)
    }
}
