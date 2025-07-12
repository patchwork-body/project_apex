#![allow(missing_docs)]
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
#[cfg(not(target_arch = "wasm32"))]
use std::pin::Pin;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

// Re-export the html macro for convenience
pub use apex_macro::tmpl;

// Re-export required 3rd party crates
pub use bytes;
// Server-side re-exports (only available for non-WASM targets)
#[cfg(not(target_arch = "wasm32"))]
pub use http;
#[cfg(not(target_arch = "wasm32"))]
pub use http_body_util;
pub use js_sys;
pub use wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
pub use web_sys;

/// Signals module for reactive state management
pub mod signals;
pub use signals::{Effect, Reactive, Signal, render_with_effect};

/// Trait that defines the view layer for components
///
/// Components must implement this trait to provide their HTML rendering logic
pub trait View {
    /// Render the component to Html
    ///
    /// This method should return the complete HTML representation of the component
    fn render(&self) -> Html;
}

type HtmlCallback = Closure<dyn Fn(web_sys::Element)>;

/// Represents rendered HTML content
///
/// This type wraps HTML strings and provides a safe way to handle HTML content
#[derive(Debug)]
pub struct Html {
    callback: HtmlCallback,
}

impl Html {
    /// Create Html with a callback function for dynamic content generation
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(web_sys::Element) + 'static,
    {
        Html {
            callback: Closure::wrap(Box::new(callback) as Box<dyn Fn(web_sys::Element)>),
        }
    }

    /// Mount the HTML into a DOM element
    ///
    /// # Arguments
    /// * `target` - Optional CSS selector for the target element (defaults to "body")
    ///
    /// # Returns
    /// * `Result<(), wasm_bindgen::JsValue>` - Ok if successful, Err with JS error if failed
    pub fn mount(&self, target: Option<&str>) -> Result<(), wasm_bindgen::JsValue> {
        use web_sys::{Element, window};

        let window = window().ok_or("No global window object")?;
        let document = window.document().ok_or("No document object")?;

        let target_selector = target.unwrap_or("body");
        let target_element: Element = if target_selector == "body" {
            document.body().ok_or("No body element")?.into()
        } else {
            document
                .query_selector(target_selector)?
                .ok_or_else(|| format!("Target element '{target_selector}' not found"))?
        };

        let inner_html = target_element.inner_html();

        web_sys::console::log_1(&format!("[APEX] target element {inner_html:?}").into());

        let func: &js_sys::Function = self.callback.as_ref().unchecked_ref();
        func.call1(&wasm_bindgen::JsValue::NULL, &target_element.clone().into())?;

        let inner_html = target_element.inner_html();

        web_sys::console::log_1(&format!("[APEX] target element {inner_html:?}").into());

        Ok(())
    }

    /// Update the mounted HTML by re-executing the callback
    /// This is useful for reactive updates when state changes
    pub fn update(&self, target: Option<&str>) -> Result<(), wasm_bindgen::JsValue> {
        self.mount(target)
    }
}

impl From<String> for Html {
    fn from(content: String) -> Self {
        Html {
            callback: Closure::new(Box::new(move |element: web_sys::Element| {
                element.set_inner_html(&content);
            })),
        }
    }
}

impl From<&str> for Html {
    fn from(content: &str) -> Self {
        let owned_content = content.to_string();
        Html {
            callback: Closure::new(Box::new(move |element: web_sys::Element| {
                element.set_inner_html(&owned_content);
            })),
        }
    }
}

impl std::fmt::Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use web_sys::window;

        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Ok(temp_element) = document.create_element("div") {
                    let func: &js_sys::Function = self.callback.as_ref().unchecked_ref();
                    if func
                        .call1(&wasm_bindgen::JsValue::NULL, &temp_element.clone().into())
                        .is_ok()
                    {
                        return write!(f, "{}", temp_element.inner_html());
                    }
                }
            }
        }
        write!(f, "")
    }
}

// Server-side code (only available for non-WASM targets)
#[cfg(not(target_arch = "wasm32"))]
mod server {
    use super::*;
    use bytes::Bytes;
    use http::{Method, Request, Response, StatusCode};
    use http_body_util::Full;
    use std::path::Path;

    /// Type alias for HTTP body
    pub type Body = Full<Bytes>;

    /// Type alias for HTTP request
    pub type HttpRequest = Request<hyper::body::Incoming>;

    /// Type alias for HTTP response
    pub type HttpResponse = Response<Body>;

