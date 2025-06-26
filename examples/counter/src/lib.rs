//! Apex Counter Example Library
//!
//! This library exposes the Counter and CounterPage components
//! for both server-side and client-side usage.

#![allow(missing_docs)]

use apex::{Html, Signal, View, component, tmpl};

/// Counter component with reactive state using signals
// #[component]
// pub struct Counter {
//     #[signal]
//     pub count: Signal<i32>,
// }

// impl View for Counter {
//     fn render(&self) -> Html {
//         let increment_handler = {
//             let count = self.count.clone();
//             move |_event: web_sys::Event| count.update(|c| *c += 1)
//         };

//         let decrement_handler = {
//             let count = self.count.clone();
//             move |_event: web_sys::Event| count.update(|c| *c -= 1)
//         };

//         tmpl! {
//             <div class="counter">
//                 <p>Count: {self.count}</p>
//                 <button onclick={increment_handler}>Increment</button>
//                 <button onclick={decrement_handler}>Decrement</button>
//             </div>
//         }
//     }
// }

/// A page component that contains the counter
#[component]
pub struct CounterPage {
    #[signal]
    pub counter: Signal<i32>,
}

impl View for CounterPage {
    fn render(&self) -> Html {
        let counter = self.counter.clone(); // Clone signal for use in template

        let inc = {
            let counter = counter.clone(); // Use self.counter.clone() to avoid conflicts

            move |_event: web_sys::Event| {
                // First get the current value, then update
                let old_value = counter.get();
                counter.update(|c| *c += 1);
                web_sys::console::log_1(
                    &format!("Counter updated from {} to {}", old_value, old_value + 1).into(),
                );
            }
        };

        tmpl! {
            <h1>Awesome stuff</h1>
            <button onclick={inc}>{counter.clone()}</button>
        }
    }
}
