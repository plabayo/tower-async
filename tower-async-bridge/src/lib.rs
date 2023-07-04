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
mod async_wrapper;

mod classic_layer;
mod classic_service;
mod classic_wrapper;

pub use async_service::AsyncServiceExt;
pub use async_wrapper::AsyncServiceWrapper;

pub use classic_layer::{ClassicLayer, ClassicLayerExt};
pub use classic_service::ClassicServiceExt;
pub use classic_wrapper::{ClassicServiceError, ClassicServiceWrapper};
