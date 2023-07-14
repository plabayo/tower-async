#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use std::net::{Ipv4Addr, SocketAddr};

use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use tower_async::{Layer, Service, ServiceBuilder};
use tower_async_bridge::ClassicLayerExt;

/// Simple Hyper HTTP server with an HTTP API
#[derive(Debug, Parser)]
struct Config {
    /// The port to listen on
    #[clap(short = 'p', long, default_value = "8080")]
    port: u16,
}

async fn hello_world() -> impl IntoResponse {
    "Hello, World!".to_string()
}

#[derive(Debug, Clone)]
struct LogService<S> {
    target: String,
    service: S,
}

impl<S> LogService<S> {
    fn new(target: String, service: S) -> Self {
        Self { target, service }
    }
}

impl<Request, S> Service<Request> for LogService<S>
where
    S: Service<Request>,
    Request: std::fmt::Debug,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        // Insert log statement here or other functionality
        let stmt = format!("request = {:?}, target = {:?}", request, self.target);
        println!("{stmt}");
        let result = self.service.call(request).await;
        if result.is_ok() {
            println!("{stmt}; succeeded");
        } else {
            println!("{stmt}; failed");
        }
        result
    }
}

#[derive(Debug, Clone)]
struct LogLayer {
    target: String,
}

impl LogLayer {
    fn new(target: String) -> Self {
        Self { target }
    }
}

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, service: S) -> Self::Service {
        LogService::new(self.target.clone(), service)
    }
}

fn app() -> Router {
    // Build our middleware stack
    let middleware = ServiceBuilder::new().layer(LogLayer::new("axum-http-server".to_string()));

    // Build route service
    Router::new()
        .route("/", get(hello_world))
        .layer(middleware.into_classic())
}

#[tokio::main]
async fn main() {
    // Setup tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let config = Config::parse();

    // Run our service
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    tracing::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .await
        .expect("server error");
}
