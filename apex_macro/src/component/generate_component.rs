use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::{
    common::to_pascal_case,
    component::{parse_props::parse_props, parse_slots::parse_slots, validate_component_function},
};

/// Generate a component from a function
pub(crate) fn generate_component(input: ItemFn) -> TokenStream {
    // Validate the function signature
    validate_component_function(&input);

    // Extract function details
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let vis = &input.vis;

    // Parse props and slots from function parameters
    let all_props = parse_props(&input);
    let _slots = parse_slots(&input);

    // Separate regular props from server context and route_data props
    let props: Vec<_> = all_props.iter().collect();

    // Convert function name to PascalCase for the struct
    let struct_name = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());
    let builder_name = syn::Ident::new(&format!("{struct_name}Builder"), fn_name.span());

    // Generate struct fields from props and slots
    let struct_fields = props.iter().map(|prop| {
        let name = &prop.name;
        let ty = &prop.ty;

        quote! {
            pub #name: #ty
        }
    });

    // Generate builder struct fields (Option for all)
    let builder_fields = props.iter().map(|prop| {
        let name = &prop.name;
        let ty = &prop.ty;
        quote! {
            #name: Option<#ty>
        }
    });

    // Generate builder setter methods
    let builder_setters = props.iter().map(|prop| {
        let name = &prop.name;
        let ty = &prop.ty;
        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });

    // Generate builder build method
    let build_field_inits = props.iter().map(|prop| {
        let name = &prop.name;
        if let Some(default) = &prop.default {
            quote! {
                #name: self.#name.unwrap_or_else(|| #default)
            }
        } else {
            let name_str = name.ident.to_string();
            quote! {
                #name: self.#name.expect(&format!("Required prop '{}' not set", #name_str))
            }
        }
    });

    // Generate local variable bindings for props and slots in render method
    let prop_bindings = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            quote! {
                let #name = self.#name.clone();
            }
        })
        .collect::<Vec<_>>();

    // Generate builder default field values
    let builder_default_fields = props.iter().map(|prop| {
        let name = &prop.name;
        quote! { #name: None }
    });

    // Generate the component struct and impl
    let output = quote! {
        #vis struct #struct_name {
            #(#struct_fields),*
        }

        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#builder_default_fields),*
                }
            }

            #(#builder_setters)*

            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#build_field_inits),*
                }
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl #struct_name {
            pub fn render(&self) -> String {
                #(#prop_bindings)*
                #fn_body
            }
        }

        #[cfg(target_arch = "wasm32")]
        impl #struct_name {
            pub fn hydrate(&self) -> impl Fn(&std::collections::HashMap<String, apex::web_sys::Text>, &std::collections::HashMap<String, apex::web_sys::Element>) {
                #(#prop_bindings)*
                #fn_body
            }
        }
    };

    output
}
