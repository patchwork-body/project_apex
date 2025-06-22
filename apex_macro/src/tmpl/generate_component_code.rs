use quote::quote;
use syn::Result;

use crate::tmpl::ComponentAttribute;

/// Generates the code to instantiate a dynamic component and renders it.
///
/// This function is a key part of the template macro expansion. It is responsible
/// for translating the HTML-like component syntax from a template into the
/// corresponding Rust code that builds and renders the component at runtime.
///
/// For a template syntax like `<MyComponent name="World" count={1+1} />`, this
/// function receives "MyComponent" as the `component_type` and the attributes
/// `name` and `count` as a `HashMap`.
///
/// The generated code performs the following steps:
/// 1. Creates a `std::collections::HashMap<String, String>`.
/// 2. Inserts each attribute from the template into the HashMap. All attribute
///    values (literals, variables, and expressions) are converted to strings.
/// 3. Calls the `from_attributes` static method on the component type, passing
///    the HashMap of stringified attributes.
/// 4. The component's `from_attributes` implementation is then responsible for
///    parsing these string values back into their expected types.
/// 5. Finally, it calls `apex::View::render()` on the newly created component
///    instance to get its `View`.
///
/// This mechanism allows for a declarative, HTML-like syntax in templates but
/// relies on runtime parsing of string-based attributes, which lacks compile-time
/// type safety. This is a known limitation of the current implementation.
pub(crate) fn generate_component_code(
    component_type: &str,
    attributes: &std::collections::HashMap<String, ComponentAttribute>,
) -> Result<proc_macro2::TokenStream> {
    let component_ident = syn::parse_str::<syn::Ident>(component_type)?;

    // Convert ComponentAttributes to a HashMap<String, String> for the from_attributes method
    let mut attr_assignments = Vec::new();

    for (key, value) in attributes {
        match value {
            ComponentAttribute::Literal(lit) => {
                attr_assignments.push(quote! {
                    attrs.insert(#key.to_string(), #lit.to_string());
                });
            }
            ComponentAttribute::Variable(var) => {
                if let Ok(var_ident) = syn::parse_str::<syn::Ident>(var) {
                    attr_assignments.push(quote! {
                        attrs.insert(#key.to_string(), #var_ident.to_string());
                    });
                }
            }
            ComponentAttribute::Expression(expr) => {
                if let Ok(expr_tokens) = syn::parse_str::<proc_macro2::TokenStream>(expr) {
                    attr_assignments.push(quote! {
                        attrs.insert(#key.to_string(), (#expr_tokens).to_string());
                    });
                }
            }
        }
    }

    Ok(quote! {
        {
            let mut attrs = std::collections::HashMap::new();
            #(#attr_assignments)*
            let component = #component_ident::from_attributes(&attrs);
            apex::View::render(&component)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::ComponentAttribute;
    use std::collections::HashMap;

    fn format_code(tokens: proc_macro2::TokenStream) -> String {
        let wrapped_tokens = quote::quote! {
            fn dummy() {
                #tokens
            }
        };

        let syntax_tree = syn::parse_file(&wrapped_tokens.to_string()).unwrap();
        let formatted = prettyplease::unparse(&syntax_tree);

        // Extract the formatted block from the dummy function
        formatted
            .lines()
            .skip(1)
            .take_while(|line| !line.starts_with('}'))
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_no_attributes() {
        let component_type = "MyComponent";
        let attributes = HashMap::new();
        let generated_code = generate_component_code(component_type, &attributes).unwrap();

        let expected_code = quote::quote! {
            {
                let mut attrs = std::collections::HashMap::new();
                let component = MyComponent::from_attributes(&attrs);
                apex::View::render(&component)
            }
        };

        assert_eq!(format_code(generated_code), format_code(expected_code));
    }

    #[test]
    fn test_with_literal_attribute() {
        let component_type = "MyComponent";
        let mut attributes = HashMap::new();

        attributes.insert(
            "name".to_owned(),
            ComponentAttribute::Literal("\"World\"".to_owned()),
        );

        let generated_code = generate_component_code(component_type, &attributes).unwrap();

        let expected_code = quote::quote! {
            {
                let mut attrs = std::collections::HashMap::new();
                attrs.insert("name".to_string(), "\"World\"".to_string());
                let component = MyComponent::from_attributes(&attrs);
                apex::View::render(&component)
            }
        };

        assert_eq!(format_code(generated_code), format_code(expected_code));
    }

    #[test]
    fn test_with_variable_attribute() {
        let component_type = "MyComponent";
        let mut attributes = HashMap::new();

        attributes.insert(
            "count".to_owned(),
            ComponentAttribute::Variable("my_var".to_owned()),
        );

        let generated_code = generate_component_code(component_type, &attributes).unwrap();
        let my_var_ident = syn::parse_str::<syn::Ident>("my_var").unwrap();

        let expected_code = quote::quote! {
            {
                let mut attrs = std::collections::HashMap::new();
                attrs.insert("count".to_string(), #my_var_ident.to_string());
                let component = MyComponent::from_attributes(&attrs);
                apex::View::render(&component)
            }
        };

        assert_eq!(format_code(generated_code), format_code(expected_code));
    }

    #[test]
    fn test_with_expression_attribute() {
        let component_type = "MyComponent";
        let mut attributes = HashMap::new();

        attributes.insert(
            "value".to_owned(),
            ComponentAttribute::Expression("1 + 1".to_owned()),
        );

        let generated_code = generate_component_code(component_type, &attributes).unwrap();

        let expected_code = quote::quote! {
            {
                let mut attrs = std::collections::HashMap::new();
                attrs.insert("value".to_string(), (1 + 1).to_string());
                let component = MyComponent::from_attributes(&attrs);
                apex::View::render(&component)
            }
        };

        assert_eq!(format_code(generated_code), format_code(expected_code));
    }

    #[test]
    fn test_with_mixed_attributes() {
        let component_type = "MyComponent";
        let mut attributes = HashMap::new();

        attributes.insert(
            "name".to_owned(),
            ComponentAttribute::Literal("\"World\"".to_owned()),
        );
        attributes.insert(
            "count".to_owned(),
            ComponentAttribute::Variable("my_var".to_owned()),
        );
        attributes.insert(
            "value".to_owned(),
            ComponentAttribute::Expression("1 + 1".to_owned()),
        );

        let generated_code = generate_component_code(component_type, &attributes).unwrap();

        // The order of insertions can vary, so we check for each part's existence
        let formatted_code = format_code(generated_code);
        assert!(formatted_code.contains("let mut attrs = std::collections::HashMap::new();"));
        assert!(
            formatted_code
                .contains("attrs.insert(\"name\".to_string(), \"\\\"World\\\"\".to_string());")
        );
        assert!(
            formatted_code.contains("attrs.insert(\"count\".to_string(), my_var.to_string());")
        );
        assert!(
            formatted_code.contains("attrs.insert(\"value\".to_string(), (1 + 1).to_string());")
        );
        assert!(formatted_code.contains("let component = MyComponent::from_attributes(&attrs);"));
        assert!(formatted_code.contains("apex::View::render(&component)"));
    }

    #[test]
    fn test_invalid_component_type() {
        let component_type = "invalid-component-type";
        let attributes = HashMap::new();
        let result = generate_component_code(component_type, &attributes);

        assert!(result.is_err());
    }

    #[test]
    fn test_with_invalid_variable_name() {
        let component_type = "MyComponent";
        let mut attributes = HashMap::new();

        attributes.insert(
            "name".to_owned(),
            ComponentAttribute::Variable("invalid-name".to_owned()),
        );

        let generated_code = generate_component_code(component_type, &attributes).unwrap();

        let expected_code = quote::quote! {
            {
                let mut attrs = std::collections::HashMap::new();
                let component = MyComponent::from_attributes(&attrs);
                apex::View::render(&component)
            }
        };

        assert_eq!(format_code(generated_code), format_code(expected_code));
    }

    #[test]
    fn test_with_unparseable_expression() {
        let component_type = "MyComponent";
        let mut attributes = HashMap::new();

        attributes.insert(
            "name".to_owned(),
            // An unmatched delimiter will cause a parsing error
            ComponentAttribute::Expression("{".to_owned()),
        );

        let generated_code = generate_component_code(component_type, &attributes).unwrap();

        let expected_code = quote::quote! {
            {
                let mut attrs = std::collections::HashMap::new();
                let component = MyComponent::from_attributes(&attrs);
                apex::View::render(&component)
            }
        };

        assert_eq!(format_code(generated_code), format_code(expected_code));
    }
}
