use crate::BoxError;

/// Checks a request asynchronously.
pub trait AsyncPredicate<Request> {
    /// The type of requests returned by [`check`].
    ///
    /// This request is forwarded to the inner service if the predicate
    /// succeeds.
    ///
    /// [`check`]: crate::filter::AsyncPredicate::check
    type Request;

    /// Check whether the given request should be forwarded.
    ///
    /// If the future resolves with [`Ok`], the request is forwarded to the inner service.
    async fn check(&mut self, request: Request) -> Result<Self::Request, BoxError>;
}
/// Checks a request synchronously.
pub trait Predicate<Request> {
    /// The type of requests returned by [`check`].
    ///
    /// This request is forwarded to the inner service if the predicate
    /// succeeds.
    ///
    /// [`check`]: crate::filter::Predicate::check
    type Request;

    /// Check whether the given request should be forwarded.
    ///
    /// If the future resolves with [`Ok`], the request is forwarded to the inner service.
    fn check(&mut self, request: Request) -> Result<Self::Request, BoxError>;
}

impl<F, T, U, R, E> AsyncPredicate<T> for F
where
    F: FnMut(T) -> U,
    E: Into<BoxError>,
{
    type Request = R;

    fn check(&mut self, request: T) -> Result<R, E> {
        use futures_util::TryFutureExt;
        self(request).err_into()
    }
}

impl<F, T, R, E> Predicate<T> for F
where
    F: FnMut(T) -> Result<R, E>,
    E: Into<BoxError>,
{
    type Request = R;

    fn check(&mut self, request: T) -> Result<Self::Request, BoxError> {
        self(request).map_err(Into::into)
    }
}
