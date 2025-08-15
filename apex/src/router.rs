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
    fn hydrate_components(
        &self,
        pathname: &str,
        expressions_map: &HashMap<String, web_sys::Text>,
        elements_map: &HashMap<String, web_sys::Element>,
    );
}

pub struct ApexRouter {
    router: Router<ApexHandler>,
    // Store route metadata for outlet handling
    routes: Vec<(String, Vec<Box<dyn ApexRoute>>)>,
    // Store hierarchical route structure for path matching
    route_tree: RouteTree,
}

struct RouteNode {
    path: String,
    handler: Option<ApexHandler>,
    children: HashMap<String, RouteNode>,
}

#[derive(Default)]
struct RouteTree {
    root: HashMap<String, RouteNode>,
}

impl ApexRouter {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            routes: Vec::new(),
            route_tree: RouteTree::default(),
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

        // Store route metadata for hierarchical matching logic
        self.routes.push((path.to_string(), Vec::new()));

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

    /// Main route handling method with hierarchical matching
    ///
    /// For incoming request /pathA/pathB:
    /// 1. If root "/" has actual children defined, matches root first and looks for children
    /// 2. Otherwise, tries exact matches from most specific to least specific
    /// 3. Falls back to root as last resort if it exists
    pub async fn handle_request(&self, path: &str) -> Option<String> {
        let segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // Try to find a matching route hierarchy
        let html = self.find_hierarchical_match(&segments).await?;

        // Inject collected INIT_DATA script if any routes had data
        #[cfg(not(target_arch = "wasm32"))]
        {
            let init_script = crate::init_data::generate_init_data_script();
            if !init_script.is_empty() {
                // Find </head> tag and inject the script before it
                if let Some(head_end) = html.find("</head>") {
                    let mut result = html.clone();
                    result.insert_str(head_end, &init_script);
                    return Some(result);
                }
            }
        }

        Some(html)
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

    /// Find hierarchical match starting from root and traversing down
    async fn find_hierarchical_match(&self, segments: &[&str]) -> Option<String> {
        // Check if root route exists and has actual children defined
        let root_has_children = self
            .routes
            .iter()
            .any(|(path, children)| path == "/" && !children.is_empty());

        // If root has actual children routes, prioritize hierarchical matching
        if root_has_children && let Some(result) = self.try_match_from_path("/", segments).await {
            return Some(result);
        }

        // Handle empty segments (root path) first
        if segments.is_empty()
            && let Some(result) = self.try_match_from_path("/", segments).await
        {
            return Some(result);
        }

        // Try exact matches from most specific to least specific
        for i in (1..=segments.len()).rev() {
            let parent_path = format!("/{}", segments[..i].join("/"));
            if let Some(result) = self.try_match_from_path(&parent_path, &segments[i..]).await {
                return Some(result);
            }
        }

        // Fall back to root in hierarchical scenarios:
        // 1. If we have a root with children defined (mount_route scenario)
        // 2. If we have multiple levels of nested routes (indicating hierarchical design)
        if !segments.is_empty() {
            let has_nested_routes = self.routes.iter().any(|(path, _)| {
                path != "/" && path.matches('/').count() > 1 // More than one slash means nested
            });
            let is_hierarchical_path = segments.len() > 1;

            if (root_has_children || is_hierarchical_path || has_nested_routes)
                && let Some(result) = self.try_match_from_path("/", segments).await
            {
                return Some(result);
            }
        }

        None
    }

    /// Try to match a path and its remaining segments hierarchically
    async fn try_match_from_path(
        &self,
        base_path: &str,
        remaining_segments: &[&str],
    ) -> Option<String> {
        // First check if the base path itself matches
        match self.router.at(base_path) {
            Ok(Match {
                value: handler,
                params,
            }) => {
                let params_map: HashMap<String, String> = params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();

                let base_result = handler(params_map).await;

                // If there are no remaining segments, return the base result
                if remaining_segments.is_empty() {
                    return Some(base_result);
                }

                // If there are remaining segments, try to match child routes hierarchically
                // This handles the case where we matched root "/" and need to find children pathA/pathB
                if base_result.contains("<!-- @outlet -->") {
                    // Try to render child routes for the remaining segments
                    if let Some(child_result) = self
                        .render_child_route_hierarchical(base_path, remaining_segments)
                        .await
                    {
                        let result = base_result.replace("<!-- @outlet -->", &child_result);
                        return Some(result);
                    }
                }

                // Return base result even if we couldn't match children
                // This ensures that if we match root "/" but can't find child routes,
                // we still return the root route content
                Some(base_result)
            }
            Err(_) => None,
        }
    }

    /// Render child routes hierarchically by trying to match segments progressively
    fn render_child_route_hierarchical<'a>(
        &'a self,
        parent_path: &'a str,
        remaining_segments: &'a [&str],
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
        Box::pin(async move {
            if remaining_segments.is_empty() {
                return None;
            }

            // Try to match the first segment as a direct child
            let first_segment_path = format!("/{}", remaining_segments[0]);

            // Find the parent route in our stored metadata
            for (route_path, children) in &self.routes {
                // Check if this route matches the parent path pattern
                if self.path_matches(route_path, parent_path) {
                    // Look for a child route that matches the first segment
                    for child in children {
                        if child.path() == first_segment_path {
                            let params = self.extract_params(route_path, parent_path);
                            let handler = child.handler();

                            // If there are more segments after this one
                            if remaining_segments.len() > 1 {
                                let child_result = handler(params).await;

                                // Check if this child has outlets for further nesting
                                if child_result.contains("<!-- @outlet -->") {
                                    // Try to render the remaining segments as grandchildren
                                    let grandchild_path =
                                        self.combine_paths(parent_path, child.path());
                                    if let Some(grandchild_result) = self
                                        .render_child_route_hierarchical(
                                            &grandchild_path,
                                            &remaining_segments[1..],
                                        )
                                        .await
                                    {
                                        return Some(
                                            child_result
                                                .replace("<!-- @outlet -->", &grandchild_result),
                                        );
                                    }
                                }

                                return Some(child_result);
                            } else {
                                // This is the final segment, render the child
                                return Some(handler(params).await);
                            }
                        }
                    }
                }
            }

            // If no direct child match found, try to find a route that matches multiple segments
            // This handles cases where we might have nested routes like /pathA/pathB
            for i in 1..=remaining_segments.len() {
                let child_path = format!("/{}", remaining_segments[..i].join("/"));

                for (route_path, children) in &self.routes {
                    if self.path_matches(route_path, parent_path) {
                        for child in children {
                            if child.path() == child_path {
                                let params = self.extract_params(route_path, parent_path);
                                let handler = child.handler();

                                // If there are remaining segments after this match
                                if i < remaining_segments.len() {
                                    let child_result = handler(params).await;

                                    if child_result.contains("<!-- @outlet -->") {
                                        let combined_path =
                                            self.combine_paths(parent_path, child.path());
                                        if let Some(grandchild_result) = self
                                            .render_child_route_hierarchical(
                                                &combined_path,
                                                &remaining_segments[i..],
                                            )
                                            .await
                                        {
                                            return Some(
                                                child_result.replace(
                                                    "<!-- @outlet -->",
                                                    &grandchild_result,
                                                ),
                                            );
                                        }
                                    }

                                    return Some(child_result);
                                } else {
                                    return Some(handler(params).await);
                                }
                            }
                        }
                    }
                }
            }

            None
        })
    }

