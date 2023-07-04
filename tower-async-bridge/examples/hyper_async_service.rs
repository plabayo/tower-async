#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use tower_async_bridge::ClassicServiceExt;

use hyper::service::make_service_fn;
use hyper::{Body, Server, StatusCode};
use hyper::{Request, Response};
use tokio::sync::Mutex;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

type Counter = i32;

#[derive(Debug, Clone)]
struct CounterWebService {
    counter: Arc<Mutex<Counter>>,
}

impl CounterWebService {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

impl tower_async_service::Service<Request<Body>> for CounterWebService {
    type Response = Response<Body>;
    type Error = hyper::Error;

    async fn call(&mut self, _req: Request<Body>) -> Result<Self::Response, Self::Error> {
        let counter = {
            let mut counter = self.counter.lock().await;
            *counter += 1;
            *counter
        };

        let response = Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(format!("Counter: {}", counter)))
            .unwrap();

        Ok(response)
    }
}

#[tokio::main]
async fn main() {
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let service = CounterWebService::new().into_classic();

    let make_service = make_service_fn(move |_conn| {
        let service = service.clone();
        async move { Ok::<_, Infallible>(service.clone()) }
    });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
