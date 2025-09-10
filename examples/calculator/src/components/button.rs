use apex::prelude::*;

#[component]
pub fn button(
    #[prop] symbol: Signal<String>,
    #[prop(default)] wide: bool,
    #[prop(default)] primary: bool,
    #[prop(default)] secondary: bool,
    #[prop(default)] onclick: EventHandler<apex::web_sys::Event>,
    #[prop(default)] onmousedown: EventHandler<apex::web_sys::Event>,
    #[prop(default)] onmouseup: EventHandler<apex::web_sys::Event>,
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

    if let Some(data_value) = props.get("data-value").as_ref() {
        let data_value = data_value.downcast_ref::<String>();
        apex::web_sys::console::log_1(&data_value.into());
    }

    tmpl! {
        <button type="button" class={classes.join(" ")} onclick={onclick} onmousedown={onmousedown} onmouseup={onmouseup}>
            <span class="button-symbol">
                {symbol.get()}
            </span>
        </button>
    }
}
