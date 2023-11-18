use super::{
    open_file::{FileOpened, FileRequestExtent, OpenFileOutput},
    ResponseBody,
};
use crate::{content_encoding::Encoding, services::fs::AsyncReadBody, BoxError};
use bytes::Bytes;
use http::{
    header::{self, ALLOW},
    HeaderValue, Request, Response, StatusCode,
};
use http_body_util::{BodyExt, Empty, Full};
use std::{convert::Infallible, io};
use tower_async_service::Service;

pub(super) async fn consume_open_file_result<ReqBody, ResBody, F>(
    open_file_result: Result<OpenFileOutput, std::io::Error>,
    mut fallback_and_request: Option<(F, Request<ReqBody>)>,
) -> Result<Response<ResponseBody>, std::io::Error>
where
    F: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Infallible> + Clone,
    ResBody: http_body::Body<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    match open_file_result {
        Ok(OpenFileOutput::FileOpened(file_output)) => Ok(build_response(*file_output)),

        Ok(OpenFileOutput::Redirect { location }) => {
            let mut res = response_with_status(StatusCode::TEMPORARY_REDIRECT);
            res.headers_mut().insert(http::header::LOCATION, location);
            Ok(res)
        }

        Ok(OpenFileOutput::FileNotFound) => {
            if let Some((fallback, request)) = fallback_and_request.take() {
                call_fallback(&fallback, request).await
            } else {
                Ok(not_found())
            }
        }

        Ok(OpenFileOutput::PreconditionFailed) => {
            Ok(response_with_status(StatusCode::PRECONDITION_FAILED))
        }

        Ok(OpenFileOutput::NotModified) => Ok(response_with_status(StatusCode::NOT_MODIFIED)),

        Err(err) => {
            #[cfg(unix)]
            // 20 = libc::ENOTDIR => "not a directory
            // when `io_error_more` landed, this can be changed
            // to checking for `io::ErrorKind::NotADirectory`.
            // https://github.com/rust-lang/rust/issues/86442
            let error_is_not_a_directory = err.raw_os_error() == Some(20);
            #[cfg(not(unix))]
            let error_is_not_a_directory = false;

            if matches!(
                err.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
            ) || error_is_not_a_directory
            {
                if let Some((fallback, request)) = fallback_and_request.take() {
                    call_fallback(&fallback, request).await
                } else {
                    Ok(not_found())
                }
            } else {
                Err(err)
            }
        }
    }
}

pub(super) fn method_not_allowed() -> Response<ResponseBody> {
    let mut res = response_with_status(StatusCode::METHOD_NOT_ALLOWED);
    res.headers_mut()
        .insert(ALLOW, HeaderValue::from_static("GET,HEAD"));
    res
}

fn response_with_status(status: StatusCode) -> Response<ResponseBody> {
    Response::builder()
        .status(status)
        .body(empty_body())
        .unwrap()
}

pub(super) fn not_found() -> Response<ResponseBody> {
    response_with_status(StatusCode::NOT_FOUND)
}

pub(super) async fn call_fallback<F, B, FResBody>(
    fallback: &F,
    req: Request<B>,
) -> Result<Response<ResponseBody>, std::io::Error>
where
    F: Service<Request<B>, Response = Response<FResBody>, Error = Infallible>,
    FResBody: http_body::Body<Data = Bytes> + Send + 'static,
    FResBody::Error: Into<BoxError>,
{
    let response = fallback.call(req).await.unwrap();
    Ok(response
        .map(|body| {
            body.map_err(|err| match err.into().downcast::<io::Error>() {
                Ok(err) => *err,
                Err(err) => io::Error::new(io::ErrorKind::Other, err),
            })
            .boxed_unsync()
        })
        .map(ResponseBody::new))
}

fn build_response(output: FileOpened) -> Response<ResponseBody> {
    let (maybe_file, size) = match output.extent {
        FileRequestExtent::Full(file, meta) => (Some(file), meta.len()),
        FileRequestExtent::Head(meta) => (None, meta.len()),
    };

    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, output.mime_header_value)
        .header(header::ACCEPT_RANGES, "bytes");

    if let Some(encoding) = output
        .maybe_encoding
        .filter(|encoding| *encoding != Encoding::Identity)
    {
        builder = builder.header(header::CONTENT_ENCODING, encoding.into_header_value());
    }

    if let Some(last_modified) = output.last_modified {
        builder = builder.header(header::LAST_MODIFIED, last_modified.0.to_string());
    }

    match output.maybe_range {
        Some(Ok(ranges)) => {
            if let Some(range) = ranges.first() {
                if ranges.len() > 1 {
                    builder
                        .header(header::CONTENT_RANGE, format!("bytes */{}", size))
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .body(body_from_bytes(Bytes::from(
                            "Cannot serve multipart range requests",
                        )))
                        .unwrap()
                } else {
                    let body = if let Some(file) = maybe_file {
                        let range_size = range.end() - range.start() + 1;
                        ResponseBody::new(
                            AsyncReadBody::with_capacity_limited(
                                file,
                                output.chunk_size,
                                range_size,
                            )
                            .boxed_unsync(),
                        )
                    } else {
                        empty_body()
                    };

                    builder
                        .header(
                            header::CONTENT_RANGE,
                            format!("bytes {}-{}/{}", range.start(), range.end(), size),
                        )
                        .header(header::CONTENT_LENGTH, range.end() - range.start() + 1)
                        .status(StatusCode::PARTIAL_CONTENT)
                        .body(body)
                        .unwrap()
                }
            } else {
                builder
                    .header(header::CONTENT_RANGE, format!("bytes */{}", size))
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .body(body_from_bytes(Bytes::from(
                        "No range found after parsing range header, please file an issue",
                    )))
                    .unwrap()
            }
        }

        Some(Err(_)) => builder
            .header(header::CONTENT_RANGE, format!("bytes */{}", size))
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .body(empty_body())
            .unwrap(),

        // Not a range request
        None => {
            let body = if let Some(file) = maybe_file {
                ResponseBody::new(
                    AsyncReadBody::with_capacity(file, output.chunk_size).boxed_unsync(),
                )
            } else {
                empty_body()
            };

            builder
                .header(header::CONTENT_LENGTH, size.to_string())
                .body(body)
                .unwrap()
        }
    }
}

fn body_from_bytes(bytes: Bytes) -> ResponseBody {
    let body = Full::from(bytes).map_err(|err| match err {}).boxed_unsync();
    ResponseBody::new(body)
}

fn empty_body() -> ResponseBody {
    let body = Empty::new().map_err(|err| match err {}).boxed_unsync();
    ResponseBody::new(body)
}
