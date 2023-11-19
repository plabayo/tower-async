# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0 (November 20, 2023)

- Adapt to new `tower_async::Service` contract:
  - `call` takes now `&self` instead of `&mut self`;
  - `call` returns `impl Future` instead of declared as `async fn`;

## 0.1.1 (July 18, 2023)

- Improve, expand and fix documentation;

## 0.1.0 (July 17, 2023)

This is the initial release of `tower-async-test`, a fork of <https://github.com/tower-rs/tower> and makes use of `async traits`
([RFC-3185: Static async fn in traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html))
to simplify things and make it more easier to integrate async functions into middleware.

This library is very different however then `tower-test`, as we have a lot less things to validate,
thanks to the simplification of the `tower-async`'s Service contract. The approach in how we aid in writing
layer tests is also different, where we make use of a black box builder approach, instead
of the macro util approach taken by `tower-test`.
