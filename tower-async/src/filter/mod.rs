//! Conditionally dispatch requests to the inner service based on the result of
//! a predicate.
//!
//! A predicate takes some request type and returns a `Result<Request, Error>`.
//! If the predicate returns [`Ok`], the inner service is called with the request
//! returned by the predicate &mdash; which may be the original request or a
//! modified one. If the predicate returns [`Err`], the request is rejected and
//! the inner service is not called.
//!
//! Predicates may either be synchronous (simple functions from a `Request` to
//! a [`Result`]) or asynchronous (functions returning [`Future`]s). Separate
//! traits, [`Predicate`] and [`AsyncPredicate`], represent these two types of
//! predicate. Note that when it is not necessary to await some other
//! asynchronous operation in the predicate, the synchronous predicate should be
//! preferred, as it introduces less overhead.
//!
//! The predicate traits are implemented for closures and function pointers.
//! However, users may also implement them for other types, such as when the
//! predicate requires some state carried between requests. For example,
//! [`Predicate`] could be implemented for a type that rejects a fixed set of
//! requests by checking if they are contained by a a [`HashSet`] or other
//! collection.
//!
//! [`Future`]: std::future::Future
//! [`HashSet`]: std::collections::HashSet
mod layer;
mod predicate;

pub use self::{
    layer::{AsyncFilterLayer, FilterLayer},
    predicate::{AsyncPredicate, Predicate},
};

use crate::BoxError;
use futures_util::TryFutureExt;
use tower_async_service::Service;

/// Conditionally dispatch requests to the inner service based on a [predicate].
///
/// [predicate]: Predicate
#[derive(Clone, Debug)]
pub struct Filter<T, U> {
    inner: T,
    predicate: U,
}

/// Conditionally dispatch requests to the inner service based on an
/// [asynchronous predicate].
///
/// [asynchronous predicate]: AsyncPredicate
#[derive(Clone, Debug)]
pub struct AsyncFilter<T, U> {
    inner: T,
    predicate: U,
}

// ==== impl Filter ====

impl<T, U> Filter<T, U> {
    /// Returns a new [`Filter`] service wrapping `inner`.
    pub fn new(inner: T, predicate: U) -> Self {
        Self { inner, predicate }
    }

    /// Returns a new [`Layer`] that wraps services with a [`Filter`] service
    /// with the given [`Predicate`].
    ///
    /// [`Layer`]: crate::Layer
    pub fn layer(predicate: U) -> FilterLayer<U> {
        FilterLayer::new(predicate)
    }

    /// Check a `Request` value against this filter's predicate.
    pub fn check<R>(&self, request: R) -> Result<U::Request, BoxError>
    where
        U: Predicate<R>,
    {
        self.predicate.check(request)
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, U, Request> Service<Request> for Filter<T, U>
where
    U: Predicate<Request>,
    T: Service<U::Request>,
    T::Error: Into<BoxError>,
{
    type Response = T::Response;
    type Error = BoxError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        match self.predicate.check(request) {
            Ok(request) => self.inner.call(request).err_into().await,
            Err(e) => Err(e),
        }
    }
}

// ==== impl AsyncFilter ====

impl<T, U> AsyncFilter<T, U> {
    /// Returns a new [`AsyncFilter`] service wrapping `inner`.
    pub fn new(inner: T, predicate: U) -> Self {
        Self { inner, predicate }
    }

    /// Returns a new [`Layer`] that wraps services with an [`AsyncFilter`]
    /// service with the given [`AsyncPredicate`].
    ///
    /// [`Layer`]: crate::Layer
    pub fn layer(predicate: U) -> FilterLayer<U> {
        FilterLayer::new(predicate)
    }

    /// Check a `Request` value against this filter's predicate.
    pub async fn check<R>(&self, request: R) -> Result<U::Request, BoxError>
    where
        U: AsyncPredicate<R>,
    {
        self.predicate.check(request).await
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, U, Request> Service<Request> for AsyncFilter<T, U>
where
    U: AsyncPredicate<Request>,
    T: Service<U::Request> + Clone,
    T::Error: Into<BoxError>,
{
    type Response = T::Response;
    type Error = BoxError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        match self.predicate.check(request).await {
            Ok(request) => self.inner.call(request).err_into().await,
            Err(e) => Err(e),
        }
    }
}