    /// Type alias for route handler function
    pub type Handler = Arc<
        dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync,
    >;

    /// Loader result that can contain data or exceptional behavior
    #[derive(Debug)]
    pub enum LoaderResult<T> {
        /// Success with data
        Ok(T),
        /// Redirect to another URL
        Redirect(String),
        /// Not found error
        NotFound,
        /// Internal server error
        ServerError(String),
        /// Custom HTTP response
        Response(HttpResponse),
    }

    impl<T> LoaderResult<T> {
        /// Create a successful result with data
        pub fn ok(data: T) -> Self {
            LoaderResult::Ok(data)
        }

        /// Create a redirect result
        pub fn redirect(url: impl Into<String>) -> Self {
            LoaderResult::Redirect(url.into())
        }

        /// Create a not found result
        pub fn not_found() -> Self {
            LoaderResult::NotFound
        }

        /// Create a server error result
        pub fn server_error(message: impl Into<String>) -> Self {
            LoaderResult::ServerError(message.into())
        }

        /// Create a custom response result
        pub fn response(response: HttpResponse) -> Self {
            LoaderResult::Response(response)
        }

        /// Convert to HTTP response, calling the component render function if successful
        pub fn into_response<F>(self, render_fn: F) -> HttpResponse
        where
            F: FnOnce(T) -> String,
        {
            match self {
                LoaderResult::Ok(data) => {
                    let html = render_fn(data);
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/html; charset=utf-8")
                        .body(Full::new(Bytes::from(html)))
                        .unwrap()
                }
                LoaderResult::Redirect(url) => Response::builder()
                    .status(StatusCode::FOUND)
                    .header("location", url)
                    .body(Full::new(Bytes::new()))
                    .unwrap(),
                LoaderResult::NotFound => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Full::new(Bytes::from("<h1>404 Not Found</h1>")))
                    .unwrap(),
                LoaderResult::ServerError(message) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Full::new(Bytes::from(format!(
                        "<h1>500 Server Error</h1><p>{}</p>",
                        message
                    ))))
                    .unwrap(),
                LoaderResult::Response(response) => response,
            }
        }
    }

    /// HTTP method and path combination for routing
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Route {
        pub method: Method,
        pub path: String,
    }

    impl Route {
        /// Create a new route
        pub fn new(method: Method, path: impl Into<String>) -> Self {
            Self {
                method,
                path: path.into(),
            }
        }
    }

    /// The ApexRouter handles HTTP request routing
    #[derive(Clone)]
    pub struct ApexRouter {
        routes: HashMap<Route, Handler>,
    }

    impl ApexRouter {
        /// Create a new ApexRouter
        pub fn new() -> Self {
            Self {
                routes: HashMap::new(),
            }
        }

        /// Add a GET route
        pub fn get<F, Fut>(mut self, path: impl Into<String>, handler: F) -> Self
        where
            F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = HttpResponse> + Send + 'static,
        {
            let route = Route::new(Method::GET, path);
            let handler = Arc::new(move |req| {
                Box::pin(handler(req)) as Pin<Box<dyn Future<Output = HttpResponse> + Send>>
            });
            self.routes.insert(route, handler);

            self
        }

        /// Add a POST route
        pub fn post<F, Fut>(mut self, path: impl Into<String>, handler: F) -> Self
        where
            F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = HttpResponse> + Send + 'static,
        {
            let route = Route::new(Method::POST, path);
            let handler = Arc::new(move |req| {
                Box::pin(handler(req)) as Pin<Box<dyn Future<Output = HttpResponse> + Send>>
            });
            self.routes.insert(route, handler);

            self
        }

        /// Add a PUT route
        pub fn put<F, Fut>(mut self, path: impl Into<String>, handler: F) -> Self
        where
            F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = HttpResponse> + Send + 'static,
        {
            let route = Route::new(Method::PUT, path);
            let handler = Arc::new(move |req| {
                Box::pin(handler(req)) as Pin<Box<dyn Future<Output = HttpResponse> + Send>>
            });
            self.routes.insert(route, handler);

            self
        }

        /// Add a DELETE route
        pub fn delete<F, Fut>(mut self, path: impl Into<String>, handler: F) -> Self
        where
            F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = HttpResponse> + Send + 'static,
        {
            let route = Route::new(Method::DELETE, path);
            let handler = Arc::new(move |req| {
                Box::pin(handler(req)) as Pin<Box<dyn Future<Output = HttpResponse> + Send>>
            });
            self.routes.insert(route, handler);

            self
        }

        /// Add a route with a custom method
        pub fn route<F, Fut>(mut self, method: Method, path: impl Into<String>, handler: F) -> Self
        where
            F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = HttpResponse> + Send + 'static,
        {
            let route = Route::new(method, path);
            let handler = Arc::new(move |req| {
                Box::pin(handler(req)) as Pin<Box<dyn Future<Output = HttpResponse> + Send>>
            });
            self.routes.insert(route, handler);

            self
        }

        /// Handle an HTTP request
        pub async fn handle(&self, req: HttpRequest) -> HttpResponse {
            let route = Route::new(req.method().clone(), req.uri().path().to_string());

            if let Some(handler) = self.routes.get(&route) {
                handler(req).await
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Full::new(Bytes::from("<h1>404 Not Found</h1>")))
                    .unwrap()
            }
        }

        /// Get all routes
        pub fn routes(&self) -> impl Iterator<Item = &Route> {
            self.routes.keys()
        }
    }

    impl Default for ApexRouter {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Serve static files from a directory
    pub async fn serve_static_file(file_path: &str, static_dir: &str) -> Option<HttpResponse> {
        let full_path = Path::new(static_dir).join(file_path.trim_start_matches('/'));

        if !full_path.exists() || !full_path.is_file() {
            return None;
        }

        // Security check: ensure the path doesn't escape the static directory
        if let Ok(canonical_static) = Path::new(static_dir).canonicalize() {
            if let Ok(canonical_file) = full_path.canonicalize() {
                if !canonical_file.starts_with(canonical_static) {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }

        let content = match tokio::fs::read(&full_path).await {
            Ok(content) => content,
            Err(_) => return None,
        };

        let content_type = match full_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("wasm") => "application/wasm",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            _ => "application/octet-stream",
        };

        Some(
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .body(Full::new(Bytes::from(content)))
                .unwrap(),
        )
    }
}

/// Universal Apex application that works on both server and client
#[derive(Default)]
pub struct Apex {
    #[cfg(not(target_arch = "wasm32"))]
    router: Option<ApexRouter>,
    #[cfg(not(target_arch = "wasm32"))]
    static_dir: Option<String>,
}

impl Apex {
    /// Create a new Apex application
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            router: None,
            #[cfg(not(target_arch = "wasm32"))]
            static_dir: None,
        }
    }

    /// Hydrate the client-side application with a component
    pub fn hydrate<T: View>(self, component: T) -> Result<(), wasm_bindgen::JsValue> {
        use web_sys::window;

        let window = window().ok_or("No global window object")?;
        let document = window.document().ok_or("No document object")?;

        let body = document.body().ok_or("No body element")?;

        let html = component.render();
        let func: &js_sys::Function = html.callback.as_ref().unchecked_ref();
        func.call1(&wasm_bindgen::JsValue::NULL, &body.into())?;

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Apex {
    /// Set the router
    pub fn router(mut self, router: ApexRouter) -> Self {
        self.router = Some(router);
        self
    }

    /// Set the static files directory (for serving WASM assets and other static files)
    pub fn static_dir(mut self, dir: impl Into<String>) -> Self {
        self.static_dir = Some(dir.into());
        self
    }

    /// Get the router
    pub fn get_router(&self) -> Option<&ApexRouter> {
        self.router.as_ref()
    }

    /// Handle an HTTP request with routing and static file serving
    pub async fn handle_request(&self, req: HttpRequest) -> HttpResponse {
        let path = req.uri().path();

        // First try to serve static files if static_dir is configured
        if let Some(static_dir) = &self.static_dir {
            if let Some(response) = server::serve_static_file(path, static_dir).await {
                return response;
            }
        }

        // If not a static file, try routing
        if let Some(router) = &self.router {
            router.handle(req).await
        } else {
            use bytes::Bytes;
            use http::{Response, StatusCode};
            use http_body_util::Full;

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from("<h1>No router configured</h1>")))
                .unwrap()
        }
    }

    /// Start the HTTP server
    pub async fn serve(
        self,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(addr).await?;
        println!("Server running on http://{}", addr);

        let service = Arc::new(self);

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let service = Arc::clone(&service);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let service = Arc::clone(&service);
                            async move { Ok::<_, hyper::Error>(service.handle_request(req).await) }
                        }),
                    )
                    .await
                {
                    eprintln!("Error serving connection: {err:?}");
                }
            });
        }
    }
}

// Re-export server types for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub use server::*;
