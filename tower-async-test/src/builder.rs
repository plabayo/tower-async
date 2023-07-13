//! Builder for creating mock services and testing them with a layer.

use std::convert::Infallible;

use tower_async_layer::Layer;
use tower_async_service::Service;

pub mod marker {
    //! Marker types for builder state,
    //! used to prevent invalid state transitions.

    /// Marker type to indicates an unspecified Type.
    #[derive(Debug, Default)]
    pub struct None;

    /// Marker type to indicates a defined Type.
    #[derive(Debug, Default)]
    pub struct Defined;

    /// Marker type to indicates a successful response (used for the test type).
    #[derive(Debug, Default)]
    pub struct Ok<T>(pub T);

    /// Marker type to indicates an error (used for the test type).
    #[derive(Debug, Default)]
    pub struct Err<T>(pub T);
}

/// Defines the test data structure used by the builder,
/// to store internally the registeresd tests.
#[derive(Debug)]
pub struct Test<In, Out> {
    output: Out,
    expected_input: Option<In>,
}

/// Builder for creating mock services and testing them with a layer.
#[derive(Debug)]
pub struct Builder<R, T, RequestState> {
    request: R,
    tests: T,
    _request_state: RequestState,
}

//////////////////////////
/// Virgin Builder
//////////////////////////

impl<R> Builder<R, marker::None, marker::None> {
    /// Creates a new builder with the given request.
    pub fn new(request: R) -> Self {
        Self {
            request,
            tests: marker::None,
            _request_state: marker::None,
        }
    }

    /// Register the sending of a (successful) response.
    pub fn send_response<Response>(
        self,
        response: Response,
    ) -> Builder<R, Vec<Test<R, marker::Ok<Response>>>, marker::None> {
        Builder {
            request: self.request,
            tests: vec![Test {
                output: marker::Ok(response),
                expected_input: None,
            }],
            _request_state: marker::None,
        }
    }

    /// Register the sending of an error.
    pub fn send_error<Error>(
        self,
        error: Error,
    ) -> Builder<R, Vec<Test<R, marker::Err<Error>>>, marker::None> {
        Builder {
            request: self.request,
            tests: vec![Test {
                output: marker::Err(error),
                expected_input: None,
            }],
            _request_state: marker::None,
        }
    }
}

//////////////////////////
/// Ok-only test builder
//////////////////////////

impl<R, Response, RequestState> Builder<R, Vec<Test<R, marker::Ok<Response>>>, RequestState> {
    /// Register the sending of an additional (successful) response.
    pub fn send_response(
        mut self,
        response: Response,
    ) -> Builder<R, Vec<Test<R, marker::Ok<Response>>>, marker::None> {
        self.tests.push(Test {
            output: marker::Ok(response),
            expected_input: None,
        });
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::None,
        }
    }

    /// Register the sending of an additional error.
    #[allow(clippy::type_complexity)]
    pub fn send_error<Error>(
        self,
        error: Error,
    ) -> Builder<R, Vec<Test<R, Result<Response, Error>>>, marker::None> {
        let mut tests: Vec<_> = self
            .tests
            .into_iter()
            .map(|test| Test {
                output: Ok(test.output.0),
                expected_input: test.expected_input,
            })
            .collect();
        tests.push(Test {
            output: Err(error),
            expected_input: None,
        });
        Builder {
            request: self.request,
            tests,
            _request_state: marker::None,
        }
    }
}

impl<R, Response, RequestState> Builder<R, Vec<Test<R, marker::Ok<Response>>>, RequestState>
where
    R: Send + Sync + std::fmt::Debug + PartialEq,
    Response: Send + Sync,
{
    /// Test the given layer with the previously registered tests.
    /// 
    /// # Panics
    /// 
    /// Panics if there are less requests returned then there
    /// are responses+errors registered.
    pub async fn test<L>(
        self,
        layer: L,
    ) -> ResponseTester<
        <<L as Layer<crate::mock::Mock<R, Response, Infallible>>>::Service as Service<R>>::Response,
        <<L as Layer<crate::mock::Mock<R, Response, Infallible>>>::Service as Service<R>>::Error,
    >
    where
        L: Layer<crate::mock::Mock<R, Response, Infallible>>,
        L::Service: Service<R>,
    {
        let tests = self
            .tests
            .into_iter()
            .map(|test| Test {
                output: Ok(test.output.0),
                expected_input: test.expected_input,
            })
            .collect();
        test_layer(layer, self.request, tests).await
    }
}

