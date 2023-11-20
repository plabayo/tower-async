use std::fmt;

use tower_async_layer::Layer;
use tower_async_service::Service;

/// Service returned by the [`MapRequest`] combinator.
///
/// [`MapRequest`]: crate::util::ServiceExt::map_request
#[derive(Clone)]
pub struct MapRequest<S, F> {
    inner: S,
    f: F,
}

impl<S, F> fmt::Debug for MapRequest<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapRequest")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

impl<S, F> MapRequest<S, F> {
    /// Creates a new [`MapRequest`] service.
    pub fn new(inner: S, f: F) -> Self {
        MapRequest { inner, f }
    }

    /// Returns a new [`Layer`] that produces [`MapRequest`] services.
    ///
    /// This is a convenience function that simply calls [`MapRequestLayer::new`].
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(f: F) -> MapRequestLayer<F> {
        MapRequestLayer { f }
    }
}

impl<S, F, R1, R2> Service<R1> for MapRequest<S, F>
where
    S: Service<R2>,
    F: Fn(R1) -> R2,
{
    type Response = S::Response;
    type Error = S::Error;

    #[inline]
    async fn call(&self, request: R1) -> Result<Self::Response, Self::Error> {
        self.inner.call((self.f)(request)).await
    }
}

/// A [`Layer`] that produces [`MapRequest`] services.
///
/// [`Layer`]: tower_async_layer::Layer
#[derive(Clone, Debug)]
pub struct MapRequestLayer<F> {
    f: F,
}

impl<F> MapRequestLayer<F> {
    /// Creates a new [`MapRequestLayer`].
    pub fn new(f: F) -> Self {
        MapRequestLayer { f }
    }
}

impl<S, F> Layer<S> for MapRequestLayer<F>
where
    F: Clone,
{
    type Service = MapRequest<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        MapRequest {
            f: self.f.clone(),
            inner,
        }
    }
}
