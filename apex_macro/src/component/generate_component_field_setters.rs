use quote::quote;

/// Generate component field setters for the from_attributes method
pub(crate) fn generate_component_field_setters(
    prop_fields: &[(&syn::Ident, &syn::Type, bool)],
) -> Vec<proc_macro2::TokenStream> {
    prop_fields
        .iter()
        .map(|(field_name, field_type, is_signal)| {
            let field_str = field_name.to_string();

            // Generate type-specific parsing logic
            let type_str = quote! { #field_type }.to_string();

            if *is_signal {
                // For signals, use the setter method
                let setter_name = syn::Ident::new(&format!("set_{field_name}"), field_name.span());

                if type_str.contains("String") {
                    quote! {
                        if let Some(value) = attrs.get(#field_str) {
                            component.#setter_name(value.clone());
                        }
                    }
                } else if type_str.contains("i32") {
                    quote! {
                        if let Some(value) = attrs.get(#field_str) {
                            if let Ok(parsed) = value.parse::<i32>() {
                                component.#setter_name(parsed);
                            }
                        }
                    }
                } else {
                    quote! {
                        if let Some(value) = attrs.get(#field_str) {
                            if let Ok(parsed) = value.parse() {
                                component.#setter_name(parsed);
                            }
                        }
                    }
                }
            } else {
                // For regular fields, assign directly
                if type_str.contains("String") {
                    quote! {
                        if let Some(value) = attrs.get(#field_str) {
                            component.#field_name = value.clone();
                        }
                    }
                } else if type_str.contains("i32") {
                    quote! {
                        if let Some(value) = attrs.get(#field_str) {
                            if let Ok(parsed) = value.parse::<i32>() {
                                component.#field_name = parsed;
                            }
                        }
                    }
                } else {
                    quote! {
                        if let Some(value) = attrs.get(#field_str) {
                            if let Ok(parsed) = value.parse() {
                                component.#field_name = parsed;
                            }
                        }
                    }
                }
            }
        })
        .collect()
}
