//! `async fn(HttpRequest) -> Result<HttpResponse, Error>`
//!
//! # Overview
//!
//! tower-http is a library that provides HTTP-specific middleware and utilities built on top of
//! [tower-async].
//!
//! All middleware uses the [http] and [http-body] crates as the HTTP abstractions. That means
//! they're compatible with any library or framework that also uses those crates, such as
//! [hyper], [tonic], and [warp].
//!
//! Note that for these frameworks you do need to use the `tower-async-bridge` crate to convert
//! between the `tower-async` and `tower` abstractions.
//!
//! # Example server
//!
//! This example shows how to apply middleware from tower-http to a [`Service`] and then run
//! that service using [hyper].
//!
//! ```rust,no_run
//! use std::{sync::Arc, net::SocketAddr, convert::Infallible, iter::once};
//!
//! use http::{Request, Response, StatusCode, header::{AUTHORIZATION, CONTENT_TYPE, HeaderName}};
//! use hyper::body::Incoming;
//! use hyper_util::rt::{TokioExecutor, TokioIo};
//! use hyper_util::server::conn::auto::Builder;
//! use tokio::net::TcpListener;
//!
//! use tower_async::{ServiceBuilder, BoxError};
//! use tower_async_http::{
//!     ServiceBuilderExt,
//!     add_extension::AddExtensionLayer,
//!     compression::CompressionLayer,
//!     propagate_header::PropagateHeaderLayer,
//!     sensitive_headers::SetSensitiveRequestHeadersLayer,
//!     set_header::SetResponseHeaderLayer,
//!     validate_request::ValidateRequestHeaderLayer,
//! };
//! use tower_async_hyper::{HyperBody, TowerHyperServiceExt};
//!
//! # struct DatabaseConnectionPool;
//! # impl DatabaseConnectionPool {
//! #     fn new() -> DatabaseConnectionPool { DatabaseConnectionPool }
//! # }
//! # fn content_length_from_response<B>(_: &http::Response<B>) -> Option<http::HeaderValue> { None }
//! # async fn update_in_flight_requests_metric(count: usize) {}
//!
//! // Our request handler. This is where we would implement the application logic
//! // for responding to HTTP requests...
//! async fn handler(request: Request<HyperBody>) -> Result<Response<HyperBody>, BoxError> {
//!     // ...
//!     # todo!()
//! }
//!
//! // Shared state across all request handlers --- in this case, a pool of database connections.
//! struct State {
//!     pool: DatabaseConnectionPool,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), BoxError> {
//!     // Construct the shared state.
//!     let state = State {
//!         pool: DatabaseConnectionPool::new(),
//!     };
//!
//!     // Use tower's `ServiceBuilder` API to build a stack of tower middleware
//!     // wrapping our request handler.
//!     let service = ServiceBuilder::new()
//!         // To be able to use `Body` `Default` middleware
//!         .map_request_body(HyperBody::from)
//!         // Mark the `Authorization` request header as sensitive so it doesn't show in logs
//!         .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
//!         // Share an `Arc<State>` with all requests
//!         .layer(AddExtensionLayer::new(Arc::new(state)))
//!         // Compress responses
//!         .compression()
//!         // Propagate `X-Request-Id`s from requests to responses
//!         .layer(PropagateHeaderLayer::new(HeaderName::from_static("x-request-id")))
//!         // If the response has a known size set the `Content-Length` header
//!         .layer(SetResponseHeaderLayer::overriding(CONTENT_TYPE, content_length_from_response))
//!         // Authorize requests using a token
//!         .layer(ValidateRequestHeaderLayer::bearer("passwordlol"))
//!         // Accept only application/json, application/* and */* in a request's ACCEPT header
//!         .layer(ValidateRequestHeaderLayer::accept("application/json"))
//!         // Wrap a `Service` in our middleware stack
//!         .service_fn(handler);
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
//!
//! Keep in mind that while this example uses [hyper], tower-http supports any HTTP
//! client/server implementation that uses the [http] and [http-body] crates.
//!
//! # Example client
//!
//! tower-http middleware can also be applied to HTTP clients:
//!
//! ```rust,no_run
//! use tower_async_http::{
//!     decompression::DecompressionLayer,
//!     set_header::SetRequestHeaderLayer,
//! };
//! use tower_async_bridge::AsyncServiceExt;
//! use tower_async::{ServiceBuilder, Service, ServiceExt};
//! use http_body_util::Full;
//! use bytes::Bytes;
//! use http::{Request, HeaderValue, header::USER_AGENT};
//! use hyper_util::{client::legacy::Client, rt::TokioExecutor};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = ServiceBuilder::new()
//!         // Set a `User-Agent` header on all requests.
//!         .layer(SetRequestHeaderLayer::overriding(
//!             USER_AGENT,
//!             HeaderValue::from_static("tower-http demo")
//!         ))
//!         // Decompress response bodies
//!         .layer(DecompressionLayer::new())
//!         // Wrap a `hyper::Client` in our middleware stack.
//!         // This is possible because `hyper::Client` implements
//!         // `tower_async::Service`.
//!         .service(Client::builder(TokioExecutor::new()).build_http().into_async());
//!
//!     // Make a request
//!     let request = Request::builder()
//!         .uri("http://example.com")
//!         .body(Full::<Bytes>::default())
//!         .unwrap();
//!
//!     let response = client
//!         .call(request)
//!         .await
//!         .unwrap();
//! }
//! ```
//!
//! # Feature Flags
//!
//! All middleware are disabled by default and can be enabled using [cargo features].
//!
//! For example, to enable the [`Timeout`] middleware, add the "timeout" feature flag in
//! your `Cargo.toml`:
//!
//! ```toml
//! tower-async-http = { version = "0.2", features = ["timeout"] }
//! ```
//!
//! You can use `"full"` to enable everything:
//!
//! ```toml
//! tower-async-http = { version = "0.2", features = ["full"] }
//! ```
//!
//! [tower-async]: https://crates.io/crates/tower-async
//! [http]: https://crates.io/crates/http
//! [http-body]: https://crates.io/crates/http-body
//! [hyper]: https://crates.io/crates/hyper
//! [tonic]: https://crates.io/crates/tonic
//! [warp]: https://crates.io/crates/warp
//! [cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
//! [`AddExtension`]: crate::add_extension::AddExtension
//! [`Service`]: https://docs.rs/tower-async/latest/tower-async/trait.Service.html
//! [`Timeout`]: crate::timeout::Timeout

