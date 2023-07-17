# Tower Async Layer

Decorates a [Tower Async] `Service`, transforming either the request or the response.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/tower-async-layer.svg
[crates-url]: https://crates.io/crates/tower-async-layer
[docs-badge]: https://docs.rs/tower-async-layer/badge.svg
[docs-url]: https://docs.rs/tower-async-layer
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/plabayo/tower-async/workflows/CI/badge.svg
[actions-url]:https://github.com/plabayo/tower-async/actions?query=workflow%3ACI

## Fork

Tower Async Layer is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

This fork is made entirely with the needs of the author in mind,
and thus might not yet contain all features you might need.

Come join us at discord at <https://discord.com/channels/1114459060050333696/1123537825929900113>
or tag `@glendc` at Tokio's Tower discord instead.

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.
Given however how big the diversange we aren't sure how likely that is.

## Overview

Often, many of the pieces needed for writing network applications can be
reused across multiple services. The `Layer` trait can be used to write
reusable components that can be applied to very different kinds of services;
for example, it can be applied to services operating on different protocols,
and to both the client and server side of a network transaction.

## License

This project is licensed under the [MIT license](LICENSE).

Big thanks and credits go towards
[the original Tower authors](https://github.com/tower-rs/tower/graphs/contributors?from=2016-08-21&to=2023-06-04&type=c)
which licensed their code under the same License type.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.

[Tower Async]: https://crates.io/crates/tower-async