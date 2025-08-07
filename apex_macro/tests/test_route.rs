#![allow(missing_docs)]

use apex_macro::{component, route};
use std::collections::HashMap;

// Test component for route testing
#[component]
fn home_component() -> String {
    "Welcome to Home!".to_owned()
}

// Test route with component
#[route(component = HomeComponent)]
fn home_route(params: HashMap<String, String>) -> String {
    // This function logic will be executed, but the component will be rendered
    let _user_id = params.get("id").unwrap_or(&"guest".to_owned());
    "Custom logic executed".to_owned()
}

// Test route without component
#[route]
fn about_route(params: HashMap<String, String>) -> String {
    format!(
        "About page for user: {}",
        params.get("id").unwrap_or(&"anonymous".to_owned())
    )
}

#[tokio::test]
async fn test_route_with_component() {
    let params = HashMap::new();
    let result = home_route(params).await;

    // Should render the HomeComponent, not the function body return value
    assert_eq!(result, "Welcome to Home!");
}

#[tokio::test]
async fn test_route_without_component() {
    let mut params = HashMap::new();
    params.insert("id".to_owned(), "testuser".to_owned());

    let result = about_route(params).await;
    assert_eq!(result, "About page for user: testuser");
}

#[tokio::test]
async fn test_route_without_component_default_param() {
    let params = HashMap::new();
    let result = about_route(params).await;
    assert_eq!(result, "About page for user: anonymous");
}
