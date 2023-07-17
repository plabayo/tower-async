# Tower Async

Tower Async is a library of modular and reusable components for building robust
networking clients and servers.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/tower-async-service.svg
[crates-url]: https://crates.io/crates/tower-async
[docs-badge]: https://docs.rs/tower-async/badge.svg
[docs-url]: https://docs.rs/tower-async
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/plabayo/tower-async/workflows/CI/badge.svg
[actions-url]:https://github.com/plabayo/tower-async/actions?query=workflow%3ACI

## Fork

Tower Async is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

Come join us at discord on the `#tower-async` public channel at [Discord](https://discord.gg/29EetaSYCD)
or tag `@glendc` at Tokio's Tower discord instead.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.
Given however how big the diversange we aren't sure how likely that is.

This set of libraries is best suited in an ecosystem of its own,
that is to say, making use only of `tower-async` libraries and dependents on it.
At the very least it is desired that `tower-async` is the puppeteer with where needed
making use of `tower` (classic) (middleware) layers.

For an example on how to operate purely within a `tower-async` environment you can
explore [the Rama codebase](https://www.github.com/plabayo/rama), a proxy framework,
written purely with a `tower-async` mindset, and the main motivator to start this fork.

You can however also bridge `tower` and `tower-async` in any other way. Please consult
[the "Bridging to Tokio's official Tower Ecosystem" chapter](#Bridging-to-Tokios-official-Tower-Ecosystem)
for more information on how to do that.

## Overview

Tower Async aims to make it as easy as possible to build robust networking clients and
servers. It is protocol agnostic, but is designed around a request / response
pattern. If your protocol is entirely stream based, Tower may not be a good fit.

Tower Async provides a simple core abstraction, the [`Service`] trait, which
represents an asynchronous function taking a request and returning either a
response or an error. This abstraction can be used to model both clients and
servers.

Generic components, like [timeouts], can be modeled as [`Service`]s
that wrap some inner service and apply
additional behavior before or after the inner service is called. This allows
implementing these components in a protocol-agnostic, composable way. Typically,
such services are referred to as _middleware_.

An additional abstraction, the [`Layer`] trait, is used to compose
middleware with [`Service`]s. If a [`Service`] can be thought of as an
asynchronous function from a request type to a response type, a [`Layer`] is
a function taking a [`Service`] of one type and returning a [`Service`] of a
different type. The [`ServiceBuilder`] type is used to add middleware to a
service by composing it with multiple [`Layer`]s.

## Difference with Tokio's official Tower Ecosystem?

- Make use of `Async Traits`
  ([RFC-3185: Static async fn in traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html))
  instead of requiring the user to manually implement Futures;
  - Which in fact forces users to Box Services that rely on futures which cannot be named,
    e.g. those returned by `async functions` that the user might have to face by using
    common utility functions from the wider _Tokio_ ecosystem;
- Drop the notion of `poll_ready` (See [the FAQ](#faq)).

## Bridging to Tokio's official Tower Ecosystem

You can make use of the `tower-async-bridge` crate as found in this repo in the [./tower-async-bridge](./tower-async-bridge/) directory,
and published at [crates.io](https://crates.io/) under the same name.

At a high level it allows you to:

- Turn a [`tower::Service`] into a [`tower_async::Service`] (requires the `into_async` feature);
- Turn a [`tower_async::Service`] into a [`tower::Service`];
- Use a [`tower_async::Layer`] within a [`tower`] environment (e.g. [`tower::ServiceBuilder`]);
- Use a [`tower::Layer`] within a [`tower_async`] environment (e.g. [`tower_async::ServiceBuilder`]) (requires the `into_async` feature);

Please check the crate's unit tests and examples to see specifically how to use the crate in order to achieve this.

Furthermore we also urge you to only use this kind of approach for transition purposes and not as a permanent way of life.
Best in our opinion is to use one or the other and not to combine the two. But if you do absolutely must
use one combined with the other, `tower-async-bridge` should allow you to do exactly that.

### The Tower Ecosystem

Tower is made up of the following crates:

* [`tower-async`] (this crate)
* [`tower-async-bridge`]
* [`tower-async-service`]
* [`tower-async-layer`]

Since the [`Service`] and [`Layer`] traits are important integration points
for all libraries using Tower, they are kept as stable as possible, and
breaking changes are made rarely. Therefore, they are defined in separate
crates, [`tower-async-service`] and [`tower-async-layer`]. This crate contains
re-exports of those core traits, implementations of commonly-used
middleware, and [utilities] for working with [`Service`]s and [`Layer`]s.

[`tower-async-bridge`] is there to bridge Tokio's official Tower ecosystem
with this (Aync Trait) version (Fork).

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
tower-async = { version = "0.4", features = ["full"] }
```

Alternatively, you can only enable some features. For example, to enable
only the [`timeout`][timeouts] middleware, write:

```toml
tower-async = { version = "0.4", features = ["timeout"] }
```

See [here][all_layers] for a complete list of all middleware provided by
Tower.

[`Service`]: https://docs.rs/tower-async/latest/tower-async/trait.Service.html
[`Layer`]: https://docs.rs/tower-async/latest/tower-async/trait.Layer.html
[all_layers]: https://docs.rs/tower-async/latest/tower-async/#modules
[timeouts]: https://docs.rs/tower-async/latest/tower-async/timeout/
[`ServiceBuilder`]: https://docs.rs/tower-async/latest/tower-async/struct.ServiceBuilder.html
[utilities]: https://docs.rs/tower-async/latest/tower-async/trait.ServiceExt.html
[`tower-async`]: https://crates.io/crates/tower
[`tower-async-bridge`]: https://crates.io/crates/tower-async-bridge
[`tower-async-service`]: https://crates.io/crates/tower-async-service
[`tower-async-layer`]: https://crates.io/crates/tower-async-layer
[open a PR]: https://github.com/plabayo/tower-async/compare

[`tower`]: https://docs.rs/tower/*/t
[`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
[`tower::ServiceBuilder`]: https://docs.rs/tower/*/tower/builder/struct.ServiceBuilder.html
[`tower::Layer`]: https://docs.rs/tower/*/tower/trait.Layer.html
[`tower_async`]: https://docs.rs/tower-async/*/tower_async
[`tower_async::Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
[`tower_async::ServiceBuilder`]: https://docs.rs/tower-async/*/tower_async/builder/struct.ServiceBuilder.html
[`tower_async::Layer`]: https://docs.rs/tower-async/*/tower_async/trait.Layer.html


## Supported Rust Versions

Tower Async requires nightly Rust for the time being and has no backwards compatibility
promises for the time being.

Once `async traits` are stabalized we'll start supporting stable rust once again,
and we can start working towards backwards compatibility.

Read <https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html> for more information
on this roadmap by the Rust Language Core Team.

## Sponsorship

Regular and onetime sponsors alike help us to pay the development and service costs
done in function of all Plabayo's Free and Open Source work.

We're also a monthly sponsor of Tokio ourselves, to give back to all
the great work done and continued effort being put in by them.

You can find more about Plabayo Sponsorship at <https://github.com/sponsors/plabayo>.

One time sponsorships (the so called "buy me a coffee", but then via GitHub Sponsors payments),
are welcome as much as regular sponsors. Not everbody have the financial means to sponsor,
so feel free [to contribute in any other way](#contribution) that you can think of.

## FAQ

> Where is the `poll_ready` method from Tower's Service?

This has been removed for the purpose of simplification and because the authors of this
fork consider it a problem out of scope:

- most Tower services / layers do not ever need the `poll_ready` method, and simply call the inner service for that;
- for some backpressure purposes you do want to know the request to know how to handle it, so `poll_ready` wouldn't work for these;

`poll_ready` was also used for load balancing services but this is considered out of scope:

- load balancing incoming network streams is in our humble opinion more something
  to be handled by your network infrastructure surrounding your service (using a... load balancer);
- and again... if you do want to load balance within a service it might be because you
  actually require context from the request to know what to do, in which case `poll_ready` wouldn't work for you;

Where you do still want to apply some kind of rate limiting, back pressure or load balancing
within a Tower (Async) Service you are to do it within the `call` function instead.

This fork is however still in its early days, so feel free to start a discussion if you feel different about this topic.
The authors of this library are always open for feedback but retain the reservation to deny any request they wish.

> Where is my favourite Tower Utility?

As all the tower code has to be manually ported, there might
be some features missing. The tower ecosystem also continues to thrive and live happy,
so there might still be new features added there as well. Feel free to chat with us
or open a ticket on GitHub in case you wish to add/port such feature(s).

Note that some features are not supported on purpose:

1. all the 'ready' related functionality was removed on purpose as we believe it to be out of scope
  - as such also all utilities that rely on this or build on top of this aren't supported

See the previous FAQ point to get our point of view related to load balancing and the like.

## License

This project is licensed under the [MIT license](LICENSE).

Big thanks and credits go towards
[the original Tower authors](https://github.com/tower-rs/tower/graphs/contributors?from=2016-08-21&to=2023-06-04&type=c)
which licensed their code under the same License type.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.
