//! Middleware for retrying "failed" requests.

pub mod budget;
mod layer;
mod policy;

pub use self::layer::RetryLayer;
pub use self::policy::Policy;

use tower_async_service::Service;

/// Configure retrying requests of "failed" responses.
///
/// A [`Policy`] classifies what is a "failed" response.
#[derive(Clone, Debug)]
pub struct Retry<P, S> {
    policy: P,
    service: S,
}

// ===== impl Retry =====

impl<P, S> Retry<P, S> {
    /// Retry the inner service depending on this [`Policy`].
    pub fn new(policy: P, service: S) -> Self {
        Retry { policy, service }
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &S {
        &self.service
    }

    /// Get a mutable reference to the inner service
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.service
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> S {
        self.service
    }
}

impl<P, S, Request> Service<Request> for Retry<P, S>
where
    P: Policy<Request, S::Response, S::Error>,
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, mut request: Request) -> Result<Self::Response, Self::Error> {
        loop {
            let cloned_request = self.policy.clone_request(&request);
            let mut result = self.service.call(request).await;
            if let Some(mut req) = cloned_request {
                if !self.policy.retry(&mut req, &mut result).await {
                    return result;
                }
                request = req;
            } else {
                return result;
            }
        }
    }
}
