//! Server-side router implementation for the Apex framework.
//!
//! This module provides the core routing functionality for server-side applications,
//! handling route registration, hierarchical route matching, and request processing.

use matchit::{Match, Router};
use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{get_matched_path, init_data::generate_init_data_script};

/// Type alias for server-side route handlers.
///
/// A server handler is a boxed closure that takes route parameters as a HashMap
/// and returns a pinned future that resolves to an HTML string response.
pub type ApexServerHandler = Box<
    dyn Fn(HashMap<String, String>) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync,
>;

/// Trait defining the interface for server-side routes.
///
/// This trait must be implemented by all route types that can be registered
/// with the server router. It provides methods for retrieving the route path,
/// handler function, and child routes for hierarchical routing.
pub trait ApexServerRoute {
    /// Returns the path pattern for this route.
    ///
    /// The path can include parameter patterns like `/users/{id}` for dynamic routing.
    /// Defaults to the root path "/" if not overridden.
    fn path(&self) -> &'static str {
        "/"
    }

    /// Returns the handler function for this route.
    ///
    /// The handler receives route parameters extracted from the URL and returns
    /// a future that resolves to an HTML string response. Defaults to an empty
    /// response if not overridden.
    fn handler(&self) -> ApexServerHandler {
        Box::new(|_| Box::pin(async { "".to_owned() }))
    }

    /// Returns child routes for hierarchical routing.
    ///
    /// Child routes are nested under this route's path and are processed
    /// in order during route resolution. Defaults to no children if not overridden.
    fn children(&self) -> Vec<Box<dyn ApexServerRoute>> {
        Vec::new()
    }
}

/// Internal structure representing a route chain for hierarchical routing.
///
/// This structure stores the handler function and parent path information
/// needed to properly resolve nested routes and maintain the route hierarchy.
struct RouteChain {
    /// Optional chain of parent paths leading to this route.
    /// Used for hierarchical route resolution and outlet rendering.
    parent_pattern: Option<Vec<String>>,
    /// The handler function that processes requests for this route.
    handler: ApexServerHandler,
}

impl std::fmt::Debug for RouteChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteChain")
            .field("parent_path", &self.parent_pattern)
            .field("handler", &"<ApexServerHandler>")
            .finish()
    }
}

/// Main server-side router for the Apex framework.
///
/// This router handles registration of routes and their hierarchical resolution.
/// It uses the matchit crate for efficient path matching and supports nested
/// routes with outlet-based content composition.
pub struct ApexServerRouter {
    /// Internal router instance that handles path matching and route storage.
    router: Router<RouteChain>,
}

impl std::fmt::Debug for ApexServerRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApexServerRouter")
            .field("router", &"<Router<RouteChain>>")
            .finish()
    }
}

impl ApexServerRouter {
    /// Creates a new server router with the given root route.
    ///
    /// # Arguments
    ///
    /// * `route` - The root route to mount at the router's base
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let router = ApexServerRouter::new(&my_root_route);
    /// ```
    pub fn new(route: &dyn ApexServerRoute) -> Self {
        let mut r = Self {
            router: Router::new(),
        };

        r.mount_root_route(route);

        r
    }

    /// Mounts a route as the root route of the router.
    ///
    /// This is a convenience method that calls `mount_route` with no parent path.
    ///
    /// # Arguments
    ///
    /// * `route` - The route to mount as the root route
    pub fn mount_root_route(&mut self, route: &dyn ApexServerRoute) {
        self.mount_route(route, None);
    }

    /// Mounts a route and its children into the router.
    ///
    /// This method recursively mounts a route and all its child routes,
    /// building the complete route hierarchy. Each route is registered
    /// with the internal matchit router for efficient path matching.
    ///
    /// # Arguments
    ///
    /// * `route` - The route to mount
    /// * `parent_path` - Optional parent path chain for nested routes
    ///
    /// # Panics
    ///
    /// Panics if a route path conflicts with an already registered route.
    pub fn mount_route(
        &mut self,
        route: &dyn ApexServerRoute,
        parent_pattern: Option<Vec<String>>,
    ) {
        let path = route.path();

        let handler = route.handler();
        let children = route.children();

        let route_chain = RouteChain {
            parent_pattern: parent_pattern.clone(),
            handler,
        };

        let mut parent_path = parent_pattern.unwrap_or_default();
        let mut route_path = String::new();

        for part in parent_path.iter() {
            route_path.push_str(part.trim_end_matches("/"));
        }

        route_path.push_str(path);

        if let Err(e) = self.router.insert(&route_path, route_chain) {
            panic!("Failed to insert route '{path}': {e}");
        }

        parent_path.push(route_path);

        for child in children.iter() {
            self.mount_route(child.as_ref(), parent_path.clone().into());
        }
    }

