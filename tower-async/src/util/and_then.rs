use futures_core::TryFuture;
use futures_util::TryFutureExt;
use std::fmt;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Service returned by the [`and_then`] combinator.
///
/// [`and_then`]: crate::util::ServiceExt::and_then
#[derive(Clone)]
pub struct AndThen<S, F> {
    inner: S,
    f: F,
}

impl<S, F> fmt::Debug for AndThen<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AndThen")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

/// A [`Layer`] that produces a [`AndThen`] service.
///
/// [`Layer`]: tower_async_layer::Layer
#[derive(Clone, Debug)]
pub struct AndThenLayer<F> {
    f: F,
}

impl<S, F> AndThen<S, F> {
    /// Creates a new `AndThen` service.
    pub fn new(inner: S, f: F) -> Self {
        AndThen { f, inner }
    }

    /// Returns a new [`Layer`] that produces [`AndThen`] services.
    ///
    /// This is a convenience function that simply calls [`AndThenLayer::new`].
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(f: F) -> AndThenLayer<F> {
        AndThenLayer { f }
    }
}

impl<S, F, Request, Fut> Service<Request> for AndThen<S, F>
where
    S: Service<Request>,
    S::Error: Into<Fut::Error>,
    F: FnOnce(S::Response) -> Fut + Clone,
    Fut: TryFuture,
{
    type Response = Fut::Ok;
    type Error = Fut::Error;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        self.inner
            .call(request)
            .err_into()
            .and_then(self.f.clone())
            .await
    }
}

impl<F> AndThenLayer<F> {
    /// Creates a new [`AndThenLayer`] layer.
    pub fn new(f: F) -> Self {
        AndThenLayer { f }
    }
}

impl<S, F> Layer<S> for AndThenLayer<F>
where
    F: Clone,
{
    type Service = AndThen<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        AndThen {
            f: self.f.clone(),
            inner,
        }
    }
}
