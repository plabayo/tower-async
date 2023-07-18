# Tower Async HTTP

Tower Async middleware and utilities for HTTP clients and servers.

It is a fork of <https://github.com/tower-rs/tower-http>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

[![Build status](https://github.com/plabayo/tower-async/workflows/CI/badge.svg)](https://github.com/plabayo/tower-async/actions)
[![Crates.io](https://img.shields.io/crates/v/tower-async-http)](https://crates.io/crates/tower-async-http)
[![Documentation](https://docs.rs/tower-async-http/badge.svg)](https://docs.rs/tower-async-http)
[![Crates.io](https://img.shields.io/crates/l/tower-async-http)](LICENSE)

More information about this crate can be found in the [crate documentation][docs].

## Middleware

Tower Async HTTP contains lots of middleware that are generally useful when building
HTTP servers and clients. Some of the highlights are:

- `Compression` and `Decompression` to compress/decompress response bodies.
- `FollowRedirect` to automatically follow redirection responses.

See the [docs] for the complete list of middleware.

Middleware uses the [http] crate as the HTTP interface so they're compatible
with any library or framework that also uses [http]. For example [hyper].

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

## FAQ

Read the full `tower-async` FAQ at <https://github.com/plabayo/tower-async#faq>.

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower HTTP by you, shall be licensed as MIT, without any
additional terms or conditions.

[http]: https://crates.io/crates/http
[docs]: https://docs.rs/tower-http
[hyper]: https://github.com/hyperium/hyper
[guides]: https://github.com/tower-rs/tower/tree/master/guides
