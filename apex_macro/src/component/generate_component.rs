use quote::quote;
use syn::{ItemFn, Result};

use crate::component::{
    parse_props::parse_props, to_pascal_case::to_pascal_case, validate_component_function,
};

/// Generate a component from a function
pub(crate) fn generate_component(input: ItemFn) -> Result<proc_macro2::TokenStream> {
    // Validate the function signature
    validate_component_function(&input)?;

    // Extract function details
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let vis = &input.vis;

    // Parse props from function parameters
    let props = parse_props(&input);

    // Convert function name to PascalCase for the struct
    let struct_name = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());
    let builder_name = syn::Ident::new(&format!("{struct_name}Builder"), fn_name.span());

    // Generate struct fields from props
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

    // Generate local variable bindings for props in render method
    let prop_bindings = props.iter().map(|prop| {
        let name = &prop.name;
        quote! {
            let #name = self.#name.clone();
        }
    });

    // Generate builder default field values
    let builder_default_fields = props.iter().map(|prop| {
        let name = &prop.name;
        quote! { #name: None }
    });

    // Generate the component struct and impl
    let output = if props.is_empty() {
        // No props - generate unit struct with builder for consistency
        quote! {
            #vis struct #struct_name;

            pub struct #builder_name;

            impl #builder_name {
                pub fn new() -> Self {
                    Self
                }

                pub fn build(self) -> #struct_name {
                    #struct_name
                }
            }

            impl #struct_name {
                pub fn builder() -> #builder_name {
                    #builder_name::new()
                }

                pub fn render(&self) -> apex::Html {
                    #fn_body
                }
            }
        }
    } else {
        // Has props - generate struct with builder
        quote! {
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

                pub fn render(&self) -> apex::Html {
                    #(#prop_bindings)*
                    #fn_body
                }
            }
        }
    };

    Ok(output)
}
