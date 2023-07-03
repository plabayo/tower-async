use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

/// Extension trait for [`tower::Service`] that provides the [`into_classic`] method.
///
/// [`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
/// [`into_classic`]: https://docs.rs/tower-async/*/tower_async/trait.ClassicServiceExt.html#method.into_classic
pub trait ClassicServiceExt<Request>: tower_async_service::Service<Request> {
    /// Turn this [`tower::Service`] into an async [`Service`].
    ///
    /// [`Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
    /// [`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
    fn into_classic(self) -> ClassicService<Self, Request>
    where
        Self: Sized,
    {
        ClassicService(Some(self), PhantomData)
    }
}

impl<S, Request> ClassicServiceExt<Request> for S where S: tower_async_service::Service<Request> {}

/// Service returned by [`ClassicServiceExt::into_classic`].
///
/// [`ClassicServiceExt::into_classic`]: https://docs.rs/tower-async/*/tower_async/trait.ClassicServiceExt.html#method.into_classic
pub struct ClassicService<S, Request>(pub(crate) Option<S>, pub(crate) PhantomData<Request>);

impl<S, Request> std::fmt::Debug for ClassicService<S, Request>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassicService")
            .field("inner", &self.0)
            .finish()
    }
}

impl<S, Request> tower_service::Service<Request> for ClassicService<S, Request>
where
    S: tower_async_service::Service<Request> + 'static,
    S::Error: 'static,
    Request: 'static,
{
    type Response = S::Response;
    type Error = ClassicServiceError<S::Error>;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(if self.0.is_some() {
            Ok(())
        } else {
            Err(ClassicServiceError::ServiceConsumed)
        })
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let mut service = self.0.take().expect("service must be present");
        let future = async move {
            let request = request;
            service
                .call(request)
                .await
                .map_err(ClassicServiceError::ServiceError)
        };
        Box::pin(future)
    }
}

/// Error returned by [`ClassicService`].
#[derive(Debug)]
pub enum ClassicServiceError<E> {
    ServiceConsumed,
    ServiceError(E),
}

impl<E> std::fmt::Display for ClassicServiceError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassicServiceError::ServiceConsumed => write!(f, "service consumed"),
            ClassicServiceError::ServiceError(e) => write!(f, "service error: {}", e),
        }
    }
}

impl<E> std::error::Error for ClassicServiceError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClassicServiceError::ServiceConsumed => None,
            ClassicServiceError::ServiceError(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;
    use tower::{Service, ServiceExt};

    #[derive(Debug)]
    struct AsyncEchoService;

    impl tower_async_service::Service<String> for AsyncEchoService {
        type Response = String;
        type Error = Infallible;

        async fn call(&mut self, req: String) -> Result<Self::Response, Self::Error> {
            Ok(req)
        }
    }

    #[tokio::test]
    async fn test_into_classic() {
        let mut service = AsyncEchoService.into_classic();
        let response = service
            .ready()
            .await
            .unwrap()
            .call("hello".to_string())
            .await
            .unwrap();
        assert_eq!(response, "hello");
    }

    #[tokio::test]
    async fn test_into_classic_twice() {
        let mut service = AsyncEchoService.into_classic();
        let response = service
            .ready()
            .await
            .unwrap()
            .call("hello".to_string())
            .await
            .unwrap();
        assert_eq!(response, "hello");
        let err = service.ready().await.unwrap_err();
        if !matches!(err, ClassicServiceError::ServiceConsumed) {
            panic!(
                "expected ClassicServiceError::ServiceConsumed, got {:?}",
                err
            );
        }
    }

    #[tokio::test]
    async fn test_into_classic_for_builder() {
        let service = AsyncEchoService;
        let mut service = tower::ServiceBuilder::new()
            .rate_limit(1, std::time::Duration::from_secs(1))
            .service(service.into_classic());

        let response = service
            .ready()
            .await
            .unwrap()
            .call("hello".to_string())
            .await
            .unwrap();
        assert_eq!(response, "hello");
    }

    #[tokio::test]
    async fn test_builder_into_classic() {
        let mut service = tower_async::ServiceBuilder::new()
            .timeout(std::time::Duration::from_secs(1))
            .service(AsyncEchoService)
            .into_classic();

        let response = service
            .ready()
            .await
            .unwrap()
            .call("hello".to_string())
            .await
            .unwrap();
        assert_eq!(response, "hello");
    }
}
