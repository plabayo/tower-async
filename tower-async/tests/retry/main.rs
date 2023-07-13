#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![cfg(feature = "retry")]
#[path = "../support.rs"]
mod support;

use tower_async::retry::{Policy, RetryLayer};
use tower_async_test::Builder;

#[tokio::test(flavor = "current_thread")]
async fn retry_errors() {
    let _t = support::trace_init();

    Builder::new("hello")
        .send_error("retry me")
        .expect_request("hello")
        .send_response("world")
        .expect_request("hello")
        .test(RetryLayer::new(RetryErrors))
        .await
        .expect_response("world");
}

#[tokio::test(flavor = "current_thread")]
async fn retry_limit() {
    let _t = support::trace_init();

    Builder::new("hello")
        .send_error("retry 1")
        .expect_request("hello")
        .send_error("retry 2")
        .expect_request("hello")
        .send_error("retry 3")
        .expect_request("hello")
        .test(RetryLayer::new(Limit(2)))
        .await
        .expect_error("retry 3");
}

#[tokio::test(flavor = "current_thread")]
async fn retry_error_inspection() {
    let _t = support::trace_init();

    Builder::new("hello")
        .send_error("retry 1")
        .expect_request("hello")
        .send_error("reject")
        .expect_request("hello")
        .test(RetryLayer::new(UnlessErr("reject")))
        .await
        .expect_error("reject");
}

#[tokio::test(flavor = "current_thread")]
async fn retry_cannot_clone_request() {
    let _t = support::trace_init();

    Builder::new("hello")
        .send_error("retry 1")
        .expect_request("hello")
        .test(RetryLayer::new(CannotClone))
        .await
        .expect_error("retry 1");
}

#[tokio::test(flavor = "current_thread")]
async fn success_with_cannot_clone() {
    let _t = support::trace_init();

    // Even though the request couldn't be cloned, if the first request succeeds,
    // it should succeed overall.
    Builder::new("hello")
        .send_response("world")
        .test(RetryLayer::new(CannotClone))
        .await
        .expect_response("world");
}

#[tokio::test(flavor = "current_thread")]
async fn retry_mutating_policy() {
    let _t = support::trace_init();

    Builder::new("hello")
        .send_error("retry 1")
        .expect_request("hello")
        .send_response("world")
        .expect_request("retrying")
        .send_response("world")
        .test(RetryLayer::new(MutatingPolicy { remaining: 2 }))
        .await
        .expect_error("out of retries");
}

#[derive(Debug, Clone, PartialEq)]
struct RetryErrors;

impl<Req, Res, Error> Policy<Req, Res, Error> for RetryErrors
where
    Req: Copy,
{
    async fn retry(&mut self, _: &mut Req, result: &mut Result<Res, Error>) -> bool {
        result.is_err()
    }

    fn clone_request(&mut self, req: &Req) -> Option<Req> {
        Some(*req)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Limit(usize);

impl<Req, Res, Error> Policy<Req, Res, Error> for Limit
where
    Req: Copy,
{
    async fn retry(&mut self, _: &mut Req, result: &mut Result<Res, Error>) -> bool {
        if result.is_err() && self.0 > 0 {
            self.0 -= 1;
            true
        } else {
            false
        }
    }

    fn clone_request(&mut self, req: &Req) -> Option<Req> {
        Some(*req)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct UnlessErr<Error>(Error);

impl<Req, Res, Error> Policy<Req, Res, Error> for UnlessErr<Error>
where
    Error: ToString,
    Req: Copy,
{
    async fn retry(&mut self, _: &mut Req, result: &mut Result<Res, Error>) -> bool {
        result
            .as_ref()
            .err()
            .and_then(|err| {
                if err.to_string() != self.0.to_string() {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
    }

    fn clone_request(&mut self, req: &Req) -> Option<Req> {
        Some(*req)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CannotClone;

impl<Req, Res, Error> Policy<Req, Res, Error> for CannotClone {
    async fn retry(&mut self, _: &mut Req, _: &mut Result<Res, Error>) -> bool {
        unreachable!("retry cannot be called since request isn't cloned");
    }

    fn clone_request(&mut self, _req: &Req) -> Option<Req> {
        None
    }
}

/// Test policy that changes the request to `retrying` during retries and the result to `"out of retries"`
/// when retries are exhausted.
#[derive(Debug, Clone, PartialEq)]
struct MutatingPolicy {
    remaining: usize,
}

impl<Res, Error> Policy<&'static str, Res, Error> for MutatingPolicy
where
    Error: From<&'static str>,
{
    async fn retry(&mut self, req: &mut &'static str, result: &mut Result<Res, Error>) -> bool {
        if self.remaining == 0 {
            *result = Err("out of retries".into());
            false
        } else {
            *req = "retrying";
            self.remaining -= 1;
            true
        }
    }

    fn clone_request(&mut self, req: &&'static str) -> Option<&'static str> {
        Some(*req)
    }
}
