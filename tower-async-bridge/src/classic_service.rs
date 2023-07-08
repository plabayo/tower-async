use crate::ClassicServiceWrapper;

/// Extension trait for [tower::Service] that provides the [ClassicServiceExt::into_classic] method.
pub trait ClassicServiceExt<Request>: tower_async_service::Service<Request> {
    /// Turn this [tower::Service] into an async [tower_async_service::Service].
    fn into_classic(self) -> ClassicServiceWrapper<Self>
    where
        Self: Sized,
    {
        ClassicServiceWrapper::new(self)
    }
}

impl<S, Request> ClassicServiceExt<Request> for S where S: tower_async_service::Service<Request> {}

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
