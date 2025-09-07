use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

/// Parses the loader_data! macro input
/// Expected format: loader_data!(route_name)
struct LoaderDataInput {
    route_name: Ident,
}

impl Parse for LoaderDataInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(LoaderDataInput {
            route_name: input.parse()?,
        })
    }
}

pub(crate) fn generate_loader_data_macro(input: TokenStream) -> TokenStream2 {
    let input: LoaderDataInput = syn::parse(input).unwrap();

    let route_name = &input.route_name;
    let helper_name = syn::Ident::new(&format!("get_{route_name}_loader_data"), route_name.span());

    // Generate different code based on the target architecture
    quote! {
        {
            #[cfg(target_arch = "wasm32")]
            {
                // On the client side, call the helper function (no parameters needed)
                #helper_name()
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // On the server side, call the helper function with data parameter
                #helper_name(data)
            }
        }
    }
}
