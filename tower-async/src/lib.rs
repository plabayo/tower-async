#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(impl_trait_projections)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![cfg_attr(test, allow(clippy::float_cmp))]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
// `rustdoc::broken_intra_doc_links` is checked on CI

//! `async fn(Request) -> Result<Response, Error>`
//!
//! ## Fork
//!
//! Tower Async is a fork of <https://github.com/tower-rs/tower>
//! and makes use of `async traits` to simplify things and make it more easier
//! to integrate async functions into middleware.
//!
//! This fork is made entirely with the needs of the author in mind,
//! and thus might not yet contain all features you might need.
//!
//! Come join us at discord at <https://discord.com/channels/1114459060050333696/1123537825929900113>
//! or tag `@glendc` at Tokio's Tower discord instead.
//!
//! Where suitable we'll keep in sync (manually) with Tower and if the
//! opportunity arises we'll contribute back "upstream" as well.
//! Given however how big the diversange we aren't sure how likely that is.
//!
//! ## Overview
//!
//! Tower Async aims to make it as easy as possible to build robust networking clients and
//! servers. It is protocol agnostic, but is designed around a request / response
//! pattern. If your protocol is entirely stream based, Tower may not be a good fit.
//!
//! Tower Async provides a simple core abstraction, the [`Service`] trait, which
//! represents an asynchronous function taking a request and returning either a
//! response or an error. This abstraction can be used to model both clients and
//! servers.
//!
//! Generic components, like [timeouts], [rate limiting], and [load balancing],
//! can be modeled as [`Service`]s that wrap some inner service and apply
//! additional behavior before or after the inner service is called. This allows
//! implementing these components in a protocol-agnostic, composable way. Typically,
//! such services are referred to as _middleware_.
//!
//! An additional abstraction, the [`Layer`] trait, is used to compose
//! middleware with [`Service`]s. If a [`Service`] can be thought of as an
//! asynchronous function from a request type to a response type, a [`Layer`] is
//! a function taking a [`Service`] of one type and returning a [`Service`] of a
//! different type. The [`ServiceBuilder`] type is used to add middleware to a
//! service by composing it with multiple [`Layer`]s.
//!
//! ## Difference with Tokio's official Tower Ecosystem?
//!
//! - Make use of `Async Traits`
//! ([RFC-3185: Static async fn in traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html))
//! instead of requiring the user to manually implement Futures;
//!    - Which in fact forces users to Box Services that rely on futures which cannot be named,
//!      e.g. those returned by `async functions` that the user might have to face by using
//!      common utility functions from the wider _Tokio_ ecosystem;
//! - Drop the notion of `poll_ready` (See [the FAQ](https://github.com/plabayo/tower-async#faq)).
//!
//! ## Bridging to Tokio's official Tower Ecosystem
//!
//! You can make use of the `tower-async-bridge` crate as found in this repo in the [./tower-async-bridge](./tower-async-bridge/) directory,
//! and published at [crates.io](https://crates.io/) under the same name.
//!
//! At a high level it allows you to:
//!
//! - Turn a [`tower::Service`] into a [`tower_async::Service`];
//! - Turn a [`tower_async::Service`] into a [`tower::Service`];
//! - Use a [`tower_async::Layer`] within a [`tower`] environment (e.g. [`tower::ServiceBuilder`]);
//! - Use a [`tower::Layer`] within a [`tower_async`] environment (e.g. [`tower_async::ServiceBuilder`]);
//!
//! Please check the crate's unit tests and examples to see specifically how to use the crate in order to achieve this.
//!
//! Furthermore we also urge you to only use this kind of approach for transition purposes and not as a permanent way of life.
//! Best in our opinion is to use one or the other and not to combine the two. But if you do absolutely must
//! use one combined with the other, `tower-async-bridge` should allow you to do exactly that.
//!
//! ### The Tower Ecosystem
//!
//! Tower is made up of the following crates:
//!
//! * [`tower-async`] (this crate)
//! * [`tower-async-bridge`]
//! * [`tower-async-service`]
//! * [`tower-async-layer`]
//!
//! Since the [`Service`] and [`Layer`] traits are important integration points
//! for all libraries using Tower, they are kept as stable as possible, and
//! breaking changes are made rarely. Therefore, they are defined in separate
//! crates, [`tower-async-service`] and [`tower-async-layer`]. This crate contains
//! re-exports of those core traits, implementations of commonly-used
//! middleware, and [utilities] for working with [`Service`]s and [`Layer`]s.
//!
//! [`tower-async-bridge`] is there to bridge Tokio's official Tower ecosystem
//! with this (Aync Trait) version (Fork).
//!
//! ## Usage
//!
//! Tower provides an abstraction layer, and generic implementations of various
//! middleware. This means that the `tower-async` crate on its own does *not* provide
//! a working implementation of a network client or server. Instead, Tower's
//! [`Service` trait][`Service`] provides an integration point between
//! application code, libraries providing middleware implementations, and
//! libraries that implement servers and/or clients for various network
//! protocols.
//!
//! Depending on your particular use case, you might use Tower in several ways:
//!
//! * **Implementing application logic** for a networked program. You might
//!   use the [`Service`] trait to model your application's behavior, and use
//!   the middleware [provided by this crate][all_layers] and by other libraries
//!   to add functionality to clients and servers provided by one or more
//!   protocol implementations.
//! * **Implementing middleware** to add custom behavior to network clients and
//!   servers in a reusable manner. This might be general-purpose middleware
//!   (and if it is, please consider releasing your middleware as a library for
//!   other Tower users!) or application-specific behavior that needs to be
//!   shared between multiple clients or servers.
//! * **Implementing a network protocol**. Libraries that implement network
//!   protocols (such as HTTP) can depend on `tower-async-service` to use the
//!   [`Service`] trait as an integration point between the protocol and user
//!   code. For example, a client for some protocol might implement [`Service`],
//!   allowing users to add arbitrary Tower middleware to those clients.
//!   Similarly, a server might be created from a user-provided [`Service`].
//!
//!   Additionally, when a network protocol requires functionality already
//!   provided by existing Tower middleware, a protocol implementation might use
//!   Tower middleware internally, as well as an integration point.
//!
//! ### Library Support
//!
//! Following are some libraries that make use of Tower Async (instead of Tower)
//! and the [`Service`] trait:
//!
//! * [`rama`]: A proxy framework to anonymise your network traffic.
//!
//! [`rama`]: https://crates.io/crates/rama
//!
//! If you're the maintainer of a crate that supports Tower Async, we'd love to add
//! your crate to this list! Please [open a PR] adding a brief description of
//! your library!
//!
//! ### Getting Started
//!
//! The various middleware implementations provided by this crate are feature
//! flagged, so that users can only compile the parts of Tower they need. By
//! default, all the optional middleware are disabled.
//!
//! To get started using all of Tower's optional middleware, add this to your
//! `Cargo.toml`:
//!
//! ```toml
//! tower-async = { version = "0.4", features = ["full"] }
//! ```
//!
//! Alternatively, you can only enable some features. For example,
//! to enable only the [`timeout`][timeouts] middleware, write:
//!
//! ```toml
//! tower-async = { version = "0.4", features = ["timeout"] }
//! ```
//!
//! See [here][all_layers] for a complete list of all middleware provided by
//! Tower.
//!
//! [`Service`]: https://docs.rs/tower-async/latest/tower-async/trait.Service.html
//! [`Layer`]: https://docs.rs/tower-async/latest/tower-async/trait.Layer.html
//! [all_layers]: https://docs.rs/tower-async/latest/tower-async/#modules
//! [timeouts]: https://docs.rs/tower-async/latest/tower-async/timeout/
//! [`ServiceBuilder`]: https://docs.rs/tower-async/latest/tower-async/struct.ServiceBuilder.html
//! [utilities]: https://docs.rs/tower-async/latest/tower-async/trait.ServiceExt.html
//! [`tower-async`]: https://crates.io/crates/tower
//! [`tower-async-bridge`]: https://crates.io/crates/tower-async-bridge
//! [`tower-async-service`]: https://crates.io/crates/tower-async-service
//! [`tower-async-layer`]: https://crates.io/crates/tower-async-layer
//! [open a PR]: https://github.com/plabayo/tower-async/compare
//!
//! [`tower`]: https://docs.rs/tower/*/t
//! [`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
//! [`tower::ServiceBuilder`]: https://docs.rs/tower/*/tower/builder/struct.ServiceBuilder.html
//! [`tower::Layer`]: https://docs.rs/tower/*/tower/trait.Layer.html
//! [`tower_async`]: https://docs.rs/tower-async/*/tower_async
//! [`tower_async::Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
//! [`tower_async::ServiceBuilder`]: https://docs.rs/tower-async/*/tower_async/builder/struct.ServiceBuilder.html
//! [`tower_async::Layer`]: https://docs.rs/tower-async/*/tower_async/trait.Layer.html
//!
//! ## Supported Rust Versions
//!
//! Tower Async requires nightly Rust for the time being and has no backwards compatibility
//! promises for the time being.
//!
//! Once `async traits` are stabalized we'll start supporting stable rust once again,
//! and we can start working towards backwards compatibility.
//!
//! Read <https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html> for more information
//! on this roadmap by the Rust Language Core Team.

#[cfg(feature = "filter")]
pub mod filter;

#[cfg(feature = "make")]
pub mod make;
#[cfg(feature = "retry")]
pub mod retry;
#[cfg(feature = "timeout")]
pub mod timeout;
#[cfg(feature = "util")]
pub mod util;

pub mod builder;
pub mod layer;

#[cfg(feature = "util")]
#[doc(inline)]
pub use self::util::{service_fn, ServiceExt};

#[doc(inline)]
pub use crate::builder::ServiceBuilder;
#[cfg(feature = "make")]
#[doc(inline)]
pub use crate::make::MakeService;
#[doc(inline)]
pub use tower_async_layer::Layer;
#[doc(inline)]
pub use tower_async_service::Service;

#[allow(unreachable_pub)]
mod sealed {
    pub trait Sealed<T> {}
}

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
