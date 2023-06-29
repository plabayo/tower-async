//! Middleware for setting timeouts on requests and responses.

mod service;

pub use service::{Timeout, TimeoutLayer};
