use crate::sealed::Sealed;
use tokio::io::{AsyncRead, AsyncWrite};
use tower_async_service::Service;

/// The [`MakeConnection`] trait is used to create transports.
///
/// The goal of this service is to allow composable methods for creating
/// `AsyncRead + AsyncWrite` transports. This could mean creating a TLS
/// based connection or using some other method to authenticate the connection.
pub trait MakeConnection<Target>: Sealed<(Target,)> {
    /// The transport provided by this service
    type Connection: AsyncRead + AsyncWrite;

    /// Errors produced by the connecting service
    type Error;

    /// Connect and return a transport asynchronously
    fn make_connection(
        &self,
        target: Target,
    ) -> impl std::future::Future<Output = Result<Self::Connection, Self::Error>>;
}

impl<S, Target> Sealed<(Target,)> for S where S: Service<Target> {}

impl<C, Target> MakeConnection<Target> for C
where
    C: Service<Target>,
    C::Response: AsyncRead + AsyncWrite,
{
    type Connection = C::Response;
    type Error = C::Error;

    async fn make_connection(&self, target: Target) -> Result<Self::Connection, Self::Error> {
        Service::call(self, target).await
    }
}
