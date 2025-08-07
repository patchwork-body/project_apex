#![allow(missing_docs)]

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::{component::generate_component, route::generate_route, tmpl::parse_tmpl};

mod component;
mod route;
pub(crate) mod tmpl;

#[proc_macro]
pub fn tmpl(input: TokenStream) -> TokenStream {
    parse_tmpl(input).into()
}

#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);

    generate_component(item_fn).into()
}

#[proc_macro_attribute]
pub fn route(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);

    generate_route(item_fn).into()
}
