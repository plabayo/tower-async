#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
// `rustdoc::broken_intra_doc_links` is checked on CI

//! Bridge traits and extensions.
//!
//! A bridge decorates an service and provides additional functionality.
//! It allows a class Tower Service to be used as an Async [`Service`].
//!
//! [`Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html

mod async_service;

pub use async_service::{AsyncService, AsyncServiceExt};
