use apex::prelude::*;

#[component]
pub fn button(
    #[prop(default)] wide: bool,
    #[prop(default)] primary: bool,
    #[prop(default)] secondary: bool,
    #[prop(default)] onclick: EventHandler<apex::web_sys::MouseEvent>,
    #[prop(default)] onmousedown: EventHandler<apex::web_sys::MouseEvent>,
    #[prop(default)] onmouseup: EventHandler<apex::web_sys::MouseEvent>,
) {
    let mut classes = vec!["button"];

    if wide {
        classes.push("button-wide");
    }

    if primary {
        classes.push("button-primary");
    }

    if secondary {
        classes.push("button-secondary");
    }

    tmpl! {
        <button type="button" class={classes.join(" ")} onclick={onclick} onmousedown={onmousedown} onmouseup={onmouseup}>
            <span class="button-symbol">
                <#slot />
            </span>
        </button>
    }
}
