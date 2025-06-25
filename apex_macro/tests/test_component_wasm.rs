#![allow(missing_docs)]
#![allow(dead_code)]
use apex::{Html, Signal, View, component, tmpl};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_component_with_event_handler() {
    #[component]
    struct EventComponent {
        #[signal]
        count: Signal<i32>,
    }

    impl View for EventComponent {
        fn render(&self) -> Html {
            let _inc_count = {
                let count = self.count.clone();
                move |_event: apex::web_sys::Event| count.update(|c| *c += 1)
            };

            tmpl! {
                <button onclick={_inc_count}>{self.count}</button>
            }
        }
    }

    let component = EventComponent::new();
    component.set_count(0);

    // Test initial render
    let html = component.render().to_string();
    assert!(html.contains("<button"));
    assert!(html.contains("0"));

    // Test signal updates work
    component.count.set(10);
    let html = component.render().to_string();
    assert!(html.contains("10"));

    // Test manual increment
    component.count.update(|c| *c += 1);
    let html = component.render().to_string();
    assert!(html.contains("11"));
}

#[wasm_bindgen_test]
fn test_dom_interactions() {
    #[component]
    struct InteractiveComponent {
        #[signal]
        clicked: Signal<bool>,
    }

    impl View for InteractiveComponent {
        fn render(&self) -> Html {
            let _toggle_clicked = {
                let clicked = self.clicked.clone();
                move |_event: apex::web_sys::Event| clicked.update(|c| *c = !*c)
            };

            let clicked_text = if self.clicked.get() {
                "Clicked!"
            } else {
                "Click me"
            };
            let status_text = if self.clicked.get() {
                "Button was clicked"
            } else {
                "Button not clicked"
            };

            tmpl! {
                <div>
                    <button id="test-btn" onclick={_toggle_clicked}>
                        {clicked_text}
                    </button>
                    <p>{status_text}</p>
                </div>
            }
        }
    }

    let component = InteractiveComponent::new();

    // Test initial state
    let html = component.render().to_string();
    assert!(html.contains("Click me"));
    assert!(html.contains("Button not clicked"));

    // Simulate click by updating signal directly
    component.clicked.set(true);
    let html = component.render().to_string();
    assert!(html.contains("Clicked!"));
    assert!(html.contains("Button was clicked"));

    // Test toggle back
    component.clicked.set(false);
    let html = component.render().to_string();
    assert!(html.contains("Click me"));
    assert!(html.contains("Button not clicked"));
}