    /// Extract route parameters from a route pattern and actual path
    /// Used by outlet helpers to extract params for child routes
    pub fn extract_route_params(
        pattern: &str,
        path: &str,
    ) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();
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

/// Extract route parameters from a route pattern and actual path
/// Standalone function for use by generated outlet helpers
pub fn extract_route_params(
    pattern: &str,
    path: &str,
) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
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

/// Server-side outlet matching helper
/// Finds which child route should render for the given request path
pub fn server_outlet_match(
    parent_path: &str,
    request_path: &str,
    children: Vec<Box<dyn ApexRoute>>,
) -> Option<Box<dyn ApexRoute>> {
    // Remove parent path from request path to get the child path
    let child_path = if parent_path == "/" {
        request_path.to_string()
    } else if request_path.starts_with(parent_path) {
        request_path[{
            let this = &parent_path;
            this.len()
        }..]
            .to_string()
    } else {
        return None;
    };

    // Find matching child route
    children
        .into_iter()
        .find(|child| path_matches_pattern(child.path(), &child_path))
        .map(|v| v as _)
}

/// Client-side outlet matching helper
/// On client side, this would integrate with client-side routing
pub fn client_outlet_match(
    parent_path: &str,
    request_path: &str,
    children: Vec<Box<dyn ApexRoute>>,
) -> Option<Box<dyn ApexRoute>> {
    // For now, use same logic as server-side
    // In a full implementation, this would integrate with browser history API
    server_outlet_match(parent_path, request_path, children)
}

/// Get client-side outlet content
/// This would be implemented differently in a full client-side routing system
#[allow(unused_variables)]
pub fn get_client_outlet_content(parent_path: &str, request_path: &str) -> Option<String> {
    // Placeholder for client-side outlet rendering
    // In a real implementation, this would:
    // 1. Check current browser URL
    // 2. Match against client-side route definitions
    // 3. Render the appropriate component
    // 4. Return the rendered HTML
    None
}

/// Helper function to check if a path matches a route pattern
pub fn path_matches_pattern(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    if pattern_segments.len() != path_segments.len() {
        return false;
    }

    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            // Parameter segment, matches any value
            continue;
        } else if pattern_seg != path_seg {
            return false;
        }
    }

    true
}

