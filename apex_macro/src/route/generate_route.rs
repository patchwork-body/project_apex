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
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Server-side outlet matching
                    apex::router::server_outlet_match(#route_path, request_path, vec![
                        #(Box::new(#children_route_names) as Box<dyn apex::router::ApexRoute>),*
                    ])
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // Client-side outlet matching
                    apex::router::client_outlet_match(#route_path, request_path, vec![
                        #(Box::new(#children_route_names) as Box<dyn apex::router::ApexRoute>),*
                    ])
                }
            }

            /// Get the child route that should render for the current request
            /// This is used in templates with {#outlet} directive
            pub fn get_outlet_content(request_path: &str) -> Option<String> {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Server-side: render the matched child route
                    if let Some(child_route) = #outlet_helper_name(request_path) {
                        let handler = child_route.handler();
                        // Extract params from the path
                        let params = apex::router::extract_route_params(child_route.path(), request_path);
                        Some(tokio::runtime::Handle::current().block_on(handler(params)))
                    } else {
                        None
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    // Client-side: get content from client-side routing
                    apex::router::get_client_outlet_content(#route_path, request_path)
                }
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
    let fn_vis = &input.vis;
    let fn_body = &input.block;
    let route_struct_name = syn::Ident::new(
        &format!("{}Route", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    // Generate children method implementation
    let children_method = generate_children_method(&args);

    // Validate function signature - should accept HashMap<String, String> params
    validate_route_function(&input);

    // Extract the params parameter name from function signature
    let params_name = extract_params_name(&input);

    if let Some(component_name) = &args.component {
        let has_children = !args.children.is_empty();

        let hydrate_components_method = quote! {
            fn hydrate_components(
                &self,
                pathname: &str,
                expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
            ) {
                #[cfg(target_arch = "wasm32")]
                {
                    web_sys::console::log_1(&format!("Hydrating route: {}, pathname: {}", self.path(), pathname).into());

                    // Check if this route matches the current pathname
                    let matches = if self.children().is_empty() {
                        apex::router::path_matches_pattern(self.path(), pathname)
                    } else {
                        apex::router::path_matches_pattern_prefix(self.path(), pathname)
                    };

                    if matches {
                        web_sys::console::log_1(&format!("Route matched! Hydrating component").into());

                        // Build and hydrate the component
                        let component = #component_name::builder().build();
                        let hydrate_fn = component.hydrate();
                        hydrate_fn(expressions_map, elements_map);

                        // After hydrating parent, hydrate matching child routes
                        // Child routes will continue with the current counter values
                        #[allow(unused_variables)]
                        let has_children = #has_children;

                        if has_children {
                            // Calculate the unmatched portion of the path
                            // If parent is /{name}/{age} and pathname is /john/23/calculator
                            // We need to extract /calculator
                            let parent_pattern = self.path();
                            let unmatched_path = apex::router::get_unmatched_path(parent_pattern, pathname);

                            web_sys::console::log_1(&format!(
                                "Parent matched {}, passing unmatched path '{}' to children",
                                parent_pattern, unmatched_path
                            ).into());

                            for child in self.children() {
                                child.hydrate_components(&unmatched_path, expressions_map, elements_map);
                            }
                        }
                    } else {
                        web_sys::console::log_1(&format!("Route did not match! Not hydrating component").into());
                    }
                }
            }
        };

        // Check if the route function returns something other than String
        let has_return_value = match &input.sig.output {
            syn::ReturnType::Type(_, ty) => {
                let type_str = quote!(#ty).to_string();
                !type_str.contains("String") && type_str != "()"
            }
            syn::ReturnType::Default => false,
        };

        if has_return_value {
            // Generate route handler that sets server context and injects INIT_DATA
            // Also generate client-side helper function
            {
                let path = args
                    .path
                    .as_ref()
                    .map(|p| p.value())
                    .unwrap_or_else(|| "/".to_owned());

                // Generate client-side helper function name
                let client_helper_name =
                    syn::Ident::new(&format!("get_{fn_name}_loader_data"), fn_name.span());

                // Generate outlet helpers
                let outlet_helpers = generate_outlet_helpers(fn_name, &path, &args);

                // Extract return type for the client helper
                let return_type = match &input.sig.output {
                    syn::ReturnType::Type(_, ty) => ty,
                    syn::ReturnType::Default => {
                        panic!("Route with return value must have explicit return type")
                    }
                };

                quote! {
                    // legacy handler fn kept for backward compatibility
                    #[cfg(not(target_arch = "wasm32"))]
                    #fn_vis async fn #fn_name(#params_name: std::collections::HashMap<String, String>) -> String {
                        let route_data = { #fn_body };
                        let component = #component_name::builder().build();
                        let html = component.render_with_data(route_data.clone());

                        // Store route data in the collector instead of injecting directly
                        let route_name = stringify!(#fn_name);
                        let _ = apex::init_data::add_route_data(route_name, &route_data);

                        html
                    }

                    // struct-based route for ApexRouter::mount_route
                    pub struct #route_struct_name;

                    impl apex::router::ApexRoute for #route_struct_name {
                        fn path(&self) -> &'static str { #path }

                        #[cfg(not(target_arch = "wasm32"))]
                        fn handler(&self) -> apex::router::ApexHandler {
                            Box::new(|#params_name: std::collections::HashMap<String, String>| {
                                Box::pin(async move {
                                    let route_data = { #fn_body };
                                    let component = #component_name::builder().build();
                                    let html = component.render_with_data(route_data.clone());

                                    // Store route data in the collector instead of injecting directly
                                    let route_name = stringify!(#fn_name);
                                    let _ = apex::init_data::add_route_data(route_name, &route_data);

                                    html
                                })
                            })
                        }

                        #[cfg(target_arch = "wasm32")]
                        fn handler(&self) -> apex::router::ApexHandler {
                            Box::new(|#params_name: std::collections::HashMap<String, String>| {
                                Box::pin(async move {
                                    "".to_string()
                                })
                            })
                        }

                        #hydrate_components_method
                        #children_method
                    }

                    // Helper function for accessing loader data (works on both client and server)
                    pub fn #client_helper_name() -> Signal<Option<#return_type>> {
                        #[cfg(target_arch = "wasm32")]
                        {
                            // On client side, get data from INIT_DATA[route_name]
                            let route_name = stringify!(#fn_name);
                            signal!(apex::init_data::get_typed_route_data::<#return_type>(route_name))
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            // On server side, get the data from server context
                            signal!(apex::server_context::get_server_context::<#return_type>())
                        }
                    }

                    // Outlet helpers for hierarchical routing
                    #outlet_helpers
                }
            }
        } else {
            // Generate route handler without passing data to component
            {
                let path = args
                    .path
                    .as_ref()
                    .map(|p| p.value())
                    .unwrap_or_else(|| "/".to_owned());

                // Generate outlet helpers
                let outlet_helpers = generate_outlet_helpers(fn_name, &path, &args);

                quote! {
                    #[cfg(not(target_arch = "wasm32"))]
                    #fn_vis async fn #fn_name(#params_name: std::collections::HashMap<String, String>) -> String {
                        let _result = { #fn_body };
                        let component = #component_name::builder().build();
                        component.render()
                    }

                    pub struct #route_struct_name;

                    impl apex::router::ApexRoute for #route_struct_name {
                        fn path(&self) -> &'static str { #path }

                        #[cfg(not(target_arch = "wasm32"))]
                        fn handler(&self) -> apex::router::ApexHandler {
                            Box::new(|#params_name: std::collections::HashMap<String, String>| {
                                Box::pin(async move {
                                    let _result = { #fn_body };
                                    let component = #component_name::builder().build();
                                    component.render()
                                })
                            })
                        }

                        #[cfg(target_arch = "wasm32")]
                        fn handler(&self) -> apex::router::ApexHandler {
                            Box::new(|#params_name: std::collections::HashMap<String, String>| {
                                Box::pin(async move {
                                    "".to_string()
                                })
                            })
                        }

                        #hydrate_components_method
                        #children_method
                    }

                    // Outlet helpers for hierarchical routing
                    #outlet_helpers
                }
            }
        }
    } else {
        // Generate route handler without component (just execute original logic)
        {
            let path = args
                .path
                .as_ref()
                .map(|p| p.value())
                .unwrap_or_else(|| "/".to_owned());

            // Generate outlet helpers
            let outlet_helpers = generate_outlet_helpers(fn_name, &path, &args);

            quote! {
                #[cfg(not(target_arch = "wasm32"))]
                #fn_vis async fn #fn_name(#params_name: std::collections::HashMap<String, String>) -> String {
                    #fn_body
                }

                #[cfg(not(target_arch = "wasm32"))]
                pub struct #route_struct_name;

                #[cfg(not(target_arch = "wasm32"))]
                impl apex::router::ApexRoute for #route_struct_name {
                    fn path(&self) -> &'static str { #path }
                    fn handler(&self) -> apex::router::ApexHandler {
                        Box::new(|#params_name: std::collections::HashMap<String, String>| {
                            Box::pin(async move { { #fn_body } })
                        })
                    }

                    fn hydrate_components(
                        &self,
                        pathname: &str,
                        expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                        elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
                    ) {
                        // Route without component - just hydrate children if any
                        for child in self.children() {
                            apex::router::hydrate_child_with_parent_path(child.as_ref(), self.path(), pathname, expressions_map, elements_map);
                        }
                    }

                    #children_method
                }

                // Outlet helpers for hierarchical routing
                #outlet_helpers
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
