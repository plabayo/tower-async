//! Bridges a `tower-async` `Service` to be used within a `hyper` (1.x) environment.
//!
//! In case you also make use of `tower-async-http`,
//! you can use its [`tower_async_http::map_request_body::MapRequestBodyLayer`] middleware
//! to convert the normal [`hyper::body::Incoming`] [`http_body::Body`] into a [`HyperBody`]
//! as it can be used with middlewares that require the [`http_body::Body`] to be [`Default`].
//!
//! [`tower_async_http::map_request_body::MapRequestBodyLayer`]: https://docs.rs/tower-async-http/latest/tower_async_http/map_request_body/struct.MapRequestBodyLayer.html
//!
//! # Example
//!
//! ```rust,no_run
//! use std::net::SocketAddr;
//!
//! use http::{Request, Response, StatusCode};
//! use hyper_util::rt::{TokioExecutor, TokioIo};
//! use hyper_util::server::conn::auto::Builder;
//! use tokio::net::TcpListener;
//! use tracing_subscriber::filter::LevelFilter;
//! use tracing_subscriber::layer::SubscriberExt;
//! use tracing_subscriber::util::SubscriberInitExt;
//! use tracing_subscriber::{fmt, EnvFilter};
//!
//! use tower_async::ServiceBuilder;
//! use tower_async_http::ServiceBuilderExt;
//! use tower_async_hyper::{HyperBody, TowerHyperServiceExt};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     tracing_subscriber::registry()
//!         .with(fmt::layer())
//!         .with(
//!             EnvFilter::builder()
//!                 .with_default_directive(LevelFilter::DEBUG.into())
//!                 .from_env_lossy(),
//!         )
//!         .init();
//!
//!     let service = ServiceBuilder::new()
//!         .map_request_body(HyperBody::from)
//!         .timeout(std::time::Duration::from_secs(5))
//!         .decompression()
//!         .compression()
//!         .follow_redirects()
//!         .trace_for_http()
//!         .service_fn(|_req: Request<HyperBody>| async move {
//!             Response::builder()
//!                 .status(StatusCode::OK)
//!                 .header("content-type", "text/plain")
//!                 .body(String::from("hello"))
//!         });
//!
//!     let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
//!     let listener = TcpListener::bind(addr).await?;
//!
//!     loop {
//!         let (stream, _) = listener.accept().await?;
//!         let service = service.clone().into_hyper_service();
//!         tokio::spawn(async move {
//!             let stream = TokioIo::new(stream);
//!             let result = Builder::new(TokioExecutor::new())
//!                 .serve_connection(stream, service)
//!                 .await;
//!             if let Err(e) = result {
//!                 eprintln!("server connection error: {}", e);
//!             }
//!         });
//!     }
//! }
//! ```

#![feature(return_type_notation)]
#![allow(incomplete_features)]

mod service;
pub use service::{BoxFuture, HyperServiceWrapper, TowerHyperServiceExt};

mod body;
pub use body::Body as HyperBody;
