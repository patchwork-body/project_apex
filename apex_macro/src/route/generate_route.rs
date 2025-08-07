use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Pat};

use super::parse_route_args::RouteArgs;

/// Generate a route handler function that can be used with ApexRouter
///
/// The macro transforms:
/// ```rust
/// #[route(component = HomeComponent)]
/// fn home(params: HashMap<String, String>) -> String {
///     // custom logic here
/// }
/// ```
///
/// Into a function that:
/// 1. Executes the original function logic
/// 2. Creates and renders the specified component
/// 3. Returns the rendered HTML as a String
pub(crate) fn generate_route(args: RouteArgs, input: ItemFn) -> TokenStream {
    // Extract function details
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_body = &input.block;

    // Validate function signature - should accept HashMap<String, String> params
    validate_route_function(&input);

    // Extract the params parameter name from function signature
    let params_name = extract_params_name(&input);

    if let Some(component_name) = &args.component {
        // Generate route handler that uses the specified component
        quote! {
            #fn_vis async fn #fn_name(#params_name: std::collections::HashMap<String, String>) -> String {
                // Execute original function logic
                let _result = {
                    #fn_body
                };

                // Create and render the component
                let component = #component_name::builder().build();
                component.render()
            }
        }
    } else {
        // Generate route handler without component (just execute original logic)
        quote! {
            #fn_vis async fn #fn_name(#params_name: std::collections::HashMap<String, String>) -> String {
                #fn_body
            }
        }
    }
}

/// Validate that the function has the correct signature for a route handler
fn validate_route_function(input: &ItemFn) {
    // Check that function has exactly one parameter of type HashMap<String, String>
    if input.sig.inputs.len() != 1 {
        panic!("Route functions must have exactly one parameter: params: HashMap<String, String>");
    }

    if let Some(FnArg::Typed(pat_type)) = input.sig.inputs.first() {
        // Basic validation - could be more thorough
        let type_str = quote!(#pat_type.ty).to_string();
        if !type_str.contains("HashMap") {
            panic!("Route function parameter should be HashMap<String, String>");
        }
    } else {
        panic!("Route functions cannot have self parameter");
    }

    // Check return type - should return String or similar
    match &input.sig.output {
        syn::ReturnType::Type(_, ty) => {
            let type_str = quote!(#ty).to_string();
            if !type_str.contains("String") {
                // Allow String return type for now
                // Could be more flexible in the future
            }
        }
        syn::ReturnType::Default => {
            // Default return type is (), which is fine for routes
        }
    }
}

/// Extract the parameter name from the function signature
fn extract_params_name(input: &ItemFn) -> &syn::Ident {
    if let Some(FnArg::Typed(pat_type)) = input.sig.inputs.first() {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            return &pat_ident.ident;
        }
    }

    panic!("Could not extract parameter name from route function");
}
