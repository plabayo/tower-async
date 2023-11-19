use std::pin::Pin;
use std::sync::Arc;

use hyper::service::Service as HyperService;

use tower_async_service::Service;

pub trait TowerHyperServiceExt<S, Request> {
    fn into_hyper_service(self) -> HyperServiceWrapper<S>;
}

impl<S, Request> TowerHyperServiceExt<S, Request> for S
where
    S: Service<Request>,
{
    fn into_hyper_service(self) -> HyperServiceWrapper<S> {
        HyperServiceWrapper {
            service: Arc::new(self),
        }
    }
}

pub struct HyperServiceWrapper<S> {
    service: Arc<S>,
}

impl<S, Request> HyperService<Request> for HyperServiceWrapper<S>
where
    S: Service<Request, call(): Send> + Send + Sync + 'static,
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let service = self.service.clone();
        let fut = async move { service.call(req).await };
        Box::pin(fut)
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use super::*;

    fn require_send<T: Send>(t: T) -> T {
        t
    }

    fn require_service<Request, S: Service<Request>>(s: S) -> S {
        s
    }

    #[tokio::test]
    async fn test_into_hyper_service() {
        let service =
            tower_async::service_fn(|req: &'static str| async move { Ok::<_, Infallible>(req) });
        let service = require_service(service);
        let hyper_service = service.into_hyper_service();
        inner_test_hyper_service(hyper_service).await;
    }

    #[tokio::test]
    async fn test_into_layered_hyper_service() {
        let service = tower_async::ServiceBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .service_fn(|req: &'static str| async move { Ok::<_, Infallible>(req) });
        let service = require_service(service);
        let hyper_service = service.into_hyper_service();
        inner_test_hyper_service(hyper_service).await;
    }

    async fn inner_test_hyper_service<H>(hyper_service: H)
    where
        H: HyperService<&'static str, Response = &'static str>,
        H::Error: std::fmt::Debug,
        H::Future: Send,
    {
        let fut = hyper_service.call("hello");
        let fut = require_send(fut);

        let res = fut.await.expect("call hyper service");
        assert_eq!(res, "hello");
    }
}
