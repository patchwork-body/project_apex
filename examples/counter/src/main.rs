//! Apex Route Macro Demo with User-Defined Loaders
//!
//! This example demonstrates the new `#[route]` macro pattern where:
//! - Loader functions are defined by user code
//! - Loaders return LoaderResult<T> for data or exceptional behavior
//! - Route macro generates handlers that call loaders and render components

#![allow(missing_docs)]

use apex::{Apex, ApexRouter, Html, View, component, route, tmpl};
use std::net::SocketAddr;

/// Application context containing global app information.
#[derive(Clone, Debug)]
pub struct AppContext {
    /// The name of the application.
    pub app_name: String,
    /// The version of the application.
    pub version: String,
}

/// Counter component with count and name properties.
#[component(tag = "my-counter")]
pub struct Counter;

impl View for Counter {
    fn render(&self) -> Html {
        let count = 42;

        tmpl! {
            <div class="counter">
                <h1>Counter</h1>
                <p>Count: {count}</p>
            </div>
        }
    }
}

#[component(
    tag = "counter-page",
    imports = [Counter]
)]
pub struct CounterPage;

impl View for CounterPage {
    fn render(&self) -> Html {
        let title = "Welcome to the Counter Page";
        let value = "Enter text";

        tmpl! {
            <div>
                <h1>{title}</h1>
                <my-counter />
                <input type="text" value={value} />
            </div>
        }
    }
}

/// Route handler for the counter page.
#[allow(missing_docs)]
#[route(
    path = "/counter",
    component = CounterPage
)]
pub async fn counter_route(req: HttpRequest, context: &AppContext) -> LoaderResult<Counter> {
    LoaderResult::ok(Counter {
        count: 0,
        name: "Counter".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let context = AppContext {
        app_name: "Apex Demo".to_owned(),
        version: "1.0.0".to_owned(),
    };

    // Create router with routes using the generated handler functions
    let router = ApexRouter::new().get("/counter", {
        let ctx = context.clone();
        move |req| {
            let ctx = ctx.clone();
            async move { counter_route(req, &ctx).await }
        }
    });

    // Create and configure the Apex application
    let app = Apex::new().context(context).router(router);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    app.serve(addr).await
}
