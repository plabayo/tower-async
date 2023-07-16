//! A middleware that limits the number of in-flight requests.
//!
//! See [`Limit`].

use tower_async_service::Service;

use crate::BoxError;

pub mod policy;

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

impl<T, P, Request> Service<Request> for Limit<T, P>
where
    T: Service<Request>,
    P: policy::Policy<Request>,
    T::Error: Into<BoxError>,
    P::Error: Into<BoxError>,
    P::Future: std::future::Future,
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
                policy::PolicyOutput::Retry(future) => {
                    future.await;
                }
            }
        }
    }
}
