#![allow(missing_docs)]
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

// Re-export the component macro for convenience
pub use apex_macro::component;
// Re-export the route macro for convenience
pub use apex_macro::route;
// Re-export the html macro for convenience
pub use apex_macro::tmpl;

// Re-export required 3rd party crates
pub use bytes;
pub use http;
pub use http_body_util;

/// Trait that defines the view layer for components
///
/// Components must implement this trait to provide their HTML rendering logic
pub trait View {
    /// Render the component to Html
    ///
    /// This method should return the complete HTML representation of the component
    fn render(&self) -> Html;
}

/// Represents rendered HTML content
///
/// This type wraps HTML strings and provides a safe way to handle HTML content
#[derive(Debug, Clone, PartialEq)]
pub struct Html {
    content: String,
}

impl Html {
    /// Create a new Html instance from a string
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    /// Create an empty Html instance
    pub fn empty() -> Self {
        Self {
            content: String::new(),
        }
    }

    /// Get the inner HTML content as a string
    pub fn into_string(self) -> String {
        self.content
    }

    /// Get a reference to the inner HTML content
    pub fn as_str(&self) -> &str {
        &self.content
    }
}

impl From<String> for Html {
    fn from(content: String) -> Self {
        Self::new(content)
    }
}

impl From<&str> for Html {
    fn from(content: &str) -> Self {
        Self::new(content)
    }
}

impl std::fmt::Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

/// Type alias for HTTP body
pub type Body = Full<Bytes>;

/// Type alias for HTTP request
pub type HttpRequest = Request<hyper::body::Incoming>;

/// Type alias for HTTP response
pub type HttpResponse = Response<Body>;

/// Type alias for route handler function
pub type Handler =
    Arc<dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> + Send + Sync>;

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

    /// Add a route with any HTTP method
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

    /// Handle an incoming HTTP request
    pub async fn handle(&self, req: HttpRequest) -> HttpResponse {
        let route = Route::new(req.method().clone(), req.uri().path().to_string());

        match self.routes.get(&route) {
            Some(handler) => handler(req).await,
            None => {
                // Return 404 Not Found
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("Not Found")))
                    .unwrap()
            }
        }
    }

    /// Get all registered routes
    pub fn routes(&self) -> impl Iterator<Item = &Route> {
        self.routes.keys()
    }
}

impl Default for ApexRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// The main Apex application builder
pub struct Apex<C = ()> {
    context: C,
    router: Option<ApexRouter>,
}

impl Apex<()> {
    /// Create a new Apex instance
    pub fn new() -> Self {
        Self {
            context: (),
            router: None,
        }
    }
}

impl<C> Apex<C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Set the context for the Apex application
    pub fn context<NewC>(self, context: NewC) -> Apex<NewC> {
        Apex {
            context,
            router: self.router,
        }
    }

    /// Set the router for the Apex application
    pub fn router(mut self, router: ApexRouter) -> Self {
        self.router = Some(router);
        self
    }

    /// Get a reference to the context
    pub fn get_context(&self) -> &C {
        &self.context
    }

    /// Get a reference to the router
    pub fn get_router(&self) -> Option<&ApexRouter> {
        self.router.as_ref()
    }

    /// Handle an HTTP request using the configured router
    pub async fn handle_request(&self, req: HttpRequest) -> HttpResponse {
        match &self.router {
            Some(router) => router.handle(req).await,
            None => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("No router configured")))
                .unwrap(),
        }
    }

    /// Start the HTTP server and serve requests
    pub async fn serve(
        self,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Apex server listening on http://{addr}");

        // Create an Arc to share the Apex instance across connections
        let apex = Arc::new(self);

        loop {
            let (stream, _) = listener.accept().await?;
            let apex = apex.clone();

            // Spawn a task to handle each connection
            tokio::task::spawn(async move {
                let service = service_fn(move |req| {
                    let apex = apex.clone();
                    async move {
                        let response = apex.handle_request(req).await;
                        Ok::<_, hyper::Error>(response)
                    }
                });

                // Wrap the stream with TokioIo to make it compatible with hyper
                let io = TokioIo::new(stream);

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

impl Default for Apex<()> {
    fn default() -> Self {
        Self::new()
    }
}
