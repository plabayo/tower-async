//! Bridges a `tower-async` `Service` to be used within a `hyper` (1.x) environment.
//!
//! # Example
//!
//! ```text
//! ```

use std::sync::Arc;

use hyper::body::Body as HyperBody;
use hyper::service::service_fn;
use hyper::service::Service as HyperService;
use hyper::Request as HyperRequest;
use hyper::Response as HyperResponse;

use tower_async_service::Service;

pub trait TowerHyperServiceExt<Request> {
    type Response;
    type Error;

    /// Convert this [`tower::Service`] service into a [`hyper::service::Service`].
    ///
    /// [`tower::Service`]: https://docs.rs/tower-async/latest/tower_async/trait.Service.html
    /// [`hyper::service::Service`]: https://docs.rs/hyper/latest/hyper/service/trait.Service.html
    fn into_hyper_service(
        self,
    ) -> impl HyperService<Request, Response = Self::Response, Error = Self::Error>;
}

impl<S, ReqBody, RespBody> TowerHyperServiceExt<HyperRequest<ReqBody>> for S
where
    S: Service<HyperRequest<ReqBody>, Response = HyperResponse<RespBody>>,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    ReqBody: HyperBody,
    RespBody: HyperBody,
{
    type Response = S::Response;
    type Error = S::Error;

    fn into_hyper_service(
        self,
    ) -> impl HyperService<HyperRequest<ReqBody>, Response = Self::Response, Error = Self::Error>
    {
        let service = Arc::new(self);
        service_fn(move |req: HyperRequest<ReqBody>| {
            let service = service.clone();
            async move { service.call(req).await }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_into_hyper_service() {
        let service = tower_async::service_fn(|req: HyperRequest<String>| async move {
            HyperResponse::builder().status(200).body(req.into_body())
        });
        let hyper_service = service.into_hyper_service();
        inner_test_hyper_service(hyper_service).await;
    }

    #[tokio::test]
    async fn test_into_layered_hyper_service() {
        let service = tower_async::ServiceBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .service_fn(|req: HyperRequest<String>| async move {
                HyperResponse::builder().status(200).body(req.into_body())
            });
        let hyper_service = service.into_hyper_service();
        inner_test_hyper_service(hyper_service).await;
    }

    async fn inner_test_hyper_service<E: std::fmt::Debug>(
        hyper_service: impl HyperService<
            HyperRequest<String>,
            Response = HyperResponse<String>,
            Error = E,
        >,
    ) {
        fn require_send<T: Send>(_t: T) {}

        let fut = hyper_service.call(
            HyperRequest::builder()
                .body(String::from("hello"))
                .expect("build http hyper request"),
        );
        require_send(fut);

        let res = fut.await.expect("call hyper service");
        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), "hello");
    }
}
