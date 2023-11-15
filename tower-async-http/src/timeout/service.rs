use http::{Request, Response, StatusCode};
use std::time::Duration;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Layer that applies the [`Timeout`] middleware which apply a timeout to requests.
///
/// See the [module docs](super) for an example.
#[derive(Debug, Clone, Copy)]
pub struct TimeoutLayer {
    timeout: Duration,
}

impl TimeoutLayer {
    /// Creates a new [`TimeoutLayer`].
    pub fn new(timeout: Duration) -> Self {
        TimeoutLayer { timeout }
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = Timeout<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Timeout::new(inner, self.timeout)
    }
}

/// Middleware which apply a timeout to requests.
///
/// If the request does not complete within the specified timeout it will be aborted and a `408
/// Request Timeout` response will be sent.
///
/// See the [module docs](super) for an example.
#[derive(Debug, Clone, Copy)]
pub struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    /// Creates a new [`Timeout`].
    pub fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `Timeout` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(timeout: Duration) -> TimeoutLayer {
        TimeoutLayer::new(timeout)
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for Timeout<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        tokio::select! {
            res = self.inner.call(req) => res,
            _ = tokio::time::sleep(self.timeout) => {
                let mut res = Response::new(ResBody::default());
                *res.status_mut() = StatusCode::REQUEST_TIMEOUT;
                Ok(res)
            }
        }
    }
}
