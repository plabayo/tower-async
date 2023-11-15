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

    async fn call(&self, _target: T) -> Result<Self::Response, Self::Error> {
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
        let shared = Shared::new(service_fn(echo::<&'static str>));

        let svc = shared.make_service(()).await.unwrap();

        let res = svc.call("foo").await.unwrap();

        assert_eq!(res, "foo");
    }

    #[tokio::test]
    async fn as_make_service_into_service() {
        let shared = Shared::new(service_fn(echo::<&'static str>));
        let shared = MakeService::<(), _>::into_service(shared);

        let svc = shared.call(()).await.unwrap();

        let res = svc.call("foo").await.unwrap();

        assert_eq!(res, "foo");
    }
}
