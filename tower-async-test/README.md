# Tower Test

Utilities for writing client and server `Service` tests.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![Discord chat][discord-badge]][discord-url]

It is a fork of <https://github.com/tower-rs/tower>
and makes use of `async traits` to simplify things and make it more easier
to integrate async functions into middleware.

Compared to other forks in this mono repository, this specific `tower-async-test` crate is however
only a spiritual fork of `tower-test`, with a completely different implementation,
as the needs are very different then when using a classifc ``

[crates-badge]: https://img.shields.io/crates/v/tower_async_test.svg
[crates-url]: https://crates.io/crates/tower-async-test
[docs-badge]: https://docs.rs/tower-async-test/badge.svg
[docs-url]: https://docs.rs/tower-async-test
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/tower-rs/tower/workflows/CI/badge.svg
[actions-url]:https://github.com/tower-rs/tower/actions?query=workflow%3ACI
[discord-badge]: https://img.shields.io/discord/500028886025895936?logo=discord&label=discord&logoColor=white
[discord-url]: https://discord.gg/EeF3cQw

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Tower by you, shall be licensed as MIT, without any additional
terms or conditions.
