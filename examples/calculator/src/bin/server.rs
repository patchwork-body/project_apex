#![allow(missing_docs)]

use apex::prelude::*;
use axum::{Router, response::Html, routing::get};
use calculator::Calculator;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                apex::apex_utils::reset_counters();

                let template = tmpl! {
                    <Calculator />
                };

                let index_html = format!(
                    r#"
                        <!DOCTYPE html>
                        <html lang="en">
                        <head>
                            <meta charset="UTF-8">
                            <meta name="viewport" content="width=device-width, initial-scale=1.0">
                            <title>Apex WASM Example</title>
                            <link rel="stylesheet" href="/static/styles.css">
                            <script type="module">
                                import init from '/static/client.js';

                                async function run() {{
                                    try {{
                                        await init();
                                        console.log('WASM loaded successfully!');
                                    }} catch (error) {{
                                        console.error('Failed to load WASM:', error);
                                    }}
                                }}

                                run();
                            </script>
                        </head>
                        <body>
                            {template}
                        </body>
                        </html>
                    "#
                );

                Html(index_html)
            }),
        )
        .nest_service("/static", ServeDir::new("static"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