    /// Handles an incoming HTTP request by matching the path and executing handlers.
    ///
    /// This method performs hierarchical route resolution, executing parent route
    /// handlers before child handlers to build the complete HTML response. The
    /// method supports outlet-based content composition for nested layouts.
    ///
    /// # Arguments
    ///
    /// * `path` - The request path to match against registered routes
    /// * `_query` - Query string parameters (currently unused)
    ///
    /// # Returns
    ///
    /// Returns `Some(String)` containing the HTML response if a route matches,
    /// or `None` if no route is found for the given path.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let response = router.handle_request("/users/123", "").await;
    /// ```
    pub async fn handle_request(&self, path: &str, query: &str) -> Option<String> {
        apex_utils::reset_counters();

        let exclude_path = query
            .split('&')
            .find(|s| s.starts_with("exclude="))
            .and_then(|s| s.split('=').nth(1))
            .unwrap_or("")
            .replace("%2F", "/"); // Handle URL-encoded slashes

        println!("exclude_path: {exclude_path}");

        if let Ok(route_match) = self.router.at(path) {
            let mut html = String::new();

            if let Some(parent_patterns_chain) = route_match.value.parent_pattern.as_ref() {
                for parent_pattern in parent_patterns_chain.iter() {
                    let matched_path = get_matched_path(parent_pattern, path);

                    if matched_path == exclude_path {
                        continue;
                    }

                    if let Ok(parent_route_match) = self.router.at(&matched_path) {
                        self.update_html(parent_route_match, &mut html).await;
                    }
                }
            }

            self.update_html(route_match, &mut html).await;

            let init_data_script = generate_init_data_script();

            // Inject the init data script into the HTML
            if !init_data_script.is_empty() {
                // Try to inject before closing </head> tag first
                if let Some(head_pos) = html.find("</head>") {
                    html.insert_str(head_pos, &init_data_script);
                } else if let Some(body_pos) = html.find("</body>") {
                    // If no </head> tag, inject before closing </body> tag
                    html.insert_str(body_pos, &init_data_script);
                } else {
                    // If no head or body tags, append to the end
                    html.push_str(&init_data_script);
                }
            }

            return Some(html);
        };

        None
    }

    /// Updates the HTML response by executing a route handler and composing content.
    ///
    /// This method extracts route parameters, executes the route handler, and
    /// integrates the result into the existing HTML response. It handles both
    /// outlet-based composition (for nested layouts) and simple concatenation.
    ///
    /// # Arguments
    ///
    /// * `route_match` - The matched route containing handler and parameters
    /// * `html` - Mutable reference to the HTML response being built
    async fn update_html(&self, route_match: Match<'_, '_, &RouteChain>, html: &mut String) {
        let handler = route_match.value.handler.as_ref();

        let params_map: HashMap<String, String> = route_match
            .params
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();

        let parent_path = route_match
            .value
            .parent_pattern
            .clone()
            .unwrap_or_default()
            .join("");

        println!("params map: {params_map:?}");

        let child_html = handler(params_map).await;

        if html.contains("<!-- @outlet-begin -->") && html.contains("<!-- @outlet-end -->") {
            Self::replace_outlet_content(&parent_path, html, &child_html);
        } else if html.is_empty() {
            html.push_str(&child_html);
        }
    }

    /// Replaces outlet content in parent HTML with child content.
    ///
    /// This method implements outlet-based content composition by finding
    /// outlet markers in the parent content and replacing the content between
    /// them with the child content. It also adds path information to the
    /// outlet markers for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `path` - The path identifier to add to outlet markers
    /// * `parent_content` - Mutable reference to the parent HTML content
    /// * `child_content` - The child content to insert into the outlet
    ///
    /// # Outlet Format
    ///
    /// Outlets are defined using HTML comments:
    /// ```html
    /// <!-- @outlet-begin -->
    /// Content to be replaced
    /// <!-- @outlet-end -->
    /// ```
    ///
    /// After replacement, they become:
    /// ```html
    /// <!-- @outlet-begin:/path -->
    /// Child content
    /// <!-- @outlet-end:/path -->
    /// ```
    fn replace_outlet_content(path: &str, parent_content: &mut String, child_content: &str) {
        let outlet_begin = "<!-- @outlet-begin -->";
        let outlet_end = "<!-- @outlet-end -->";

        let Some(mut start) = parent_content.find(outlet_begin) else {
            return;
        };

        // Add path to the outlet begin
        let outlet_begin_with_path = format!("<!-- @outlet-begin:{path} -->");
        let outlet_end_with_path = format!("<!-- @outlet-end:{path} -->");

        // Replace the outlet begin with the new path
        *parent_content = parent_content.replace(outlet_begin, &outlet_begin_with_path);
        *parent_content = parent_content.replace(outlet_end, &outlet_end_with_path);

        start += outlet_begin_with_path.len();

        let Some(end) = parent_content.find(&outlet_end_with_path) else {
            return;
        };

        parent_content.replace_range(start..end, child_content);
    }
}
