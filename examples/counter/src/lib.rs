//! Apex Counter Example Library
//!
//! This library exposes the Counter and CounterPage components
//! for both server-side and client-side usage.

#![allow(missing_docs)]

use apex::{component, signal, tmpl, Html, Signal, View};

/// A page component that contains the counter
#[component]
pub struct CounterPage {}

impl View for CounterPage {
    fn render(&self) -> Html {
        let counter = signal!(0);

        let inc = {
            let counter = counter.clone();

            move |_event: web_sys::Event| {
                counter.update(|c| *c += 1);
                web_sys::console::log_1(
                    &format!("Incremented counter to {}", counter.get()).into(),
                );
            }
        };

        tmpl! {
            <h1>Awesome</h1>
            <button onclick={inc}>Increment: {counter}</button>
        }
    }
}

// Example of the new component macro
use apex_macro::component;

#[component]
fn simple_greeting() -> Html {
    tmpl! {
        <div class="greeting">
            <h2>Hello from Component Macro!</h2>
            <p>This component was created using the new #[component] syntax.</p>
        </div>
    }
}

// You can now use SimpleGreeting as a component:
// let greeting = SimpleGreeting;
// let html = greeting.render();

// Example of using the component inside another template
pub fn example_page() -> Html {
    tmpl! {
        <div class="page">
            <h1>Welcome to Apex!</h1>
            <SimpleGreeting />
            <p>Components can be easily composed together.</p>
        </div>
    }
}

// Example of component with props
#[component]
fn user_badge(#[prop] name: &'static str, #[prop] role: &'static str) -> Html {
    tmpl! {
        <div class="user-badge">
            <span class="name">{name}</span>
            <span class="role">{role}</span>
        </div>
    }
}

// Example usage with props
pub fn team_page() -> Html {
    tmpl! {
        <div class="team">
            <h2>Our Team</h2>
            <UserBadge name="Alice" role="Developer" />
            <UserBadge name="Bob" role="Designer" />
        </div>
    }
}

// Example of component with function props
#[component]
fn button_with_handler(
    #[prop] text: &'static str,
    #[prop] on_click: std::sync::Arc<dyn Fn()>,
) -> Html {
    tmpl! {
        <button onclick={on_click}>{text}</button>
    }
}

// Example usage with function props
pub fn interactive_page() -> Html {
    let handler = std::sync::Arc::new(|| {
        println!("Button clicked!");
    }) as std::sync::Arc<dyn Fn()>;

    tmpl! {
        <div class="interactive">
            <h1>Interactive Components</h1>
            <ButtonWithHandler text="Click me!" on_click={handler.clone()} />
            <p>Check the console for click events.</p>
        </div>
    }
}
