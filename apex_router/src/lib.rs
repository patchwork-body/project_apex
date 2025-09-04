use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Handler function type for server-side route rendering
pub type ApexHandler = Box<
    dyn Fn(HashMap<String, String>) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync,
>;

// pub use matchit;
// pub mod init_data;

// mod client_router;
// mod server_router;

// pub struct ApexRouter {
//     router: Router<ApexServerRoute>,
//     routes: Vec<(String, Vec<Box<dyn ApexServerRoute>>)>,
// }

// impl ApexRouter {
//     pub fn new() -> Self {
//         Self {
//             router: Router::new(),
//             routes: Vec::new(),
//         }
//     }

//     pub fn route<F, Fut>(mut self, path: &str, handler: F) -> Self
//     where
//         F: Fn(HashMap<String, String>) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = String> + Send + 'static,
//     {
//         let boxed_handler: ApexHandler = Box::new(move |params| Box::pin(handler(params)));

//         if let Err(e) = self.router.insert(path, boxed_handler) {
//             panic!("Failed to insert route '{path}': {e}");
//         }

//         // Store route metadata for hierarchical matching logic
//         self.routes.push((path.to_owned(), Vec::new()));

//         self
//     }

//     /// Mount a macro-generated route struct implementing `ApexRoute`
//     pub fn mount_route<R: ApexRoute>(mut self, route: R) -> Self {
//         let path = route.path();
//         let handler = route.handler();
//         let children = route.children();

//         if let Err(e) = self.router.insert(path, handler) {
//             panic!("Failed to insert route '{path}': {e}");
//         }

//         // Store route metadata for outlet handling
//         self.routes.push((path.to_owned(), children));

//         self
//     }

//     /// Combine parent and child paths for nested routing
//     fn combine_paths(&self, parent: &str, child: &str) -> String {
//         let parent = parent.trim_end_matches('/');
//         let child = child.trim_start_matches('/');

//         if parent.is_empty() {
//             format!("/{child}")
//         } else if child.is_empty() {
//             parent.to_owned()
//         } else {
//             format!("{parent}/{child}")
//         }
//     }

//     /// Main route handling method with hierarchical matching
//     ///
//     /// For incoming request /pathA/pathB:
//     /// 1. If root "/" has actual children defined, matches root first and looks for children
//     /// 2. Otherwise, tries exact matches from most specific to least specific
//     /// 3. Falls back to root as last resort if it exists
//     pub async fn handle_request(&self, path: &str, query: &str) -> Option<String> {
//         apex_utils::reset_counters();

//         let exclude_path = query
//             .split('&')
//             .find(|s| s.starts_with("exclude="))
//             .and_then(|s| s.split('=').nth(1))
//             .unwrap_or("")
//             .replace("%2F", "/"); // Handle URL-encoded slashes

//         // Try to find a matching route hierarchy
//         let html = self.find_hierarchical_match(path, &exclude_path).await?;

//         // Inject collected INIT_DATA script if any routes had data
//         #[cfg(not(target_arch = "wasm32"))]
//         {
//             let init_script = crate::init_data::generate_init_data_script();
//             if !init_script.is_empty() {
//                 // Find </head> tag and inject the script before it
//                 if let Some(head_end) = html.find("</head>") {
//                     let mut result = html.clone();
//                     result.insert_str(head_end, &init_script);
//                     return Some(result);
//                 }
//             }
//         }

//         Some(html)
//     }

//     /// Check if a route pattern matches a path (handles parameters like /{name}/{age})
//     fn path_matches(&self, pattern: &str, path: &str) -> bool {
//         let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
//         let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

//         if pattern_segments.len() != path_segments.len() {
//             return false;
//         }

//         for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
//             if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
//                 // This is a parameter, it matches any value
//                 continue;
//             }

//             if pattern_seg != path_seg {
//                 return false;
//             }
//         }

//         true
//     }

//     /// Extract parameters from a path using a pattern
//     fn extract_params(&self, pattern: &str, path: &str) -> HashMap<String, String> {
//         let mut params = HashMap::new();
//         let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
//         let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

//         for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
//             if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
//                 let param_name = &pattern_seg[1..pattern_seg.len() - 1];
//                 params.insert(param_name.to_owned(), (*path_seg).to_owned());
//             }
//         }

