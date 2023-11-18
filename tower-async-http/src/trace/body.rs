use super::{OnBodyChunk, OnEos, OnFailure};
use crate::classify::ClassifyEos;
use futures_core::ready;
use http_body::{Body, Frame};
use pin_project_lite::pin_project;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use tracing::Span;

pin_project! {
    /// Response body for [`Trace`].
    ///
    /// [`Trace`]: super::Trace
    pub struct ResponseBody<B, C, OnBodyChunk, OnEos, OnFailure> {
        #[pin]
        pub(crate) inner: B,
        pub(crate) classify_eos: Option<C>,
        pub(crate) on_eos: Option<(OnEos, Instant)>,
        pub(crate) on_body_chunk: OnBodyChunk,
        pub(crate) on_failure: Option<OnFailure>,
        pub(crate) start: Instant,
        pub(crate) span: Span,
    }
}

impl<B, C, OnBodyChunkT, OnEosT, OnFailureT> Body
    for ResponseBody<B, C, OnBodyChunkT, OnEosT, OnFailureT>
where
    B: Body,
    B::Error: fmt::Display + 'static,
    C: ClassifyEos,
    OnEosT: OnEos,
    OnBodyChunkT: OnBodyChunk<B::Data>,
    OnFailureT: OnFailure<C::FailureClass>,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let _guard = this.span.enter();

        let result = if let Some(result) = ready!(this.inner.poll_frame(cx)) {
            result
        } else {
            return Poll::Ready(None);
        };

        let latency = this.start.elapsed();
        *this.start = Instant::now();

        match &result {
            Ok(frame) => {
                if let Some(data) = frame.data_ref() {
                    this.on_body_chunk.on_body_chunk(data, latency, this.span);
                }
            }
            Err(err) => {
                if let Some((classify_eos, on_failure)) =
                    this.classify_eos.take().zip(this.on_failure.take())
                {
                    let failure_class = classify_eos.classify_error(err);
                    on_failure.on_failure(failure_class, latency, this.span);
                }
            }
        }

        Poll::Ready(Some(result))
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

impl<B: Default, C: Default, OnBodyChunk: Default, OnEos: Default, OnFailure: Default> Default
    for ResponseBody<B, C, OnBodyChunk, OnEos, OnFailure>
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            classify_eos: Default::default(),
            on_eos: Default::default(),
            on_body_chunk: Default::default(),
            on_failure: Default::default(),
            start: Instant::now(),
            span: Span::current(),
        }
    }
}
