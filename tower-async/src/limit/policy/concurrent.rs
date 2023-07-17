//! A policy that limits the number of concurrent requests.
//!
//! See [`ConcurrentPolicy`].
//!
//! # Examples
//!
//! ```
//! use tower_async::{
//!     limit::{Limit, policy::ConcurrentPolicy},
//!     Service, ServiceExt, service_fn,
//! };
//! # use std::convert::Infallible;
//!
//! # #[tokio::main]
//! # async fn main() {
//!
//! let service = service_fn(|_| async {
//!     Ok::<_, Infallible>(())
//! });
//! let mut service = Limit::new(service, ConcurrentPolicy::new(2));
//!
//! let response = service.oneshot(()).await;
//! assert!(response.is_ok());
//! # }
//! ```

use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};

use crate::util::backoff::Backoff;

use super::{Policy, PolicyOutput};

/// A policy that limits the number of concurrent requests.
#[derive(Debug)]
pub struct ConcurrentPolicy<B> {
    max: usize,
    current: Arc<Mutex<usize>>,
    backoff: B,
}

impl<B> Clone for ConcurrentPolicy<B>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        ConcurrentPolicy {
            max: self.max,
            current: self.current.clone(),
            backoff: self.backoff.clone(),
        }
    }
}

impl ConcurrentPolicy<()> {
    /// Create a new concurrent policy,
    /// which aborts the request if the limit is reached.
    pub fn new(max: usize) -> Self {
        ConcurrentPolicy {
            max,
            current: Arc::new(Mutex::new(0)),
            backoff: (),
        }
    }
}

impl<B> ConcurrentPolicy<B> {
    /// Create a new concurrent policy,
    /// which backs off if the limit is reached,
    /// using the given backoff policy.
    pub fn with_backoff(max: usize, backoff: B) -> Self {
        ConcurrentPolicy {
            max,
            current: Arc::new(Mutex::new(0)),
            backoff,
        }
    }
}

/// The guard that releases the concurrent request limit.
#[derive(Debug)]
pub struct ConcurrentGuard {
    current: Arc<Mutex<usize>>,
}

impl Drop for ConcurrentGuard {
    fn drop(&mut self) {
        let mut current = self.current.lock().unwrap();
        *current -= 1;
    }
}

impl<B, Request> Policy<Request> for ConcurrentPolicy<B>
where
    B: Backoff,
{
    type Guard = ConcurrentGuard;
    type Error = Infallible;
    type Future = B::Future;

    async fn check(
        &mut self,
        _: &mut Request,
    ) -> PolicyOutput<Self::Guard, Self::Error, Self::Future> {
        let mut current = self.current.lock().unwrap();
        if *current < self.max {
            *current += 1;
            PolicyOutput::Ready(ConcurrentGuard {
                current: self.current.clone(),
            })
        } else {
            PolicyOutput::Retry(self.backoff.next_backoff())
        }
    }
}

/// The error that indicates the request is aborted,
/// because the concurrent request limit is reached.
#[derive(Debug)]
pub struct LimitReached;

impl std::fmt::Display for LimitReached {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LimitReached")
    }
}

impl std::error::Error for LimitReached {}

impl<Request> Policy<Request> for ConcurrentPolicy<()> {
    type Guard = ConcurrentGuard;
    type Error = LimitReached;
    type Future = std::future::Ready<()>;

    async fn check(
        &mut self,
        _: &mut Request,
    ) -> PolicyOutput<Self::Guard, Self::Error, Self::Future> {
        let mut current = self.current.lock().unwrap();
        if *current < self.max {
            *current += 1;
            PolicyOutput::Ready(ConcurrentGuard {
                current: self.current.clone(),
            })
        } else {
            PolicyOutput::Abort(LimitReached)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_ready<G, E, F>(output: PolicyOutput<G, E, F>) -> G {
        match output {
            PolicyOutput::Ready(guard) => guard,
            _ => panic!("unexpected output, expected ready"),
        }
    }

    fn assert_abort<G, E, F>(output: PolicyOutput<G, E, F>) {
        match output {
            PolicyOutput::Abort(_) => (),
            _ => panic!("unexpected output, expected abort"),
        }
    }

    #[tokio::test]
    async fn concurrent_policy() {
        let mut policy = ConcurrentPolicy::new(2);
        let mut request = ();

        let guard_1 = assert_ready(policy.check(&mut request).await);
        let guard_2 = assert_ready(policy.check(&mut request).await);

        assert_abort(policy.check(&mut request).await);

        drop(guard_1);
        let _guard_3 = assert_ready(policy.check(&mut request).await);

        assert_abort(policy.check(&mut request).await);

        drop(guard_2);
        assert_ready(policy.check(&mut request).await);
    }
}
