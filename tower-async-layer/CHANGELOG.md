# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.0 (July 17, 2023)

This is the initial release of `tower-async-layer`, a fork of <https://github.com/tower-rs/tower> and makes use of `async traits`
([RFC-3185: Static async fn in traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html))
to simplify things and make it more easier to integrate async functions into middleware.
