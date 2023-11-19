//! Middleware that decompresses request and response bodies.
//!
//! # Examples
//!
//! #### Request
//! ```rust
//! use bytes::{Bytes, BytesMut};
//! use flate2::{write::GzEncoder, Compression};
//! use http::{header, HeaderValue, Request, Response};
//! use http_body_util::{Full, BodyExt};
//! use std::{error::Error, io::Write};
//! use tower_async::{Service, ServiceBuilder, service_fn, ServiceExt};
//! use tower_async_http::{BoxError, decompression::{DecompressionBody, RequestDecompressionLayer}};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), BoxError> {
//! // A request encoded with gzip coming from some HTTP client.
//! let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
//! encoder.write_all(b"Hello?")?;
//! let request = Request::builder()
//!     .header(header::CONTENT_ENCODING, "gzip")
//!     .body(Full::from(encoder.finish()?))?;
//!
//! // Our HTTP server
//! let mut server = ServiceBuilder::new()
//!     // Automatically decompress request bodies.
//!     .layer(RequestDecompressionLayer::new())
//!     .service(service_fn(handler));
//!
//! // Send the request, with the gzip encoded body, to our server.
//! let _response = server.call(request).await?;
//!
//! // Handler receives request whose body is decoded when read
//! async fn handler(mut req: Request<DecompressionBody<Full<Bytes>>>) -> Result<Response<Full<Bytes>>, BoxError>{
//!     let data = req.into_body().collect().await?.to_bytes();
//!     assert_eq!(&data[..], b"Hello?");
//!     Ok(Response::new(Full::from("Hello, World!")))
//! }
//! # Ok(())
//! # }
//! ```
//!
//! #### Response
//! ```rust
//! use bytes::{Bytes, BytesMut};
//! use http::{Request, Response};
//! use http_body_util::{BodyExt, Full};
//! use std::convert::Infallible;
//! use tower_async::{Service, ServiceExt, ServiceBuilder, service_fn};
//! use tower_async_http::{compression::Compression, decompression::DecompressionLayer, BoxError};
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), tower_async_http::BoxError> {
//! # async fn handle(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//! #     let body = Full::from("Hello, World!");
//! #     Ok(Response::new(body))
//! # }
//!
//! // Some opaque service that applies compression.
//! let service = Compression::new(service_fn(handle));
//!
//! // Our HTTP client.
//! let mut client = ServiceBuilder::new()
//!     // Automatically decompress response bodies.
//!     .layer(DecompressionLayer::new())
//!     .service(service);
//!
//! // Call the service.
//! //
//! // `DecompressionLayer` takes care of setting `Accept-Encoding`.
//! let request = Request::new(Full::<Bytes>::default());
//!
//! let response = client
//!     .call(request)
//!     .await?;
//!
//! // Read the body
//! let body = response.into_body();
//! let bytes = body.collect().await?.to_bytes().to_vec();
//! let body = String::from_utf8(bytes).map_err(Into::<BoxError>::into)?;
//!
//! assert_eq!(body, "Hello, World!");
//! #
//! # Ok(())
//! # }
//! ```

mod request;

mod body;
mod layer;
mod service;

pub use self::{body::DecompressionBody, layer::DecompressionLayer, service::Decompression};

pub use self::request::layer::RequestDecompressionLayer;
pub use self::request::service::RequestDecompression;

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    use crate::compression::Compression;
    use crate::test_helpers::{Body, TowerHttpBodyExt};

    use bytes::BytesMut;
    use flate2::write::GzEncoder;
    use http::Response;
    use hyper::{Error, Request};
    use tower_async::{service_fn, Service};

    #[tokio::test]
    async fn works() {
        let client = Decompression::new(Compression::new(service_fn(handle)));

        let req = Request::builder()
            .header("accept-encoding", "gzip")
            .body(Body::empty())
            .unwrap();
        let res = client.call(req).await.unwrap();

        // read the body, it will be decompressed automatically
        let mut body = res.into_body();
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk[..]);
        }
        let decompressed_data = String::from_utf8(data.freeze().to_vec()).unwrap();

        assert_eq!(decompressed_data, "Hello, World!");
    }

    #[tokio::test]
    async fn decompress_multi_gz() {
        let client = Decompression::new(service_fn(handle_multi_gz));

        let req = Request::builder()
            .header("accept-encoding", "gzip")
            .body(Body::empty())
            .unwrap();
        let res = client.call(req).await.unwrap();

        // read the body, it will be decompressed automatically
        let mut body = res.into_body();
        let mut data = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk[..]);
        }
        let decompressed_data = String::from_utf8(data.freeze().to_vec()).unwrap();

        assert_eq!(decompressed_data, "Hello, World!");
    }

    async fn handle(_req: Request<Body>) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from("Hello, World!")))
    }

    async fn handle_multi_gz(_req: Request<Body>) -> Result<Response<Body>, Error> {
        let mut buf = Vec::new();
        let mut enc1 = GzEncoder::new(&mut buf, Default::default());
        enc1.write_all(b"Hello, ").unwrap();
        enc1.finish().unwrap();

        let mut enc2 = GzEncoder::new(&mut buf, Default::default());
        enc2.write_all(b"World!").unwrap();
        enc2.finish().unwrap();

        let mut res = Response::new(Body::from(buf));
        res.headers_mut()
            .insert("content-encoding", "gzip".parse().unwrap());
        Ok(res)
    }

    // TODO: revisit
    // #[allow(dead_code)]
    // async fn is_compatible_with_hyper() {
    //     use tower_async_bridge::AsyncServiceExt;
    //     let mut client = Decompression::new(Client::new().into_async());

    //     let req = Request::new(Body::empty());

    //     let _: Response<DecompressionBody<Body>> = client.call(req).await.unwrap();
    // }
}
