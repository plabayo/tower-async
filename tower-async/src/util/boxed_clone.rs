use std::fmt;
use std::future::Future;
use tower_async_layer::{layer_fn, LayerFn};
use tower_async_service::Service;

use super::boxed::erase::ServiceDyn;

/// A [`Clone`] + [`Send`] boxed [`Service`].
///
/// [`BoxCloneService`] turns a service into a trait object, allowing the
/// response future type to be dynamic, and allowing the service to be cloned.
///
/// This is similar to [`BoxService`](super::BoxService) except the resulting
/// service implements [`Clone`].
///
/// # Example
///
/// ```
/// use tower_async::{Service, ServiceBuilder, BoxError, util::BoxCloneService};
/// use std::time::Duration;
/// #
/// # struct Request;
/// # struct Response;
/// # impl Response {
/// #     fn new() -> Self { Self }
/// # }
///
/// // This service has a complex type that is hard to name
/// let service = ServiceBuilder::new()
///     .map_request(|req| {
///         println!("received request");
///         req
///     })
///     .map_response(|res| {
///         println!("response produced");
///         res
///     })
///     .timeout(Duration::from_secs(10))
///     .service_fn(|req: Request| async {
///         Ok::<_, BoxError>(Response::new())
///     });
/// # let service = assert_service(service);
///
/// // `BoxCloneService` will erase the type so it's nameable
/// let service: BoxCloneService<Request, Response, BoxError> = BoxCloneService::new(service);
/// # let service = assert_service(service);
///
/// // And we can still clone the service
/// let cloned_service = service.clone();
/// #
/// # fn assert_service<S, R>(svc: S) -> S
/// # where S: Service<R> { svc }
/// ```
pub struct BoxCloneService<T, U, E>(Box<dyn CloneService<T, Response = U, Error = E> + Send>);

impl<T, U, E> BoxCloneService<T, U, E> {
    /// Create a new `BoxCloneService`.
    pub fn new<S>(inner: S) -> Self
    where
        S: ServiceDyn<T, Response = U, Error = E> + Clone + Send + 'static,
    {
        BoxCloneService(Box::new(inner))
    }

    /// Returns a [`Layer`] for wrapping a [`Service`] in a [`BoxCloneService`]
    /// middleware.
    ///
    /// [`Layer`]: crate::Layer
    pub fn layer<S>() -> LayerFn<fn(S) -> Self>
    where
        S: Service<T, Response = U, Error = E> + Clone + Send + 'static,
        T: 'static,
    {
        layer_fn(Self::new)
    }
}

impl<T, U, E> Service<T> for BoxCloneService<T, U, E> {
    type Response = U;
    type Error = E;

    #[inline]
    fn call(&self, request: T) -> impl Future<Output = Result<Self::Response, Self::Error>> {
        self.0.call(request)
    }
}

impl<T, U, E> Clone for BoxCloneService<T, U, E> {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

trait CloneService<R>: ServiceDyn<R> {
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<R, Response = Self::Response, Error = Self::Error> + Send>;
}

impl<R, T> CloneService<R> for T
where
    T: ServiceDyn<R> + Send + Clone + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<R, Response = T::Response, Error = T::Error> + Send> {
        Box::new(self.clone())
    }
}

impl<T, U, E> fmt::Debug for BoxCloneService<T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BoxCloneService").finish()
    }
}
