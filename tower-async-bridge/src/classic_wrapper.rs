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
    Request: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, request: Request) -> Self::Future {
        let service = self.inner.take().expect("service must be present");

        let future = async move {
            let mut service = service;

            service.call(request).await
        };

        Box::pin(future)
    }
}
