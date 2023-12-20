use std::{
    collections::HashMap,
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use bytes::Bytes;
use clap::Parser;
use http::{
    header::{self, CONTENT_TYPE},
    HeaderValue, Method, StatusCode,
};
use http_body_util::Full;
use hyper::{Request, Response};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::net::TcpListener;
use tower_async::{
    limit::policy::{ConcurrentPolicy, LimitReached},
    service_fn,
    util::BoxService,
    BoxError, Service, ServiceBuilder, ServiceExt,
};
use tower_async_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, ServiceBuilderExt,
};
use tower_async_hyper::{HyperBody, TowerHyperServiceExt};

/// Simple Hyper server with an HTTP API
#[derive(Debug, Parser)]
struct Config {
    /// The port to listen on
    #[clap(short = 'p', long, default_value = "8080")]
    port: u16,
}

pub type WebRequest = Request<HyperBody>;
pub type WebResponse = Response<Full<Bytes>>;

pub trait IntoWebResponse {
    fn into_web_response(self) -> WebResponse;
}

impl IntoWebResponse for WebResponse {
    fn into_web_response(self) -> WebResponse {
        self
    }
}

impl IntoWebResponse for Infallible {
    fn into_web_response(self) -> WebResponse {
        panic!("BUG");
    }
}

impl IntoWebResponse for StatusCode {
    fn into_web_response(self) -> WebResponse {
        Response::builder()
            .status(self)
            .body(Full::default())
            .expect("the web response to be build")
    }
}

impl IntoWebResponse for &'static str {
    fn into_web_response(self) -> WebResponse {
        Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
            .body(Full::from(self))
            .expect("the web response to be build")
    }
}

impl IntoWebResponse for String {
    fn into_web_response(self) -> WebResponse {
        Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
            .body(Full::from(self))
            .expect("the web response to be build")
    }
}

#[derive(Debug, Clone, Default)]
pub struct UriParams {
    params: Option<HashMap<String, String>>,
}

