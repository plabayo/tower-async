use crate::{AsyncServiceWrapper, ClassicServiceWrapper};

/// ClassicLayerExt adds a method to _any_ [`tower_async_layer::Layer`] that
/// wraps it in a [ClassicLayer] so that it can be used within a [`tower_layer::Layer`] environment.
///
/// [`tower_async_layer::Layer`]: https://docs.rs/tower-async-layer/*/tower_async_layer/trait.Layer.html
/// [`tower_layer::Layer`]: https://docs.rs/tower-layer/*/tower_layer/trait.Layer.html
pub trait ClassicLayerExt<S>: tower_async_layer::Layer<S> {
    /// Wrap a [`tower_async_layer::Layer`],
    /// so that it can be used within a [`tower_layer::Layer`] environment.
    ///
    /// [`tower_async_layer::Layer`]: https://docs.rs/tower-async-layer/*/tower_async_layer/trait.Layer.html
    /// [`tower_layer::Layer`]: https://docs.rs/tower-layer/*/tower_layer/trait.Layer.html
    fn into_classic(self) -> ClassicLayer<Self, S>
    where
        Self: Sized,
    {
        ClassicLayer::new(self)
    }
}

impl<L, S> ClassicLayerExt<S> for L where L: tower_async_layer::Layer<S> + Sized {}

impl<L, S> From<L> for ClassicLayer<L, S>
where
    L: tower_async_layer::Layer<S>,
{
    fn from(inner: L) -> Self {
        Self::new(inner)
    }
}

/// A wrapper around a [`tower_layer::Layer`] that implements
/// [`tower_async_layer::Layer`] and is the type returned
/// by [ClassicLayerExt::into_classic].
///
/// [`tower_layer::Layer`]: https://docs.rs/tower-layer/*/tower_layer/trait.Layer.html
/// [`tower_async_layer::Layer`]: https://docs.rs/tower-async-layer/*/tower_async_layer/trait.Layer.html
pub struct ClassicLayer<L, S> {
    inner: L,
    _marker: std::marker::PhantomData<S>,
}

impl<L, S> std::fmt::Debug for ClassicLayer<L, S>
where
    L: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassicLayer")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<L, S> Clone for ClassicLayer<L, S>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<L, S> ClassicLayer<L, S> {
    /// Create a new [ClassicLayer] wrapping `inner`.
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<L, S> tower_layer::Layer<S> for ClassicLayer<L, S>
where
    L: tower_async_layer::Layer<AsyncServiceWrapper<S>>,
{
    type Service =
        ClassicServiceWrapper<<L as tower_async_layer::Layer<AsyncServiceWrapper<S>>>::Service>;

    #[inline]
    fn layer(&self, service: S) -> Self::Service {
        let service = AsyncServiceWrapper::new(service);
        let service = self.inner.layer(service);
        ClassicServiceWrapper::new(service)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use tower::ServiceExt;

    use super::*;

    #[derive(Debug)]
    struct AsyncDelayService<S> {
        inner: S,
        delay: std::time::Duration,
    }

    impl<S> AsyncDelayService<S> {
        fn new(inner: S, delay: std::time::Duration) -> Self {
            Self { inner, delay }
        }
    }

    impl<S, Request> tower_async_service::Service<Request> for AsyncDelayService<S>
    where
        S: tower_async_service::Service<Request>,
    {
        type Response = S::Response;
        type Error = S::Error;

        async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
            tokio::time::sleep(self.delay).await;
            self.inner.call(request).await
        }
    }

    #[derive(Debug)]
    struct AsyncDelayLayer {
        delay: std::time::Duration,
    }

    impl AsyncDelayLayer {
        fn new(delay: std::time::Duration) -> Self {
            Self { delay }
        }
    }

    impl<S> tower_async_layer::Layer<S> for AsyncDelayLayer {
        type Service = AsyncDelayService<S>;

        fn layer(&self, service: S) -> Self::Service {
            AsyncDelayService::new(service, self.delay)
        }
    }

    #[derive(Debug)]
    struct EchoService;

    impl<Request> tower_service::Service<Request> for EchoService {
        type Response = Request;
        type Error = Infallible;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request) -> Self::Future {
            std::future::ready(Ok(req))
        }
    }

    /// Test that a regular async (trait) layer can be used in a classic tower builder.
    /// While this is not the normal use case of this crate, it might as well be supported
    /// for those cases where one _has_ to use a classic tower envirioment,
    /// but wants to (already be able to) use an async layer.
    #[tokio::test]
    async fn test_classic_layer_in_classic_tower_builder() {
        let service = tower::ServiceBuilder::new()
            .rate_limit(1, std::time::Duration::from_millis(200))
            .layer(AsyncDelayLayer::new(std::time::Duration::from_millis(100)).into_classic())
            .service(EchoService);

        let response = service.oneshot("hello").await.unwrap();
        assert_eq!(response, "hello");
    }
}
