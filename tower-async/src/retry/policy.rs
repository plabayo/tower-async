/// A "retry policy" to classify if a request should be retried.
///
/// # Example
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use tower_async::retry::Policy;
///
/// type Req = String;
/// type Res = String;
///
/// struct Attempts(Arc<Mutex<usize>>);
///
/// impl<E> Policy<Req, Res, E> for Attempts {
///     async fn retry(&self, req: &mut Req, result: &mut Result<Res, E>) -> bool {
///         match result {
///             Ok(_) => {
///                 // Treat all `Response`s as success,
///                 // so don't retry...
///                 false
///             },
///             Err(_) => {
///                 // Treat all errors as failures...
///                 // But we limit the number of attempts...
///                 let mut retries = self.0.lock().unwrap();
///                 if *retries > 0 {
///                     // Try again!
///                     *retries -= 1;
///                     true
///                 } else {
///                     // Used all our attempts, no retry...
///                     false
///                 }
///             }
///         }
///     }
///
///     fn clone_request(&self, req: &Req) -> Option<Req> {
///         Some(req.clone())
///     }
/// }
/// ```
pub trait Policy<Req, Res, E> {
    /// Check the policy if a certain request should be retried.
    ///
    /// This method is passed a reference to the original request, and either
    /// the [`Service::Response`] or [`Service::Error`] from the inner service.
    ///
    /// If the request should **not** be retried, return `None`.
    ///
    /// If the request *should* be retried, return `Some` future that will delay
    /// the next retry of the request. This can be used to sleep for a certain
    /// duration, to wait for some external condition to be met before retrying,
    /// or resolve right away, if the request should be retried immediately.
    ///
    /// ## Mutating Requests
    ///
    /// The policy MAY chose to mutate the `req`: if the request is mutated, the
    /// mutated request will be sent to the inner service in the next retry.
    /// This can be helpful for use cases like tracking the retry count in a
    /// header.
    ///
    /// ## Mutating Results
    ///
    /// The policy MAY chose to mutate the result. This enables the retry
    /// policy to convert a failure into a success and vice versa. For example,
    /// if the policy is used to poll while waiting for a state change, the
    /// policy can switch the result to emit a specific error when retries are
    /// exhausted.
    ///
    /// The policy can also record metadata on the request to include
    /// information about the number of retries required or to record that a
    /// failure failed after exhausting all retries.
    ///
    /// [`Service::Response`]: crate::Service::Response
    /// [`Service::Error`]: crate::Service::Error
    fn retry(
        &self,
        req: &mut Req,
        result: &mut Result<Res, E>,
    ) -> impl std::future::Future<Output = bool>;

    /// Tries to clone a request before being passed to the inner service.
    ///
    /// If the request cannot be cloned, return [`None`]. Moreover, the retry
    /// function will not be called if the [`None`] is returned.
    fn clone_request(&self, req: &Req) -> Option<Req>;
}
