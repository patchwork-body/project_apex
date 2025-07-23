#![allow(missing_docs)]

use apex::wasm_bindgen::JsCast;
use std::rc::Rc;

use apex::prelude::*;

#[component]
pub fn Button(
    #[prop] symbol: String,
    #[prop(default = false)] wide: bool,
    #[prop(default = false)] primary: bool,
    #[prop(default = false)] secondary: bool,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onclick: Rc<dyn Fn(web_sys::Event)>,
) -> Html {
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
        <button type="button" class={classes.join(" ")} onclick={onclick}>
            <span class="button-symbol">
                {&symbol}
            </span>
        </button>
    }
}

#[component]
pub fn calculator() -> Html {
    let value = signal!("0".to_owned());

    let add_symbol = action!(value => |event| {
        let symbol = event.current_target().unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap().inner_text();

        value.update(|v| {
            if v == "0" {
                symbol
            } else {
                format!("{v}{symbol}")
            }
        });
    });

    tmpl! {
        <div class="calculator">
            <div class="display">
                {$value}
            </div>

            <div class="buttons">
                <Button secondary={true} symbol={if value.get() == "0" { "AC".to_string() } else { "<-".to_string() }} />
                <Button secondary={true} symbol="±" />
                <Button secondary={true} symbol="%" />
                <Button primary={true} symbol="÷" />
                <Button symbol="7" onclick={add_symbol.clone()} />
                <Button symbol="8" onclick={add_symbol.clone()} />
                <Button symbol="9" onclick={add_symbol.clone()} />
                <Button primary={true} symbol="×" />
                <Button symbol="4" onclick={add_symbol.clone()} />
                <Button symbol="5" onclick={add_symbol.clone()} />
                <Button symbol="6" onclick={add_symbol.clone()} />
                <Button primary={true} symbol="-" />
                <Button symbol="1" onclick={add_symbol.clone()} />
                <Button symbol="2" onclick={add_symbol.clone()} />
                <Button symbol="3" onclick={add_symbol.clone()} />
                <Button primary={true} symbol="+" />
                <Button wide={true} symbol="0" onclick={add_symbol.clone()} />
                <Button symbol="." onclick={add_symbol.clone()} />
                <Button primary={true} symbol="=" />
            </div>
        </div>
    }
}
