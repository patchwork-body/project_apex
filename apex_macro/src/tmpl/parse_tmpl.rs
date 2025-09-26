use proc_macro::TokenStream;
use quote::quote;

use crate::tmpl::{parse_tmpl_into_ast::*, render_ast::*};

pub(crate) fn parse_tmpl(input: TokenStream) -> proc_macro2::TokenStream {
    let input_str = input.to_string();
    let parsed_content = parse_tmpl_into_ast(&input_str);
    let (render_instructions, hydration_expressions) = render_ast(&parsed_content);

    quote! {
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut buffer = String::with_capacity(1024);
                #(#render_instructions)*

                buffer
            }

            #[cfg(target_arch = "wasm32")]
            {
                let hydrate = move |state: std::rc::Rc<std::cell::RefCell<apex_router::client_router::State>>| {
                    #(#hydration_expressions)*
                };

                hydrate
            }
        }
    }
}
