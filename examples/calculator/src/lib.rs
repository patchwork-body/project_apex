#![allow(missing_docs)]

use apex::wasm_bindgen::JsCast;
use std::rc::Rc;
use wasm_bindgen::prelude::Closure;

use apex::prelude::*;

#[component]
pub fn button(
    #[prop] symbol: Signal<String>,
    #[prop(default = false)] wide: bool,
    #[prop(default = false)] primary: bool,
    #[prop(default = false)] secondary: bool,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onclick: Rc<dyn Fn(web_sys::Event)>,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onmousedown: Rc<dyn Fn(web_sys::Event)>,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onmouseup: Rc<dyn Fn(web_sys::Event)>,
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
        <button type="button" class={classes.join(" ")} onclick={onclick} onmousedown={onmousedown} onmouseup={onmouseup}>
            <span class="button-symbol">
                {$symbol}
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

    let clear_symbol = value.derive(|v| {
        if v == "0" {
            "AC".to_owned()
        } else {
            "<-".to_owned()
        }
    });

    let timeout_id = signal!(None::<i32>);

    let remove_last_symbol = action!(value, timeout_id => |_event| {
        if let Some(id) = timeout_id.get() {
            web_sys::window().unwrap().clear_timeout_with_handle(id);
        }

        value.update(|v| {
            if v.len() == 1 {
                "0".to_owned()
            } else {
                let mut v = v.clone();
                v.pop();
                v
            }
        });
    });

    let set_timeout_to_clear_value = action!(value, timeout_id => |_event| {
        let value = value.clone();

        let closure = Closure::wrap(Box::new(move || {
            value.set("0".to_owned());
        }) as Box<dyn Fn()>);

        let id = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            )
            .unwrap();

        timeout_id.set(Some(id));

        closure.forget(); // Prevent the closure from being dropped
    });

    tmpl! {
        <div class="calculator">
            <div class="display">
                {$value}
            </div>

            <div class="buttons">
                <Button secondary={true} symbol={clear_symbol.clone()} onmousedown={set_timeout_to_clear_value.clone()} onmouseup={remove_last_symbol.clone()} />
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
