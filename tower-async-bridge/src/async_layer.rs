use crate::{AsyncServiceWrapper, ClassicServiceWrapper};

/// AsyncLayerExt adds a method to _any_ [tower_layer::Layer] that
/// wraps it in an [AsyncLayer] so that it can be used within a [tower_async_layer::Layer] environment.
pub trait AsyncLayerExt<S>: tower_layer::Layer<S> {
    /// Wrap a [tower_layer::Layer],
    /// so that it can be used within a [tower_async_layer::Layer] environment.
    fn into_async(self) -> AsyncLayer<Self, S>
    where
        Self: Sized,
    {
        AsyncLayer::new(self)
    }
}

impl<L, S> AsyncLayerExt<S> for L where L: tower_layer::Layer<S> + Sized {}

impl<L, S> From<L> for AsyncLayer<L, S>
where
    L: tower_layer::Layer<S>,
{
    fn from(inner: L) -> Self {
        Self::new(inner)
    }
}

/// A wrapper around a [tower_layer::Layer] that implements
/// [tower_async_layer::Layer] and is the type returned
/// by [AsyncLayerExt::into_async].
pub struct AsyncLayer<L, S> {
    inner: L,
    _marker: std::marker::PhantomData<S>,
}

impl<L, S> std::fmt::Debug for AsyncLayer<L, S>
where
    L: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncLayer")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<L, S> AsyncLayer<L, S> {
    /// Create a new [AsyncLayer] wrapping `inner`.
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<L, S> tower_async_layer::Layer<S> for AsyncLayer<L, S>
where
    L: tower_layer::Layer<ClassicServiceWrapper<S>>,
{
    type Service =
        AsyncServiceWrapper<<L as tower_layer::Layer<ClassicServiceWrapper<S>>>::Service>;

    fn layer(&self, service: S) -> Self::Service {
        let service = ClassicServiceWrapper::new(service);
        let service = self.inner.layer(service);
        AsyncServiceWrapper::new(service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pin_project_lite::pin_project;
    use std::convert::Infallible;
    use tower_async::ServiceExt;

    #[derive(Debug)]
    struct DelayService<S> {
        inner: S,
        delay: std::time::Duration,
    }

    impl<S> DelayService<S> {
        fn new(inner: S, delay: std::time::Duration) -> Self {
            Self { inner, delay }
        }
    }

    impl<S, Request> tower_service::Service<Request> for DelayService<S>
    where
        S: tower_service::Service<Request>,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = DelayFuture<tokio::time::Sleep, S::Future>;

        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, request: Request) -> Self::Future {
            DelayFuture::new(tokio::time::sleep(self.delay), self.inner.call(request))
        }
    }

    enum DelayFutureState {
        Delaying,
        Serving,
    }

    pin_project! {
        struct DelayFuture<T, U> {
            state: DelayFutureState,
            #[pin]
            delay: T,
            #[pin]
            serve: U,
        }
    }

    impl<T, U> DelayFuture<T, U> {
        fn new(delay: T, serve: U) -> Self {
            Self {
                state: DelayFutureState::Delaying,
                delay,
                serve,
            }
        }
    }

    impl<T, U> std::future::Future for DelayFuture<T, U>
    where
        T: std::future::Future,
        U: std::future::Future,
    {
        type Output = U::Output;

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            let this = self.project();
            match this.state {
                DelayFutureState::Delaying => {
                    let _ = futures_core::ready!(this.delay.poll(cx));
                    *this.state = DelayFutureState::Serving;
                    this.serve.poll(cx)
                }
                DelayFutureState::Serving => this.serve.poll(cx),
            }
        }
    }

    #[derive(Debug)]
    struct DelayLayer {
        delay: std::time::Duration,
    }

    impl DelayLayer {
        fn new(delay: std::time::Duration) -> Self {
            Self { delay }
        }
    }

    impl<S> tower_layer::Layer<S> for DelayLayer {
        type Service = DelayService<S>;

        fn layer(&self, service: S) -> Self::Service {
            DelayService::new(service, self.delay)
        }
    }

    #[derive(Debug)]
    struct AsyncEchoService;

    impl<Request> tower_async_service::Service<Request> for AsyncEchoService {
        type Response = Request;
        type Error = Infallible;

        async fn call(&mut self, req: Request) -> Result<Self::Response, Self::Error> {
            Ok(req)
        }
    }

    /// Test that a classic Tower layer can be used in an async tower builder.
    /// While this is not the normal use case of this crate, it might as well be supported
    /// for those cases where one _has_ to use a classic layer in an async tower envirioment,
    /// because for example the functionality was not yet ported to an async trait version.
    #[tokio::test]
    async fn test_async_layer_in_async_tower_builder() {
        let service = tower_async::ServiceBuilder::new()
            .timeout(std::time::Duration::from_millis(200))
            .layer(DelayLayer::new(std::time::Duration::from_millis(100)).into_async())
            .service(AsyncEchoService);

        let response = service.oneshot("hello").await.unwrap();
        assert_eq!(response, "hello");
    }
}