//         params
//     }

//     /// Find hierarchical match starting from root and traversing down
//     async fn find_hierarchical_match(&self, path: &str, exclude_path: &str) -> Option<String> {
//         let segments: Vec<&str> = path
//             .trim_start_matches('/')
//             .split('/')
//             .filter(|s| !s.is_empty())
//             .collect();

//         let exclude_segments: Vec<&str> = exclude_path
//             .trim_start_matches('/')
//             .trim_end_matches('/')
//             .split('/')
//             .filter(|s| !s.is_empty())
//             .collect();

//         if let Ok(match_result) = self.router.at(path) {
//             let params_map: HashMap<String, String> = match_result
//                 .params
//                 .iter()
//                 .map(|(k, v)| (k.to_owned(), v.to_owned()))
//                 .collect();

//             let result = (match_result.value)(params_map).await;

//             // If we have exclude segments, render only the child content
//             if !exclude_segments.is_empty()
//                 && segments.len() > exclude_segments.len()
//                 && segments[..exclude_segments.len()] == exclude_segments[..]
//             {
//                 let child_segments = &segments[exclude_segments.len()..];
//                 if !child_segments.is_empty() {
//                     return self.render_child_content_only(child_segments).await;
//                 }
//             }

//             return Some(result);
//         }

//         // If no direct match, fall back to hierarchical matching
//         // Check if any route has children defined
//         let has_children = self.routes.iter().any(|(_, children)| !children.is_empty());

//         // If any route has children, prioritize hierarchical matching
//         if has_children {
//             // Apply exclude logic before full hierarchical matching
//             if !exclude_segments.is_empty()
//                 && segments.len() > exclude_segments.len()
//                 && segments[..exclude_segments.len()] == exclude_segments[..]
//             {
//                 let child_segments = &segments[exclude_segments.len()..];
//                 if !child_segments.is_empty()
//                     && let Some(child_result) = self.render_child_content_only(child_segments).await
//                 {
//                     return Some(child_result);
//                 }
//             }

//             if let Some(result) = self.match_route_recursively("/", &segments).await {
//                 return Some(result);
//             }
//         }

//         // Handle empty segments (root path) first
//         if segments.is_empty()
//             && let Some(result) = self.match_route_recursively("/", &segments).await
//         {
//             return Some(result);
//         }

//         // Try exact matches from most specific to least specific
//         for i in (1..=segments.len()).rev() {
//             let parent_path = format!("/{}", segments[..i].join("/"));

//             if let Some(result) = self
//                 .match_route_recursively(&parent_path, &segments[i..])
//                 .await
//             {
//                 return Some(result);
//             }
//         }

//         // Fall back to root in hierarchical scenarios:
//         // 1. If we have a root with children defined (mount_route scenario)
//         // 2. If we have multiple levels of nested routes (indicating hierarchical design)
//         if !segments.is_empty() {
//             let has_nested_routes = self.routes.iter().any(|(path, _)| {
//                 path != "/" && path.matches('/').count() > 1 // More than one slash means nested
//             });
//             let is_hierarchical_path = segments.len() > 1;

//             if (has_children || is_hierarchical_path || has_nested_routes)
//                 && let Some(result) = self.match_route_recursively("/", &segments).await
//             {
//                 return Some(result);
//             }
//         }

//         None
//     }

//     /// Unified recursive method for hierarchical route matching and rendering
//     /// Combines the functionality of try_match_from_path and render_child_route_hierarchical
//     fn match_route_recursively<'a>(
//         &'a self,
//         current_path: &'a str,
//         remaining_segments: &'a [&str],
//     ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
//         Box::pin(async move {
//             // First try to match the current path directly using the matchit router
//             if let Ok(Match {
//                 value: handler,
//                 params,
//             }) = self.router.at(current_path)
//             {
//                 let params_map: HashMap<String, String> = params
//                     .iter()
//                     .map(|(k, v)| (k.to_owned(), v.to_owned()))
//                     .collect();

//                 let base_result = handler(params_map).await;

//                 // If there are no remaining segments, we're done
//                 if remaining_segments.is_empty() {
//                     return Some(base_result);
//                 }

