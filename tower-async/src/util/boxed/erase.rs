use std::future::Future;
use std::pin::Pin;

use tower_async_service::Service;

pub trait ServiceDyn<Request> {
    type Response;
    type Error;

    fn call(
        &self,
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + '_>>;
}

impl<T, Request> ServiceDyn<Request> for T
where
    T: Service<Request, call(): Send + Sync> + Send + Sync + 'static,
    T::Response: Send + Sync + 'static,
    T::Error: Send + Sync + 'static,
    Request: Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;

    fn call(
        &self,
        req: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + '_>> {
        Box::pin(<Self as Service<Request>>::call(self, req))
    }
}
