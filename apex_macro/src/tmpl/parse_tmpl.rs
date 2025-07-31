use proc_macro::TokenStream;
use quote::quote;

use crate::tmpl::{parse_tmpl_into_ast::*, render_ast::*};

pub(crate) fn parse_tmpl(input: TokenStream) -> proc_macro2::TokenStream {
    let input_str = input.to_string();
    let parsed_content = parse_tmpl_into_ast(&input_str);
    let render = render_ast(&parsed_content);

    quote! {
        apex::Html::new(move |element: apex::web_sys::Element| {
            #(#render)*

            let element_clone = element.clone();
            let callback: apex::wasm_bindgen::closure::Closure<dyn Fn()> = apex::wasm_bindgen::closure::Closure::new(Box::new(move || {
                let _ = element_clone.remove();
            }) as Box<dyn Fn()>);

            callback.into_js_value().dyn_into().unwrap()
        })
    }
}
