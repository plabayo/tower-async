pub(crate) mod erase;
mod layer;
mod layer_clone;
mod sync;

#[allow(unreachable_pub)] // https://github.com/rust-lang/rust/issues/57411
pub use self::{layer::BoxLayer, layer_clone::BoxCloneServiceLayer, sync::BoxService};
