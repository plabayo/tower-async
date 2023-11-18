use std::net::SocketAddr;

use http::{Request, Response, StatusCode};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;
use tower_async::ServiceBuilder;
use tower_async_http::ServiceBuilderExt;
use tower_async_hyper::TowerHyperServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = ServiceBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .map_request(|req| req)
        // .decompression()
        // .compression()
        // .follow_redirects()
        // .trace_for_http()
        .service_fn(|_req: Request<Incoming>| async move {
            // req.into_body().data().await;
            // let bytes = body::to_bytes(req.into_body()).await?;
            // let body = String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8");
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain")
                .body(String::from("hello"))
        });

    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let service = service.clone().into_hyper_service();
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