impl UriParams {
    pub fn insert(&mut self, name: String, value: String) {
        self.params
            .get_or_insert_with(HashMap::new)
            .insert(name, value);
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&str> {
        self.params
            .as_ref()
            .and_then(|params| params.get(name.as_ref()))
            .map(String::as_str)
    }
}

#[derive(Debug)]
struct RouterEndpoint {
    matcher: EndpointMatcher,
    service: BoxService<WebRequest, WebResponse, WebResponse>,
}

impl RouterEndpoint {
    pub(crate) fn new(
        method: Method,
        path: &'static str,
        service: BoxService<WebRequest, WebResponse, WebResponse>,
    ) -> Self {
        Self {
            matcher: EndpointMatcher::new(method, path),
            service,
        }
    }
}

#[derive(Debug)]
enum PathFragment {
    Literal(&'static str),
    Param(&'static str),
    // Note if you also want to support some kind of Glob (*) stuff, you can also do that,
    // but let's keep it as simple as possible
}

#[derive(Debug)]
struct EndpointMatcher {
    fragments: Vec<PathFragment>,
    method: Method,
}

impl EndpointMatcher {
    pub fn new(method: Method, path: &'static str) -> Self {
        let fragments: Vec<PathFragment> = path
            .split('/')
            .filter_map(|s| {
                if s.is_empty() {
                    return None;
                }
                if s.starts_with(':') {
                    Some(PathFragment::Param(s.trim_start_matches(':')))
                } else {
                    Some(PathFragment::Literal(s))
                }
            })
            .collect();
        Self { fragments, method }
    }

    pub fn match_request(&self, method: &Method, path: &str) -> Option<UriParams> {
        if method != self.method {
            return None;
        }

        let fragments_iter = self
            .fragments
            .iter()
            .map(Some)
            .chain(std::iter::repeat(None));

        let mut params = UriParams::default();

        for (segment, fragment) in path.split('/').map(Some).zip(fragments_iter) {
            match (segment, fragment) {
                (Some(segment), Some(fragment)) => match fragment {
                    PathFragment::Literal(literal) => {
                        if !literal.eq_ignore_ascii_case(segment) {
                            return None;
                        }
                    }
                    PathFragment::Param(name) => {
                        params.insert(name.to_string(), segment.to_string());
                    }
                },
                (None, None) => {
                    break;
                }
                _ => {
                    return None;
                }
            }
        }

        Some(params)
    }
}

#[derive(Debug, Default)]
pub struct Router {
    endpoints: Arc<Vec<RouterEndpoint>>,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            endpoints: self.endpoints.clone(),
        }
    }
}

impl Router {
    // NOTE: you would not change this function signature since my original PR,
    // I Only changed this to make my example work
    pub fn on<F, Fut, O, E>(&mut self, method: Method, endpoint: &'static str, f: F)
    where
        F: Fn(WebRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<O, E>> + Send + Sync + 'static,
        E: IntoWebResponse + Send + 'static,
        O: IntoWebResponse + Send + 'static,
    {
        let svc = service_fn(f)
            .map_response(IntoWebResponse::into_web_response)
            .map_err(IntoWebResponse::into_web_response)
            .boxed();
        self.endpoints
            .push(RouterEndpoint::new(method, endpoint, svc));
    }
}

impl Service<WebRequest> for Router {
    type Response = WebResponse;
    type Error = Infallible;

    fn call(
        &self,
        mut req: WebRequest,
    ) -> impl std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + 'static
    {
        let endpoints = self.endpoints.clone();
        async move {
            let method = req.method();
            let path = req.uri().path().trim_matches('/');

            for endpoint in endpoints.iter() {
                if let Some(params) = endpoint.matcher.match_request(method, path.as_ref()) {
                    req.extensions_mut().insert(params);
                    return match endpoint.service.call(req).await {
                        Ok(res) => Ok(res),
                        Err(err) => Ok(err.into_web_response()),
                    };
                }
            }

            Ok(StatusCode::NOT_FOUND.into_web_response())
        }
    }
}

async fn render_page_fast(_request: WebRequest) -> Result<String, Infallible> {
    Ok(render_page("This was a fast response."))
}

async fn render_page_slow(_request: WebRequest) -> Result<String, Infallible> {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok(render_page("This was a slow response."))
}

fn render_page(msg: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hyper Http Server Example</title>
</head>
<body>
    <h1>Hello!</h1>
    <p>{msg}<p>
</body>
</html>
"##
    )
}

#[tokio::main]
async fn main() {
    // Setup tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let config = Config::parse();

    let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();

    let mut router = Router::default();
    router.on(Method::GET, "/fast", render_page_fast);
    router.on(Method::GET, "/slow", render_page_slow);

    let web_service = ServiceBuilder::new()
        .map_request_body(HyperBody::from)
        .compression()
        .sensitive_request_headers(sensitive_headers.clone())
        .layer(
            TraceLayer::new_for_http()
            .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
            })
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        )
        .sensitive_response_headers(sensitive_headers)
        .timeout(Duration::from_secs(10))
        .map_result(map_limit_result)
        .limit(ConcurrentPolicy::new(1))
        .service(router)
        .into_hyper_service();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let service = web_service.clone();
        tokio::spawn(async move {
            let stream = TokioIo::new(stream);
            let result = Builder::new(TokioExecutor::new())
                .serve_connection(stream, service)
                .await;
            if let Err(e) = result {
                eprintln!("server connection error: {}", e);
            }
        });
    }
}

fn map_limit_result(result: Result<WebResponse, BoxError>) -> Result<WebResponse, BoxError> {
    if let Err(err) = &result {
        if err.is::<LimitReached>() {
            return Ok(StatusCode::TOO_MANY_REQUESTS.into_web_response());
        }
    }
    result
}
