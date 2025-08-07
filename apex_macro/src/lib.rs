#![allow(missing_docs)]

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::{component::generate_component, tmpl::parse_tmpl};

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
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);
    let route_args = route::parse_route_args::parse_route_args(args);

    route::generate_route::generate_route(route_args, item_fn).into()
}
