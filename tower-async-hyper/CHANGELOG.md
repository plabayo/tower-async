# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0 (November 20, 2023)

- Initial release. Bridges `hyper` (v1) with `tower-async`.
- Compatible with new `tower_async::Service` contract:
  - `call` takes now `&self` instead of `&mut self`;
  - `call` returns `impl Future` instead of declared as `async fn`;
