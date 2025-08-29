use quote::quote;
use syn::{FnArg, ItemFn};

pub(crate) fn validate_route_function(input: &ItemFn) {
    if input.sig.inputs.len() != 1 {
        panic!("Route functions must have exactly one parameter: params: HashMap<String, String>");
    }

    if let Some(FnArg::Typed(pat_type)) = input.sig.inputs.first() {
        let type_str = quote!(#pat_type.ty).to_string();
        if !type_str.contains("HashMap") {
            panic!("Route function parameter should be HashMap<String, String>");
        }
    } else {
        panic!("Route functions cannot have self parameter");
    }
}
