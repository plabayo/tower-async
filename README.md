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

Where suitable we'll keep in sync (manually) with Tower and if the
opportunity arises we'll contribute back "upstream" as well.

## Supported Rust Versions

Tower Async requires nightly Rust for the time being and has no backwards compatibility
promises for the time being.

Once `async traits` are stabalized we'll start supporting stable rust once again,
and we can start working towards backwards compatibility.

Read <https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html> for more information
on this roadmap by the Rust Language Core Team.

## Getting Started

If you're brand new to Tower and want to start with the basics we recommend you
check out some of our [guides].

## License

This project is licensed under the [MIT license](LICENSE).
Big thanks and credits go towards the original Tower authors which
licensed their code under the same License type.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower Async by you, shall be licensed as MIT, without any additional
terms or conditions.

[guides]: https://github.com/plabayo/tower-async/tree/master/guides
