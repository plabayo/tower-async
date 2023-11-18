use super::body::create_error_response;
use super::{RequestBodyLimitLayer, ResponseBody};

use http::{Request, Response};
use http_body::Body;
use http_body_util::Limited;
use tower_async_service::Service;

/// Middleware that intercepts requests with body lengths greater than the
/// configured limit and converts them into `413 Payload Too Large` responses.
///
/// See the [module docs](crate::limit) for an example.
#[derive(Clone, Copy, Debug)]
pub struct RequestBodyLimit<S> {
    pub(crate) inner: S,
    pub(crate) limit: usize,
}

impl<S> RequestBodyLimit<S> {
    /// Create a new `RequestBodyLimit` with the given body length limit.
    pub fn new(inner: S, limit: usize) -> Self {
        Self { inner, limit }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `RequestBodyLimit` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(limit: usize) -> RequestBodyLimitLayer {
        RequestBodyLimitLayer::new(limit)
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for RequestBodyLimit<S>
where
    ResBody: Body,
    S: Service<Request<Limited<ReqBody>>, Response = Response<ResBody>>,
{
    type Response = Response<ResponseBody<ResBody>>;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let content_length = req
            .headers()
            .get(http::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok()?.parse::<usize>().ok());

        let body_limit = match content_length {
            Some(len) if len > self.limit => return Ok(create_error_response()),
            Some(len) => self.limit.min(len),
            None => self.limit,
        };

        let req = req.map(|body| Limited::new(body, body_limit));
        Ok(self.inner.call(req).await?.map(ResponseBody::new))
    }
}
