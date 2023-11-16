// use std::convert::Infallible;

// use http::{Request, Response, StatusCode};
// use hyper::body::Bytes;
// use hyper_util::rt::TokioExecutor;
// use hyper_util::server::conn::auto::Builder;
// use tokio::net::TcpListener;
// use tower_async::ServiceBuilder;
// use tower_async_http::ServiceBuilderExt;
// use tower_async_hyper::TowerHyperServiceExt;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let service = ServiceBuilder::new()
//         .timeout(std::time::Duration::from_secs(5))
//         .compression()
//         .service_fn(|req: Request<Bytes>| async move {
//             Ok::<_, Infallible>(
//                 Response::builder()
//                     .status(StatusCode::OK)
//                     .header("content-type", "text/plain")
//                     .body(Bytes::from_static(&b"Hello, world!"[..]))
//                     .unwrap(),
//             )
//         });

//     let addr = ([127, 0, 0, 1], 8080).into();
//     let listener = TcpListener::bind(addr).await?;

//     loop {
//         let (stream, _) = listener.accept().await?;
//         let service = service.clone();
//         let service = service.clone().into_hyper();
//         tokio::spawn(async move {
//             let result = Builder::new(TokioExecutor::new())
//                 .serve_connection(stream, service)
//                 .await;
//             if let Err(e) = result {
//                 eprintln!("server connection error: {}", e);
//             }
//         });
//     }
// }

fn main() {}