//                 // If we have remaining segments and this route has outlets, try to render children
//                 if self.has_outlets(&base_result)
//                     && let Some(child_result) = self
//                         .render_children_recursively(current_path, remaining_segments)
//                         .await
//                 {
//                     return Some(self.replace_outlet_content(
//                         current_path,
//                         &base_result,
//                         &child_result,
//                     ));
//                 }

//                 // Return base result even if we couldn't match children
//                 return Some(base_result);
//             }

//             // If direct match failed, try to find child routes in stored metadata
//             self.render_children_recursively(current_path, remaining_segments)
//                 .await
//         })
//     }

//     /// Helper method to recursively render child routes
//     fn render_children_recursively<'a>(
//         &'a self,
//         parent_path: &'a str,
//         remaining_segments: &'a [&str],
//     ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + 'a>> {
//         Box::pin(async move {
//             if remaining_segments.is_empty() {
//                 return None;
//             }

//             // Try to match the first segment as a direct child
//             let first_segment_path = format!("/{}", remaining_segments[0]);

//             // Find the parent route in our stored metadata
//             for (route_path, children) in &self.routes {
//                 // Check if this route matches the parent path pattern
//                 if self.path_matches(route_path, parent_path) {
//                     // Look for a child route that matches the first segment
//                     for child in children {
//                         if child.path() == first_segment_path {
//                             let params = self.extract_params(route_path, parent_path);
//                             let handler = child.handler();

//                             // If there are more segments after this one
//                             if remaining_segments.len() > 1 {
//                                 let child_result = handler(params).await;

//                                 // Check if this child has outlets for further nesting
//                                 if self.has_outlets(&child_result) {
//                                     // Try to render the remaining segments as grandchildren
//                                     let grandchild_path =
//                                         self.combine_paths(parent_path, child.path());
//                                     if let Some(grandchild_result) = self
//                                         .match_route_recursively(
//                                             &grandchild_path,
//                                             &remaining_segments[1..],
//                                         )
//                                         .await
//                                     {
//                                         return Some(self.replace_outlet_content(
//                                             route_path,
//                                             &child_result,
//                                             &grandchild_result,
//                                         ));
//                                     }
//                                 }

//                                 return Some(child_result);
//                             } else {
//                                 // This is the final segment, render the child
//                                 return Some(handler(params).await);
//                             }
//                         }
//                     }
//                 }
//             }

//             // If no direct child match found, try to find a route that matches multiple segments
//             // This handles cases where we might have nested routes like /pathA/pathB
//             for i in 1..=remaining_segments.len() {
//                 let child_path = format!("/{}", remaining_segments[..i].join("/"));

//                 for (route_path, children) in &self.routes {
//                     if self.path_matches(route_path, parent_path) {
//                         for child in children {
//                             if child.path() == child_path {
//                                 let params = self.extract_params(route_path, parent_path);
//                                 let handler = child.handler();

//                                 // If there are remaining segments after this match
//                                 if i < remaining_segments.len() {
//                                     let child_result = handler(params).await;

//                                     if self.has_outlets(&child_result) {
//                                         let combined_path =
//                                             self.combine_paths(parent_path, child.path());
//                                         if let Some(grandchild_result) = self
//                                             .match_route_recursively(
//                                                 &combined_path,
//                                                 &remaining_segments[i..],
//                                             )
//                                             .await
//                                         {
//                                             return Some(self.replace_outlet_content(
//                                                 route_path,
//                                                 &child_result,
//                                                 &grandchild_result,
//                                             ));
//                                         }
//                                     }

//                                     return Some(child_result);
//                                 } else {
//                                     return Some(handler(params).await);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }

//             None
//         })
//     }

//     /// Helper method to check if content has outlets
//     fn has_outlets(&self, content: &str) -> bool {
//         content.contains("<!-- @outlet-begin -->") && content.contains("<!-- @outlet-end -->")
//     }

//     /// Helper method to replace outlet content
//     fn replace_outlet_content(
//         &self,
//         path: &str,
//         parent_content: &str,
//         child_content: &str,
//     ) -> String {
//         let outlet_begin = "<!-- @outlet-begin -->";
//         let outlet_end = "<!-- @outlet-end -->";

//         let Some(mut start) = parent_content.find(outlet_begin) else {
//             return parent_content.to_string();
//         };

//         // Add path to the outlet begin
//         let outlet_begin_with_path = format!("<!-- @outlet-begin:{path} -->");
//         let outlet_end_with_path = format!("<!-- @outlet-end:{path} -->");

