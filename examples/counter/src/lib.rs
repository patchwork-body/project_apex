#![allow(missing_docs)]

use apex::prelude::*;

#[component]
pub fn counter() -> Html {
    let count = signal!(0);

    let inc = action!(count => {
        count.update(|c| c + 1);
    });

    let dec = action!(count => {
        count.update(|c| c - 1);
    });

    tmpl! {
        <div class="counter">
            <button onclick={inc}>Inc</button>
            <p class="count">{$count}</p>
            <button onclick={dec}>Dec</button>
        </div>
    }
}
