use super::{body::BodyInner, DecompressionBody, DecompressionLayer};
use crate::compression_utils::{AcceptEncoding, CompressionLevel, WrapBody};
use crate::content_encoding::SupportedEncodings;
use http::{
    header::{self, ACCEPT_ENCODING},
    Request, Response,
};
use http_body::Body;
use tower_async_service::Service;

/// Decompresses response bodies of the underlying service.
///
/// This adds the `Accept-Encoding` header to requests and transparently decompresses response
/// bodies based on the `Content-Encoding` header.
///
/// See the [module docs](crate::decompression) for more details.
#[derive(Debug, Clone)]
pub struct Decompression<S> {
    pub(crate) inner: S,
    pub(crate) accept: AcceptEncoding,
}

impl<S> Decompression<S> {
    /// Creates a new `Decompression` wrapping the `service`.
    pub fn new(service: S) -> Self {
        Self {
            inner: service,
            accept: AcceptEncoding::default(),
        }
    }

    define_inner_service_accessors!();

    /// Returns a new [`Layer`] that wraps services with a `Decompression` middleware.
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer() -> DecompressionLayer {
        DecompressionLayer::new()
    }

    /// Sets whether to request the gzip encoding.
    #[cfg(feature = "decompression-gzip")]
    pub fn gzip(mut self, enable: bool) -> Self {
        self.accept.set_gzip(enable);
        self
    }

    /// Sets whether to request the Deflate encoding.
    #[cfg(feature = "decompression-deflate")]
    pub fn deflate(mut self, enable: bool) -> Self {
        self.accept.set_deflate(enable);
        self
    }

    /// Sets whether to request the Brotli encoding.
    #[cfg(feature = "decompression-br")]
    pub fn br(mut self, enable: bool) -> Self {
        self.accept.set_br(enable);
        self
    }

    /// Sets whether to request the Zstd encoding.
    #[cfg(feature = "decompression-zstd")]
    pub fn zstd(mut self, enable: bool) -> Self {
        self.accept.set_zstd(enable);
        self
    }

    /// Disables the gzip encoding.
    ///
    /// This method is available even if the `gzip` crate feature is disabled.
    pub fn no_gzip(mut self) -> Self {
        self.accept.set_gzip(false);
        self
    }

    /// Disables the Deflate encoding.
    ///
    /// This method is available even if the `deflate` crate feature is disabled.
    pub fn no_deflate(mut self) -> Self {
        self.accept.set_deflate(false);
        self
    }

    /// Disables the Brotli encoding.
    ///
    /// This method is available even if the `br` crate feature is disabled.
    pub fn no_br(mut self) -> Self {
        self.accept.set_br(false);
        self
    }

    /// Disables the Zstd encoding.
    ///
    /// This method is available even if the `zstd` crate feature is disabled.
    pub fn no_zstd(mut self) -> Self {
        self.accept.set_zstd(false);
        self
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for Decompression<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Body,
{
    type Response = Response<DecompressionBody<ResBody>>;
    type Error = S::Error;

    fn call(&mut self, mut req: Request<ReqBody>) -> Result<Self::Response, Self::Error> {
        if let header::Entry::Vacant(entry) = req.headers_mut().entry(ACCEPT_ENCODING) {
            if let Some(accept) = self.accept.to_header_value() {
                entry.insert(accept);
            }
        }

        let res = self.inner.call(req)?;
        let (mut parts, body) = res.into_parts();

        let res =
            if let header::Entry::Occupied(entry) = parts.headers.entry(header::CONTENT_ENCODING) {
                let body = match entry.get().as_bytes() {
                    #[cfg(feature = "decompression-gzip")]
                    b"gzip" if self.accept.gzip() => DecompressionBody::new(BodyInner::gzip(
                        WrapBody::new(body, CompressionLevel::default()),
                    )),

                    #[cfg(feature = "decompression-deflate")]
                    b"deflate" if self.accept.deflate() => DecompressionBody::new(
                        BodyInner::deflate(WrapBody::new(body, CompressionLevel::default())),
                    ),

                    #[cfg(feature = "decompression-br")]
                    b"br" if self.accept.br() => DecompressionBody::new(BodyInner::brotli(
                        WrapBody::new(body, CompressionLevel::default()),
                    )),

                    #[cfg(feature = "decompression-zstd")]
                    b"zstd" if self.accept.zstd() => DecompressionBody::new(BodyInner::zstd(
                        WrapBody::new(body, CompressionLevel::default()),
                    )),

                    _ => {
                        return Ok(Response::from_parts(
                            parts,
                            DecompressionBody::new(BodyInner::identity(body)),
                        ))
                    }
                };

                entry.remove();
                parts.headers.remove(header::CONTENT_LENGTH);

                Response::from_parts(parts, body)
            } else {
                Response::from_parts(parts, DecompressionBody::new(BodyInner::identity(body)))
            };

        Ok(res)
    }
}
