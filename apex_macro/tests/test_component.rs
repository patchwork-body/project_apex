#![allow(missing_docs)]
use apex::{Html, View, component, tmpl};

#[test]
fn test_primitive_component() {
    #[component]
    struct MyCounter;

    impl View for MyCounter {
        fn render(&self) -> Html {
            tmpl! {
                <div>MyCounter</div>
            }
        }
    }

    assert_eq!(
        MyCounter::new().render().to_string(),
        "<div>MyCounter</div>"
    );
}

#[test]
fn test_component_with_attributes() {
    #[component]
    struct MyComponent {
        name: String,
        count: i32,
    }

    impl View for MyComponent {
        fn render(&self) -> Html {
            tmpl! {
                <h1>{self.name}</h1>
                <p>{self.count}</p>
            }
        }
    }

    let mut counter = MyComponent::new();
    counter.set_name("MyCounter".to_owned());
    counter.set_count(10);

    assert_eq!(counter.render().to_string(), "<h1>MyCounter</h1><p>10</p>");
}

#[test]
fn test_component_with_other_component() {
    #[component]
    struct Awesome;

    impl View for Awesome {
        fn render(&self) -> Html {
            tmpl! {
                <div>Awesome</div>
            }
        }
    }

    #[component]
    struct MyComponent;

    impl View for MyComponent {
        fn render(&self) -> Html {
            tmpl! {
                <Awesome />
                <div>My Counter</div>
            }
        }
    }

    assert_eq!(
        MyComponent::new().render().to_string(),
        "<div>Awesome</div><div>My Counter</div>"
    );
}
