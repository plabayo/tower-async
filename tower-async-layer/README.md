# Tower Layer

Decorates a [Tower Async] `Service`, transforming either the request or the response.

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

Tower Async Layer is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.

## Overview

Often, many of the pieces needed for writing network applications can be
reused across multiple services. The `Layer` trait can be used to write
reusable components that can be applied to very different kinds of services;
for example, it can be applied to services operating on different protocols,
and to both the client and server side of a network transaction.

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.

[Tower Async]: https://crates.io/crates/tower-async