#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_docs
)]
#![deny(unreachable_pub)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]

#[macro_use]
pub(crate) mod macros;

#[cfg(test)]
mod test_helpers;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "set-header")]
pub mod set_header;

#[cfg(feature = "propagate-header")]
pub mod propagate_header;

#[cfg(any(
    feature = "compression-br",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-zstd",
))]
pub mod compression;

#[cfg(feature = "add-extension")]
pub mod add_extension;

#[cfg(feature = "sensitive-headers")]
pub mod sensitive_headers;

#[cfg(any(
    feature = "decompression-br",
    feature = "decompression-deflate",
    feature = "decompression-gzip",
    feature = "decompression-zstd",
))]
pub mod decompression;

#[cfg(any(
    feature = "compression-br",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-zstd",
    feature = "decompression-br",
    feature = "decompression-deflate",
    feature = "decompression-gzip",
    feature = "decompression-zstd",
    feature = "fs" // Used for serving precompressed static files as well
))]
mod content_encoding;

#[cfg(any(
    feature = "compression-br",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-zstd",
    feature = "decompression-br",
    feature = "decompression-deflate",
    feature = "decompression-gzip",
    feature = "decompression-zstd",
))]
mod compression_utils;

#[cfg(any(
    feature = "compression-br",
    feature = "compression-deflate",
    feature = "compression-gzip",
    feature = "compression-zstd",
    feature = "decompression-br",
    feature = "decompression-deflate",
    feature = "decompression-gzip",
    feature = "decompression-zstd",
))]
pub use compression_utils::CompressionLevel;

#[cfg(feature = "map-response-body")]
pub mod map_response_body;

#[cfg(feature = "map-request-body")]
pub mod map_request_body;

#[cfg(feature = "trace")]
pub mod trace;

#[cfg(feature = "follow-redirect")]
pub mod follow_redirect;

#[cfg(feature = "limit")]
pub mod limit;

#[cfg(feature = "cors")]
pub mod cors;

#[cfg(feature = "request-id")]
pub mod request_id;

#[cfg(feature = "catch-panic")]
pub mod catch_panic;

#[cfg(feature = "set-status")]
pub mod set_status;

#[cfg(feature = "timeout")]
pub mod timeout;

#[cfg(feature = "normalize-path")]
pub mod normalize_path;

pub mod classify;
pub mod services;

#[cfg(feature = "util")]
mod builder;

#[cfg(feature = "util")]
#[doc(inline)]
pub use self::builder::ServiceBuilderExt;

#[cfg(feature = "validate-request")]
pub mod validate_request;

/// The latency unit used to report latencies by middleware.
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum LatencyUnit {
    /// Use seconds.
    Seconds,
    /// Use milliseconds.
    Millis,
    /// Use microseconds.
    Micros,
    /// Use nanoseconds.
    Nanos,
}

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

mod sealed {
    #[allow(unreachable_pub)]
    pub trait Sealed<T> {}
}
