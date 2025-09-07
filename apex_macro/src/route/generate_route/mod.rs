mod extract_params_name;
mod generate_children_method;
mod validate_route_function;

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use super::parse_route_args::RouteArgs;
use crate::common::to_pascal_case;

use extract_params_name::extract_params_name;
use generate_children_method::generate_children_method;
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

    let hydrate_component_method = if let Some(component_name) = args.component.as_ref() {
        quote! {
            fn hydrate_component(
                &self,
                expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
            ) {
                let component = #component_name::builder().build();
                let hydrate_fn = component.hydrate();
                hydrate_fn(expressions_map, elements_map);
            }
        }
    } else {
        quote! {}
    };

    let has_return_value = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => {
            let type_str = quote!(#ty).to_string();
            type_str != "()"
        }
        syn::ReturnType::Default => false,
    };

    let loader_data_helper = if has_return_value {
        let helper_name = syn::Ident::new(&format!("get_{fn_name}_loader_data"), fn_name.span());

        let return_type = match &input.sig.output {
            syn::ReturnType::Type(_, ty) => ty,
            syn::ReturnType::Default => {
                panic!("Route with return value must have explicit return type")
            }
        };

        quote! {
            #[cfg(target_arch = "wasm32")]
            pub(crate) fn #helper_name() -> Signal<Option<#return_type>> {
                let route_name = stringify!(#fn_name);
                signal!(apex::apex_router::init_data::get_typed_route_data::<#return_type>(route_name))
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub(crate) fn #helper_name(data: &std::collections::HashMap<String, serde_json::Value>) -> Signal<Option<#return_type>> {
                let route_name = stringify!(#fn_name);

                signal!(
                    data.get(route_name)
                        .and_then(|value| serde_json::from_value::<#return_type>(value.clone()).ok())
                )
            }
        }
    } else {
        quote! {}
    };

    let has_component = args.component.is_some();

    let handler_method_logic = if has_return_value && has_component {
        let Some(component_name) = args.component.as_ref() else {
            panic!("Unhandled error, component name is not set");
        };

        quote! {
            let route_data = { #fn_body };
            let component = #component_name::builder().build();

            let route_name = stringify!(#fn_name);

            if let Ok(serialized_data) = serde_json::to_value(&route_data) {
                data.insert(route_name.to_owned(), serialized_data);
            }

            let html = component.render(&data);

            html
        }
    } else if has_component {
        let Some(component_name) = args.component.as_ref() else {
            panic!("Unhandled error, component name is not set");
        };

        quote! {
            { #fn_body };
            let component = #component_name::builder().build();
            component.render(&data)
        }
    } else {
        quote! {
            let route_data = { #fn_body };
            let route_name = stringify!(#fn_name);

            if let Ok(serialized_data) = serde_json::to_value(&route_data) {
                data.insert(route_name.to_owned(), serialized_data);
            }

            route_data
        }
    };

    let path = args
        .path
        .as_ref()
        .map(|p| p.value())
        .unwrap_or_else(|| "/".to_owned());

    let server_children_method =
        generate_children_method(&args, quote!(apex::apex_router::ApexServerRoute));

    let client_children_method =
        generate_children_method(&args, quote!(apex::apex_router::ApexClientRoute));

    let server_route = quote! {
        #[cfg(not(target_arch = "wasm32"))]
        pub struct #route_struct_name;

        #[cfg(not(target_arch = "wasm32"))]
        impl #route_struct_name {
            pub fn new() -> Self {
                Self
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl apex::apex_router::ApexServerRoute for #route_struct_name {
            fn path(&self) -> &'static str { #path }

            fn handler(&self) -> apex::apex_router::ApexServerHandler {
                Box::new(|#params_name: std::collections::HashMap<String, String>| {
                    Box::pin(async move {
                        let mut data: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

                        let html = {
                            #handler_method_logic
                        };

                        (html, data)
                    })
                })
            }

            #server_children_method
        }
    };

    let client_route = if has_component {
        quote! {
            #[cfg(target_arch = "wasm32")]
            pub struct #route_struct_name;

            #[cfg(target_arch = "wasm32")]
            impl #route_struct_name {
                pub fn new() -> Self {
                    Self
                }
            }

            #[cfg(target_arch = "wasm32")]
            impl apex::apex_router::ApexClientRoute for #route_struct_name {
                fn path(&self) -> &'static str { #path }
                #hydrate_component_method
                #client_children_method
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #server_route
        #client_route
        #loader_data_helper
    }
}
