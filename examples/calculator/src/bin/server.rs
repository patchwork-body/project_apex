#![allow(missing_docs)]

use apex::router::ApexRouter;
use bytes::Bytes;
use calculator::RootPageRoute;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode, body::Incoming as IncomingBody};
use hyper_util::rt::TokioIo;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn serve_static(path: &str) -> Result<Response<BoxBody>, hyper::Error> {
    let file_path = format!("static{path}");

    if Path::new(&file_path).exists() {
        match fs::read(&file_path) {
            Ok(content) => {
                let content_type = match Path::new(&file_path).extension().and_then(|s| s.to_str())
                {
                    Some("html") => "text/html",
                    Some("css") => "text/css",
                    Some("js") => "application/javascript",
                    Some("wasm") => "application/wasm",
                    Some("png") => "image/png",
                    Some("jpg" | "jpeg") => "image/jpeg",
                    Some("svg") => "image/svg+xml",
                    _ => "application/octet-stream",
                };

                match Response::builder()
                    .header("content-type", content_type)
                    .body(full(content))
                {
                    Ok(response) => Ok(response),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(full("Error building response"))
                        .unwrap()),
                }
            }
            Err(_) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Error reading file"))
                .unwrap()),
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full("File not found"))
            .unwrap())
    }
}

async fn handle_request(
    req: Request<IncomingBody>,
    apex_router: Arc<ApexRouter>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let path = req.uri().path();

    // Handle static files
    if path.starts_with("/static/") {
        return serve_static(&path[7..]).await; // Remove "/static" prefix
    }

    // Handle dynamic routes with Apex router
    match apex_router.handle_request(path).await {
        Some(content) => Ok(Response::builder()
            .header("content-type", "text/html")
            .body(full(content))
            .unwrap()),
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full("Not Found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let apex_router = Arc::new(ApexRouter::new().mount_route(RootPageRoute));

    let listener = TcpListener::bind("0.0.0.0:9999").await?;
    println!("Server running on http://0.0.0.0:9999");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let apex_router = Arc::clone(&apex_router);

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                let apex_router = Arc::clone(&apex_router);
                async move { handle_request(req, apex_router).await }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {err:?}");
            }
        });
    }
}
