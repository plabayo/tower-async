use super::erase::ServiceDyn;
use tower_async_layer::{layer_fn, LayerFn};
use tower_async_service::Service;

use std::fmt;
use std::future::Future;

/// A boxed `Service + Send` trait object.
///
/// [`BoxService`] turns a service into a trait object, allowing the response
/// future type to be dynamic. This type requires both the service and the
/// response future to be [`Send`].
///
/// If you need a boxed [`Service`] that implements [`Clone`] consider using
/// [`BoxCloneService`](crate::util::BoxCloneService).
///
/// Dynamically dispatched [`Service`] objects allow for erasing the underlying
/// [`Service`] type and using the `Service` instances as opaque handles. This can
/// be useful when the service instance cannot be explicitly named for whatever
/// reason.
///
/// # Examples
///
/// ```
/// use futures_util::future::ready;
/// # use tower_async_service::Service;
/// # use tower_async::util::{BoxService, service_fn};
/// // Respond to requests using a closure, but closures cannot be named...
/// # pub fn main() {
/// let svc = service_fn(|mut request: String| async move {
///     request.push_str(" response");
///     Ok(request)
/// });
///
/// let service: BoxService<String, String, ()> = BoxService::new(svc);
/// # drop(service);
/// }
/// ```
///
/// [`Service`]: crate::Service
/// [`Rc`]: std::rc::Rc
pub struct BoxService<T, U, E> {
    inner: Box<dyn ServiceDyn<T, Response = U, Error = E> + Send + Sync + 'static>,
}

impl<T, U, E> BoxService<T, U, E> {
    #[allow(missing_docs)]
    pub fn new<S>(inner: S) -> Self
    where
        S: ServiceDyn<T, Response = U, Error = E> + Send + Sync + 'static,
    {
        // rust can't infer the type
        let inner: Box<dyn ServiceDyn<T, Response = U, Error = E> + Send + Sync + 'static> = Box::new(inner);
        BoxService { inner }
    }

    /// Returns a [`Layer`] for wrapping a [`Service`] in a [`BoxService`]
    /// middleware.
    ///
    /// [`Layer`]: crate::Layer
    pub fn layer<S>() -> LayerFn<fn(S) -> Self>
    where
        S: Service<T, Response = U, Error = E, call(): Send + Sync> + Send + Sync + 'static,
        U: Send + Sync + 'static,
        E: Send + Sync + 'static,
        T: Send + 'static,
    {
        layer_fn(Self::new)
    }
}

impl<T, U, E> Service<T> for BoxService<T, U, E>
    where
        U: Send + Sync + 'static,
        E: Send + Sync + 'static,
        T: Send + 'static,
{
    type Response = U;
    type Error = E;

    fn call(&self, request: T) -> impl Future<Output = Result<U, E>> {
        self.inner.call(request)
    }
}

impl<T, U, E> fmt::Debug for BoxService<T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BoxService").finish()
    }
}
