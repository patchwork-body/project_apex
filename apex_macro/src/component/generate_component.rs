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

    // Generate struct fields from props
    let struct_fields = props.iter().map(|prop| {
        let name = &prop.name;
        let ty = &prop.ty;
        quote! {
            pub #name: #ty
        }
    });

    // Generate local variable bindings for props in render method
    let prop_bindings = props.iter().map(|prop| {
        let name = &prop.name;

        quote! {
            let #name = self.#name.clone();
        }
    });

    // Generate the component struct and impl
    let output = if props.is_empty() {
        // No props - generate unit struct
        quote! {
            #vis struct #struct_name;

            impl #struct_name {
                pub fn render(&self) -> apex::Html {
                    #fn_body
                }
            }
        }
    } else {
        // Has props - generate struct with fields
        quote! {
            #vis struct #struct_name {
                #(#struct_fields),*
            }

            impl #struct_name {
                pub fn render(&self) -> apex::Html {
                    #(#prop_bindings)*
                    #fn_body
                }
            }
        }
    };

    Ok(output)
}
