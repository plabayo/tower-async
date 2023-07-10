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

//! Mock `Service` that can be used in tests
//! to assert that the `Service` receives the expected requests
//! and to send back responses.
//!
//! # Examples
//!
//! ```rust
//! use tower_async_test::runner;
//! use tower_async_layer::Identity;
//!
//! #[tokio::main]
//! async fn main() {
//!     // simple test showing how to use the runner,
//!     // to test a `tower_async::Layer` implementation,
//!     // in this case `tower_async_layer::Identity`.
//!     let mut runner = runner(&mut Identity::new());
//!
//!     runner
//!         .test_ok("ping", "pong")
//!         .expect_request("ping")
//!         .expect_response("pong")
//!         .run().await;
//! }
//! ```

use std::{convert::Infallible, sync::Arc};

use tokio::sync::Mutex;
use tower_async_layer::Layer;
use tower_async_service::Service;

pub(crate) mod mock;

/// Runtime to allow you to run tests against a `Layer`.
#[derive(Debug)]
pub struct TestRunner<L, S, Request, CoreResponse = (), CoreError = Infallible> {
    handle: mock::SyncHandle<Request, CoreResponse, CoreError>,
    service: Arc<Mutex<S>>,
    _phantom: std::marker::PhantomData<(L, Request, CoreResponse, CoreError)>,
}

impl<L, Request, CoreResponse, CoreError>
    TestRunner<L, L::Service, Request, CoreResponse, CoreError>
where
    L: Layer<mock::Mock<Request, CoreResponse, CoreError>>,
{
    /// Construct a new `TestRunner` that will run tests against the given layer.
    pub fn new(layer: &mut L) -> Self {
        let (service, handle) = mock::spawn();
        let service = layer.layer(service);
        TestRunner {
            handle,
            service: Arc::new(Mutex::new(service)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<L, S, Request, CoreResponse> TestRunner<L, S, Request, CoreResponse, Infallible>
where
    L: Layer<mock::Mock<Request, CoreResponse, Infallible>, Service = S>,
    L::Service: Service<Request>,
    S: Service<Request>,
    Request: std::fmt::Debug + PartialEq,
    <<L as Layer<mock::Mock<Request, CoreResponse, Infallible>>>::Service as Service<Request>>::Response: std::fmt::Debug + PartialEq,
    <<L as Layer<mock::Mock<Request, CoreResponse, Infallible>>>::Service as Service<Request>>::Error: std::fmt::Debug + PartialEq,
{
    /// Construct a `TestBuilder` that will send the given request to the service,
    /// and where the innter (mocked) service will return the given success response.
    pub fn test_ok(
        &mut self,
        request: Request,
        response: CoreResponse,
    ) -> TestBuilder<
        S,
        Request,
        CoreResponse,
        <<L as Layer<mock::Mock<Request, CoreResponse, Infallible>>>::Service as Service<Request>>::Response,
        Infallible,
        <<L as Layer<mock::Mock<Request, CoreResponse, Infallible>>>::Service as Service<Request>>::Error,
    > {
        TestBuilder::new(self.handle.clone(), self.service.clone(), request, Ok(response))
    }
}

impl<L, S, Request, CoreError> TestRunner<L, S, Request, (), CoreError>
where
    L: Layer<mock::Mock<Request, (), CoreError>, Service = S>,
    L::Service: Service<Request>,
    S: Service<Request>,
    Request: std::fmt::Debug + PartialEq,
    <<L as Layer<mock::Mock<Request, (), CoreError>>>::Service as Service<Request>>::Response:
        std::fmt::Debug + PartialEq,
    <<L as Layer<mock::Mock<Request, (), CoreError>>>::Service as Service<Request>>::Error:
        std::fmt::Debug + PartialEq,
{
    /// Construct a `TestBuilder` that will send the given request to the service,
    /// and where the innter (mocked) service will return the given error.
    pub fn test_err(
        &mut self,
        request: Request,
        error: CoreError,
    ) -> TestBuilder<
        S,
        Request,
        (),
        <<L as Layer<mock::Mock<Request, (), CoreError>>>::Service as Service<Request>>::Response,
        CoreError,
        <<L as Layer<mock::Mock<Request, (), CoreError>>>::Service as Service<Request>>::Error,
    > {
        TestBuilder::new(
            self.handle.clone(),
            self.service.clone(),
            request,
            Err(error),
        )
    }
}

/// Builder for a single test.
#[derive(Debug)]
pub struct TestBuilder<S, Request, CoreResponse, ServiceResponse, CoreError, ServiceError> {
    handle: mock::SyncHandle<Request, CoreResponse, CoreError>,
    service: Arc<Mutex<S>>,
    request: Request,
    result: Result<CoreResponse, CoreError>,
    expected_request: Option<Request>,
    expected_result: Option<Result<ServiceResponse, ServiceError>>,
}

impl<S, Request, CoreResponse, ServiceResponse, CoreError, ServiceError>
    TestBuilder<S, Request, CoreResponse, ServiceResponse, CoreError, ServiceError>
where
    S: Service<Request, Response = ServiceResponse, Error = ServiceError>,
    Request: std::fmt::Debug + PartialEq,
    ServiceResponse: std::fmt::Debug + PartialEq,
    ServiceError: std::fmt::Debug + PartialEq,
{
    /// Construct a new `TestBuilder` that will send the given request to the service,
    /// and which will return the given result from within the core (Mocked) service.
    fn new(
        handle: mock::SyncHandle<Request, CoreResponse, CoreError>,
        service: Arc<Mutex<S>>,
        request: Request,
        result: Result<CoreResponse, CoreError>,
    ) -> Self {
        Self {
            handle,
            service,
            request,
            result,
            expected_request: None,
            expected_result: None,
        }
    }

    /// Expect the given request to be received by the (inner) mocked service.
    pub fn expect_request(mut self, request: Request) -> Self {
        self.expected_request = Some(request);
        self
    }

    /// Expect the given Response to be returned by the outer service.
    pub fn expect_response(mut self, response: ServiceResponse) -> Self {
        self.expected_result = Some(Ok(response));
        self
    }

    /// Expect the given Error to be returned by the outer service.
    pub fn expect_error(mut self, err: ServiceError) -> Self {
        self.expected_result = Some(Err(err));
        self
    }

    /// Run the test.
    pub async fn run(self) {
        let TestBuilder {
            handle,
            service,
            request,
            result,
            expected_request,
            expected_result,
        } = self;

        {
            handle.lock().await.push_result(result);
        }

        let result = { service.lock().await.call(request).await };

        let received_request = { handle.lock().await.pop_request() };

        if let Some(expected_request) = expected_request {
            assert_eq!(received_request, expected_request);
        }

        if let Some(expected_result) = expected_result {
            assert_eq!(result, expected_result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_async_layer::Identity;

    #[tokio::test]
    async fn test_runner() {
        // simple test showing how to use the runner,
        // to test a `tower_async::Layer` implementation,
        // in this case `tower_async_layer::Identity`.
        let mut runner = TestRunner::new(&mut Identity::new());

        runner
            .test_ok("ping", "pong")
            .expect_request("ping")
            .expect_response("pong")
            .run()
            .await;
    }
}
