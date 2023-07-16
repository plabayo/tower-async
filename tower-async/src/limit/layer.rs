use super::Limit;
use tower_async_layer::Layer;

/// Limit requests based on a policy
#[derive(Debug, Clone)]
pub struct LimitLayer<P> {
    policy: P,
}

impl<P> LimitLayer<P> {
    /// Creates a new [`LimitLayer`] from a limit policy.
    pub fn new(policy: P) -> Self {
        LimitLayer { policy }
    }
}

impl<P, S> Layer<S> for LimitLayer<P>
where
    P: Clone,
{
    type Service = Limit<P, S>;

    fn layer(&self, service: S) -> Self::Service {
        let policy = self.policy.clone();
        Limit::new(policy, service)
    }
}
