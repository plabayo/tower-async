use tokio_test::{assert_pending};
use tower_async_test::{assert_request_eq, mock};

#[tokio::test(flavor = "current_thread")]
async fn single_request_ready() {
    let (mut service, mut handle) = mock::spawn();

    assert_pending!(handle.poll_request());

    let response = service.call("hello");

    assert_request_eq!(handle, "hello").send_response("world");

    assert_eq!(response.await.unwrap(), "world");
}
