use super::layer::RequestDecompressionLayer;
use crate::compression_utils::CompressionLevel;
use crate::{
    compression_utils::AcceptEncoding, decompression::body::BodyInner,
    decompression::DecompressionBody, BoxError,
};
use bytes::Buf;
use http::{header, HeaderValue, Request, Response, StatusCode};
use http_body::Body;
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Empty};
use tower_async_service::Service;

#[cfg(any(
    feature = "decompression-gzip",
    feature = "decompression-deflate",
    feature = "decompression-br",
    feature = "decompression-zstd",
))]
use crate::content_encoding::SupportedEncodings;

/// Decompresses request bodies and calls its underlying service.
///
/// Transparently decompresses request bodies based on the `Content-Encoding` header.
/// When the encoding in the `Content-Encoding` header is not accepted an `Unsupported Media Type`
/// status code will be returned with the accepted encodings in the `Accept-Encoding` header.
///
/// Enabling pass-through of unaccepted encodings will not return an `Unsupported Media Type` but
/// will call the underlying service with the unmodified request if the encoding is not supported.
/// This is disabled by default.
///
/// See the [module docs](crate::decompression) for more details.
#[derive(Debug, Clone)]
pub struct RequestDecompression<S> {
    pub(super) inner: S,
    pub(super) accept: AcceptEncoding,
    pub(super) pass_through_unaccepted: bool,
}

impl<S, ReqBody, ResBody, D> Service<Request<ReqBody>> for RequestDecompression<S>
where
    S: Service<Request<DecompressionBody<ReqBody>>, Response = Response<ResBody>>,
    ReqBody: Body,
    ResBody: Body<Data = D> + Send + 'static,
    S::Error: Into<BoxError>,
    <ResBody as Body>::Error: Into<BoxError>,
    D: Buf + 'static,
{
    type Response = Response<UnsyncBoxBody<D, BoxError>>;
    type Error = BoxError;

    async fn call(&self, req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        let (mut parts, body) = req.into_parts();

        let body =
            if let header::Entry::Occupied(entry) = parts.headers.entry(header::CONTENT_ENCODING) {
                match entry.get().as_bytes() {
                    #[cfg(feature = "decompression-gzip")]
                    b"gzip" if self.accept.gzip() => {
                        entry.remove();
                        parts.headers.remove(header::CONTENT_LENGTH);
                        BodyInner::gzip(crate::compression_utils::WrapBody::new(
                            body,
                            CompressionLevel::default(),
                        ))
                    }
                    #[cfg(feature = "decompression-deflate")]
                    b"deflate" if self.accept.deflate() => {
                        entry.remove();
                        parts.headers.remove(header::CONTENT_LENGTH);
                        BodyInner::deflate(crate::compression_utils::WrapBody::new(
                            body,
                            CompressionLevel::default(),
                        ))
                    }
                    #[cfg(feature = "decompression-br")]
                    b"br" if self.accept.br() => {
                        entry.remove();
                        parts.headers.remove(header::CONTENT_LENGTH);
                        BodyInner::brotli(crate::compression_utils::WrapBody::new(
                            body,
                            CompressionLevel::default(),
                        ))
                    }
                    #[cfg(feature = "decompression-zstd")]
                    b"zstd" if self.accept.zstd() => {
                        entry.remove();
                        parts.headers.remove(header::CONTENT_LENGTH);
                        BodyInner::zstd(crate::compression_utils::WrapBody::new(
                            body,
                            CompressionLevel::default(),
                        ))
                    }
                    b"identity" => BodyInner::identity(body),
                    _ if self.pass_through_unaccepted => BodyInner::identity(body),
                    _ => return unsupported_encoding(self.accept).await,
                }
            } else {
                BodyInner::identity(body)
            };
        let body = DecompressionBody::new(body);
        let req = Request::from_parts(parts, body);
        self.inner
            .call(req)
            .await
            .map(|res| res.map(|body| body.map_err(Into::into).boxed_unsync()))
            .map_err(Into::into)
    }
}

async fn unsupported_encoding<D>(
    accept: AcceptEncoding,
) -> Result<Response<UnsyncBoxBody<D, BoxError>>, BoxError>
where
    D: Buf + 'static,
{
    let res = Response::builder()
        .header(
            header::ACCEPT_ENCODING,
            accept
                .to_header_value()
                .unwrap_or(HeaderValue::from_static("identity")),
        )
        .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
        .body(Empty::new().map_err(Into::into).boxed_unsync())
        .unwrap();
    Ok(res)
}

impl<S> RequestDecompression<S> {
    /// Creates a new `RequestDecompression` wrapping the `service`.
    pub fn new(service: S) -> Self {
        Self {
            inner: service,
            accept: AcceptEncoding::default(),
            pass_through_unaccepted: false,
        }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `RequestDecompression` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer() -> RequestDecompressionLayer {
        RequestDecompressionLayer::new()
    }

    /// Passes through the request even when the encoding is not supported.
    ///
    /// By default pass-through is disabled.
    pub fn pass_through_unaccepted(mut self, enabled: bool) -> Self {
        self.pass_through_unaccepted = enabled;
        self
    }

    /// Sets whether to support gzip encoding.
    #[cfg(feature = "decompression-gzip")]
    pub fn gzip(mut self, enable: bool) -> Self {
        self.accept.set_gzip(enable);
        self
    }

    /// Sets whether to support Deflate encoding.
    #[cfg(feature = "decompression-deflate")]
    pub fn deflate(mut self, enable: bool) -> Self {
        self.accept.set_deflate(enable);
        self
    }

    /// Sets whether to support Brotli encoding.
    #[cfg(feature = "decompression-br")]
    pub fn br(mut self, enable: bool) -> Self {
        self.accept.set_br(enable);
        self
    }

    /// Sets whether to support Zstd encoding.
    #[cfg(feature = "decompression-zstd")]
    pub fn zstd(mut self, enable: bool) -> Self {
        self.accept.set_zstd(enable);
        self
    }

    /// Disables support for gzip encoding.
    ///
    /// This method is available even if the `gzip` crate feature is disabled.
    pub fn no_gzip(mut self) -> Self {
        self.accept.set_gzip(false);
        self
    }

    /// Disables support for Deflate encoding.
    ///
    /// This method is available even if the `deflate` crate feature is disabled.
    pub fn no_deflate(mut self) -> Self {
        self.accept.set_deflate(false);
        self
    }

    /// Disables support for Brotli encoding.
    ///
    /// This method is available even if the `br` crate feature is disabled.
    pub fn no_br(mut self) -> Self {
        self.accept.set_br(false);
        self
    }

    /// Disables support for Zstd encoding.
    ///
    /// This method is available even if the `zstd` crate feature is disabled.
    pub fn no_zstd(mut self) -> Self {
        self.accept.set_zstd(false);
        self
    }
}
