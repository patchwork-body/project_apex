#![allow(missing_docs)]

use apex::prelude::*;
use std::rc::Rc;

#[component]
pub fn button(#[prop] onclick: Rc<dyn Fn()>) -> Html {
    tmpl! {
        <button onclick={onclick}>Inc</button>
    }
}

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
        <div>
            <p>{$count}</p>
        </div>
    }
}
