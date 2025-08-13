use matchit::{Match, Router};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Public handler type used by routes
pub type ApexHandler = Box<
    dyn Fn(HashMap<String, String>) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync,
>;

/// Trait implemented by macro-generated route structs
pub trait ApexRoute: Send + Sync {
    /// Static path for this route (e.g., "/users/:id")
    fn path(&self) -> &'static str;
    /// Handler function invoked by the router
    fn handler(&self) -> ApexHandler;
    /// Children routes for nested routing
    fn children(&self) -> Vec<Box<dyn ApexRoute>> {
        Vec::new()
    }
}

pub struct ApexRouter {
    router: Router<ApexHandler>,
    // Store route metadata for outlet handling
    routes: Vec<(String, Vec<Box<dyn ApexRoute>>)>,
}

impl ApexRouter {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            routes: Vec::new(),
        }
    }

    pub fn route<F, Fut>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(HashMap<String, String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let boxed_handler: ApexHandler = Box::new(move |params| Box::pin(handler(params)));

        if let Err(e) = self.router.insert(path, boxed_handler) {
            panic!("Failed to insert route '{path}': {e}");
        }

        self
    }

    /// Mount a macro-generated route struct implementing `ApexRoute`
    pub fn mount_route<R: ApexRoute>(mut self, route: R) -> Self {
        let path = route.path();
        let handler = route.handler();
        let children = route.children();

        if let Err(e) = self.router.insert(path, handler) {
            panic!("Failed to insert route '{path}': {e}");
        }

        // Store route metadata for outlet handling
        self.routes.push((path.to_string(), children));

        // Don't mount children routes as direct routes - they will be handled through outlets
        // for child in route.children() {
        //     let child_path = self.combine_paths(path, child.path());
        //     let child_handler = child.handler();
        //     if let Err(e) = self.router.insert(&child_path, child_handler) {
        //         panic!("Failed to insert child route '{child_path}': {e}");
        //     }
        // }

        self
    }

    /// Combine parent and child paths for nested routing
    fn combine_paths(&self, parent: &str, child: &str) -> String {
        let parent = parent.trim_end_matches('/');
        let child = child.trim_start_matches('/');

        if parent.is_empty() {
            format!("/{child}")
        } else if child.is_empty() {
            parent.to_string()
        } else {
            format!("{parent}/{child}")
        }
    }

    pub async fn handle_request(&self, path: &str) -> Option<String> {
        match self.router.at(path) {
            Ok(Match {
                value: handler,
                params,
            }) => {
                let params_map: HashMap<String, String> = params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                Some(handler(params_map).await)
            }
            Err(_) => None,
        }
    }

    /// Handle request with outlet support - renders parent route with child content injected
    pub async fn handle_request_with_outlets(&self, path: &str) -> Option<String> {
        // First try to match the exact path
        if let Some(result) = self.handle_request(path).await {
            return Some(result);
        }

        // If no exact match, try to find parent routes that might have outlets
        let path_segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // Try different parent/child combinations
        for i in (1..path_segments.len()).rev() {
            let parent_path = format!("/{}", path_segments[..i].join("/"));
            let child_path = format!("/{}", path_segments[i..].join("/"));

            if let Some(parent_html) = self.handle_request(&parent_path).await {
                // Check if parent has outlet
                if parent_html.contains("<!-- @outlet -->") {
                    // We need to find and render the child route manually since it's not mounted as a direct route
                    // For now, we need to access the route's children and find the matching one
                    if let Some(child_html) =
                        self.render_child_route(&parent_path, &child_path).await
                    {
                        // Replace outlet with child content
                        let result = parent_html.replace("<!-- @outlet -->", &child_html);
                        return Some(result);
                    }
                }
            }
        }

        None
    }

    /// Render a child route by finding it in the parent route's children
    async fn render_child_route(&self, parent_path: &str, child_path: &str) -> Option<String> {
        // Find the parent route in our stored metadata
        for (route_path, children) in &self.routes {
            // Check if this route matches the parent path pattern
            if self.path_matches(route_path, parent_path) {
                // Look for a child route that matches the child path
                for child in children {
                    if child.path() == child_path {
                        // Found matching child route, render it
                        let params = self.extract_params(route_path, parent_path);
                        let handler = child.handler();
                        return Some(handler(params).await);
                    }
                }
            }
        }

        None
    }

    /// Check if a route pattern matches a path (handles parameters like /{name}/{age})
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        if pattern_segments.len() != path_segments.len() {
            return false;
        }

        for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
            if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
                // This is a parameter, it matches any value
                continue;
            } else if pattern_seg != path_seg {
                return false;
            }
        }

        true
    }

    /// Extract parameters from a path using a pattern
    fn extract_params(&self, pattern: &str, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

        for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
            if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
                let param_name = &pattern_seg[1..pattern_seg.len() - 1];
                params.insert(param_name.to_string(), path_seg.to_string());
            }
        }

        params
    }
}

