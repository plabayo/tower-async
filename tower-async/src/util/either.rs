//! Contains [`Either`] and related types and functions.
//!
//! See [`Either`] documentation for more details.

use tower_async_layer::Layer;
use tower_async_service::Service;

/// Combine two different service types into a single type.
///
/// Both services must be of the same request, response, and error types.
/// [`Either`] is useful for handling conditional branching in service middleware
/// to different inner service types.
#[derive(Clone, Copy, Debug)]
pub enum Either<A, B> {
    #[allow(missing_docs)]
    Left(A),
    #[allow(missing_docs)]
    Right(B),
}

impl<A, B, Request> Service<Request> for Either<A, B>
where
    A: Service<Request>,
    B: Service<Request, Response = A::Response, Error = A::Error>,
{
    type Response = A::Response;
    type Error = A::Error;

    async fn call(&mut self, request: Request) -> Result<Self::Response, Self::Error> {
        match self {
            Either::Left(service) => service.call(request).await,
            Either::Right(service) => service.call(request).await,
        }
    }
}

impl<S, A, B> Layer<S> for Either<A, B>
where
    A: Layer<S>,
    B: Layer<S>,
{
    type Service = Either<A::Service, B::Service>;

    fn layer(&self, inner: S) -> Self::Service {
        match self {
            Either::Left(layer) => Either::Left(layer.layer(inner)),
            Either::Right(layer) => Either::Right(layer.layer(inner)),
        }
    }
}