/// Helper function to check if a path matches a route pattern as a prefix
/// This is useful for parent routes that need to match when the path continues beyond their pattern
pub fn path_matches_pattern_prefix(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    // Path must have at least as many segments as the pattern
    if path_segments.len() < pattern_segments.len() {
        return false;
    }

    // Check only the segments covered by the pattern
    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            // Parameter segment, matches any value
            continue;
        } else if pattern_seg != path_seg {
            return false;
        }
    }

    true
}

/// Get the unmatched portion of a path after matching a pattern
/// For example: pattern="/{name}/{age}", path="/john/23/calculator" returns "/calculator"
pub fn get_unmatched_path(pattern: &str, path: &str) -> String {
    let pattern_segments: Vec<&str> = pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let path_segments: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    // If path has fewer segments than pattern, return empty
    if path_segments.len() <= pattern_segments.len() {
        return String::from("/");
    }

    // Get the remaining segments after the pattern matches
    let remaining_segments = &path_segments[pattern_segments.len()..];

    if remaining_segments.is_empty() {
        String::from("/")
    } else {
        format!("/{}", remaining_segments.join("/"))
    }
}

/// Helper function to hydrate child routes with parent path context
/// This combines the parent path with child path to check against the full pathname
#[cfg(target_arch = "wasm32")]
pub fn hydrate_child_with_parent_path(
    child: &dyn ApexRoute,
    parent_path: &str,
    pathname: &str,
    expressions_map: &std::collections::HashMap<String, web_sys::Text>,
    elements_map: &std::collections::HashMap<String, web_sys::Element>,
) {
    // Combine parent and child paths to get the full path
    let parent_clean = parent_path.trim_end_matches('/');
    let child_clean = child.path().trim_start_matches('/');

    let full_child_path = if parent_clean.is_empty() || parent_clean == "/" {
        format!("/{}", child_clean)
    } else {
        format!("{}/{}", parent_clean, child_clean)
    };

    web_sys::console::log_1(
        &format!(
            "checking child - full path: {}, pathname: {}",
            full_child_path, pathname
        )
        .into(),
    );

    // Check if the full child path matches the pathname
    if path_matches_pattern(&full_child_path, pathname) {
        web_sys::console::log_1(
            &format!(
                "matched child: full path: {}, pathname: {}",
                full_child_path, pathname
            )
            .into(),
        );

        // Directly hydrate the child component
        // The counter state is maintained because we don't reset it
        child.hydrate_components(pathname, expressions_map, elements_map);
    }
}

