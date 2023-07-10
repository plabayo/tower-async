#![allow(dead_code)]

use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio_stream::Stream;

pub(crate) fn trace_init() -> tracing::subscriber::DefaultGuard {
    let subscriber = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::TRACE)
        .with_thread_names(true)
        .finish();
    tracing::subscriber::set_default(subscriber)
}

pin_project_lite::pin_project! {
    #[derive(Clone, Debug)]
    pub struct IntoStream<S> {
        #[pin]
        inner: S
    }
}

impl<S> IntoStream<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<I> Stream for IntoStream<mpsc::Receiver<I>> {
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_recv(cx)
    }
}

impl<I> Stream for IntoStream<mpsc::UnboundedReceiver<I>> {
    type Item = I;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_recv(cx)
    }
}

#[derive(Clone, Debug)]
pub struct AssertSpanSvc {
    span: tracing::Span,
    polled: bool,
}

pub struct AssertSpanError(String);

impl fmt::Debug for AssertSpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Display for AssertSpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for AssertSpanError {}

impl AssertSpanSvc {
    pub fn new(span: tracing::Span) -> Self {
        Self {
            span,
            polled: false,
        }
    }

    fn check(&self, func: &str) -> Result<(), AssertSpanError> {
        let current_span = tracing::Span::current();
        tracing::debug!(?current_span, ?self.span, %func);
        if current_span == self.span {
            return Ok(());
        }

        Err(AssertSpanError(format!(
            "{} called outside expected span\n expected: {:?}\n  current: {:?}",
            func, self.span, current_span
        )))
    }
}
