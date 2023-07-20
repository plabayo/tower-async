#![allow(unused_imports)]

use crate::compression_utils::CompressionLevel;
use crate::{
    compression_utils::{AsyncReadBody, BodyIntoStream, DecorateAsyncRead, WrapBody},
    BoxError,
};
#[cfg(feature = "decompression-br")]
use async_compression::tokio::bufread::BrotliDecoder;
#[cfg(feature = "decompression-gzip")]
use async_compression::tokio::bufread::GzipDecoder;
#[cfg(feature = "decompression-deflate")]
use async_compression::tokio::bufread::ZlibDecoder;
#[cfg(feature = "decompression-zstd")]
use async_compression::tokio::bufread::ZstdDecoder;
use bytes::{Buf, Bytes};
use futures_util::ready;
use http::HeaderMap;
use http_body::Body;
use pin_project_lite::pin_project;
use std::task::Context;
use std::{io, marker::PhantomData, pin::Pin, task::Poll};
use tokio_util::io::StreamReader;

pin_project! {
    /// Response body of [`RequestDecompression`] and [`Decompression`].
    ///
    /// [`RequestDecompression`]: super::RequestDecompression
    /// [`Decompression`]: super::Decompression
    pub struct DecompressionBody<B>
    where
        B: Body
    {
        #[pin]
        pub(crate) inner: BodyInner<B>,
    }
}

impl<B> Default for DecompressionBody<B>
where
    B: Body + Default,
{
    fn default() -> Self {
        Self {
            inner: BodyInner::Identity {
                inner: B::default(),
            },
        }
    }
}

impl<B> DecompressionBody<B>
where
    B: Body,
{
    pub(crate) fn new(inner: BodyInner<B>) -> Self {
        Self { inner }
    }
}

#[cfg(any(
    not(feature = "decompression-gzip"),
    not(feature = "decompression-deflate"),
    not(feature = "decompression-br"),
    not(feature = "decompression-zstd")
))]
pub(crate) enum Never {}

#[cfg(feature = "decompression-gzip")]
type GzipBody<B> = WrapBody<GzipDecoder<B>>;
#[cfg(not(feature = "decompression-gzip"))]
type GzipBody<B> = (Never, PhantomData<B>);

#[cfg(feature = "decompression-deflate")]
type DeflateBody<B> = WrapBody<ZlibDecoder<B>>;
#[cfg(not(feature = "decompression-deflate"))]
type DeflateBody<B> = (Never, PhantomData<B>);

#[cfg(feature = "decompression-br")]
type BrotliBody<B> = WrapBody<BrotliDecoder<B>>;
#[cfg(not(feature = "decompression-br"))]
type BrotliBody<B> = (Never, PhantomData<B>);

#[cfg(feature = "decompression-zstd")]
type ZstdBody<B> = WrapBody<ZstdDecoder<B>>;
#[cfg(not(feature = "decompression-zstd"))]
type ZstdBody<B> = (Never, PhantomData<B>);

pin_project! {
    #[project = BodyInnerProj]
    pub(crate) enum BodyInner<B>
    where
        B: Body,
    {
        Gzip {
            #[pin]
            inner: GzipBody<B>,
        },
        Deflate {
            #[pin]
            inner: DeflateBody<B>,
        },
        Brotli {
            #[pin]
            inner: BrotliBody<B>,
        },
        Zstd {
            #[pin]
            inner: ZstdBody<B>,
        },
        Identity {
            #[pin]
            inner: B,
        },
    }
}

impl<B: Body> BodyInner<B> {
    #[cfg(feature = "decompression-gzip")]
    pub(crate) fn gzip(inner: WrapBody<GzipDecoder<B>>) -> Self {
        Self::Gzip { inner }
    }

    #[cfg(feature = "decompression-deflate")]
    pub(crate) fn deflate(inner: WrapBody<ZlibDecoder<B>>) -> Self {
        Self::Deflate { inner }
    }

    #[cfg(feature = "decompression-br")]
    pub(crate) fn brotli(inner: WrapBody<BrotliDecoder<B>>) -> Self {
        Self::Brotli { inner }
    }

    #[cfg(feature = "decompression-zstd")]
    pub(crate) fn zstd(inner: WrapBody<ZstdDecoder<B>>) -> Self {
        Self::Zstd { inner }
    }