//         // Replace the outlet begin with the new path
//         let parent_content = parent_content.replace(outlet_begin, &outlet_begin_with_path);
//         let parent_content = parent_content.replace(outlet_end, &outlet_end_with_path);

//         start += outlet_begin_with_path.len();

//         let Some(end) = parent_content.find(&outlet_end_with_path) else {
//             return parent_content.to_string();
//         };

//         let mut result = parent_content.to_string();
//         result.replace_range(start..end, child_content);
//         result
//     }

//     /// Render only child content without parent layout when using exclude segments
//     async fn render_child_content_only(&self, child_segments: &[&str]) -> Option<String> {
//         let child_path = format!("/{}", child_segments.join("/"));

//         // Look through our stored routes to find a child route that matches
//         for (_, children) in &self.routes {
//             for child_route in children {
//                 if child_route.path() == child_path {
//                     // Execute the child handler with empty params
//                     let handler = child_route.handler();
//                     let result = handler(HashMap::new()).await;
//                     return Some(result);
//                 }
//             }
//         }

//         None
//     }

//     /// Extract route parameters from a route pattern and actual path
//     /// Used by outlet helpers to extract params for child routes
//     pub fn extract_route_params(
//         pattern: &str,
//         path: &str,
//     ) -> std::collections::HashMap<String, String> {
//         let mut params = std::collections::HashMap::new();
//         let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
//         let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

//         for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
//             if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
//                 let param_name = &pattern_seg[1..pattern_seg.len() - 1];
//                 params.insert(param_name.to_owned(), (*path_seg).to_owned());
//             }
//         }

//         params
//     }
// }

// impl Default for ApexRouter {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// Get the matched portion of a path after matching a pattern
// For example: pattern="/{name}/{age}", path="/john/23/calculator" returns "/john/23"
pub fn get_matched_path(pattern: &str, path: &str) -> String {
    let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    if path_segments.len() < pattern_segments.len() {
        return String::from("/");
    }

    let mut matched_path = vec![];

    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        if pattern_seg == path_seg {
            matched_path.push(path_seg.to_owned());
        }
    }

    format!("/{}", matched_path.join("/"))
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
        return String::from("");
    }

    // Get the remaining segments after the pattern matches
    let remaining_segments = &path_segments[pattern_segments.len()..];

    if remaining_segments.is_empty() {
        String::from("")
    } else {
        format!("/{}", remaining_segments.join("/"))
    }
}

// /// Helper function to check if a path matches a route pattern as a prefix
// /// This is useful for parent routes that need to match when the path continues beyond their pattern
// pub fn path_matches_pattern_prefix(pattern: &str, path: &str) -> bool {
//     let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
//     let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

//     // Path must have at least as many segments as the pattern
//     if path_segments.len() < pattern_segments.len() {
//         return false;
//     }

//     // Check only the segments covered by the pattern
//     for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
//         if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
//             // Parameter segment, matches any value
//             continue;
//         } else if pattern_seg != path_seg {
//             return false;
//         }
//     }

//     true
// }

// /// Helper function to check if a path matches a route pattern
#[cfg(target_arch = "wasm32")]
pub use client_router::{
    get_unmatched_path_after_router_match, path_matches_prefix_with_router,
    path_matches_with_router, register_client_route,
};

// /// Initialize client-side routing by registering all route patterns
// /// This should be called once during application startup
// #[cfg(target_arch = "wasm32")]
// pub fn init_client_routing(route_patterns: &[&str]) {
//     for pattern in route_patterns {
//         register_client_route(pattern);
//     }
// }

// /// Optimized path matching using global matchit router (client-side only)
#[cfg(target_arch = "wasm32")]
pub fn path_matches_pattern_optimized(pattern: &str, path: &str) -> bool {
    if let Some(matched_pattern) = path_matches_with_router(path) {
        matched_pattern == pattern
    } else {
        false
    }
}

// /// Optimized prefix path matching using global matchit router (client-side only)
#[cfg(target_arch = "wasm32")]
pub fn path_matches_pattern_prefix_optimized(pattern: &str, path: &str) -> bool {
    path_matches_prefix_with_router(pattern, path)
}

