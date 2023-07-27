use std::convert::Infallible;
use tower_async_service::Service;

/// A [`MakeService`] that produces services by cloning an inner service.
///
/// [`MakeService`]: super::MakeService
#[derive(Debug, Clone, Copy)]
pub struct Shared<S> {
    service: S,
}

impl<S> Shared<S> {
    /// Create a new [`Shared`] from a service.
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S, T> Service<T> for Shared<S>
where
    S: Clone,
{
    type Response = S;
    type Error = Infallible;

    async fn call(&mut self, _target: T) -> Result<Self::Response, Self::Error> {
        Ok(self.service.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make::MakeService;
    use crate::service_fn;

    async fn echo<R>(req: R) -> Result<R, Infallible> {
        Ok(req)
    }

    #[tokio::test]
    async fn as_make_service() {
        let mut shared = Shared::new(service_fn(echo::<&'static str>));

        let mut svc = shared.make_service(()).await.unwrap();

        let res = svc.call("foo").await.unwrap();

        assert_eq!(res, "foo");
    }

    #[tokio::test]
    async fn as_make_service_into_service() {
        let shared = Shared::new(service_fn(echo::<&'static str>));
        let mut shared = MakeService::<(), _>::into_service(shared);

        let mut svc = shared.call(()).await.unwrap();

        let res = svc.call("foo").await.unwrap();

        assert_eq!(res, "foo");
    }

    #[derive(Debug, Clone)]
    struct EchoService;

    impl<R> Service<R> for EchoService {
        type Response = R;
        type Error = Infallible;

        async fn call(&mut self, req: R) -> Result<Self::Response, Self::Error> {
            Ok(req)
        }
    }

    async fn higher_order_async_fn<F, R>(mut factory: F, req: R) -> R
    where
        F: MakeService<(), R, Response = R>,
        F::MakeError: std::fmt::Debug,
        F::Error: std::fmt::Debug,
        F::Response: std::fmt::Debug,
        F::Service: Service<R, call(): Send> + Send + 'static,
        R: Send + 'static,
    {
        let mut svc = factory.make_service(()).await.unwrap();

        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let resp = svc.call(req).await.unwrap();
            tx.send(resp).unwrap();
        })
        .await
        .unwrap();

        rx.await.unwrap()
    }

    #[tokio::test]
    async fn higher_order_async_fn_with_shared_service_struct() {
        let shared = Shared::new(EchoService);

        let res = higher_order_async_fn(shared, "foo").await;

        assert_eq!(res, "foo");
    }

    #[tokio::test]
    async fn higher_order_async_fn_with_shared_service_fn() {
        let shared = Shared::new(service_fn(echo::<&'static str>));

        let res = higher_order_async_fn(shared, "foo").await;

        assert_eq!(res, "foo");
    }
}
