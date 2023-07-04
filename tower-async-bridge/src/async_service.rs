use crate::AsyncServiceWrapper;

/// Extension for a [`tower::Service`] to turn it into an async [`Service`].
///
/// [`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
/// [`Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
pub trait AsyncServiceExt<Request>: tower_service::Service<Request> {
    /// Turn this [`tower::Service`] into an async [`Service`].
    ///
    /// [`Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
    fn into_async(self) -> AsyncServiceWrapper<Self>
    where
        Self: Sized,
    {
        AsyncServiceWrapper::new(self)
    }
}

impl<S, Request> AsyncServiceExt<Request> for S where S: tower_service::Service<Request> {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        convert::Infallible,
        future::Future,
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };

    use tower::Service;
    use tower_async::{ServiceBuilder, ServiceExt};

    struct EchoService;

    impl Service<String> for EchoService {
        type Response = String;
        type Error = Infallible;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: String) -> Self::Future {
            // create a response in a future.
            let fut = async { Ok(req) };

            // Return the response as an immediate future
            Box::pin(fut)
        }
    }

    struct AsyncEchoService;

    impl tower_async::Service<String> for AsyncEchoService {
        type Response = String;
        type Error = Infallible;

        async fn call(&mut self, req: String) -> Result<Self::Response, Self::Error> {
            Ok(req)
        }
    }

    #[tokio::test]
    async fn test_async_service_ext() {
        let service = EchoService;
        let service = ServiceBuilder::new()
            .timeout(Duration::from_secs(1))
            .service(service.into_async()); // use tower service as async service

        let response = service.oneshot("hello".to_string()).await.unwrap();
        assert_eq!(response, "hello");
    }
}