impl<R, Response> Builder<R, Vec<Test<R, marker::Ok<Response>>>, marker::None> {
    /// Register the expectation of a request,
    /// for the same cycle as the previously added successful response.
    pub fn expect_request(
        mut self,
        request: R,
    ) -> Builder<R, Vec<Test<R, marker::Ok<Response>>>, marker::Defined> {
        self.tests.last_mut().unwrap().expected_input = Some(request);
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::Defined,
        }
    }
}

//////////////////////////
/// Error-only test builder
//////////////////////////

impl<R, Error, RequestState> Builder<R, Vec<Test<R, marker::Err<Error>>>, RequestState> {
    /// Register the sending of an additional (successful) response.
    ///
    #[allow(clippy::type_complexity)]
    pub fn send_response<Response>(
        self,
        response: Response,
    ) -> Builder<R, Vec<Test<R, Result<Response, Error>>>, marker::None> {
        let mut tests: Vec<_> = self
            .tests
            .into_iter()
            .map(|test| Test {
                output: Err(test.output.0),
                expected_input: test.expected_input,
            })
            .collect();
        tests.push(Test {
            output: Ok(response),
            expected_input: None,
        });
        Builder {
            request: self.request,
            tests,
            _request_state: marker::None,
        }
    }

    /// Register the sending of an additional error.
    pub fn send_error(
        mut self,
        error: Error,
    ) -> Builder<R, Vec<Test<R, marker::Err<Error>>>, marker::None> {
        self.tests.push(Test {
            output: marker::Err(error),
            expected_input: None,
        });
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::None,
        }
    }
}

impl<R, Error, RequestState> Builder<R, Vec<Test<R, marker::Err<Error>>>, RequestState>
where
    R: Send + Sync + std::fmt::Debug + PartialEq,
    Error: Send + Sync,
{
    /// Test the given layer with the previously registered tests.
    /// 
    /// # Panics
    /// 
    /// Panics if there are less requests returned then there
    /// are responses+errors registered.
    pub async fn test<L>(
        self,
        layer: L,
    ) -> ResponseTester<
        <<L as Layer<crate::mock::Mock<R, (), Error>>>::Service as Service<R>>::Response,
        <<L as Layer<crate::mock::Mock<R, (), Error>>>::Service as Service<R>>::Error,
    >
    where
        L: Layer<crate::mock::Mock<R, (), Error>>,
        L::Service: Service<R>,
    {
        let tests = self
            .tests
            .into_iter()
            .map(|test| Test {
                output: Err(test.output.0),
                expected_input: test.expected_input,
            })
            .collect();
        test_layer(layer, self.request, tests).await
    }
}

impl<R, Error> Builder<R, Vec<Test<R, marker::Err<Error>>>, marker::None> {
    /// Register the expectation of a request,
    /// for the same cycle as the previously added error.
    pub fn expect_request(
        mut self,
        request: R,
    ) -> Builder<R, Vec<Test<R, marker::Err<Error>>>, marker::Defined> {
        self.tests.last_mut().unwrap().expected_input = Some(request);
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::Defined,
        }
    }
}

//////////////////////////
/// Full Result (Ok+Err mix) test builder
//////////////////////////

impl<R, Response, Error, RequestState>
    Builder<R, Vec<Test<R, Result<Response, Error>>>, RequestState>
{
    /// Register the sending of an additional (successful) response.
    #[allow(clippy::type_complexity)]
    pub fn send_response(
        mut self,
        response: Response,
    ) -> Builder<R, Vec<Test<R, Result<Response, Error>>>, marker::None> {
        self.tests.push(Test {
            output: Ok(response),
            expected_input: None,
        });
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::None,
        }
    }

    /// Register the sending of an additional error.
    #[allow(clippy::type_complexity)]
    pub fn send_error(
        mut self,
        error: Error,
    ) -> Builder<R, Vec<Test<R, Result<Response, Error>>>, marker::None> {
        self.tests.push(Test {
            output: Err(error),
            expected_input: None,
        });
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::None,
        }
    }
}

impl<R, Response, Error, RequestState>
    Builder<R, Vec<Test<R, Result<Response, Error>>>, RequestState>
