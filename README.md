# Tower Async

Tower Async is a library of modular and reusable components for building robust
networking clients and servers. It is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/tower-async.svg
[crates-url]: https://crates.io/crates/tower-async
[docs-badge]: https://docs.rs/tower-async/badge.svg
[docs-url]: https://docs.rs/tower-async
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/plabayo/tower-async/workflows/CI/badge.svg
[actions-url]:https://github.com/plabayo/tower-async/actions?query=workflow%3ACI

## Overview

Tower Async aims to make it as easy as possible to build robust networking clients and
servers. It is protocol agnostic, but is designed around a request / response
pattern. If your protocol is entirely stream based, Tower Async may not be a good fit.

It is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

This fork is made entirely with the needs of the author in mind,
and thus might not yet contain all features you might need.

Come join us at discord at <https://discord.com/channels/1114459060050333696/1123537825929900113>
or tag `@glendc` at Tokio's Tower discord instead.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.
Given however how big the diversange we aren't sure how likely that is.

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

- Turn a [`tower::Service`] into a [`tower_async::Service`];
- Turn a [`tower_async::Service`] into a [`tower::Service`];
- Use a [`tower_async::Layer`] within a [`tower`] environment (e.g. [`tower::ServiceBuilder`]);
- Use a [`tower::Layer`] within a [`tower_async`] environment (e.g. [`tower_async::ServiceBuilder`]);

Please check the crate's unit tests and examples to see specifically how to use the crate in order to achieve this.

Furthermore we also urge you to only use this kind of approach for transition purposes and not as a permanent way of life.
Best in our opinion is to use one or the other and not to combine the two. But if you do absolutely must
use one combined with the other, `tower-async-bridge` should allow you to do exactly that.

## Supported Rust Versions

Tower Async requires nightly Rust for the time being and has no backwards compatibility
promises for the time being.

Once `async traits` are stabalized we'll start supporting stable rust once again,
and we can start working towards backwards compatibility.

Read <https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html> for more information
on this roadmap by the Rust Language Core Team.

## Getting Started

If you're brand new to Tower and want to start with the basics we recommend you
check out some of the original Tower [guides].

We work exactly the same as Tower, expect in an async manner and slightly easier to use as such.
But the core ideas are obviously the same, so it should never the less help you to get started.

## Sponsorship

Regular and onetime sponsors alike help us to pay the development and service costs
done in function of all Plabayo's Free and Open Source work.

We're also a monthly sponsor of Tokio ourselves, to give back to all
the great work done and continued effort being put in by them.

You can find more about Plabayo Sponsorship at <https://github.com/sponsors/plabayo>.

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

For the sake of simplicitly, the sanity of the author of this fork,
and the ability to ship an async version of Tower on a reasonable timescale,
not all features that Tower support are supported (yet) in this fork.

Note that some features are not supported on purpose:

1. all the 'ready' related functionality was removed on purpose as we believe it to be out of scope
  - as such also all utilities that rely on this or build on top of this aren't supported

See the previous FAQ point to get our point of view related to load balancing and the like.

We do think there is plenty of room for growth and improvement.
Following utilities probably do still a place here and we welcome contributons:

- As `Service` functionality: `retry`
- As `MakeService` functionality: `limit`

And there are probably some more. The test coverage is also significantly less, so also
here do we welcome contributions.

And in general welcome any contributions.
Best to do come and chat with us prior to starting any big endavours.

> Where is the `tower-async-http` Classifier and Tracing code?

As this feature requires on the Retry functionality of `tower`,
which we have yet to port to `tower-async`, it is not yet available.

We very much welcome contributions to make this a possibility.

## License

This project is licensed under the [MIT license](LICENSE).

Big thanks and credits go towards
[the original Tower authors](https://github.com/tower-rs/tower/graphs/contributors?from=2016-08-21&to=2023-06-04&type=c)
which licensed their code under the same License type.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.


[`tower`]: https://docs.rs/tower/*/t
[`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
[`tower::ServiceBuilder`]: https://docs.rs/tower/*/tower/builder/struct.ServiceBuilder.html
[`tower::Layer`]: https://docs.rs/tower/*/tower/trait.Layer.html
[`tower_async`]: https://docs.rs/tower-async/*/tower_async
[`tower_async::Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
[`tower_async::ServiceBuilder`]: https://docs.rs/tower-async/*/tower_async/builder/struct.ServiceBuilder.html
[`tower_async::Layer`]: https://docs.rs/tower-async/*/tower_async/trait.Layer.html

[guides]: https://github.com/tower-rs/tower/tree/master/guides
