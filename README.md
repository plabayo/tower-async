# Tower Async

Tower Async is a library of modular and reusable components for building robust
networking clients and servers. It is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/tower_async.svg
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
and makes use of `async traits` ([RFC-3185: Static async fn in traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html)) to simplify things and make it more easier to integrate async functions into middleware.

> The authors of this repository are not affiliated with the
> official Tokio and Tower codebases, other than that we as Plabayo
> do sponsor [the Tokio project via Github Sponsors](https://github.com/sponsors/tokio-rs).

Tower async was featured as crate of the week in "This week in Rust #505": <https://this-week-in-rust.org/blog/2023/07/26/this-week-in-rust-505/#crate-of-the-week>.

> If you want to see a prime example of how much simpler
> using [`tower_async`] is versus [`tower`], you can see an example here:
>
> A delay service using [`tower`]: <https://github.com/plabayo/tower-async/blob/4ae0c4747fac6cc69b27c87a7ea5cacdd8bab3fb/tower-async-bridge/src/into_async/async_layer.rs#L91-L169>.
>
> That same service can be written in [`tower_async`] as follows:
>
> ```rust
> #[derive(Debug)]
> struct DelayService<S> {
>     inner: S,
>     delay: std::time::Duration,
> }
> 
> impl<S> DelayService<S> {
>     fn new(inner: S, delay: std::time::Duration) -> Self {
>         Self { inner, delay }
>     }
> }
> 
> impl<S, Request> tower_async_service::Service<Request> for DelayService<S>
> where
>     S: tower_async_service::Service<Request>,
> {
>     type Response = S::Response;
>     type Error = S::Error;
> 
>     async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
>         tokio::time::sleep(self.delay).await;
>         self.inner.call(request)
>     }
> }
> ```
>
> If you compare that with the linked `tower` version you can probably agree
> that things are a lot simpler if you do not have to hand write future state machines
> yourself, which is the reason why we have the `async` sugar in the first place.
>
> Of course I do acknowledge that if you make use of amazing utilities provided
> by crates such as <https://docs.rs/futures-util-preview/latest/futures_util/future/index.html>
> That you an write pretty much the same code without having to handwrite the future yourself.
>
> This is however not always possible, and it does mean that you need to (1) know about this
> and (2) pull it as a dependency and all that it brings with it. While in reallity you really
> just want to be able write your middleware in an `async` manner.
>
> We fully acknowledge that `tower` _had_ to use the approach it used as for many years this
> was simply the only sensible thing to do, unless you want to force your users
> to make use of the <https://docs.rs/async-trait/latest/async_trait/>, a choice not
> everyone is willing to make, and sometimes they might not even have the luxary to do so.

Come join us at discord on the `#tower-async` public channel at [Discord](https://discord.gg/29EetaSYCD)
or tag `@glendc` at Tokio's Tower discord instead.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.
Given however how big the diversange we aren't sure how likely that is.

This set of libraries is best suited in an ecosystem of its own,
that is to say, making use only of [`tower_async`] libraries and dependents on it.
At the very least it is desired that [`tower_async`] is the puppeteer with where needed
making use of [`tower`] (classic) (middleware) layers.

For an example on how to operate purely within a [`tower_async`] environment you can
explore [the Rama codebase](https://www.github.com/plabayo/rama), a proxy framework,
written purely with a [`tower_async`] mindset, and the main motivator to start this fork.

You can however also bridge [`tower`] and [`tower_async`] in any other way. Please consult
[the "Bridging to Tokio's official Tower Ecosystem" chapter](#Bridging-to-Tokios-official-Tower-Ecosystem)
for more information on how to do that.

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

You can find an Axum service example of this and why this might be useful at
[./tower-async-http/examples/axum-key-value-store](./tower-async-http/examples/README.md).

The above example shows how you can use `Tower-Async*` in any location where you would otherwise use `Tower`.
As such it is also possible to use these crates on projects that use `Tonic`, `Hyper`, `Warp`, among others.

## Supported Rust Versions

Tower Async requires nightly Rust for the time being and has no backwards compatibility
promises for the time being.

Once `async traits` are stabilized we'll start supporting stable rust once again,
and we can start working towards backwards compatibility.

Read <https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html> for more information
on this roadmap by the Rust Language Core Team.

## Getting Started

If you're brand new to Tower and want to start with the basics we recommend you
check out some of the original Tower [guides].

We work exactly the same as Tower, expect in an async manner and slightly easier to use as such.
But the core ideas are obviously the same, so it should never the less help you to get started.

Browse the examples at [`tower-async-http/examples`](https://github.com/plabayo/tower-async/tree/master/tower-async-http/examples) to see some examples
on how to use [`tower_async`] and its sibling crates. While these are focussed on http examples,
note that:

- [`tower_async`] can work for any request-response flow (akin to `tower`);
- you can also use [`tower_async`] with http web services without making use of the `tower-async-http` crate,
  it only is there to provide extra middleware for http-specific purposes, but this is all optional.

The documentation also contains some smaller examples and of course the codebase can be read as well,
together with its unit tests.

> Are you still new to Rust, or feel you have much to learn on your Rust learning journey?
> You might find it useful to use <https://rust-lang.guide/> as your learning guide,
> which might also help you get you on your way to understand the more complicated parts
> of this monorepo codebase.

## Sponsorship

Regular and onetime sponsors alike help us to pay the development and service costs
done in function of all Plabayo's Free and Open Source work.

We're also a monthly sponsor of Tokio ourselves, to give back to all
the great work done and continued effort being put in by them.

You can find more about Plabayo Sponsorship at <https://github.com/sponsors/plabayo>.

One time sponsorships (the so called "buy me a coffee", but then via GitHub Sponsors payments),
are welcome as much as regular sponsors. Not everybody have the financial means to sponsor,
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

We do not have a roadmap for Tower Async. But here are some ideas on what you can contribute:

- bug reports;
- spelling checks;
- port over missing features from "classic" tower ecosystem;
- make ported async tower code more idiomatic;
- add a feature of your own desire;
- fix an open issue.


[`tower`]: https://docs.rs/tower/*/tower
[`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
[`tower::ServiceBuilder`]: https://docs.rs/tower/*/tower/builder/struct.ServiceBuilder.html
[`tower::Layer`]: https://docs.rs/tower/*/tower/trait.Layer.html
[`tower_async`]: https://docs.rs/tower-async/*/tower_async
[`tower_async::Service`]: https://docs.rs/tower-async-service/*/tower_async_service/trait.Service.html
[`tower_async::ServiceBuilder`]: https://docs.rs/tower-async/*/tower_async/builder/struct.ServiceBuilder.html
[`tower_async::Layer`]: https://docs.rs/tower-async-layer/*/tower_async_layer/trait.Layer.html

[guides]: https://github.com/tower-rs/tower/tree/master/guides