// /// Optimized unmatched path extraction using router data
#[cfg(target_arch = "wasm32")]
pub fn get_unmatched_path_optimized(pattern: &str, path: &str) -> String {
    get_unmatched_path_after_router_match(pattern, path)
}

// /// Legacy path matching function - kept for compatibility
// pub fn path_matches_pattern(pattern: &str, path: &str) -> bool {
//     let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
//     let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

//     if pattern_segments.len() != path_segments.len() {
//         return false;
//     }

//     for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
//         if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
//             // Parameter segment, matches any value
//             continue;
//         }

//         if pattern_seg != path_seg {
//             return false;
//         }
//     }

//     true
// }

// /// Finds which child route should render for the given request path
// pub fn outlet_match(
//     parent_path: &str,
//     request_path: &str,
//     children: Vec<Box<dyn ApexRoute>>,
// ) -> Option<Box<dyn ApexRoute>> {
//     // Remove parent path from request path to get the child path
//     let child_path = if parent_path == "/" {
//         request_path.to_string()
//     } else if request_path.starts_with(parent_path) {
//         request_path[{
//             let this = &parent_path;
//             this.len()
//         }..]
//             .to_string()
//     } else {
//         return None;
//     };

//     // Find matching child route
//     children
//         .into_iter()
//         .find(|child| path_matches_pattern(child.path(), &child_path))
//         .map(|v| v as _)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_basic_routing() {
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Home".to_string() })
//             .route("/about", |_| async { "About".to_string() });

//         assert_eq!(
//             router.handle_request("/", "").await,
//             Some("Home".to_string())
//         );
//         assert_eq!(
//             router.handle_request("/about", "").await,
//             Some("About".to_string())
//         );
//         assert_eq!(router.handle_request("/missing", "").await, None);
//     }

//     #[tokio::test]
//     async fn test_router_builder_pattern() {
//         async fn root_handler(_params: HashMap<String, String>) -> String {
//             "Root page".to_string()
//         }

//         async fn about_handler(_params: HashMap<String, String>) -> String {
//             "About page".to_string()
//         }

//         let router = ApexRouter::new()
//             .route("/", root_handler)
//             .route("/about", about_handler);

//         assert_eq!(
//             router.handle_request("/", "").await,
//             Some("Root page".to_string())
//         );
//         assert_eq!(
//             router.handle_request("/about", "").await,
//             Some("About page".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_path_parameters() {
//         let router = ApexRouter::new()
//             .route("/users/{id}", |params| async move {
//                 format!(
//                     "User ID: {}",
//                     params.get("id").unwrap_or(&"unknown".to_string())
//                 )
//             })
//             .route("/posts/{id}/comments/{comment_id}", |params| async move {
//                 format!(
//                     "Post ID: {}, Comment ID: {}",
//                     params.get("id").unwrap_or(&"unknown".to_string()),
//                     params.get("comment_id").unwrap_or(&"unknown".to_string())
//                 )
//             });

//         assert_eq!(
//             router.handle_request("/users/123", "").await,
//             Some("User ID: 123".to_string())
//         );

//         assert_eq!(
//             router.handle_request("/posts/456/comments/789", "").await,
//             Some("Post ID: 456, Comment ID: 789".to_string())
//         );

//         assert_eq!(router.handle_request("/users", "").await, None);
//     }

//     #[tokio::test]
//     async fn test_wildcard_routes() {
//         let router = ApexRouter::new().route("/static/{*filepath}", |params| async move {
//             format!(
//                 "Static file: {}",
//                 params.get("filepath").unwrap_or(&"index.html".to_string())
//             )
//         });

//         assert_eq!(
//             router.handle_request("/static/css/style.css", "").await,
//             Some("Static file: css/style.css".to_string())
//         );

//         assert_eq!(
//             router.handle_request("/static/js/app.js", "").await,
//             Some("Static file: js/app.js".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_path_combination() {
//         let router = ApexRouter::new();

