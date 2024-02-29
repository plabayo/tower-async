use crate::util::BoxCloneService;
use std::{fmt, sync::Arc};
use tower_async_layer::{layer_fn, Layer};
use tower_async_service::Service;

/// A [`Clone`] + [`Send`] boxed [`Layer`].
///
/// [`BoxCloneServiceLayer`] turns a layer into a trait object, allowing both the [`Layer`] itself
/// and the output [`Service`] to be dynamic, while having consistent types.
///
/// This [`Layer`] produces [`BoxCloneService`] instances erasing the type of the
/// [`Service`] produced by the wrapped [`Layer`].
///
/// This is similar to [`BoxLayer`](super::BoxLayer) except the layer and resulting
/// service implements [`Clone`].
///
/// # Example
///
/// `BoxCloneServiceLayer` can, for example, be useful to create layers dynamically that otherwise wouldn't have
/// the same types, when the underlying service must be clone (for example, when building a MakeService)
/// In this example, we include a [`Timeout`] layer only if an environment variable is set. We can use
/// `BoxCloneService` to return a consistent type regardless of runtime configuration:
///
/// ```
/// use std::time::Duration;
/// use tower_async::{Service, ServiceBuilder, BoxError};
/// use tower_async::util::{BoxCloneServiceLayer, BoxCloneService};
///
/// #
/// # struct Request;
/// # struct Response;
/// # impl Response {
/// #     fn new() -> Self { Self }
/// # }
///
/// fn common_layer<S, T>() -> BoxCloneServiceLayer<S, T, S::Response, BoxError>
/// where
///     S: Service<T> + Clone + Send + 'static,
///     S::Error: Into<BoxError> + 'static,
///     T: 'static,
/// {
///     let builder = ServiceBuilder::new();
///
///     if std::env::var("SET_TIMEOUT").is_ok() {
///         let layer = builder
///             .timeout(Duration::from_secs(30))
///             .into_inner();
///
///         BoxCloneServiceLayer::new(layer)
///     } else {
///         let layer = builder
///             .map_err(Into::into)
///             .into_inner();
///
///         BoxCloneServiceLayer::new(layer)
///     }
/// }
///
/// // We can clone the layer (this is true of BoxLayer as well)
/// let boxed_clone_layer = common_layer();
///
/// let cloned_layer = boxed_clone_layer.clone();
///
/// // Using the `BoxCloneServiceLayer` we can create a `BoxCloneService`
/// let service: BoxCloneService<Request, Response, BoxError> = ServiceBuilder::new().layer(boxed_clone_layer)
///      .service_fn(|req: Request| async {
///         Ok::<_, BoxError>(Response::new())
///     });
///
/// # let service = assert_service(service);
///
/// // And we can still clone the service
/// let cloned_service = service.clone();
/// #
/// # fn assert_service<S, R>(svc: S) -> S
/// # where S: Service<R> { svc }
///
/// ```
///
/// [`Layer`]: tower_async_layer::Layer
/// [`Service`]: tower_async_service::Service
/// [`BoxService`]: super::BoxService
/// [`Timeout`]: crate::timeout
pub struct BoxCloneServiceLayer<In, T, U, E> {
    boxed: Arc<dyn Layer<In, Service = BoxCloneService<T, U, E>> + Send + 'static>,
}

impl<In, T, U, E> BoxCloneServiceLayer<In, T, U, E> {
    /// Create a new [`BoxCloneServiceLayer`].
    pub fn new<L>(inner_layer: L) -> Self
    where
        L: Layer<In> + Send + 'static,
        L::Service: Service<T, Response = U, Error = E, call(): Send + Sync> + Send + Sync + Clone + 'static,
        U: Send + Sync + 'static,
        E: Send + Sync + 'static,
        T: Send + 'static,
    {
        let layer = layer_fn(move |inner: In| {
            let out = inner_layer.layer(inner);
            BoxCloneService::new(out)
        });

        Self {
            boxed: Arc::new(layer),
        }
    }
}

impl<In, T, U, E> Layer<In> for BoxCloneServiceLayer<In, T, U, E> {
    type Service = BoxCloneService<T, U, E>;

    fn layer(&self, inner: In) -> Self::Service {
        self.boxed.layer(inner)
    }
}

impl<In, T, U, E> Clone for BoxCloneServiceLayer<In, T, U, E> {
    fn clone(&self) -> Self {
        Self {
            boxed: Arc::clone(&self.boxed),
        }
    }
}

impl<In, T, U, E> fmt::Debug for BoxCloneServiceLayer<In, T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BoxCloneServiceLayer").finish()
    }
}