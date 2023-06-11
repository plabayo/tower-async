//! Spawn mock services onto a mock task.

use tokio_test::task;
use tower_async_service::Service;

/// Service spawned on a mock task
#[derive(Debug)]
pub struct Spawn<T> {
    inner: T,
    task: task::Spawn<()>,
}

impl<T> Spawn<T> {
    /// Create a new spawn.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            task: task::spawn(()),
        }
    }

    /// Check if this service has been woken up.
    pub fn is_woken(&self) -> bool {
        self.task.is_woken()
    }

    /// Get how many futurs are holding onto the waker.
    pub fn waker_ref_count(&self) -> usize {
        self.task.waker_ref_count()
    }

    /// Call the inner Service.
    pub async fn call<Request>(&mut self, req: Request) -> Result<T::Response, T::Error>
    where
        T: Service<Request>,
    {
        self.inner.call(req).await
    }

    /// Get the inner service.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Get a reference to the inner service.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner service.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Clone> Clone for Spawn<T> {
    fn clone(&self) -> Self {
        Spawn::new(self.inner.clone())
    }
}
