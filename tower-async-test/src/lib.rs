#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]
#![allow(elided_lifetimes_in_paths)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
// `rustdoc::broken_intra_doc_links` is checked on CI

//! This crate is to be used to test [`tower_async_layer::Layer`]s,
//! by helping you write tests to gurantee this.
//!
//! The guarantees that it test are:
//!
//! - the [`tower_async_service::Service`] wrapped by the [`tower_async_layer::Layer`]
//!   receives the expected requests, and that your layer react as expected on the sent responses or errors.
//! - the [`tower_async_layer::Layer`] sends back the expected response or error.
//!
//! It does so by providing a [`crate::Builder`] that you can use to define the
//! test flow and expectations. It does this by a generated [`crate::mock::Mock`] [`tower_async_service::Service`]
//! that is used as the core [`tower_async_service::Service`] to help you
//! test your own [`tower_async_layer::Layer`]s with the [`crate::mock::Mock`] [`tower_async_service::Service`].
//!
//! The [`crate::mock::Mock`] service cannot be used directly, but is instead use
//! automatically for any _test_ spawned using the [`crate::Builder`] and specifically
//! its [`crate::Builder::test`] method.
//!
//! # Examples
//!
//! ```
//! use tower_async_test::Builder;
//! use tower_async_layer::Identity;
//!
//! #[tokio::main]
//! async fn main() {
//!     Builder::new("ping")
//!         .send_response("pong")
//!         .expect_request("ping")
//!         .test(Identity::new())
//!         .await
//!         .expect_response("pong");
//! }
//! ```

pub mod builder;
pub mod mock;

pub use builder::Builder;

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;
    use tower_async_layer::{Identity, Layer};
    use tower_async_service::Service;

    #[tokio::test]
    async fn test_runner_ok_with_success() {
        Builder::new("ping")
            .send_response("pong")
            .expect_request("ping")
            .test(Identity::new())
            .await
            .expect_response("pong");
    }

    #[tokio::test]
    #[should_panic]
    async fn test_runner_ok_with_success_panics() {
        Builder::new("ping")
            .send_response("pong")
            .expect_request("pong")
            .test(Identity::new())
            .await
            .expect_response("pong");
    }

    #[derive(Debug)]
    struct ApologeticService<S> {
        inner: S,
    }

    impl<S, Request> Service<Request> for ApologeticService<S>
    where
        S: Service<Request>,
    {
        type Response = ();
        type Error = &'static str;

        async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
            let _ = self.inner.call(request).await;
            Err("Sorry!")
        }
    }

    struct ApolgeticLayer;

    impl<S> Layer<S> for ApolgeticLayer {
        type Service = ApologeticService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            ApologeticService { inner }
        }
    }

    #[tokio::test]
    async fn test_runner_ok_with_failure() {
        Builder::new("ping")
            .send_response("pong")
            .expect_request("ping")
            .test(ApolgeticLayer)
            .await
            .expect_error("Sorry!");
    }

    #[tokio::test]
    #[should_panic]
    async fn test_runner_ok_with_failure_panics() {
        Builder::new("ping")
            .send_response("pong")
            .expect_request("ping")
            .test(ApolgeticLayer)
            .await
            .expect_response(());
    }

    #[tokio::test]
    async fn test_runner_err_with_error() {
        Builder::new("ping")
            .send_error("oops")
            .expect_request("ping")
            .test(Identity::new())
            .await
            .expect_error("oops");
    }

    #[tokio::test]
    #[should_panic]
    async fn test_runner_err_with_error_panics() {
        Builder::new("ping")
            .send_error("oops")
            .expect_request("ping")
            .test(Identity::new())
            .await
            .expect_response(());
    }

    #[derive(Debug)]
    struct DebugFmtService<S> {
        inner: S,
    }

    impl<S, Request> Service<Request> for DebugFmtService<S>
    where
        S: Service<Request>,
        S::Response: std::fmt::Debug,
        S::Error: std::fmt::Debug,
    {
        type Response = String;
        type Error = Infallible;

        async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
            Ok(format!(
                "DebugFmtService: {:?}",
                self.inner.call(request).await
            ))
        }
    }

    struct DebugFmtLayer;

    impl<S> Layer<S> for DebugFmtLayer {
        type Service = DebugFmtService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            DebugFmtService { inner }
        }
    }

    #[tokio::test]
    async fn test_runner_err_with_response() {
        Builder::new("ping")
            .send_error("Sorry!")
            .expect_request("ping")
            .test(DebugFmtLayer)
            .await
            .expect_response("DebugFmtService: Err(\"Sorry!\")".to_string());
    }

    #[tokio::test]
    #[should_panic]
    async fn test_runner_err_with_response_panics() {
        Builder::new("ping")
            .send_error("Sorry!")
            .expect_request("ping")
            .test(DebugFmtLayer)
            .await
            .expect_response("Sorry!".to_string());
    }
}
