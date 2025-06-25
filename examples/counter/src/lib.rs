//! Apex Counter Example Library
//!
//! This library exposes the Counter and CounterPage components
//! for both server-side and client-side usage.

#![allow(missing_docs)]

use apex::{Html, Signal, View, component, tmpl};

/// Counter component with reactive state using signals
#[component]
pub struct Counter {
    #[signal]
    pub count: Signal<i32>,
}

impl View for Counter {
    fn render(&self) -> Html {
        let increment_handler = {
            let count = self.count.clone();
            move |_event: web_sys::Event| {
                web_sys::console::log_1(&"Incrementing".into());
                count.update(|c| *c += 1);
                web_sys::console::log_1(&format!("Count: {}", count.get()).into());
            }
        };

        let decrement_handler = {
            let count = self.count.clone();
            move |_event: web_sys::Event| {
                web_sys::console::log_1(&"Decrementing".into());
                count.update(|c| *c -= 1);
                web_sys::console::log_1(&format!("Count: {}", count.get()).into());
            }
        };

        tmpl! {
            <div class="counter">
                <p>Count: {self.count}</p>
                <button onclick={increment_handler}>Increment</button>
                <button onclick={decrement_handler}>Decrement</button>
            </div>
        }
    }
}

/// A page component that contains the counter
#[component]
pub struct CounterPage {
    #[signal]
    pub counter: Signal<i32>,
}

impl View for CounterPage {
    fn render(&self) -> Html {
        let inc = {
            let counter = self.counter.clone();
            move |_event: web_sys::Event| {
                counter.update(|c| *c += 1);
                web_sys::console::log_1(&format!("Counter: {}", counter.get()).into());
            }
        };

        tmpl! {
            <h1>Awesome stuff</h1>
            <button onclick={inc}>{self.counter}</button>
        }
    }
}