/// Server-side stub for hydrate_child_with_parent_path
#[cfg(not(target_arch = "wasm32"))]
pub fn hydrate_child_with_parent_path(
    _child: &dyn ApexRoute,
    _parent_path: &str,
    _pathname: &str,
    _expressions_map: &std::collections::HashMap<String, web_sys::Text>,
    _elements_map: &std::collections::HashMap<String, web_sys::Element>,
) {
    // No-op on server side
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

    #[tokio::test]
    async fn test_hierarchical_routing_with_root() {
        let router = ApexRouter::new()
            .route("/", |_| async { "Root".to_string() })
            .route("/pathA", |_| async { "PathA".to_string() })
            .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() });

        // Test exact matches first
        assert_eq!(router.handle_request("/").await, Some("Root".to_string()));
        assert_eq!(
            router.handle_request("/pathA").await,
            Some("PathA".to_string())
        );
        assert_eq!(
            router.handle_request("/pathA/pathB").await,
            Some("PathA/PathB".to_string())
        );

        // Test hierarchical matching - should start from root and traverse down
        // For /pathA/pathB, should match root first, then try to find pathA/pathB
        assert_eq!(
            router.handle_request("/pathA/pathB").await,
            Some("PathA/PathB".to_string())
        );
    }

    #[tokio::test]
    async fn test_hierarchical_routing_without_root() {
        let router = ApexRouter::new()
            .route("/pathA", |_| async { "PathA".to_string() })
            .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() })
            .route("/other", |_| async { "Other".to_string() });

        // When no root route exists, should find best matching parent
        assert_eq!(
            router.handle_request("/pathA/pathB").await,
            Some("PathA/PathB".to_string())
        );

        // Should match pathA when requesting /pathA/nonexistent
        assert_eq!(
            router.handle_request("/pathA/nonexistent").await,
            Some("PathA".to_string())
        );

        // Should match other route
        assert_eq!(
            router.handle_request("/other").await,
            Some("Other".to_string())
        );
    }

    #[tokio::test]
    async fn test_hierarchical_routing_with_outlets() {
        let router = ApexRouter::new()
            .route("/", |_| async { "Root with <!-- @outlet -->".to_string() })
            .route("/dashboard", |_| async {
                "Dashboard with <!-- @outlet -->".to_string()
            });

        // Test that outlet placeholder is preserved when no child matches
        assert_eq!(
            router.handle_request("/dashboard/profile").await,
            Some("Dashboard with <!-- @outlet -->".to_string())
        );
    }

    #[tokio::test]
    async fn test_hierarchical_routing_fallback_behavior() {
        let router = ApexRouter::new()
            .route("/admin", |_| async { "Admin".to_string() })
            .route("/admin/users", |_| async { "Admin Users".to_string() })
            .route("/public", |_| async { "Public".to_string() });

        // Should find admin route when requesting /admin/users/profile
        assert_eq!(
            router.handle_request("/admin/users/profile").await,
            Some("Admin Users".to_string())
        );

        // Should find admin route when requesting /admin/settings
        assert_eq!(
            router.handle_request("/admin/settings").await,
            Some("Admin".to_string())
        );

        // Should not match anything for completely unrelated path
        assert_eq!(router.handle_request("/nonexistent/path").await, None);
    }

    #[tokio::test]
    async fn test_hierarchical_routing_with_parameters() {
        let router = ApexRouter::new()
            .route("/users/{id}", |params| async move {
                format!(
                    "User: {}",
                    params.get("id").unwrap_or(&"unknown".to_string())
                )
            })
            .route("/users/{id}/posts", |params| async move {
                format!(
                    "Posts for user: {}",
                    params.get("id").unwrap_or(&"unknown".to_string())
                )
            });

        // Should match parameterized route
        assert_eq!(
            router.handle_request("/users/123").await,
            Some("User: 123".to_string())
        );

        // Should match nested parameterized route
        assert_eq!(
            router.handle_request("/users/123/posts").await,
            Some("Posts for user: 123".to_string())
        );

        // Should fallback to parent route when child doesn't exist
        assert_eq!(
            router.handle_request("/users/123/settings").await,
            Some("User: 123".to_string())
        );
    }

    #[tokio::test]
    async fn test_hierarchical_routing_exact_matches_without_children() {
        // Test that exact matches take priority when root has no actual children defined
        let router = ApexRouter::new()
            .route("/", |_| async { "Root with <!-- @outlet -->".to_string() })
            .route("/pathA", |_| async { "PathA".to_string() })
            .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() });

        // For /pathA/pathB, should match the exact route since it exists
        // Root-first matching only applies when using mount_route with actual children
        assert_eq!(
            router.handle_request("/pathA/pathB").await,
            Some("PathA/PathB".to_string())
        );

        // Direct matches should still work
        assert_eq!(
            router.handle_request("/").await,
            Some("Root with <!-- @outlet -->".to_string())
        );

        assert_eq!(
            router.handle_request("/pathA").await,
            Some("PathA".to_string())
        );
    }

    #[tokio::test]
    async fn test_hierarchical_routing_no_root_fallback() {
        // Test that when no root route exists, router finds best entry point
        let router = ApexRouter::new()
            .route("/pathA", |_| async { "PathA".to_string() })
            .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() })
            .route("/other", |_| async { "Other".to_string() });

        // For /pathA/pathB, should match the exact route since it exists
        assert_eq!(
            router.handle_request("/pathA/pathB").await,
            Some("PathA/PathB".to_string())
        );

        // For /pathA/pathC, should fallback to /pathA since pathC doesn't exist
        assert_eq!(
            router.handle_request("/pathA/pathC").await,
            Some("PathA".to_string())
        );

        // For /pathA/pathB/pathC, should match /pathA/pathB and return its result
        assert_eq!(
            router.handle_request("/pathA/pathB/pathC").await,
            Some("PathA/PathB".to_string())
        );

        // For completely different path, should match if it exists
        assert_eq!(
            router.handle_request("/other").await,
            Some("Other".to_string())
        );

        // For non-existent path with no fallback, should return None
        assert_eq!(router.handle_request("/nonexistent").await, None);
    }

    #[tokio::test]
    async fn test_hierarchical_routing_longest_match_priority() {
        // Test that longer, more specific routes are matched before shorter ones
        let router = ApexRouter::new()
            .route("/api", |_| async { "API Root".to_string() })
            .route("/api/users", |_| async { "Users API".to_string() })
            .route("/api/users/profile", |_| async {
                "User Profile".to_string()
            });

        // Should match most specific route first
        assert_eq!(
            router.handle_request("/api/users/profile").await,
            Some("User Profile".to_string())
        );

        // Should match intermediate route
        assert_eq!(
            router.handle_request("/api/users").await,
            Some("Users API".to_string())
        );

        // Should fallback to shorter route when longer doesn't exist
        assert_eq!(
            router.handle_request("/api/users/settings").await,
            Some("Users API".to_string())
        );

        // Should fallback to root API route when no other matches
        assert_eq!(
            router.handle_request("/api/posts").await,
            Some("API Root".to_string())
        );
    }

    #[tokio::test]
    async fn test_hierarchical_routing_with_root_and_children() {
        // Test combination of root route with child routes
        let router = ApexRouter::new()
            .route("/", |_| async { "Root".to_string() })
            .route("/dashboard", |_| async { "Dashboard".to_string() })
            .route("/dashboard/settings", |_| async {
                "Dashboard Settings".to_string()
            })
            .route("/profile", |_| async { "Profile".to_string() });

        // Root should be matched first for any request
        assert_eq!(router.handle_request("/").await, Some("Root".to_string()));

        // Direct child routes should work
        assert_eq!(
            router.handle_request("/dashboard").await,
            Some("Dashboard".to_string())
        );

        // Nested routes should work
        assert_eq!(
            router.handle_request("/dashboard/settings").await,
            Some("Dashboard Settings".to_string())
        );

        // Unknown nested path should fallback to parent
        assert_eq!(
            router.handle_request("/dashboard/unknown").await,
            Some("Dashboard".to_string())
        );

        // Unknown root path should fallback to root
        assert_eq!(
            router.handle_request("/unknown").await,
            Some("Root".to_string())
        );
    }
}
