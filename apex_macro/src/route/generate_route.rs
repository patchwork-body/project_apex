use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Pat};

use super::parse_route_args::RouteArgs;
use crate::component::to_pascal_case::to_pascal_case;

/// Generate the children method implementation for ApexRoute trait
fn generate_children_method(args: &RouteArgs) -> proc_macro2::TokenStream {
    if args.children.is_empty() {
        quote! {
            fn children(&self) -> Vec<Box<dyn apex::router::ApexRoute>> {
                vec![]
            }
        }
    } else {
        let children_route_names = &args.children;
        let children_inits = children_route_names.iter().map(|child| {
            quote! {
                Box::new(#child) as Box<dyn apex::router::ApexRoute>
            }
        });

        quote! {
            fn children(&self) -> Vec<Box<dyn apex::router::ApexRoute>> {
                vec![#(#children_inits),*]
            }
        }
    }
}

/// Generate outlet matching helper functions for server and client
fn generate_outlet_helpers(
    fn_name: &syn::Ident,
    route_path: &str,
    args: &RouteArgs,
) -> proc_macro2::TokenStream {
    let outlet_helper_name = syn::Ident::new(&format!("{fn_name}_outlet_matcher"), fn_name.span());

    if args.children.is_empty() {
        // No children, no outlet matching needed
        quote! {}
    } else {
        let children_route_names = &args.children;

        quote! {
            /// Helper function to match child routes for outlet rendering
            /// Returns the route struct that should render in the outlet for the given path
            pub fn #outlet_helper_name(request_path: &str) -> Option<Box<dyn apex::router::ApexRoute>> {
                apex::router::outlet_match(#route_path, request_path, vec![
                    #(Box::new(#children_route_names) as Box<dyn apex::router::ApexRoute>),*
                ])
            }
        }
    }
}

/// Generate a route handler function that can be used with ApexRouter
///
/// The macro transforms:
/// ```rust,ignore
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
    let fn_body = &input.block;
    let route_struct_name = syn::Ident::new(
        &format!("{}Route", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    // Validate function signature - should accept HashMap<String, String> params
    validate_route_function(&input);

    // Extract the params parameter name from function signature
    let params_name = extract_params_name(&input);

    let has_children = !args.children.is_empty();
    let has_component = args.component.is_some();

    let hydrate_components_method_logic = if has_component {
        let Some(component_name) = args.component.as_ref() else {
            panic!("Route with return value must have a component");
        };

        quote! {
            #[cfg(target_arch = "wasm32")]
            {
                // Check if this route matches the current pathname
                let matches = if self.children().is_empty() {
                    apex::router::path_matches_pattern(self.path(), pathname)
                } else {
                    apex::router::path_matches_pattern_prefix(self.path(), pathname)
                };

                if matches {
                    let mut unmatched_exclude_path = exclude_path.to_string();

                    if apex::router::path_matches_pattern_prefix(self.path(), exclude_path) {
                        unmatched_exclude_path = apex::router::get_unmatched_path(self.path(), exclude_path);
                    } else {
                        let component = #component_name::builder().build();
                        let hydrate_fn = component.hydrate();
                        hydrate_fn(expressions_map, elements_map);
                    }

                    // After hydrating parent, hydrate matching child routes
                    // Child routes will continue with the current counter values
                    #[allow(unused_variables)]
                    let has_children = #has_children;

                    if has_children {
                        // Calculate the unmatched portion of the path
                        let parent_pattern = self.path();
                        let unmatched_path = apex::router::get_unmatched_path(parent_pattern, pathname);

                        for child in self.children() {
                            child.hydrate_components(&unmatched_path, &unmatched_exclude_path, expressions_map, elements_map);
                        }
                    }
                }
            }
        }
    } else {
        quote! {
            for child in self.children() {
                apex::router::hydrate_child_with_parent_path(child.as_ref(), self.path(), pathname, expressions_map, elements_map);
            }
        }
    };

    let has_return_value = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => {
            let type_str = quote!(#ty).to_string();
            type_str != "()"
        }
        syn::ReturnType::Default => false,
    };

    let loader_data_helper = if has_return_value {
        let client_helper_name =
            syn::Ident::new(&format!("get_{fn_name}_loader_data"), fn_name.span());

        let return_type = match &input.sig.output {
            syn::ReturnType::Type(_, ty) => ty,
            syn::ReturnType::Default => {
                panic!("Route with return value must have explicit return type")
            }
        };

        quote! {
            pub fn #client_helper_name() -> Signal<Option<#return_type>> {
                #[cfg(target_arch = "wasm32")]
                {
                    let route_name = stringify!(#fn_name);
                    signal!(apex::init_data::get_typed_route_data::<#return_type>(route_name))
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    signal!(apex::server_context::get_server_context::<#return_type>())
                }
            }
        }
    } else {
        quote! {}
    };

    let handler_method_logic = if has_return_value && has_component {
        let Some(component_name) = args.component.as_ref() else {
            panic!("Route with return value must have a component");
        };

        quote! {
            let route_data = { #fn_body };
            let component = #component_name::builder().build();
            let html = component.render_with_data(route_data.clone());

            let route_name = stringify!(#fn_name);
            let _ = apex::init_data::add_route_data(route_name, &route_data);

            html
        }
    } else if has_component {
        let Some(component_name) = args.component.as_ref() else {
            panic!("Route with return value must have a component");
        };

        quote! {
            { #fn_body };
            let component = #component_name::builder().build();
            component.render()
        }
    } else {
        quote! {
            let route_data = { #fn_body };
            let route_name = stringify!(#fn_name);
            let _ = apex::init_data::add_route_data(route_name, &route_data);

            route_data
        }
    };

    let path = args
        .path
        .as_ref()
        .map(|p| p.value())
        .unwrap_or_else(|| "/".to_owned());

    let children_method = generate_children_method(&args);
    let outlet_helpers = generate_outlet_helpers(fn_name, &path, &args);

    quote! {
        pub struct #route_struct_name;

        impl apex::router::ApexRoute for #route_struct_name {
            fn path(&self) -> &'static str { #path }

            #[cfg(not(target_arch = "wasm32"))]
            fn handler(&self) -> apex::router::ApexHandler {
                Box::new(|#params_name: std::collections::HashMap<String, String>| {
                    Box::pin(async move {
                        #handler_method_logic
                    })
                })
            }

            fn hydrate_components(
                &self,
                pathname: &str,
                exclude_path: &str,
                expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
            ) {
                #hydrate_components_method_logic
            }

            #children_method
        }

        #loader_data_helper
        #outlet_helpers
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
