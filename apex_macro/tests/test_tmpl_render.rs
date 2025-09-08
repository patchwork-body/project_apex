#![allow(missing_docs)]

use apex::prelude::*;

#[test]
fn test_text() {
    assert_eq!(tmpl! { Hello, world! }, "Hello, world!");
}

#[test]
fn test_element() {
    assert_eq!(
        tmpl! { <div>Hello, world!</div> },
        "<div>Hello, world!</div>"
    );
}

#[test]
fn test_element_with_attributes() {
    assert_eq!(
        tmpl! { <div id="test" class="test">Hello, world!</div> },
        "<div class=\"test\"id=\"test\">Hello, world!</div>"
    );
}

#[test]
fn test_component() {
    #[component]
    fn my_component() {
        tmpl! { <div>Hello, world!</div> }
    }

    let data = &std::collections::HashMap::new();

    assert_eq!(tmpl! { <MyComponent /> }, "<div>Hello, world!</div>");
}

#[test]
fn test_component_with_static_prop() {
    #[component]
    fn my_component(#[prop] name: &'static str) {
        tmpl! { <div>Hello, {name}!</div> }
    }

    let data = &std::collections::HashMap::new();

    let result = tmpl! { <MyComponent name="John" /> };

    // Check that the result contains the expected structure with expression comments
    assert!(result.contains("<div>Hello, <!-- @expr-text-begin:"));
    assert!(result.contains("-->John<!-- @expr-text-end:"));
    assert!(result.contains("-->!</div>"));
}

#[test]
fn test_component_with_dynamic_prop() {
    #[component]
    fn my_component(#[prop] name: Signal<String>) {
        tmpl! { <div>Hello, {name.get()}!</div> }
    }

    let signal = Signal::new("John".to_owned());

    let data = &std::collections::HashMap::new();

    let result = tmpl! { <MyComponent name={signal} /> };

    // Check that the result contains the expected structure with expression comments
    assert!(result.contains("<div>Hello, <!-- @expr-text-begin:"));
    assert!(result.contains("-->John<!-- @expr-text-end:"));
    assert!(result.contains("-->!</div>"));
}

#[test]
fn test_same_component_multiple_times() {
    #[component]
    fn my_component(#[prop] name: Signal<String>) {
        tmpl! { <div>Hello, {name.get()}!</div> }
    }

    let data = &std::collections::HashMap::new();

    let result = tmpl! { <MyComponent name={Signal::new("John".to_owned())} /> <MyComponent name={Signal::new("Jane".to_owned())} /> };

    // Check that both components are rendered with expression comments
    assert!(result.contains("-->John<!-- @expr-text-end:"));
    assert!(result.contains("-->Jane<!-- @expr-text-end:"));
    // Verify we have two separate divs
    assert_eq!(
        result.matches("<div>Hello, <!-- @expr-text-begin:").count(),
        2
    );
}
