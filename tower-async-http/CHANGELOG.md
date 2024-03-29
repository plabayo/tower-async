# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0 (November 20, 2023)

- Update to http-body 1.0;
- Update to http 1.0;
- Make library ready for `Hyper 1.0` usage;
- Adapt to new `tower_async::Service` contract:
  - `call` takes now `&self` instead of `&mut self`;
  - `call` returns `impl Future` instead of declared as `async fn`;

## 0.1.4 (September 10, 2023)

Sync with <https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.4.4>.

### Added

- **trace**: Default implementations for trace bodies.

### Fixed

- lots of changes due to new clippy rules or modifications to existing ones.

## 0.1.3 (July 24, 2023)

### Changed

- update `http-range-header` from `0.3.0` to `0.4.0` and fix range OOB test from reject to accept;

## 0.1.2 (July 20, 2023)

Sync with original `tower-http` codebase from [`0.4.1`](https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.4.1)
to [`0.4.3`](https://github.com/tower-rs/tower-http/releases/tag/tower-http-0.4.3).

### Added

- **cors:** Add support for private network preflights ([tower-rs/tower-http#373])
- **compression:** Implement `Default` for `DecompressionBody` ([tower-rs/tower-http#370])

### Changed

- **compression:** Update to async-compression 0.4 ([tower-rs/tower-http#371])

### Fixed

- **compression:** Override default brotli compression level 11 -> 4 ([tower-rs/tower-http#356])
- **trace:** Simplify dynamic tracing level application ([tower-rs/tower-http#380])
- **normalize_path:** Fix path normalization for preceding slashes ([tower-rs/tower-http#359])

[tower-rs/tower-http#356]: https://github.com/tower-rs/tower-http/pull/356
[tower-rs/tower-http#359]: https://github.com/tower-rs/tower-http/pull/359
[tower-rs/tower-http#370]: https://github.com/tower-rs/tower-http/pull/370
[tower-rs/tower-http#371]: https://github.com/tower-rs/tower-http/pull/371
[tower-rs/tower-http#373]: https://github.com/tower-rs/tower-http/pull/373
[tower-rs/tower-http#380]: https://github.com/tower-rs/tower-http/pull/380

## 0.1.1 (July 18, 2023)

- Improve, expand and fix documentation;

## 0.1.0 (July 17, 2023)

This is the initial release of `tower-async-http`, a fork of <https://github.com/tower-rs/tower-http> and makes use of `async traits`
([RFC-3185: Static async fn in traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html))
to simplify things and make it more easier to integrate async functions into middleware.

Notable differences:

- make use of `tower-async-service` instead of `tower-service`;
- make use of `tower-async-layer` instead of `tower-layer`,
