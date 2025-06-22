//! Apex Route Macro Demo with User-Defined Loaders and Signals
//!
//! This example demonstrates the new `#[route]` macro pattern and signals-based state management where:
//! - Loader functions are defined by user code
//! - Loaders return LoaderResult<T> for data or exceptional behavior
//! - Route macro generates handlers that call loaders and render components
//! - Components use signals for reactive state management

#![allow(missing_docs)]

use apex::{Apex, ApexRouter, Html, Signal, View, component, route, tmpl};
use std::net::SocketAddr;

/// Application context containing global app information.
#[derive(Clone, Debug)]
pub struct AppContext {
    /// The name of the application.
    pub app_name: String,
    /// The version of the application.
    pub version: String,
}

/// Counter component with reactive state using signals
#[component]
pub struct Counter {
    #[signal]
    count: Signal<i32>,

    #[prop(default = "Counter")]
    name: String,
}

impl View for Counter {
    fn render(&self) -> Html {
        tmpl! {
            <div class="counter">
                <h1>Reactive {self.name}</h1>
                <p>Count: {self.count}</p>
                <button onclick={|_| {
                    self.count.update(|c| *c += 1);
                }}>Increment</button>
                <button onclick={|_| {
                    self.count.update(|c| *c -= 1);
                }}>Decrement</button>
                <button onclick={|_| {
                    self.count.set(0);
                }}>Reset</button>
            </div>
        }
    }
}

/// A page component that contains the counter
#[component]
pub struct CounterPage {
    #[signal]
    page_title: Signal<String>,
}

impl View for CounterPage {
    fn render(&self) -> Html {
        tmpl! {
            <div>
                <h1>{self.page_title}</h1>
                <Counter name="My Counter" />
                <input name="page_title" type="text" value={self.page_title} />
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
pub async fn counter_route(req: HttpRequest, context: &AppContext) -> LoaderResult<CounterPage> {
    let mut page = CounterPage::new();
    page.set_page_title("Welcome to the Reactive Counter Page".to_string());
    LoaderResult::ok(page)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let context = AppContext {
        app_name: "Apex Signals Demo".to_owned(),
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

    println!("🚀 Starting Apex Signals Demo server on http://127.0.0.1:3000/counter");

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    app.serve(addr).await
}
