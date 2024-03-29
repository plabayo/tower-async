use std::sync::Arc;

use async_lock::Mutex;

/// A wrapper around a [`tower_service::Service`] that implements
/// [`tower_async_service::Service`].
///
/// [`tower_service::Service`]: https://docs.rs/tower/*/tower/trait.Service.html
/// [`tower_async_service::Service`]: https://docs.rs/tower-async/*/tower_async/trait.Service.html
#[derive(Debug)]
pub struct AsyncServiceWrapper<S> {
    inner: Arc<Mutex<S>>,
}

impl<S> Clone for AsyncServiceWrapper<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S> AsyncServiceWrapper<S> {
    /// Create a new [`AsyncServiceWrapper`] wrapping `inner`.
    pub fn new(inner: S) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl<S, Request> tower_async_service::Service<Request> for AsyncServiceWrapper<S>
where
    S: tower_service::Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;

    #[inline]
    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        use tower::ServiceExt;
        self.inner.lock().await.ready().await?.call(request).await
    }
}
