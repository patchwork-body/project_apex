#![allow(missing_docs)]

use apex::prelude::*;
use apex::router::ApexRouter;
use axum::{Router, extract::Path, response::Html, routing::get};
use calculator::CalculatorPage;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[route(
    component = CalculatorPage
)]
fn root_page(params: HashMap<String, String>) {
    println!("Root page accessed with params: {params:?}");
}

#[tokio::main]
async fn main() {
    // Create the Apex router with our routes
    let apex_router = Arc::new(ApexRouter::new().route(
        "/",
        |_params: HashMap<String, String>| async {
            apex::apex_utils::reset_counters();

            tmpl! {
                <CalculatorPage />
            }
        },
    ));

    // Create Axum app that delegates to Apex router for dynamic routes
    // but still handles static files directly
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
        .route(
            "/",
            get({
                let apex_router = Arc::clone(&apex_router);
                move || async move {
                    match apex_router.handle_request("/").await {
                        Some(content) => Html(content),
                        None => Html("Not Found".to_owned()),
                    }
                }
            }),
        )
        .nest_service("/static", ServeDir::new("static"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
