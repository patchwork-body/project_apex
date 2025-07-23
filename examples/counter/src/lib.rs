#![allow(missing_docs)]

use apex::prelude::*;
use std::rc::Rc;

#[component]
pub fn counter() -> Html {
    let count = signal!(0);

    let inc = {
        let count = count.clone();

        Rc::new(move || {
            count.update(|c| c + 1);
        })
    };

    let dec = {
        let count = count.clone();

        Rc::new(move || {
            count.update(|c| c - 1);
        })
    };

    tmpl! {
        <div class="counter">
            <button onclick={inc}>Inc</button>
            <p class="count">{$count}</p>
            <button onclick={dec}>Dec</button>
        </div>
    }
}
