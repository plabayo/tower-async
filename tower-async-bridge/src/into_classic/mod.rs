mod classic_service;
mod classic_wrapper;

pub use classic_service::ClassicServiceExt;
pub use classic_wrapper::ClassicServiceWrapper;

#[cfg(feature = "into_async")]
mod classic_layer;
#[cfg(feature = "into_async")]
pub use classic_layer::{ClassicLayer, ClassicLayerExt};