impl Default for ApexRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_routing() {
        let router = ApexRouter::new()
            .route("/", |_| async { "Home".to_string() })
            .route("/about", |_| async { "About".to_string() });

        assert_eq!(router.handle_request("/").await, Some("Home".to_string()));
        assert_eq!(
            router.handle_request("/about").await,
            Some("About".to_string())
        );
        assert_eq!(router.handle_request("/missing").await, None);
    }

    #[tokio::test]
    async fn test_router_builder_pattern() {
        async fn root_handler(_params: HashMap<String, String>) -> String {
            "Root page".to_string()
        }

        async fn about_handler(_params: HashMap<String, String>) -> String {
            "About page".to_string()
        }

        let router = ApexRouter::new()
            .route("/", root_handler)
            .route("/about", about_handler);

        assert_eq!(
            router.handle_request("/").await,
            Some("Root page".to_string())
        );
        assert_eq!(
            router.handle_request("/about").await,
            Some("About page".to_string())
        );
    }

    #[tokio::test]
    async fn test_path_parameters() {
        let router = ApexRouter::new()
            .route("/users/{id}", |params| async move {
                format!(
                    "User ID: {}",
                    params.get("id").unwrap_or(&"unknown".to_string())
                )
            })
            .route("/posts/{id}/comments/{comment_id}", |params| async move {
                format!(
                    "Post ID: {}, Comment ID: {}",
                    params.get("id").unwrap_or(&"unknown".to_string()),
                    params.get("comment_id").unwrap_or(&"unknown".to_string())
                )
            });

        assert_eq!(
            router.handle_request("/users/123").await,
            Some("User ID: 123".to_string())
        );

        assert_eq!(
            router.handle_request("/posts/456/comments/789").await,
            Some("Post ID: 456, Comment ID: 789".to_string())
        );

        assert_eq!(router.handle_request("/users").await, None);
    }

    #[tokio::test]
    async fn test_wildcard_routes() {
        let router = ApexRouter::new().route("/static/{*filepath}", |params| async move {
            format!(
                "Static file: {}",
                params.get("filepath").unwrap_or(&"index.html".to_string())
            )
        });

        assert_eq!(
            router.handle_request("/static/css/style.css").await,
            Some("Static file: css/style.css".to_string())
        );

        assert_eq!(
            router.handle_request("/static/js/app.js").await,
            Some("Static file: js/app.js".to_string())
        );
    }

    #[tokio::test]
    async fn test_path_combination() {
        let router = ApexRouter::new();

        assert_eq!(router.combine_paths("/", "profile"), "/profile");
        assert_eq!(
            router.combine_paths("/dashboard", "profile"),
            "/dashboard/profile"
        );
        assert_eq!(
            router.combine_paths("/dashboard/", "/profile"),
            "/dashboard/profile"
        );
        assert_eq!(
            router.combine_paths("/dashboard", "/profile"),
            "/dashboard/profile"
        );
        assert_eq!(router.combine_paths("", "profile"), "/profile");
        assert_eq!(router.combine_paths("/dashboard", ""), "/dashboard");
    }
}