    pub(crate) fn identity(inner: B) -> Self {
        Self::Identity { inner }
    }
}

impl<B> Body for DecompressionBody<B>
where
    B: Body,
    B::Error: Into<BoxError>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.project().inner.project() {
            #[cfg(feature = "decompression-gzip")]
            BodyInnerProj::Gzip { inner } => inner.poll_data(cx),
            #[cfg(feature = "decompression-deflate")]
            BodyInnerProj::Deflate { inner } => inner.poll_data(cx),
            #[cfg(feature = "decompression-br")]
            BodyInnerProj::Brotli { inner } => inner.poll_data(cx),
            #[cfg(feature = "decompression-zstd")]
            BodyInnerProj::Zstd { inner } => inner.poll_data(cx),
            BodyInnerProj::Identity { inner } => match ready!(inner.poll_data(cx)) {
                Some(Ok(mut buf)) => {
                    let bytes = buf.copy_to_bytes(buf.remaining());
                    Poll::Ready(Some(Ok(bytes)))
                }
                Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
                None => Poll::Ready(None),
            },

            #[cfg(not(feature = "decompression-gzip"))]
            BodyInnerProj::Gzip { inner } => match inner.0 {},
            #[cfg(not(feature = "decompression-deflate"))]
            BodyInnerProj::Deflate { inner } => match inner.0 {},
            #[cfg(not(feature = "decompression-br"))]
            BodyInnerProj::Brotli { inner } => match inner.0 {},
            #[cfg(not(feature = "decompression-zstd"))]
            BodyInnerProj::Zstd { inner } => match inner.0 {},
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        match self.project().inner.project() {
            #[cfg(feature = "decompression-gzip")]
            BodyInnerProj::Gzip { inner } => inner.poll_trailers(cx),
            #[cfg(feature = "decompression-deflate")]
            BodyInnerProj::Deflate { inner } => inner.poll_trailers(cx),
            #[cfg(feature = "decompression-br")]
            BodyInnerProj::Brotli { inner } => inner.poll_trailers(cx),
            #[cfg(feature = "decompression-zstd")]
            BodyInnerProj::Zstd { inner } => inner.poll_trailers(cx),
            BodyInnerProj::Identity { inner } => inner.poll_trailers(cx).map_err(Into::into),

            #[cfg(not(feature = "decompression-gzip"))]
            BodyInnerProj::Gzip { inner } => match inner.0 {},
            #[cfg(not(feature = "decompression-deflate"))]
            BodyInnerProj::Deflate { inner } => match inner.0 {},
            #[cfg(not(feature = "decompression-br"))]
            BodyInnerProj::Brotli { inner } => match inner.0 {},
            #[cfg(not(feature = "decompression-zstd"))]
            BodyInnerProj::Zstd { inner } => match inner.0 {},
        }
    }
}

#[cfg(feature = "decompression-gzip")]
impl<B> DecorateAsyncRead for GzipDecoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = GzipDecoder<Self::Input>;

    fn apply(input: Self::Input, _quality: CompressionLevel) -> Self::Output {
        let mut decoder = GzipDecoder::new(input);
        decoder.multiple_members(true);
        decoder
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
}

#[cfg(feature = "decompression-deflate")]
impl<B> DecorateAsyncRead for ZlibDecoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = ZlibDecoder<Self::Input>;

    fn apply(input: Self::Input, _quality: CompressionLevel) -> Self::Output {
        ZlibDecoder::new(input)
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
}

#[cfg(feature = "decompression-br")]
impl<B> DecorateAsyncRead for BrotliDecoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = BrotliDecoder<Self::Input>;

    fn apply(input: Self::Input, _quality: CompressionLevel) -> Self::Output {
        BrotliDecoder::new(input)
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
}

#[cfg(feature = "decompression-zstd")]
impl<B> DecorateAsyncRead for ZstdDecoder<B>
where
    B: Body,
{
    type Input = AsyncReadBody<B>;
    type Output = ZstdDecoder<Self::Input>;

    fn apply(input: Self::Input, _quality: CompressionLevel) -> Self::Output {
        ZstdDecoder::new(input)
    }

    fn get_pin_mut(pinned: Pin<&mut Self::Output>) -> Pin<&mut Self::Input> {
        pinned.get_pin_mut()
    }
}
