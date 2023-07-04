/// Service returned by [crate::ClassicServiceExt::into_classic].
#[derive(Debug)]
pub struct ClassicServiceWrapper<S> {
    inner: Option<S>,
}

impl<S> ClassicServiceWrapper<S> {
    /// Create a new [ClassicServiceWrapper] wrapping `inner`.
    pub fn new(inner: S) -> Self {
        Self { inner: Some(inner) }
    }
}

impl<S> Clone for ClassicServiceWrapper<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S, Request> tower_service::Service<Request> for ClassicServiceWrapper<S>
where
    S: tower_async_service::Service<Request, call(): Send> + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = ClassicServiceError<S::Error>;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(if self.inner.is_some() {
            Ok(())
        } else {
            Err(ClassicServiceError::ServiceConsumed)
        })
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let service = self.inner.take().expect("service must be present");

        let future = async move {
            let mut service = service;

            let future = service.call(request);
            future.await.map_err(ClassicServiceError::ServiceError)
        };

        Box::pin(future)
    }
}

/// Error returned by [ClassicServiceWrapper].
#[derive(Debug)]
pub enum ClassicServiceError<E> {
    /// Error to indicate that the service has already been consumed,
    /// and cannot be used again.
    ServiceConsumed,
    /// Error returned by the wrapped service.
    ServiceError(E),
}

impl<E> std::fmt::Display for ClassicServiceError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassicServiceError::ServiceConsumed => write!(f, "service consumed"),
            ClassicServiceError::ServiceError(e) => write!(f, "service error: {}", e),
        }
    }
}

impl<E> std::error::Error for ClassicServiceError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClassicServiceError::ServiceConsumed => None,
            ClassicServiceError::ServiceError(e) => Some(e),
        }
    }
}
