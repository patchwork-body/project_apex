#![allow(missing_docs)]

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::{component::generate_component, tmpl::parse_tmpl};

mod component;
pub(crate) mod tmpl;

#[proc_macro]
pub fn tmpl(input: TokenStream) -> TokenStream {
    parse_tmpl(input).into()
}

#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    generate_component(input_fn).into()
}
