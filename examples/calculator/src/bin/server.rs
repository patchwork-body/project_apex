#![allow(missing_docs)]

use apex::router::ApexRouter;
use axum::{Router, extract::Path, response::Html, routing::get};
use calculator::RootPageRoute;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let apex_router = Arc::new(ApexRouter::new().mount_route(RootPageRoute));

    let app = Router::new()
        .route(
            "/{*path}",
            get({
                let apex_router = Arc::clone(&apex_router);

                move |Path(path): Path<String>| async move {
                    let full_path = format!("/{path}");

                    match apex_router.handle_request(&full_path).await {
                        Some(content) => Html(content),
                        None => {
                            // Handle root path specifically
                            if path.is_empty() || path == "/" {
                                match apex_router.handle_request("/").await {
                                    Some(content) => Html(content),
                                    None => Html("Not Found".to_owned()),
                                }
                            } else {
                                Html("Not Found".to_owned())
                            }
                        }
                    }
                }
            }),
        )
        .nest_service("/static", ServeDir::new("static"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
