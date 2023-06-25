//! Middleware for retrying "failed" requests.

pub mod backoff;
pub mod budget;
mod layer;
mod policy;

pub use self::layer::RetryLayer;
pub use self::policy::Policy;

use pin_project_lite::pin_project;
use tower_async_service::Service;

pin_project! {
    /// Configure retrying requests of "failed" responses.
    ///
    /// A [`Policy`] classifies what is a "failed" response.
    ///
    /// # Clone
    ///
    /// This middleware requires that the inner `Service` implements [`Clone`],
    /// because the `Service` must be stored in each [`ResponseFuture`] in
    /// order to retry the request in the event of a failure. If the inner
    /// `Service` type does not implement `Clone`, the [`Buffer`] middleware
    /// can be added to make any `Service` cloneable.
    ///
    /// [`Buffer`]: crate::buffer::Buffer
    ///
    /// The `Policy` must also implement `Clone`. This middleware will
    /// clone the policy for each _request session_. This means a new clone
    /// of the policy will be created for each initial request and any subsequent
    /// retries of that request. Therefore, any state stored in the `Policy` instance
    /// is for that request session only. In order to share data across request
    /// sessions, that shared state may be stored in an [`Arc`], so that all clones
    /// of the `Policy` type reference the same instance of the shared state.
    ///
    /// [`Arc`]: std::sync::Arc
    #[derive(Clone, Debug)]
    pub struct Retry<P, S> {
        policy: P,
        service: S,
    }
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
    P: Policy<Request, S::Response, S::Error> + Clone,
    S: Service<Request> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        let cloned = self.policy.clone_request(&request);
        loop {
            let result = self.service.call(request).await;
            match self.policy.retry(&cloned, &mut result).await {
                Some(_) => continue,
                None => return result,
            }
        }
    }
}
