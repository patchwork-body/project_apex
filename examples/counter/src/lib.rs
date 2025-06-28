//! Apex Counter Example Library
//!
//! This library exposes the Counter and CounterPage components
//! for both server-side and client-side usage.

#![allow(missing_docs)]

use apex::{Html, Signal, View, component, signal, tmpl};

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
