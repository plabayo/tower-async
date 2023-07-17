#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use std::{
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use bytes::Bytes;
use clap::Parser;
use http::{header, StatusCode};
use hyper::service::make_service_fn;
use tower_async::{
    limit::policy::{ConcurrentPolicy, LimitReached},
    BoxError, Service, ServiceBuilder,
};
use tower_async_bridge::ClassicServiceExt;
use tower_async_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};

/// Simple Hyper server with an HTTP API
#[derive(Debug, Parser)]
struct Config {
    /// The port to listen on
    #[clap(short = 'p', long, default_value = "8080")]
    port: u16,
}

type Request = hyper::Request<hyper::Body>;
type Response = hyper::Response<hyper::Body>;

#[derive(Debug, Clone)]
struct WebServer {
    start_time: std::time::Instant,
}

impl WebServer {
    fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    async fn render_page_fast(&self) -> Response {
        self.render_page(StatusCode::OK, "This was a fast response.")
    }

    async fn render_page_slow(&self) -> Response {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        self.render_page(StatusCode::OK, "This was a slow response.")
    }

    async fn render_page_not_found(&self, path: &str) -> Response {
        self.render_page(
            StatusCode::NOT_FOUND,
            format!("The path {} was not found.", path).as_str(),
        )
    }

    fn render_page(&self, status: StatusCode, msg: &str) -> Response {
        hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "text/html")
            .status(status)
            .body(
                format!(
                    r##"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Hyper Http Server Example</title>
    </head>
    <body>
        <h1>Hello!</h1>
        <p>{msg}<p>
        <p>Server has been running {} seconds.</p>
    </body>
</html>
"##,
                    self.start_time.elapsed().as_secs()
                )
                .into(),
            )
            .unwrap()
    }
}

impl Service<Request> for WebServer {
    type Response = Response;
    type Error = Infallible;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        Ok(match request.uri().path() {
            "/fast" => self.render_page_fast().await,
            "/slow" => self.render_page_slow().await,
            path => self.render_page_not_found(path).await,
        })
    }
}

#[tokio::main]
async fn main() {
    // Setup tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let config = Config::parse();

    let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();

    let web_service = ServiceBuilder::new()
        .compression()
        .sensitive_request_headers(sensitive_headers.clone())
        .layer(
            TraceLayer::new_for_http()
            .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
            })
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        )
        .sensitive_response_headers(sensitive_headers)
        .timeout(Duration::from_secs(10))
        .map_result(|result: Result<Response, BoxError>| {
            if let Err(err) = &result {
                if err.is::<LimitReached>() {
                    return Ok(hyper::Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body(hyper::Body::empty())
                        .unwrap());
                }
            }
            result
        })
        .limit(ConcurrentPolicy::new(1))
        .service(WebServer::new())
        .into_classic();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    tracing::info!("Listening on {}", addr);
    // Serve our application
    hyper::Server::bind(&addr)
        .serve(make_service_fn(|_| {
            let web_service = web_service.clone();
            async move { Ok::<_, Infallible>(web_service.clone()) }
        }))
        .await
        .expect("server error");
}
