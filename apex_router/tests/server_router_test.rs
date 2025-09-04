#![allow(missing_docs)]

use apex_router::{ApexHandler, ApexServerRoute, ApexServerRouter};
use std::collections::HashMap;

// Simple test route implementation
struct TestRoute {
    path: &'static str,
    content: &'static str,
}

impl ApexServerRoute for TestRoute {
    fn path(&self) -> &'static str {
        self.path
    }
    fn handler(&self) -> ApexHandler {
        let content = self.content.to_owned();
        Box::new(move |_params: HashMap<String, String>| {
            let content = content.clone();
            Box::pin(async move { content })
        })
    }

    fn children(&self) -> Vec<Box<dyn ApexServerRoute>> {
        Vec::new()
    }
}

#[tokio::test]
async fn test_basic_routing() {
    // Mount test routes
    let route1 = TestRoute {
        path: "/",
        content: "Home",
    };

    let router = ApexServerRouter::new(&route1);

    let route2 = TestRoute {
        path: "/about",
        content: "About",
    };

    let router = ApexServerRouter::new(&route2);

    // Test root route
    let result = router.handle_request("/", "").await;
    assert_eq!(result, Some("Home".to_owned()));

    // Test about route
    let result = router.handle_request("/about", "").await;
    assert_eq!(result, Some("About".to_owned()));

    // Test non-existent route - with root fallback behavior, this returns root
    let result = router.handle_request("/nonexistent", "").await;
    // This actually falls back to root due to hierarchical routing behavior
    assert_eq!(result, Some("Home".to_owned()));
}

#[tokio::test]
async fn test_hierarchical_routing() {
    // Mount nested routes
    let route1 = TestRoute {
        path: "/dashboard",
        content: "Dashboard with <!-- @outlet-begin --><!-- @outlet-end -->",
    };

    let mut router = ApexServerRouter::new(&route1);

    // Mount nested routes
    let route1 = TestRoute {
        path: "/dashboard",
        content: "Dashboard with <!-- @outlet-begin --><!-- @outlet-end -->",
    };

    router.mount_root_route(&route1);

    let route2 = TestRoute {
        path: "/dashboard/settings",
        content: "Settings",
    };

    router.mount_root_route(&route2);

    // Test parent route
    let result = router.handle_request("/dashboard", "").await;
    assert_eq!(
        result,
        Some("Dashboard with <!-- @outlet-begin --><!-- @outlet-end -->".to_owned())
    );

    // Test child route
    let result = router.handle_request("/dashboard/settings", "").await;
    assert_eq!(result, Some("Settings".to_owned()));

    // Test hierarchical fallback - should find parent route
    let result = router.handle_request("/dashboard/nonexistent", "").await;
    assert_eq!(
        result,
        Some("Dashboard with <!-- @outlet-begin --><!-- @outlet-end -->".to_owned())
    );
}
