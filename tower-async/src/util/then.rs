use std::{fmt, future::Future};

use tower_async_layer::Layer;
use tower_async_service::Service;

/// [`Service`] returned by the [`then`] combinator.
///
/// [`then`]: crate::util::ServiceExt::then
#[derive(Clone)]
pub struct Then<S, F> {
    inner: S,
    f: F,
}

impl<S, F> fmt::Debug for Then<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Then")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

/// A [`Layer`] that produces a [`Then`] service.
///
/// [`Layer`]: tower_async_layer::Layer
#[derive(Debug, Clone)]
pub struct ThenLayer<F> {
    f: F,
}

impl<S, F> Then<S, F> {
    /// Creates a new `Then` service.
    pub fn new(inner: S, f: F) -> Self {
        Then { f, inner }
    }

    /// Returns a new [`Layer`] that produces [`Then`] services.
    ///
    /// This is a convenience function that simply calls [`ThenLayer::new`].
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(f: F) -> ThenLayer<F> {
        ThenLayer { f }
    }
}

impl<S, F, Request, Response, Error, Fut> Service<Request> for Then<S, F>
where
    S: Service<Request>,
    S::Error: Into<Error>,
    F: Fn(Result<S::Response, S::Error>) -> Fut,
    Fut: Future<Output = Result<Response, Error>>,
{
    type Response = Response;
    type Error = Error;

    #[inline]
    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        let result = self.inner.call(request).await;
        (self.f)(result).await
    }
}

impl<F> ThenLayer<F> {
    /// Creates a new [`ThenLayer`] layer.
    pub fn new(f: F) -> Self {
        ThenLayer { f }
    }
}

impl<S, F> Layer<S> for ThenLayer<F>
where
    F: Clone,
{
    type Service = Then<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        Then {
            f: self.f.clone(),
            inner,
        }
    }
}