where
    R: Send + Sync + std::fmt::Debug + PartialEq,
    Response: Send + Sync,
    Error: Send + Sync,
{
    /// Test the given layer with the previously registered tests.
    /// 
    /// # Panics
    /// 
    /// Panics if there are less requests returned then there
    /// are responses+errors registered.
    pub async fn test<L>(
        self,
        layer: L,
    ) -> ResponseTester<
        <<L as Layer<crate::mock::Mock<R, Response, Error>>>::Service as Service<R>>::Response,
        <<L as Layer<crate::mock::Mock<R, Response, Error>>>::Service as Service<R>>::Error,
    >
    where
        L: Layer<crate::mock::Mock<R, Response, Error>>,
        L::Service: Service<R>,
    {
        test_layer(layer, self.request, self.tests).await
    }
}

#[allow(clippy::type_complexity)]
impl<R, Response, Error> Builder<R, Vec<Test<R, Result<Response, Error>>>, marker::None> {
    /// Register the expectation of a request,
    /// for the same cycle as the previously added result.
    pub fn expect_request(
        mut self,
        request: R,
    ) -> Builder<R, Vec<Test<R, Result<Response, Error>>>, marker::Defined> {
        self.tests.last_mut().unwrap().expected_input = Some(request);
        Builder {
            request: self.request,
            tests: self.tests,
            _request_state: marker::Defined,
        }
    }
}

//////////////////////////
/// Shared Inner Functions
//////////////////////////

async fn test_layer<L, Request, Response, Error>(
    layer: L,
    request: Request,
    tests: Vec<Test<Request, Result<Response, Error>>>,
) -> ResponseTester<<<L as Layer<crate::mock::Mock<Request, Response, Error>>>::Service as Service<Request>>::Response, <<L as Layer<crate::mock::Mock<Request, Response, Error>>>::Service as Service<Request>>::Error>
where
    L: Layer<crate::mock::Mock<Request, Response, Error>>,
    L::Service: Service<Request>,
    Request: Send + Sync + std::fmt::Debug + PartialEq,
    Response: Send + Sync,
    Error: Send + Sync,
{
    let (service, handle) = crate::mock::spawn();

    let layer = layer;
    let mut service = layer.layer(service);

    let (input_results, expected_inputs): (Vec<_>, Vec<_>) = tests
        .into_iter()
        .map(|test| (test.output, test.expected_input))
        .unzip();

    {
        let mut handle = handle.lock().await;
        for result in input_results {
            handle.push_result(result);
        }
    }

    let response = service.call(request).await;

    {
        let mut handle = handle.lock().await;
        for expected_input in expected_inputs {
            let request = handle.pop_request();
            if let Some(expected_request) = expected_input {
                assert_eq!(request, expected_request);
            }
        }
    }

    ResponseTester::new(response)
}

//////////////////////////
/// ResponseTester
//////////////////////////

/// Helper type for testing the response of a layer's service.
#[derive(Debug)]
pub struct ResponseTester<Response, Error> {
    result: Result<Response, Error>,
}

/// Helper type for testing the response of a layer's service.
impl<Response, Error> ResponseTester<Response, Error> {
    /// Creates a new `ResponseTester` with the given result.
    pub(crate) fn new(result: Result<Response, Error>) -> Self {
        Self { result }
    }
}

impl<Response, Error> ResponseTester<Response, Error>
where
    Response: PartialEq + std::fmt::Debug,
    Error: std::fmt::Debug,
{
    /// Asserts that the response is equal to the given expected response.
    /// 
    /// # Panics
    /// 
    /// Panics if the response is an error or if the response is not equal to the given expected
    /// response.
    pub fn expect_response(self, expected: Response) {
        match self.result {
            Ok(response) => assert_eq!(response, expected),
            Err(err) => panic!("expected response, got error: {:?}", err),
        }
    }
}

impl<Response, Error> ResponseTester<Response, Error>
where
    Response: std::fmt::Debug,
    Error: PartialEq + std::fmt::Debug,
{
    /// Asserts that the response is equal to the given expected error.
    /// 
    /// # Panics
    /// 
    /// Panics if the response is not an error or if the error is not equal to the given expected
    /// error.
    pub fn expect_error(self, expected: Error) {
        match self.result {
            Ok(response) => panic!("expected error, got response: {:?}", response),
            Err(err) => assert_eq!(err, expected),
        }
    }
}
