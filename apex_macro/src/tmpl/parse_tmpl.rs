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
        })
    }
}
