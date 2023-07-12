//! Mock `Service` that can be used in tests
//! and other mock related utilities.

use std::{collections::VecDeque, sync::Arc};

use tokio::sync::Mutex;
use tower_async_service::Service;

/// Mock `Service` that can be used in tests.
///
/// # Examples
///
/// ```rust
/// use tower_async_service::Service;
/// use tower_async_test::mock;
///
/// # async fn test() {
/// let (mut service, mut handle) = mock::spawn();
///
/// let response = service.call("hello");
///
/// assert_request_eq!(handle, "hello").send_response("world");
///
/// assert_eq!(response.await.unwrap(), "world");
/// # }
/// ```
#[derive(Debug)]
pub struct Mock<Request, Response, Error> {
    handle: SyncHandle<Request, Response, Error>,
}

/// Creates a new mock `Service` and with the default driver implementation,
/// which can be used to assert that the `Service` receives the expected requests,
/// and to send back responses.
pub fn spawn<Request, Response, Error>() -> (
    Mock<Request, Response, Error>,
    SyncHandle<Request, Response, Error>,
)
where
    Request: Send + Sync,
    Response: Send + Sync,
    Error: Send + Sync,
{
    let handle = Arc::new(Mutex::new(Handle::new()));
    let mock = Mock {
        handle: handle.clone(),
    };
    (mock, handle)
}

impl<Request, Response, Error> Service<Request> for Mock<Request, Response, Error> {
    type Response = Response;
    type Error = Error;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        let mut handle = self.handle.lock().await;
        handle.push_request(request);
        handle.pop_result()
    }
}

/// A Sync `Handle` to a mock `Service`.
pub type SyncHandle<Request, Response, Error> = Arc<Mutex<Handle<Request, Response, Error>>>;

/// The default `Handle` implementation.
#[derive(Debug)]
pub struct Handle<Request, Response, Error> {
    requests: VecDeque<Request>,
    results: VecDeque<Result<Response, Error>>,
}

impl<Request, Response, Error> Handle<Request, Response, Error> {
    /// Returns a new `Handle`, only usable once you inserted some results.
    pub fn new() -> Self {
        Self {
            requests: VecDeque::new(),
            results: VecDeque::new(),
        }
    }

    /// Inserts a new request that was received by the mock `Service`.
    pub fn push_request(&mut self, request: Request) {
        self.requests.push_back(request);
    }

    /// Inserts a new result to be returned by the mock `Service`.
    pub fn push_result(&mut self, result: Result<Response, Error>) {
        self.results.push_back(result);
    }

    /// Returns the oldest request received by the mock `Service`.
    ///
    /// # Panics
    ///
    /// Panics if no request has been received.
    pub fn pop_request(&mut self) -> Request {
        self.requests.pop_front().unwrap()
    }

    /// Returns the oldest result to be returned by the mock `Service`.
    ///
    /// # Panics
    ///
    /// Panics if no result has been inserted.
    pub fn pop_result(&mut self) -> Result<Response, Error> {
        self.results.pop_front().unwrap()
    }
}

impl<Request, Response, Error> Default for Handle<Request, Response, Error> {
    fn default() -> Self {
        Self::new()
    }
}
