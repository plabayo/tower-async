# Tower Async

Tower Async is a library of modular and reusable components for building robust
networking clients and servers.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/tower-async-service.svg
[crates-url]: https://crates.io/crates/tower-async-service
[docs-badge]: https://docs.rs/tower-async-service/badge.svg
[docs-url]: https://docs.rs/tower-async-service
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/plabayo/tower-async/workflows/CI/badge.svg
[actions-url]:https://github.com/plabayo/tower-async/actions?query=workflow%3ACI

## Fork

Tower Async is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.

## Overview

Tower Async aims to make it as easy as possible to build robust networking clients and
servers. It is protocol agnostic, but is designed around a request / response
pattern. If your protocol is entirely stream based, Tower may not be a good fit.

Tower Async provides a simple core abstraction, the [`Service`] trait, which
represents an asynchronous function taking a request and returning either a
response or an error. This abstraction can be used to model both clients and
servers.

Generic components, like [timeouts], [rate limiting], and [load balancing],
can be modeled as [`Service`]s that wrap some inner service and apply
additional behavior before or after the inner service is called. This allows
implementing these components in a protocol-agnostic, composable way. Typically,
such services are referred to as _middleware_.

An additional abstraction, the [`Layer`] trait, is used to compose
middleware with [`Service`]s. If a [`Service`] can be thought of as an
asynchronous function from a request type to a response type, a [`Layer`] is
a function taking a [`Service`] of one type and returning a [`Service`] of a
different type. The [`ServiceBuilder`] type is used to add middleware to a
service by composing it with multiple [`Layer`]s.

### The Tower Ecosystem

Tower is made up of the following crates:

* [`tower-async`] (this crate)
* [`tower-async-service`]
* [`tower-async-layer`]
* [`tower-async-test`]

Since the [`Service`] and [`Layer`] traits are important integration points
for all libraries using Tower, they are kept as stable as possible, and
breaking changes are made rarely. Therefore, they are defined in separate
crates, [`tower-async-service`] and [`tower-async-layer`]. This crate contains
re-exports of those core traits, implementations of commonly-used
middleware, and [utilities] for working with [`Service`]s and [`Layer`]s.
Finally, the [`tower-async-test`] crate provides tools for testing programs using
Tower.

## Usage

Tower provides an abstraction layer, and generic implementations of various
middleware. This means that the `tower-async` crate on its own does *not* provide
a working implementation of a network client or server. Instead, Tower's
[`Service` trait][`Service`] provides an integration point between
application code, libraries providing middleware implementations, and
libraries that implement servers and/or clients for various network
protocols.

Depending on your particular use case, you might use Tower in several ways: 

* **Implementing application logic** for a networked program. You might
  use the [`Service`] trait to model your application's behavior, and use
  the middleware [provided by this crate][all_layers] and by other libraries
  to add functionality to clients and servers provided by one or more
  protocol implementations.
* **Implementing middleware** to add custom behavior to network clients and
  servers in a reusable manner. This might be general-purpose middleware
  (and if it is, please consider releasing your middleware as a library for
  other Tower users!) or application-specific behavior that needs to be
  shared between multiple clients or servers.
* **Implementing a network protocol**. Libraries that implement network
  protocols (such as HTTP) can depend on `tower-async-service` to use the
  [`Service`] trait as an integration point between the protocol and user
  code. For example, a client for some protocol might implement [`Service`],
  allowing users to add arbitrary Tower middleware to those clients.
  Similarly, a server might be created from a user-provided [`Service`].

  Additionally, when a network protocol requires functionality already
  provided by existing Tower middleware, a protocol implementation might use
  Tower middleware internally, as well as an integration point.

### Library Support

Following are some libraries that make use of Tower Async (instead of Tower)
and the [`Service`] trait:

* [`rama`]: A proxy framework to anonymise your network traffic.

[`rama`]: https://crates.io/crates/rama

If you're the maintainer of a crate that supports Tower Async, we'd love to add
your crate to this list! Please [open a PR] adding a brief description of
your library!

### Getting Started

The various middleware implementations provided by this crate are feature
flagged, so that users can only compile the parts of Tower they need. By
default, all the optional middleware are disabled.

To get started using all of Tower's optional middleware, add this to your
`Cargo.toml`:

```toml
tower = { version = "0.4", features = ["full"] }
```

Alternatively, you can only enable some features. For example, to enable
only the [`retry`] and [`timeout`][timeouts] middleware, write:

```toml
tower = { version = "0.4", features = ["retry", "timeout"] }
```

See [here][all_layers] for a complete list of all middleware provided by
Tower.

[`Service`]: https://docs.rs/tower-async/latest/tower-async/trait.Service.html
[`Layer`]: https://docs.rs/tower-async/latest/tower-async/trait.Layer.html
[all_layers]: https://docs.rs/tower-async/latest/tower-async/#modules
[timeouts]: https://docs.rs/tower-async/latest/tower-async/timeout/
[rate limiting]: https://docs.rs/tower-async/latest/tower-async/limit/rate
[load balancing]: https://docs.rs/tower-async/latest/tower-async/balance/
[`ServiceBuilder`]: https://docs.rs/tower-async/latest/tower-async/struct.ServiceBuilder.html
[utilities]: https://docs.rs/tower-async/latest/tower-async/trait.ServiceExt.html
[`tower-async`]: https://crates.io/crates/tower
[`tower-async-service`]: https://crates.io/crates/tower-async-service
[`tower-async-layer`]: https://crates.io/crates/tower-async-layer
[`tower-async-test`]: https://crates.io/crates/tower-async-test
[`retry`]: https://docs.rs/tower-async/latest/tower-async/retry
[open a PR]: https://github.com/plabayo/tower-async/compare


## Supported Rust Versions

Tower Async requires nightly Rust for the time being and has no backwards compatibility
promises for the time being.

Once `async traits` are stabalized we'll start supporting stable rust once again,
and we can start working towards backwards compatibility.

Read <https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html> for more information
on this roadmap by the Rust Language Core Team.

## License

This project is licensed under the [MIT license](LICENSE).

Big thanks and credits go towards
[the original Tower authors](https://github.com/tower-rs/tower/graphs/contributors?from=2016-08-21&to=2023-06-04&type=c)
which licensed their code under the same License type.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.
