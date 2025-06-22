use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, Fields, Result};

/// Component configuration parsed from macro arguments
#[derive(Debug)]
pub(crate) struct ComponentConfig {
    // No longer need tag and imports - derive from struct name
}

/// Parse the component macro arguments to extract configuration
pub(crate) fn parse_component_args(_args: TokenStream) -> Result<ComponentConfig> {
    // No arguments needed anymore - component name derived from struct name
    Ok(ComponentConfig {})
}

/// Generate the component implementation
pub(crate) fn generate_component(
    input: &DeriveInput,
    _config: &ComponentConfig,
) -> Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let vis = &input.vis;

    // Use struct name as the component tag name
    let tag_name = struct_name.to_string();

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unit => {
                // Unit structs have no fields, handle separately
                let expanded = quote! {
                    #[doc = concat!("Web component with tag: ", #tag_name)]
                    #[derive(Debug, Clone)]
                    #vis struct #struct_name;

                    impl #struct_name {
                        /// Create a new component instance
                        pub fn new() -> Self {
                            Self
                        }

                        /// Get the component's tag name
                        pub fn tag_name() -> &'static str {
                            #tag_name
                        }

                        /// Create component from attributes map (unit struct - ignores attributes)
                        pub fn from_attributes(_attrs: &std::collections::HashMap<String, String>) -> Self {
                            Self
                        }
                    }

                    impl Default for #struct_name {
                        fn default() -> Self {
                            Self::new()
                        }
                    }
                };

                return Ok(expanded);
            }
            Fields::Unnamed(_fields) => {
                return Err(syn::Error::new_spanned(
                    struct_name,
                    "Component macro does not support unnamed fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                struct_name,
                "Component macro can only be applied to structs",
            ));
        }
    };

    // Process each field to extract prop information
    let mut prop_fields = Vec::new();
    let mut prop_defaults = Vec::new();
    let mut prop_setters = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check for #[prop] attributes
        let mut default_value = None;
        for attr in &field.attrs {
            if attr.path().is_ident("prop") {
                // Simple string parsing approach for syn 2.0
                let attr_str = attr.to_token_stream().to_string();
                if attr_str.contains("default") {
                    // Extract value between = and next token/end
                    if let Some(eq_pos) = attr_str.find('=') {
                        let after_eq = &attr_str[eq_pos + 1..];
                        // Find the numeric value
                        let value_part = after_eq.trim().split(')').next().unwrap_or("").trim();
                        if let Ok(val) = value_part.parse::<i32>() {
                            default_value = Some(val);
                        }
                    }
                }
            }
        }

        prop_fields.push((field_name, field_type));
        prop_defaults.push(default_value.unwrap_or(0));

        // Generate setter method
        let setter_name = syn::Ident::new(&format!("set_{field_name}"), field_name.span());
        prop_setters.push(quote! {
            #[doc = concat!("Set the ", stringify!(#field_name), " property")]
            pub fn #setter_name(&mut self, value: #field_type) {
                self.#field_name = value;
            }
        });
    }

    // Generate the component implementation
    let field_names: Vec<_> = prop_fields.iter().map(|(name, _)| name).collect();
    let field_types: Vec<_> = prop_fields.iter().map(|(_, ty)| ty).collect();
    let field_defaults: Vec<_> = prop_defaults
        .iter()
        .map(|val| {
            if *val == 0 {
                quote! { Default::default() }
            } else {
                quote! { #val }
            }
        })
        .collect();

    // Generate component field setters for the from_attributes method
    let field_setters = generate_component_field_setters(&prop_fields);

    let expanded = quote! {
        #[doc = concat!("Web component with tag: ", #tag_name)]
        #[derive(Debug, Clone)]
        #vis struct #struct_name {
            #(
                #[doc = "Component property"]
                pub #field_names: #field_types,
            )*
        }

        impl #struct_name {
            /// Create a new component instance
            pub fn new() -> Self {
                Self {
                    #(#field_names: #field_defaults,)*
                }
            }

            /// Get the component's tag name
            pub fn tag_name() -> &'static str {
                #tag_name
            }

            /// Create component from attributes map
            pub fn from_attributes(attrs: &std::collections::HashMap<String, String>) -> Self {
                let mut component = Self::new();
                #(#field_setters)*
                component
            }

            #(#prop_setters)*
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self::new()
            }
        }

        // Register component in the global component registry
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static #struct_name: () = {
            // This would be used by a component registry system
            // For now, it's just a placeholder for future expansion
        };
    };

    Ok(expanded)
}

/// Generate component field setters for the from_attributes method
fn generate_component_field_setters(
    prop_fields: &[(&syn::Ident, &syn::Type)],
) -> Vec<proc_macro2::TokenStream> {
    prop_fields
        .iter()
        .map(|(field_name, field_type)| {
            let field_str = field_name.to_string();

            // Generate type-specific parsing logic
            let type_str = quote! { #field_type }.to_string();

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
                // Default: try to parse as string and convert
                quote! {
                    if let Some(value) = attrs.get(#field_str) {
                        // Default string conversion for unknown types
                        if let Ok(parsed) = value.parse() {
                            component.#field_name = parsed;
                        }
                    }
                }
            }
        })
        .collect()
}
