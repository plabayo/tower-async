//! Bridges a `tower-async` `Service` to be used within a `hyper` (1.x) environment.
//!
//! # Example
//!
//! ```text
//! ```

use std::{future::Future, pin::Pin, sync::Arc};

use hyper::service::Service as HyperService;
use tower_async_service::Service;

pub trait TowerHyperServiceExt<Request> {
    /// Convert this service into a `tower::Service`.
    fn into_hyper(self) -> TowerHyperService<Self>;
}

impl<S, Request> TowerHyperServiceExt<Request> for S
where
    S: Service<Request>,
{
    fn into_hyper(self) -> TowerHyperService<Self> {
        TowerHyperService::new(self)
    }
}

pub struct TowerHyperService<S: ?Sized> {
    inner: Arc<S>,
}

impl<S> TowerHyperService<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl<S, Request> HyperService<Request> for TowerHyperService<S>
where
    Request: Send + 'static,
    S: Service<Request> + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let service = self.inner.clone();

        let future = async move { service.call(req).await };
        Box::pin(future)
    }
}

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

#[cfg(test)]
mod test {
    use super::*;
    // use tower_async::ServiceExt;

    #[tokio::test]
    async fn test_new_hyper_service() {
        let service = tower_async::service_fn(|req: String| async move { Ok::<_, ()>(req) });
        let hyper_service = TowerHyperService::new(service);
        let res = hyper_service.call("foo".to_string()).await.unwrap();
        assert_eq!(res, "foo");
    }

    #[tokio::test]
    async fn test_into_hyper_service() {
        let service = tower_async::service_fn(|req: String| async move { Ok::<_, ()>(req) });
        let hyper_service = service.into_hyper();
        let res = hyper_service.call("foo".to_string()).await.unwrap();
        assert_eq!(res, "foo");
    }

    // #[tokio::test]
    // async fn test_new_layered_hyper_service() {
    //     let service = tower_async::ServiceBuilder::new()
    //         .timeout(std::time::Duration::from_secs(5))
    //         .service_fn(|req: String| async move { Ok::<_, ()>(req) });
    //     let hyper_service = TowerHyperService::new(service);
    //     let res = hyper_service.call("foo".to_string()).await.unwrap();
    //     assert_eq!(res, "foo");
    // }

    // #[tokio::test]
    // async fn test_into_layered_hyper_service() {
    //     let service = tower_async::ServiceBuilder::new()
    //         .timeout(std::time::Duration::from_secs(5))
    //         .service_fn(|req: String| async move { Ok::<_, ()>(req) });

    //     let res = service.oneshot("foo".to_string()).await.unwrap();
    //     assert_eq!(res, "foo");

    //     // let hyper_service = service.into_hyper();
    //     // let res = hyper_service.call("foo".to_string()).await.unwrap();
    //     // assert_eq!(res, "foo");
    // }
}
