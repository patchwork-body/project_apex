mod extract_params_name;
mod generate_children_method;
mod generate_outlet_helpers;
mod validate_route_function;

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use super::parse_route_args::RouteArgs;
use crate::common::to_pascal_case;

use extract_params_name::extract_params_name;
use generate_children_method::generate_children_method;
use generate_outlet_helpers::generate_outlet_helpers;
use validate_route_function::validate_route_function;

pub(crate) fn generate_route(args: RouteArgs, input: ItemFn) -> TokenStream {
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let route_struct_name = syn::Ident::new(
        &format!("{}Route", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    validate_route_function(&input);

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

                    if #has_children {
                        let unmatched_path = apex::router::get_unmatched_path(self.path(), pathname);

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
                let parent_clean = parent_path.trim_end_matches('/');
                let child_clean = child.path().trim_start_matches('/');

                let full_child_path = if parent_clean.is_empty() || parent_clean == "/" {
                    format!("/{}", child_clean)
                } else {
                    format!("{}/{}", parent_clean, child_clean)
                };

                if path_matches_pattern(&full_child_path, pathname) {
                    child.hydrate_components(pathname, exclude_path, expressions_map, elements_map);
                }
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
            panic!("Unhandled error, component name is not set");
        };

        quote! {
            let route_data = { #fn_body };
            let component = #component_name::builder().build();

            let route_name = stringify!(#fn_name);
            let _ = apex::init_data::add_route_data(route_name, &route_data);
            apex::server_context::set_server_context(route_data);

            let html = component.render();

            html
        }
    } else if has_component {
        let Some(component_name) = args.component.as_ref() else {
            panic!("Unhandled error, component name is not set");
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
            apex::server_context::set_server_context(route_data);

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
