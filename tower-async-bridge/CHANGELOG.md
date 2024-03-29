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

This is the initial release of `tower-async-bridge`, and is meant to bridge services and/or layers
from the <https://github.com/tower-rs/tower> ecosystem with those from the `tower-async` ecosystem
(meaning written using technology of this repository).

The bridging can go in both directions, but does require the `into_async` feature to be enabled
in case you want to bridge classic (<https://github.com/tower-rs/tower>) services and/or layers
into their `async (static fn) trait` counterparts.
