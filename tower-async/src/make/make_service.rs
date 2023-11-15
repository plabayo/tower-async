//! Contains [`MakeService`] which is a trait alias for a [`Service`] of [`Service`]s.

use crate::sealed::Sealed;
use std::fmt;
use std::marker::PhantomData;
use tower_async_service::Service;

pub(crate) mod shared;

/// Creates new [`Service`] values.
///
/// Acts as a service factory. This is useful for cases where new [`Service`]
/// values must be produced. One case is a TCP server listener. The listener
/// accepts new TCP streams, obtains a new [`Service`] value using the
/// [`MakeService`] trait, and uses that new [`Service`] value to process inbound
/// requests on that new TCP stream.
///
/// This is essentially a trait alias for a [`Service`] of [`Service`]s.
pub trait MakeService<Target, Request>: Sealed<(Target, Request)> {
    /// Responses given by the service
    type Response;

    /// Errors produced by the service
    type Error;

    /// The [`Service`] value created by this factory
    type Service: Service<Request, Response = Self::Response, Error = Self::Error>;

    /// Errors produced while building a service.
    type MakeError;

    /// Create and return a new service value asynchronously.
    fn make_service(
        &self,
        target: Target,
    ) -> impl std::future::Future<Output = Result<Self::Service, Self::MakeError>>;

    /// Consume this [`MakeService`] and convert it into a [`Service`].
    ///
    /// # Example
    /// ```
    /// use std::convert::Infallible;
    /// use tower_async::Service;
    /// use tower_async::make::MakeService;
    /// use tower_async::service_fn;
    ///
    /// # fn main() {
    /// # async {
    /// // A `MakeService`
    /// let make_service = service_fn(|make_req: ()| async {
    ///     Ok::<_, Infallible>(service_fn(|req: String| async {
    ///         Ok::<_, Infallible>(req)
    ///     }))
    /// });
    ///
    /// // Convert the `MakeService` into a `Service`
    /// let mut svc = make_service.into_service();
    ///
    /// // Make a new service
    /// let mut new_svc = svc.call(()).await.unwrap();
    ///
    /// // Call the service
    /// let res = new_svc.call("foo".to_string()).await.unwrap();
    /// # };
    /// # }
    /// ```
    fn into_service(self) -> IntoService<Self, Request>
    where
        Self: Sized,
    {
        IntoService {
            make: self,
            _marker: PhantomData,
        }
    }

    /// Convert this [`MakeService`] into a [`Service`] without consuming the original [`MakeService`].
    ///
    /// # Example
    /// ```
    /// use std::convert::Infallible;
    /// use tower_async::Service;
    /// use tower_async::make::MakeService;
    /// use tower_async::service_fn;
    ///
    /// # fn main() {
    /// # async {
    /// // A `MakeService`
    /// let mut make_service = service_fn(|make_req: ()| async {
    ///     Ok::<_, Infallible>(service_fn(|req: String| async {
    ///         Ok::<_, Infallible>(req)
    ///     }))
    /// });
    ///
    /// // Convert the `MakeService` into a `Service`
    /// let mut svc = make_service.as_service();
    ///
    /// // Make a new service
    /// let mut new_svc = svc.call(()).await.unwrap();
    ///
    /// // Call the service
    /// let res = new_svc.call("foo".to_string()).await.unwrap();
    ///
    /// // The original `MakeService` is still accessible
    /// let new_svc = make_service.make_service(()).await.unwrap();
    /// # };
    /// # }
    /// ```
    fn as_service(&self) -> AsService<Self, Request>
    where
        Self: Sized,
    {
        AsService {
            make: self,
            _marker: PhantomData,
        }
    }
}

impl<M, S, Target, Request> Sealed<(Target, Request)> for M
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
}

impl<M, S, Target, Request> MakeService<Target, Request> for M
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Service = S;
    type MakeError = M::Error;

    async fn make_service(&self, target: Target) -> Result<Self::Service, Self::MakeError> {
        Service::call(self, target).await
    }
}

/// Service returned by [`MakeService::into_service`][into].
///
/// See the documentation on [`into_service`][into] for details.
///
/// [into]: MakeService::into_service
pub struct IntoService<M, Request> {
    make: M,
    _marker: PhantomData<Request>,
}

impl<M, Request> Clone for IntoService<M, Request>
where
    M: Clone,
{
    fn clone(&self) -> Self {
        Self {
            make: self.make.clone(),
            _marker: PhantomData,
        }
    }
}

impl<M, Request> fmt::Debug for IntoService<M, Request>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoService")
            .field("make", &self.make)
            .finish()
    }
}

impl<M, S, Target, Request> Service<Target> for IntoService<M, Request>
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
    type Response = M::Response;
    type Error = M::Error;

    #[inline]
    async fn call(&self, target: Target) -> Result<Self::Response, Self::Error> {
        self.make.make_service(target).await
    }
}

/// Service returned by [`MakeService::as_service`][as].
///
/// See the documentation on [`as_service`][as] for details.
///
/// [as]: MakeService::as_service
pub struct AsService<'a, M, Request> {
    make: &'a M,
    _marker: PhantomData<Request>,
}

impl<M, Request> fmt::Debug for AsService<'_, M, Request>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsService")
            .field("make", &self.make)
            .finish()
    }
}

impl<M, S, Target, Request> Service<Target> for AsService<'_, M, Request>
where
    M: Service<Target, Response = S>,
    S: Service<Request>,
{
    type Response = M::Response;
    type Error = M::Error;

    #[inline]
    async fn call(&self, target: Target) -> Result<Self::Response, Self::Error> {
        self.make.make_service(target).await
    }
}
