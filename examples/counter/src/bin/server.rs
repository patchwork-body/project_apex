//! Apex Universal Entrypoint Demo - Server Side
//!
//! This example demonstrates the universal Apex entrypoint for server-side rendering
//! with routing and static file serving for WASM assets.

#![allow(missing_docs)]
#![cfg(not(target_arch = "wasm32"))]

use apex::{Apex, ApexRouter, route};
use counter::CounterPage;
use std::net::SocketAddr;

/// Application context containing global app information.
#[derive(Clone, Debug)]
pub struct AppContext {
    /// The name of the application.
    pub app_name: String,
    /// The version of the application.
    pub version: String,
}

/// Route handler for the counter page.
#[allow(missing_docs)]
#[route(
    path = "/counter",
    component = CounterPage
)]
pub async fn counter_route(req: HttpRequest, context: &AppContext) -> LoaderResult<CounterPage> {
    let mut page = CounterPage::new();
    page.set_page_title("Welcome to the Reactive Counter Page".to_string());
    LoaderResult::ok(page)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let context = AppContext {
        app_name: "Apex Universal Demo".to_owned(),
        version: "1.0.0".to_owned(),
    };

    // Create router with routes
    let router = ApexRouter::new().get("/", {
        let ctx = context.clone();
        move |req| {
            let ctx = ctx.clone();
            async move { counter_route(req, &ctx).await }
        }
    });

    // Create and configure the universal Apex application
    let app = Apex::new().router(router).static_dir("./pkg"); // Serve WASM and JS files from pkg directory

    println!("🚀 Starting Apex Universal Demo server:");
    println!("   • Server-side rendered page: http://127.0.0.1:3000/counter");
    println!("   • Client-side hydrated page: http://127.0.0.1:3000/");
    println!("   • Static assets served from: ./pkg/");

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    app.serve(addr).await
}
