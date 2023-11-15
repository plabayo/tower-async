use std::{future::Future, pin::Pin, sync::Arc};

use hyper::service::Service as HyperService;
use tower_async_service::Service;

pub trait TowerHyperServiceExt<Request>: Service<Request> {
    /// Convert this service into a `tower::Service`.
    fn into_hyper(self) -> TowerHyperService<Self>
    where
        Self: Sized,
    {
        TowerHyperService {
            inner: Arc::new(self),
        }
    }
}

pub struct TowerHyperService<S> {
    inner: Arc<S>,
}

impl<S, Request> HyperService<Request> for TowerHyperService<S>
where
    Request: Send + 'static,
    S: Service<Request> + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request) -> Self::Future {
        let service = self.inner.clone();
        let future = async move { service.call(req).await };
        Box::pin(future)
    }
}
