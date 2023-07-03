#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
// `rustdoc::broken_intra_doc_links` is checked on CI

//! Bridge traits and extensions.
//!
//! A bridge decorates an service and provides additional functionality.
//! It allows a class Tower Service to be used as an Async [`Service`].
//!
//! [`Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html

use std::marker::PhantomData;
use tower::ServiceExt;

/// Extension for a [`tower::Service`] to turn it into an async [`Service`].
///
/// [`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
/// [`Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
pub trait AsyncServiceExt<Request>: tower_service::Service<Request> {
    /// Turn this [`tower::Service`] into an async [`Service`].
    fn into_async(self) -> AsyncService<Self, Request>
    where
        Self: Sized,
    {
        AsyncService(self, PhantomData)
    }
}

impl<S, Request> AsyncServiceExt<Request> for S where S: tower_service::Service<Request> {}

/// Service returned by [`AsyncServiceExt::into_async`].
///
/// [`AsyncServiceExt::into_async`]: https://docs.rs/tower-async-bridge/*/tower_async_bridge/trait.AsyncServiceExt.html#method.into_async
pub struct AsyncService<S, Request>(pub(crate) S, pub(crate) PhantomData<Request>);

impl<S, Request> std::fmt::Debug for AsyncService<S, Request>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncService")
            .field("inner", &self.0)
            .finish()
    }
}

impl<S, Request> tower_async_service::Service<Request> for AsyncService<S, Request>
where
    S: tower_service::Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        self.0.ready().await?.call(request).await
    }
}

// /// Extension for a [`tower::Layer`] to turn it into an async [`Layer`].
// ///
// /// [`tower::Layer`]: https://docs.rs/tower/*/tower/trait.Layer.html
// /// [`Layer`]: https://docs.rs/tower-async/*/tower_async/trait.Layer.html
// pub trait AsyncLayerExt<Service, Request>: tower_layer::Layer<Service> {
//     /// Turn this [`tower::Service`] into an async [`Service`].
//     fn into_async(self) -> AsyncLayer<Self, Service, Request>
//     where
//         Self: Sized,
//     {
//         AsyncLayer(self, PhantomData, PhantomData)
//     }
// }

// impl<L, Service, Request> AsyncLayerExt<Service, Request> for L
// where
//     L: tower_layer::Layer<Service>,
//     L::Service: AsyncServiceExt<Request>,
// {
// }

// /// Layer returned by [`AsyncLayerExt::into_async`].
// ///
// /// [`AsyncServiceExt::into_async`]: https://docs.rs/tower-async-bridge/*/tower_async_bridge/trait.AsyncLayerExt.html#method.into_async
// pub struct AsyncLayer<L, Service, Request>(
//     pub(crate) L,
//     pub(crate) PhantomData<Service>,
//     pub(crate) PhantomData<Request>,
// );

// impl<L, Service, Request> std::fmt::Debug for AsyncLayer<L, Service, Request>
// where
//     L: std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("AsyncLayer")
//             .field("inner", &self.0)
//             .finish()
//     }
// }

// impl<L, Service, Request> tower_async_layer::Layer<Service> for AsyncLayer<L, Service, Request>
// where
//     L: tower_layer::Layer<Service>,
//     L::Service: AsyncServiceExt<Request>,
// {
//     type Service = AsyncService<L::Service, Request>;

//     fn layer(&self, service: Service) -> Self::Service {
//         let service = self.0.layer(service);
//         service.into_async()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    use std::{
        convert::Infallible,
        future::Future,
        pin::Pin,
        task::{Context, Poll},
        time::Duration,
    };

    use tower::Service;
    use tower_async::{ServiceBuilder, ServiceExt};

    struct EchoService;

    impl Service<String> for EchoService {
        type Response = String;
        type Error = Infallible;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: String) -> Self::Future {
            // create a response in a future.
            let fut = async { Ok(req) };

            // Return the response as an immediate future
            Box::pin(fut)
        }
    }

    struct AsyncEchoService;

    impl tower_async::Service<String> for AsyncEchoService {
        type Response = String;
        type Error = Infallible;

        async fn call(&mut self, req: String) -> Result<Self::Response, Self::Error> {
            Ok(req)
        }
    }

    #[tokio::test]
    async fn test_async_service_ext() {
        let service = EchoService;
        let service = ServiceBuilder::new()
            .timeout(Duration::from_secs(1))
            .service(service.into_async()); // use tower service as async service

        let response = service.oneshot("hello".to_string()).await.unwrap();
        assert_eq!(response, "hello");
    }

    // #[tokio::test]
    // async fn test_async_layer_ext() {
    //     let service = AsyncEchoService;
    //     let service = ServiceBuilder::new()
    //         .layer(tower::layer::util::Identity::new().into_async())
    //         .service(service); // use tower service as async service

    //     let response = service.oneshot("hello".to_string()).await.unwrap();
    //     assert_eq!(response, "hello");
    // }
}
