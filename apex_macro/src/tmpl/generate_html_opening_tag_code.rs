use crate::tmpl::ComponentAttribute;
use quote::quote;

/// Generate HTML opening tag code with support for dynamic attributes
///
/// This function generates Rust code that produces HTML opening tags with attributes.
/// It does NOT generate closing tags - only the opening tag portion.
///
/// # Examples
/// - `<div>` for a tag with no attributes
/// - `<div class="container">` for a tag with a literal attribute
/// - `<input value="{user_input}">` for a tag with a variable attribute
/// - `<span data-count="{count + 1}">` for a tag with an expression attribute
///
/// # Arguments
/// * `tag` - The HTML tag name (e.g., "div", "span", "input")
/// * `attributes` - A map of attribute names to their values (literals, variables, or expressions)
/// * `self_closing` - Whether the tag is self-closing
/// * `element_id` - Optional element ID for event listener registration
///
/// # Returns
/// A `TokenStream` that generates a String containing the HTML opening tag
pub(crate) fn generate_html_opening_tag_code(
    tag: &str,
    attributes: &std::collections::HashMap<String, ComponentAttribute>,
    self_closing: bool,
    element_id: Option<&str>,
) -> proc_macro2::TokenStream {
    let closing_bracket = if self_closing { " />" } else { ">" };

    // Filter out event handlers from HTML attributes
    let mut html_attributes: std::collections::HashMap<String, ComponentAttribute> = attributes
        .iter()
        .filter_map(|(k, v)| {
            match v {
                ComponentAttribute::EventHandler(_) => None, // Exclude event handlers from HTML
                _ => Some((k.clone(), v.clone())),
            }
        })
        .collect();

    // Add element ID if provided (for event listener registration)
    if let Some(id) = element_id {
        html_attributes.insert("id".to_owned(), ComponentAttribute::Literal(id.to_owned()));
    }

    if html_attributes.is_empty() {
        quote! { format!("<{}{}", #tag, #closing_bracket) }
    } else {
        let mut attr_keys: Vec<_> = html_attributes.keys().collect();
        attr_keys.sort();

        let attr_parts: Vec<proc_macro2::TokenStream> = attr_keys
            .iter()
            .map(|k| {
                let v = &html_attributes[*k];
                match v {
                    ComponentAttribute::Literal(lit) => {
                        quote! { format!("{}=\"{}\"", #k, #lit) }
                    }
                    ComponentAttribute::Variable(var) => {
                        if let Ok(var_ident) = syn::parse_str::<syn::Ident>(var) {
                            quote! { format!("{}=\"{}\"", #k, #var_ident) }
                        } else {
                            quote! { format!("{}=\"{}\"", #k, #var) }
                        }
                    }
                    ComponentAttribute::Expression(expr) => {
                        if let Ok(expr_node) = syn::parse_str::<syn::Expr>(expr) {
                            quote! { format!("{}=\"{}\"", #k, (#expr_node)) }
                        } else {
                            // Fallback for things that are not valid expressions,
                            // treat them as literal strings.
                            quote! { format!("{}=\"{}\"", #k, #expr) }
                        }
                    }
                    ComponentAttribute::EventHandler(_) => {
                        // This should never happen since we filtered them out, but just in case
                        quote! { format!("") }
                    }
                }
            })
            .collect();

        quote! {
            {
                let attrs = vec![#(#attr_parts),*];
                format!("<{} {}{}", #tag, attrs.join(" "), #closing_bracket)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use std::collections::HashMap;

    #[test]
    fn test_generate_html_opening_tag_code_empty_attributes() {
        let tag = "div";
        let attributes = HashMap::new();

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();
        let expected = quote! { format!("<{}{}", "div", ">") }.to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_html_opening_tag_code_single_literal_attribute() {
        let tag = "div";
        let mut attributes = HashMap::new();

        attributes.insert(
            "class".to_owned(),
            ComponentAttribute::Literal("container".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("format !"));
        assert!(result.contains("class"));
        assert!(result.contains("container"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_single_variable_attribute() {
        let tag = "input";
        let mut attributes = HashMap::new();

        attributes.insert(
            "value".to_owned(),
            ComponentAttribute::Variable("user_input".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("user_input"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_single_expression_attribute() {
        let tag = "span";
        let mut attributes = HashMap::new();

        attributes.insert(
            "data-count".to_owned(),
            ComponentAttribute::Expression("count + 1".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("count + 1"));
        assert!(result.contains("data-count"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_multiple_mixed_attributes() {
        let tag = "button";
        let mut attributes = HashMap::new();

        attributes.insert(
            "class".to_owned(),
            ComponentAttribute::Literal("btn-primary".to_owned()),
        );
        attributes.insert(
            "id".to_owned(),
            ComponentAttribute::Variable("button_id".to_owned()),
        );
        attributes.insert(
            "data-value".to_owned(),
            ComponentAttribute::Expression("counter * 2".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("class"));
        assert!(result.contains("btn-primary"));
        assert!(result.contains("id"));
        assert!(result.contains("button_id"));
        assert!(result.contains("data-value"));
        assert!(result.contains("counter * 2"));
        assert!(result.contains("button"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_special_html_tags() {
        let test_cases = vec!["img", "br", "hr", "input", "meta"];

        for tag in test_cases {
            let attributes = HashMap::new();
            let result = generate_html_opening_tag_code(tag, &attributes, true, None).to_string();

            assert!(result.contains(tag));
        }
    }

    #[test]
    fn test_generate_html_opening_tag_code_attribute_with_special_characters() {
        let tag = "div";
        let mut attributes = HashMap::new();

        attributes.insert(
            "data-special".to_owned(),
            ComponentAttribute::Literal("value with spaces & symbols".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("data-special"));
        assert!(result.contains("value with spaces & symbols"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_invalid_variable_name() {
        let tag = "div";
        let mut attributes = HashMap::new();

        // Test with a variable name that can't be parsed as a valid Rust identifier
        attributes.insert(
            "class".to_owned(),
            ComponentAttribute::Variable("123invalid".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("123invalid"));
        assert!(result.contains("class"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_invalid_expression() {
        let tag = "div";
        let mut attributes = HashMap::new();

        // Test with an expression that can't be parsed as valid Rust code
        attributes.insert(
            "data-value".to_owned(),
            ComponentAttribute::Expression("invalid syntax here +++".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("invalid syntax here +++"));
        assert!(result.contains("data-value"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_empty_attribute_values() {
        let tag = "input";
        let mut attributes = HashMap::new();

        attributes.insert(
            "placeholder".to_owned(),
            ComponentAttribute::Literal("".to_owned()),
        );
        attributes.insert(
            "value".to_owned(),
            ComponentAttribute::Variable("empty_var".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("placeholder"));
        assert!(result.contains("value"));
        assert!(result.contains("empty_var"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_complex_expression() {
        let tag = "div";
        let mut attributes = HashMap::new();

        attributes.insert(
            "style".to_owned(),
            ComponentAttribute::Expression(
                "format!(\"width: {}px; height: {}px\", width, height)".to_owned(),
            ),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("style"));
        assert!(result.contains("format !"));
        assert!(result.contains("width"));
        assert!(result.contains("height"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_boolean_like_attributes() {
        let tag = "input";
        let mut attributes = HashMap::new();

        attributes.insert(
            "disabled".to_owned(),
            ComponentAttribute::Variable("is_disabled".to_owned()),
        );
        attributes.insert(
            "checked".to_owned(),
            ComponentAttribute::Expression("user.is_admin()".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("disabled"));
        assert!(result.contains("is_disabled"));
        assert!(result.contains("checked"));
        assert!(result.contains("is_admin"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_with_helper_attributes() {
        let tag = "section";
        let mut attributes = HashMap::new();

        attributes.insert(
            "literal".to_owned(),
            ComponentAttribute::Literal("test-value".to_owned()),
        );
        attributes.insert(
            "variable".to_owned(),
            ComponentAttribute::Variable("test_var".to_owned()),
        );
        attributes.insert(
            "expression".to_owned(),
            ComponentAttribute::Expression("test_expr()".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("literal"));
        assert!(result.contains("test-value"));
        assert!(result.contains("variable"));
        assert!(result.contains("test_var"));
        assert!(result.contains("expression"));
        assert!(result.contains("test_expr"));
    }

    #[test]
    fn test_generate_html_opening_tag_code_with_quotes_in_values() {
        let tag = "input";
        let mut attributes = HashMap::new();

        attributes.insert(
            "placeholder".to_owned(),
            ComponentAttribute::Literal("Enter \"quoted\" text".to_owned()),
        );

        let result = generate_html_opening_tag_code(tag, &attributes, false, None).to_string();

        assert!(result.contains("placeholder"));
        assert!(result.contains("quoted"));
    }
}
