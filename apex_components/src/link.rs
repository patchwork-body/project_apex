#![allow(missing_docs)]

use apex::prelude::*;
use apex::wasm_bindgen;
use apex::web_sys;

#[component]
pub fn link(#[prop] href: String, #[prop] text: String) {
    let handle_click = action!(href @ web_sys::MouseEvent => |event| {
            event.prevent_default();
            let detail = wasm_bindgen::JsValue::from_str(&href);
            let event_init = web_sys::CustomEventInit::new();

            event_init.set_detail(&detail);

            let custom_event = web_sys::CustomEvent::new_with_event_init_dict(
                "apex:navigate",
                &event_init,
            )
            .unwrap();

            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .dispatch_event(&custom_event)
                .unwrap();
    });

    tmpl! {
        <a href={href} onclick={handle_click}>{text}</a>
    }
}
