use futures_util::FutureExt;
use std::fmt;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Service returned by the [`map_result`] combinator.
///
/// [`map_result`]: crate::util::ServiceExt::map_result
#[derive(Clone)]
pub struct MapResult<S, F> {
    inner: S,
    f: F,
}

impl<S, F> fmt::Debug for MapResult<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResult")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

/// A [`Layer`] that produces a [`MapResult`] service.
///
/// [`Layer`]: tower_async_layer::Layer
#[derive(Debug, Clone)]
pub struct MapResultLayer<F> {
    f: F,
}

impl<S, F> MapResult<S, F> {
    /// Creates a new [`MapResult`] service.
    pub fn new(inner: S, f: F) -> Self {
        MapResult { f, inner }
    }

    /// Returns a new [`Layer`] that produces [`MapResult`] services.
    ///
    /// This is a convenience function that simply calls [`MapResultLayer::new`].
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(f: F) -> MapResultLayer<F> {
        MapResultLayer { f }
    }
}

impl<S, F, Request, Response, Error> Service<Request> for MapResult<S, F>
where
    S: Service<Request>,
    Error: From<S::Error>,
    F: FnOnce(Result<S::Response, S::Error>) -> Result<Response, Error> + Clone,
{
    type Response = Response;
    type Error = Error;

    #[inline]
    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        self.inner.call(request).map(self.f.clone()).await
    }
}

impl<F> MapResultLayer<F> {
    /// Creates a new [`MapResultLayer`] layer.
    pub fn new(f: F) -> Self {
        MapResultLayer { f }
    }
}

impl<S, F> Layer<S> for MapResultLayer<F>
where
    F: Clone,
{
    type Service = MapResult<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        MapResult {
            f: self.f.clone(),
            inner,
        }
    }
}