//         assert_eq!(router.combine_paths("/", "profile"), "/profile");
//         assert_eq!(
//             router.combine_paths("/dashboard", "profile"),
//             "/dashboard/profile"
//         );
//         assert_eq!(
//             router.combine_paths("/dashboard/", "/profile"),
//             "/dashboard/profile"
//         );
//         assert_eq!(
//             router.combine_paths("/dashboard", "/profile"),
//             "/dashboard/profile"
//         );
//         assert_eq!(router.combine_paths("", "profile"), "/profile");
//         assert_eq!(router.combine_paths("/dashboard", ""), "/dashboard");
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_with_root() {
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Root".to_string() })
//             .route("/pathA", |_| async { "PathA".to_string() })
//             .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() });

//         // Test exact matches first
//         assert_eq!(
//             router.handle_request("/", "").await,
//             Some("Root".to_string())
//         );
//         assert_eq!(
//             router.handle_request("/pathA", "").await,
//             Some("PathA".to_string())
//         );
//         assert_eq!(
//             router.handle_request("/pathA/pathB", "").await,
//             Some("PathA/PathB".to_string())
//         );

//         // Test hierarchical matching - should start from root and traverse down
//         // For /pathA/pathB, should match root first, then try to find pathA/pathB
//         assert_eq!(
//             router.handle_request("/pathA/pathB", "").await,
//             Some("PathA/PathB".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_without_root() {
//         let router = ApexRouter::new()
//             .route("/pathA", |_| async { "PathA".to_string() })
//             .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() })
//             .route("/other", |_| async { "Other".to_string() });

//         // When no root route exists, should find best matching parent
//         assert_eq!(
//             router.handle_request("/pathA/pathB", "").await,
//             Some("PathA/PathB".to_string())
//         );

//         // Should match pathA when requesting /pathA/nonexistent
//         assert_eq!(
//             router.handle_request("/pathA/nonexistent", "").await,
//             Some("PathA".to_string())
//         );

//         // Should match other route
//         assert_eq!(
//             router.handle_request("/other", "").await,
//             Some("Other".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_with_outlets() {
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Root with <!-- @outlet -->".to_string() })
//             .route("/dashboard", |_| async {
//                 "Dashboard with <!-- @outlet -->".to_string()
//             });

//         // Test that outlet placeholder is preserved when no child matches
//         assert_eq!(
//             router.handle_request("/dashboard/profile", "").await,
//             Some("Dashboard with <!-- @outlet -->".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_fallback_behavior() {
//         let router = ApexRouter::new()
//             .route("/admin", |_| async { "Admin".to_string() })
//             .route("/admin/users", |_| async { "Admin Users".to_string() })
//             .route("/public", |_| async { "Public".to_string() });

//         // Should find admin route when requesting /admin/users/profile
//         assert_eq!(
//             router.handle_request("/admin/users/profile", "").await,
//             Some("Admin Users".to_string())
//         );

//         // Should find admin route when requesting /admin/settings
//         assert_eq!(
//             router.handle_request("/admin/settings", "").await,
//             Some("Admin".to_string())
//         );

//         // Should not match anything for completely unrelated path
//         assert_eq!(router.handle_request("/nonexistent/path", "").await, None);
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_with_parameters() {
//         let router = ApexRouter::new()
//             .route("/users/{id}", |params| async move {
//                 format!(
//                     "User: {}",
//                     params.get("id").unwrap_or(&"unknown".to_string())
//                 )
//             })
//             .route("/users/{id}/posts", |params| async move {
//                 format!(
//                     "Posts for user: {}",
//                     params.get("id").unwrap_or(&"unknown".to_string())
//                 )
//             });

//         // Should match parameterized route
//         assert_eq!(
//             router.handle_request("/users/123", "").await,
//             Some("User: 123".to_string())
//         );

//         // Should match nested parameterized route
//         assert_eq!(
//             router.handle_request("/users/123/posts", "").await,
//             Some("Posts for user: 123".to_string())
//         );

//         // Should fallback to parent route when child doesn't exist
//         assert_eq!(
//             router.handle_request("/users/123/settings", "").await,
//             Some("User: 123".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_exact_matches_without_children() {
//         // Test that exact matches take priority when root has no actual children defined
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Root with <!-- @outlet -->".to_string() })
//             .route("/pathA", |_| async { "PathA".to_string() })
//             .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() });

//         // For /pathA/pathB, should match the exact route since it exists
//         // Root-first matching only applies when using mount_route with actual children
//         assert_eq!(
//             router.handle_request("/pathA/pathB", "").await,
//             Some("PathA/PathB".to_string())
//         );

//         // Direct matches should still work
//         assert_eq!(
//             router.handle_request("/", "").await,
//             Some("Root with <!-- @outlet -->".to_string())
//         );

//         assert_eq!(
//             router.handle_request("/pathA", "").await,
//             Some("PathA".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_no_root_fallback() {
//         // Test that when no root route exists, router finds best entry point
//         let router = ApexRouter::new()
//             .route("/pathA", |_| async { "PathA".to_string() })
//             .route("/pathA/pathB", |_| async { "PathA/PathB".to_string() })
//             .route("/other", |_| async { "Other".to_string() });

//         // For /pathA/pathB, should match the exact route since it exists
//         assert_eq!(
//             router.handle_request("/pathA/pathB", "").await,
//             Some("PathA/PathB".to_string())
//         );

//         // For /pathA/pathC, should fallback to /pathA since pathC doesn't exist
//         assert_eq!(
//             router.handle_request("/pathA/pathC", "").await,
//             Some("PathA".to_string())
//         );

//         // For /pathA/pathB/pathC, should match /pathA/pathB and return its result
//         assert_eq!(
//             router.handle_request("/pathA/pathB/pathC", "").await,
//             Some("PathA/PathB".to_string())
//         );

//         // For completely different path, should match if it exists
//         assert_eq!(
//             router.handle_request("/other", "").await,
//             Some("Other".to_string())
//         );

//         // For non-existent path with no fallback, should return None
//         assert_eq!(router.handle_request("/nonexistent", "").await, None);
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_longest_match_priority() {
//         // Test that longer, more specific routes are matched before shorter ones
//         let router = ApexRouter::new()
//             .route("/api", |_| async { "API Root".to_string() })
//             .route("/api/users", |_| async { "Users API".to_string() })
//             .route("/api/users/profile", |_| async {
//                 "User Profile".to_string()
//             });

//         // Should match most specific route first
//         assert_eq!(
//             router.handle_request("/api/users/profile", "").await,
//             Some("User Profile".to_string())
//         );

//         // Should match intermediate route
//         assert_eq!(
//             router.handle_request("/api/users", "").await,
//             Some("Users API".to_string())
//         );

//         // Should fallback to shorter route when longer doesn't exist
//         assert_eq!(
//             router.handle_request("/api/users/settings", "").await,
//             Some("Users API".to_string())
//         );

//         // Should fallback to root API route when no other matches
//         assert_eq!(
//             router.handle_request("/api/posts", "").await,
//             Some("API Root".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_hierarchical_routing_with_root_and_children() {
//         // Test combination of root route with child routes
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Root".to_string() })
//             .route("/dashboard", |_| async { "Dashboard".to_string() })
//             .route("/dashboard/settings", |_| async {
//                 "Dashboard Settings".to_string()
//             })
//             .route("/profile", |_| async { "Profile".to_string() });

//         // Root should be matched first for any request
//         assert_eq!(
//             router.handle_request("/", "").await,
//             Some("Root".to_string())
//         );

//         // Direct child routes should work
//         assert_eq!(
//             router.handle_request("/dashboard", "").await,
//             Some("Dashboard".to_string())
//         );

//         // Nested routes should work
//         assert_eq!(
//             router.handle_request("/dashboard/settings", "").await,
//             Some("Dashboard Settings".to_string())
//         );

//         // Unknown nested path should fallback to parent
//         assert_eq!(
//             router.handle_request("/dashboard/unknown", "").await,
//             Some("Dashboard".to_string())
//         );

//         // Unknown root path should fallback to root
//         assert_eq!(
//             router.handle_request("/unknown", "").await,
//             Some("Root".to_string())
//         );
//     }

//     #[tokio::test]
//     async fn test_exclude_segments_functionality() {
//         // Test the exclude segments functionality
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Root".to_string() })
//             .route("/calculator", |_| async { "Calculator".to_string() })
//             .route("/john/23/calculator", |_| async {
//                 "Full Path Match".to_string()
//             });

//         // Normal request without exclude should match the exact full path if it exists
//         assert_eq!(
//             router.handle_request("/john/23/calculator", "").await,
//             Some("Full Path Match".to_string()) // Should match the exact full path route
//         );

//         // Request with exclude segments should filter out /john/23 and match only /calculator
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=john/23")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding /john/23
//         );

//         // Test with different exclude segments
//         assert_eq!(
//             router
//                 .handle_request("/user/42/calculator", "exclude=user/42")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding /user/42
//         );

//         // Test when exclude segments don't match the beginning of the path
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=wrong/path")
//                 .await,
//             Some("Full Path Match".to_string()) // Should match full path since exclude doesn't apply
//         );

//         // Test with a path that doesn't have a full match but has exclude segments
//         assert_eq!(
//             router
//                 .handle_request("/other/path/calculator", "exclude=other/path")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding /other/path
//         );
//     }

//     #[tokio::test]
//     async fn test_exclude_segments_with_slashes() {
//         // Test the exclude segments functionality with leading/trailing slashes like in real URLs
//         let router = ApexRouter::new()
//             .route("/", |_| async { "Root".to_string() })
//             .route("/calculator", |_| async { "Calculator".to_string() });

//         // Test with leading slash: exclude=/john/23/
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=/john/23/")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding /john/23/
//         );

//         // Test with leading slash only: exclude=/john/23
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=/john/23")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding /john/23
//         );

//         // Test with trailing slash only: exclude=john/23/
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=john/23/")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding john/23/
//         );

//         // Test with URL-encoded slashes: exclude=%2Fjohn%2F23%2F
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=%2Fjohn%2F23%2F")
//                 .await,
//             Some("Calculator".to_string()) // Should match /calculator after excluding URL-encoded /john/23/
//         );
//     }

//     #[tokio::test]
//     async fn test_exclude_segments_realistic_hierarchical_scenario() {
//         // This test simulates the real calculator app scenario where:
//         // - There's a root route /{name}/{age} with children
//         // - /calculator is only a child route, not standalone

//         // Create a mock route structure similar to the calculator app
//         struct MockRootRoute {
//             path: String,
//             children: Vec<Box<dyn ApexRoute>>,
//         }

//         struct MockChildRoute {
//             path: String,
//         }

//         impl ApexRoute for MockRootRoute {
//             fn path(&self) -> &'static str {
//                 // This would normally be "/{name}/{age}" but we'll use a simple version for testing
//                 "/test/route"
//             }

//             fn handler(&self) -> ApexHandler {
//                 Box::new(|_| {
//                     Box::pin(async move {
//                         "Root with <!-- @outlet-begin --><!-- @outlet-end -->".to_string()
//                     })
//                 })
//             }

//             fn children(&self) -> Vec<Box<dyn ApexRoute>> {
//                 vec![Box::new(MockChildRoute {
//                     path: "/calculator".to_string(),
//                 })]
//             }

//             fn hydrate_components(
//                 &self,
//                 _pathname: &str,
//                 _exclude_path: &str,
//                 _expressions_map: &HashMap<String, web_sys::Text>,
//                 _elements_map: &HashMap<String, web_sys::Element>,
//             ) {
//             }
//         }

//         impl ApexRoute for MockChildRoute {
//             fn path(&self) -> &'static str {
//                 "/calculator"
//             }

//             fn handler(&self) -> ApexHandler {
//                 Box::new(|_| Box::pin(async move { "Calculator Content".to_string() }))
//             }

//             fn hydrate_components(
//                 &self,
//                 _pathname: &str,
//                 _exclude_path: &str,
//                 _expressions_map: &HashMap<String, web_sys::Text>,
//                 _elements_map: &HashMap<String, web_sys::Element>,
//             ) {
//             }
//         }

//         let router = ApexRouter::new().mount_route(MockRootRoute {
//             path: "/test/route".to_string(),
//             children: vec![],
//         });

//         // This should work: hierarchical route with child
//         assert_eq!(
//             router.handle_request("/test/route/calculator", "").await,
//             Some(
//                 "Root with <!-- @outlet-begin -->Calculator Content<!-- @outlet-end -->"
//                     .to_string()
//             )
//         );

//         // This should also work with exclude segments - the key insight is that
//         // we need to match the pattern but render with filtered segments
//         assert_eq!(
//             router
//                 .handle_request("/john/23/calculator", "exclude=john/23")
//                 .await,
//             None // This should be None because /calculator doesn't exist as standalone
//         );
//     }
// }

mod client_router;
pub mod init_data;
mod server_router;

pub use client_router::{ApexClientRoute, ApexClientRouter};
pub use server_router::{ApexServerRoute, ApexServerRouter};
