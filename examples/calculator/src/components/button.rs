use apex::prelude::*;

#[component]
pub fn button(
    #[prop] symbol: Signal<String>,
    #[prop(default = false)] wide: bool,
    #[prop(default = false)] primary: bool,
    #[prop(default = false)] secondary: bool,
    #[prop(default = noop_action())] onclick: Action,
    #[prop(default = noop_action())] onmousedown: Action,
    #[prop(default = noop_action())] onmouseup: Action,
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
                {symbol.get()}
            </span>
        </button>
    }
}
