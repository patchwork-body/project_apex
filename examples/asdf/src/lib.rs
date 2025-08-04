#![allow(missing_docs)]

use apex::{prelude::*, wasm_bindgen};
use wasm_bindgen::prelude::*;

#[component]
pub fn Title() {
    tmpl2! {
        <h1>Hello, World!</h1>
    }
}

#[component]
pub fn counter() {
    let counter = signal!(0);

    let onclick = action!(counter => |_event| {
        counter.update(|c| c + 1);
    });

    tmpl2! {
        <div>
            <Title />
            <h1>Hello, {counter.get()}!</h1>
            <button data-counter={counter.get()} onclick={onclick}>Click me {counter.get()}</button>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        let t = tmpl2! {
            <Counter />
        };

        apex::Apex::hydrate2(t);
    }
}
