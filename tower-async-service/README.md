# Tower Async Service

The foundational `Service` trait that [Tower Async] is based on.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/tower_async_service.svg
[crates-url]: https://crates.io/crates/tower-async-service
[docs-badge]: https://docs.rs/tower-async-service/badge.svg
[docs-url]: https://docs.rs/tower-async-service
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/plabayo/tower-async/workflows/CI/badge.svg
[actions-url]:https://github.com/plabayo/tower-async/actions?query=workflow%3ACI

## Fork

Tower Async Service is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

This fork is made entirely with the needs of the author in mind,
and thus might not yet contain all features you might need.

Come join us at discord on the `#tower-async` public channel at [Discord](https://discord.gg/29EetaSYCD)
or tag `@glendc` at Tokio's Tower discord instead.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.
Given however how big the diversange we aren't sure how likely that is.

## Overview

The [`Service`] trait provides the foundation upon which [Tower] is built. It is a
simple, but powerful trait. At its heart, `Service` is just an asynchronous
function of request to response.

```
async fn(Request) -> Result<Response, Error>
```

Implementations of `Service` take a request, the type of which varies per
protocol, and returns a future representing the eventual completion or failure
of the response.

Services are used to represent both clients and servers. An *instance* of
`Service` is used through a client; a server *implements* `Service`.

By using standardizing the interface, middleware can be created. Middleware
*implement* `Service` by passing the request to another `Service`. The
middleware may take actions such as modify the request.

[`Service`]: https://docs.rs/tower-async-async-service/latest/tower_async_service/trait.Service.html
[Tower Async]: https://crates.io/crates/tower-async-async

## License

This project is licensed under the [MIT license](LICENSE).

Big thanks and credits go towards
[the original Tower authors](https://github.com/tower-rs/tower/graphs/contributors?from=2016-08-21&to=2023-06-04&type=c)
which licensed their code under the same License type.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.
