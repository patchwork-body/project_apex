use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::component::{
    parse_props::parse_props, parse_slots::parse_slots, to_pascal_case::to_pascal_case,
    validate_component_function,
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
    let props = parse_props(&input);
    let slots = parse_slots(&input);

    // Convert function name to PascalCase for the struct
    let struct_name = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());
    let builder_name = syn::Ident::new(&format!("{struct_name}Builder"), fn_name.span());

    // Generate struct fields from props and slots
    let struct_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            let ty = &prop.ty;
            quote! {
                pub #name: #ty
            }
        })
        .chain(slots.iter().map(|slot| {
            let name = &slot.name;
            quote! {
                pub #name: apex::Html
            }
        }))
        .chain([quote! {
            pub children: Option<apex::Html>
        }]);

    // Generate builder struct fields (Option for all)
    let builder_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            let ty = &prop.ty;
            quote! {
                #name: Option<#ty>
            }
        })
        .chain(slots.iter().map(|slot| {
            let name = &slot.name;
            quote! {
                #name: Option<apex::Html>
            }
        }))
        .chain([quote! {
            children: Option<apex::Html>
        }]);

    // Generate builder setter methods
    let builder_setters = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            let ty = &prop.ty;
            quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = Some(value);
                    self
                }
            }
        })
        .chain(
            slots
                .iter()
                .map(|slot| {
                    let name = &slot.name;
                    quote! {
                        pub fn #name(mut self, value: apex::Html) -> Self {
                            self.#name = Some(value);
                            self
                        }
                    }
                })
                .chain([quote! {
                    pub fn children(mut self, value: apex::Html) -> Self {
                        self.children = Some(value);
                        self
                    }
                }]),
        );

    // Generate builder build method
    let build_field_inits = props
        .iter()
        .map(|prop| {
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
        })
        .chain(slots.iter().map(|slot| {
            let name = &slot.name;
            if let Some(default) = &slot.default {
                quote! {
                    #name: self.#name.unwrap_or_else(|| #default)
                }
            } else {
                let name_str = name.ident.to_string();
                quote! {
                    #name: self.#name.expect(&format!("Required slot '{}' not set", #name_str))
                }
            }
        }))
        .chain([quote! {
            children: self.children.clone()
        }]);

    // Generate local variable bindings for props and slots in render method
    let prop_bindings = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            quote! {
                let #name = self.#name.clone();
            }
        })
        .chain(slots.iter().map(|slot| {
            let name = &slot.name;
            quote! {
                let #name = self.#name.clone();
            }
        }))
        .chain([quote! {
            let children = self.children.clone();
        }]);

    // Generate builder default field values
    let builder_default_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            quote! { #name: None }
        })
        .chain(slots.iter().map(|slot| {
            let name = &slot.name;
            quote! { #name: None }
        }))
        .chain([quote! {
            children: None
        }]);

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

        impl apex::View for #struct_name {
            fn render(&self) -> apex::Html {
                #(#prop_bindings)*
                #fn_body
            }
        }
    };

    output
}
