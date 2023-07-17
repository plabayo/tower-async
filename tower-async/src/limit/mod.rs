//! A middleware that limits the number of in-flight requests.
//!
//! See [`Limit`].

use tower_async_service::Service;

use crate::BoxError;

pub mod policy;
pub use policy::{Policy, PolicyOutput};

mod layer;
pub use layer::LimitLayer;

/// Limit requests based on a policy
#[derive(Debug)]
pub struct Limit<T, P> {
    inner: T,
    policy: P,
}

impl<T, P> Limit<T, P> {
    /// Creates a new [`Limit`] from a limit policy,
    /// wrapping the given service.
    pub fn new(inner: T, policy: P) -> Self {
        Limit { inner, policy }
    }
}

impl<T, P> Clone for Limit<T, P>
where
    T: Clone,
    P: Clone,
{
    fn clone(&self) -> Self {
        Limit {
            inner: self.inner.clone(),
            policy: self.policy.clone(),
        }
    }
}

impl<T, P, Request> Service<Request> for Limit<T, P>
where
    T: Service<Request>,
    T::Error: Into<BoxError>,
    P: policy::Policy<Request>,
    P::Error: Into<BoxError>,
{
    type Response = T::Response;
    type Error = BoxError;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        let mut request = request;
        loop {
            match self.policy.check(&mut request).await {
                policy::PolicyOutput::Ready(guard) => {
                    let _ = guard;
                    return self.inner.call(request).await.map_err(Into::into);
                }
                policy::PolicyOutput::Abort(err) => return Err(err.into()),
                policy::PolicyOutput::Retry => (),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use crate::limit::policy::ConcurrentPolicy;
    use crate::service_fn;

    use super::*;

    use futures_util::future::join_all;
    use tower_async_layer::Layer;
    use tower_async_service::Service;

    #[tokio::test]
    async fn test_limit() {
        async fn handle_request<Request>(req: Request) -> Result<Request, Infallible> {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok(req)
        }

        let layer: LimitLayer<ConcurrentPolicy<()>> = LimitLayer::new(ConcurrentPolicy::new(1));

        let mut service_1 = layer.layer(service_fn(handle_request));
        let mut service_2 = layer.layer(service_fn(handle_request));

        let future_1 = service_1.call("Hello");
        let future_2 = service_2.call("Hello");

        let mut results = join_all(vec![future_1, future_2]).await;
        let result_1 = results.pop().unwrap();
        let result_2 = results.pop().unwrap();

        // check that one request succeeded and the other failed
        if result_1.is_err() {
            assert_eq!(result_2.unwrap(), "Hello");
        } else {
            assert_eq!(result_1.unwrap(), "Hello");
            assert!(result_2.is_err());
        }
    }
}
