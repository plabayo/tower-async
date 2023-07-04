#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![forbid(unsafe_code)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(associated_type_bounds)]
#![feature(return_type_notation)]
// `rustdoc::broken_intra_doc_links` is checked on CI

//! Tower Async Bridge traits and extensions.
//!
//! You can make use of this crate in order to:
//!
//! - Turn a [`tower::Service`] into a [`tower_async::Service`];
//! - Turn a [`tower_async::Service`] into a [`tower::Service`];
//! - Use a [`tower_async::Layer`] within a [`tower`] environment (e.g. [`tower::ServiceBuilder`]);
//! - Use a [`tower::Layer`] within a [`tower_async`] environment (e.g. [`tower_async::ServiceBuilder`]);
//!
//! Please check the crate's unit tests and examples to see specifically how to use the crate in order to achieve this.
//!
//! Furthermore we also urge you to only use this kind of approach for transition purposes and not as a permanent way of life.
//! Best in our opinion is to use one or the other and not to combine the two. But if you do absolutely must
//! use one combined with the other, this crate should allow you to do exactly that.
//!
//! [`tower`]: https://docs.rs/tower/*/t
//! [`tower::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
//! [`tower::ServiceBuilder`]: https://docs.rs/tower/*/tower/builder/struct.ServiceBuilder.html
//! [`tower::Layer`]: https://docs.rs/tower/*/tower/trait.Layer.html
//! [`tower_async`]: https://docs.rs/tower-async/*/tower_async
//! [`tower_async::Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
//! [`tower_async::ServiceBuilder`]: https://docs.rs/tower-async/*/tower_async/builder/struct.ServiceBuilder.html
//! [`tower_async::Layer`]: https://docs.rs/tower-async/*/tower_async/trait.Layer.html

mod async_layer;
mod async_service;
mod async_wrapper;

mod classic_layer;
mod classic_service;
mod classic_wrapper;

pub use async_layer::{AsyncLayer, AsyncLayerExt};
pub use async_service::AsyncServiceExt;
pub use async_wrapper::AsyncServiceWrapper;

pub use classic_layer::{ClassicLayer, ClassicLayerExt};
pub use classic_service::ClassicServiceExt;
pub use classic_wrapper::{ClassicServiceError, ClassicServiceWrapper};
