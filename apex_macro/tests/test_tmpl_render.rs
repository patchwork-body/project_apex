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
    apex_utils::reset_counters();

    #[component]
    fn my_component(#[prop] name: &'static str) {
        tmpl! { <div>Hello, {name}!</div> }
    }

    let data = &std::collections::HashMap::new();

    assert_eq!(
        tmpl! { <MyComponent name="John" /> },
        "<div>Hello, <!-- @expr-text-begin:0 -->John<!-- @expr-text-end:0 -->!</div>"
    );
}

#[test]
fn test_component_with_dynamic_prop() {
    apex_utils::reset_counters();

    #[component]
    fn my_component(#[prop] name: Signal<String>) {
        tmpl! { <div>Hello, {name.get()}!</div> }
    }

    let signal = Signal::new("John".to_owned());

    let data = &std::collections::HashMap::new();

    assert_eq!(
        tmpl! { <MyComponent name={signal} /> },
        "<div>Hello, <!-- @expr-text-begin:0 -->John<!-- @expr-text-end:0 -->!</div>"
    );
}

#[test]
fn test_same_component_multiple_times() {
    apex_utils::reset_counters();

    #[component]
    fn my_component(#[prop] name: Signal<String>) {
        tmpl! { <div>Hello, {name.get()}!</div> }
    }

    let data = &std::collections::HashMap::new();

    assert_eq!(
        tmpl! { <MyComponent name={Signal::new("John".to_owned())} /> <MyComponent name={Signal::new("Jane".to_owned())} /> },
        "<div>Hello, <!-- @expr-text-begin:0 -->John<!-- @expr-text-end:0 -->!</div><div>Hello, <!-- @expr-text-begin:1 -->Jane<!-- @expr-text-end:1 -->!</div>"
    );
}
