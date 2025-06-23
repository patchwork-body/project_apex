use crate::tmpl::ComponentAttribute;
use syn::Result;

/// Parse a single attribute string into a structured key-value pair
///
/// ## Purpose and Context
///
/// The `parse_single_attribute` function is a critical component of the Apex template engine's
/// macro system. It serves as the foundational parser for individual component and HTML element
/// attributes within the framework's HTML-like template syntax. This function enables the Apex
/// framework to understand and process different types of attribute values, bridging the gap
/// between declarative template syntax and runtime component instantiation.
///
/// ### Why This Function Exists
///
/// Apex uses a **string-based attribute parsing approach** as part of its current component
/// property passing system. This approach allows for HTML-like syntax in templates while
/// maintaining flexibility in handling different value types. The function is essential because:
///
/// 1. **Template Syntax Support**: Enables HTML-like component declarations (e.g., `<button class="primary" count={counter} />`)
/// 2. **Runtime Type Resolution**: Converts string-based attributes to typed component properties
/// 3. **Macro Expansion**: Part of the procedural macro system that transforms templates into Rust code
/// 4. **Attribute Type Detection**: Differentiates between literals, variables, and expressions
///
/// ### Integration with Apex Architecture
///
/// This function operates within a larger parsing pipeline:
/// ```ignore
/// Template String → parse_production_component_attributes_from_str() → parse_single_attribute() → ComponentAttribute
/// ```
///
/// The parsed attributes are then used by `generate_dynamic_component_code()` to create runtime
/// component instantiation code that calls `Component::from_attributes()` with a `HashMap<String, String>`.
///
/// ## Functionality
///
/// The function analyzes attribute strings and categorizes values into three distinct types:
///
/// ### 1. **Literal Values** (`ComponentAttribute::Literal`)
/// - **Quoted strings**: `class="primary"` → `"primary"`
/// - **Unquoted values**: `disabled=true` → `"true"`
/// - Used for static, compile-time known values
///
/// ### 2. **Variable References** (`ComponentAttribute::Variable`)
/// - **Simple variables**: `count={counter}` → `counter`
/// - Must contain only alphanumeric characters and underscores
/// - Refers to variables in the template's scope
///
/// ### 3. **Complex Expressions** (`ComponentAttribute::Expression`)
/// - **Computed values**: `total={items.len() + 1}` → `items.len() + 1`
/// - **Function calls**: `onclick={handle_click()}` → `handle_click()`
/// - Any braced content that's not a simple variable
///
/// ## Algorithm Details
///
/// 1. **Attribute Detection**: Searches for '=' separator to identify key-value pairs
/// 2. **Key Extraction**: Trims and extracts everything before '=' as the attribute name
/// 3. **Value Analysis**: Determines value type based on syntax:
///    - `"..."` → Literal (removes quotes)
///    - `{simple_name}` → Variable (validates identifier format)
///    - `{complex_expr}` → Expression (preserves entire expression)
///    - `unquoted` → Literal (preserves as-is)
/// 4. **Result Construction**: Returns structured `ComponentAttribute` enum variant
///
/// ## Error Handling
///
/// - **Missing '='**: Returns `Ok(None)` for malformed attributes
/// - **Invalid syntax**: Gracefully handles malformed expressions by treating them as literals
/// - **Empty values**: Processes empty strings as valid literal values
///
/// ## Examples
///
/// ```rust,ignore
/// // Literal string attribute
/// let result = parse_single_attribute("class=\"btn-primary\"");
/// // Returns: Ok(Some(("class", ComponentAttribute::Literal("btn-primary"))))
///
/// // Variable reference
/// let result = parse_single_attribute("count={counter}");
/// // Returns: Ok(Some(("count", ComponentAttribute::Variable("counter"))))
///
/// // Complex expression
/// let result = parse_single_attribute("total={items.len() + 1}");
/// // Returns: Ok(Some(("total", ComponentAttribute::Expression("items.len() + 1"))))
///
/// // Unquoted literal
/// let result = parse_single_attribute("disabled=true");
/// // Returns: Ok(Some(("disabled", ComponentAttribute::Literal("true"))))
///
/// // Malformed attribute (no equals sign)
/// let result = parse_single_attribute("standalone");
/// // Returns: Ok(None)
/// ```
///
/// ## Template Usage Context
///
/// This function processes attributes from template syntax like:
/// ```ignore
/// <custom-button
///     class="primary"          // → Literal
///     count={counter}          // → Variable
///     total={items.len()}      // → Expression
///     disabled=true            // → Literal
/// />
/// ```
///
/// ## Generated Code Impact
///
/// The parsed attributes influence the generated Rust code structure:
/// ```rust,ignore
/// // Template: <Button count={counter} />
/// // Generated code includes:
/// {
///     let mut attrs = std::collections::HashMap::new();
///     attrs.insert("count".to_string(), counter.to_string());
///     let component = Button::from_attributes(&attrs);
///     apex::View::render(&component);
/// }
/// ```
///
/// ## Current Limitations & Future Considerations
///
/// The current string-based approach has trade-offs outlined in the framework's architecture:
/// - **Pros**: Simple implementation, HTML-like syntax, runtime flexibility
/// - **Cons**: No compile-time type safety, runtime parsing overhead, limited type support
///
/// Alternative approaches being considered include builder patterns, direct instantiation,
/// and trait-based properties for improved type safety and performance.
///
/// ## Parameters
///
/// * `attr_str` - A string slice containing a single attribute declaration (e.g., "class=\"primary\"")
///
/// ## Returns
///
/// * `Ok(Some((key, value)))` - Successfully parsed attribute with structured value type
/// * `Ok(None)` - Input string doesn't contain a valid key-value pair (no '=' found)
/// * `Err(syn::Error)` - Parsing error (currently unused but reserves for future validation)
pub(crate) fn parse_single_attribute(
    attr_str: &str,
) -> Result<Option<(String, ComponentAttribute)>> {
    if let Some(eq_pos) = attr_str.find('=') {
        let key = attr_str[..eq_pos].trim().to_owned();
        let value = attr_str[eq_pos + 1..].trim();

        // Check if this is an event handler (starts with "on")
        let is_event_handler = key.starts_with("on") && key.len() > 2;

        let attr_value = if value.starts_with('"') && value.ends_with('"') {
            if is_event_handler {
                ComponentAttribute::EventHandler(value[1..value.len() - 1].to_string())
            } else {
                ComponentAttribute::Literal(value[1..value.len() - 1].to_string())
            }
        } else if value.starts_with('{') && value.ends_with('}') {
            let inner = &value[1..value.len() - 1];

            if is_event_handler {
                // Event handlers are always treated as expressions/variables, not literals
                ComponentAttribute::EventHandler(inner.to_owned())
            } else if inner.chars().all(|c| c.is_alphanumeric() || c == '_') {
                ComponentAttribute::Variable(inner.to_owned())
            } else {
                ComponentAttribute::Expression(inner.to_owned())
            }
        } else if is_event_handler {
            ComponentAttribute::EventHandler(value.to_owned())
        } else {
            ComponentAttribute::Literal(value.to_owned())
        };

        Ok(Some((key, attr_value)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_equals_sign_returns_none() {
        // Test the branch where no '=' is found
        let result = parse_single_attribute("standalone").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_string_returns_none() {
        // Test edge case with empty string
        let result = parse_single_attribute("").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_whitespace_only_returns_none() {
        // Test edge case with whitespace only
        let result = parse_single_attribute("   ").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_quoted_literal_attribute() {
        // Test quoted string - should return Literal with quotes removed
        let result = parse_single_attribute("class=\"btn-primary\"").unwrap();
        assert_eq!(
            result,
            Some((
                "class".to_owned(),
                ComponentAttribute::Literal("btn-primary".to_owned())
            ))
        );
    }

    #[test]
    fn test_quoted_empty_literal() {
        // Test quoted empty string
        let result = parse_single_attribute("placeholder=\"\"").unwrap();
        assert_eq!(
            result,
            Some((
                "placeholder".to_owned(),
                ComponentAttribute::Literal("".to_owned())
            ))
        );
    }

    #[test]
    fn test_variable_attribute() {
        // Test simple variable - alphanumeric + underscores only
        let result = parse_single_attribute("count={counter}").unwrap();
        assert_eq!(
            result,
            Some((
                "count".to_owned(),
                ComponentAttribute::Variable("counter".to_owned())
            ))
        );
    }

    #[test]
    fn test_variable_with_underscores() {
        // Test variable with underscores
        let result = parse_single_attribute("value={my_var_name}").unwrap();
        assert_eq!(
            result,
            Some((
                "value".to_owned(),
                ComponentAttribute::Variable("my_var_name".to_owned())
            ))
        );
    }

    #[test]
    fn test_variable_with_numbers() {
        // Test variable with numbers
        let result = parse_single_attribute("id={var123}").unwrap();
        assert_eq!(
            result,
            Some((
                "id".to_owned(),
                ComponentAttribute::Variable("var123".to_owned())
            ))
        );
    }

    #[test]
    fn test_expression_attribute() {
        // Test complex expression - contains non-alphanumeric/underscore characters
        let result = parse_single_attribute("total={items.len() + 1}").unwrap();
        assert_eq!(
            result,
            Some((
                "total".to_owned(),
                ComponentAttribute::Expression("items.len() + 1".to_owned())
            ))
        );
    }

    #[test]
    fn test_expression_with_dots() {
        // Test expression with dots
        let result = parse_single_attribute("name={user.profile.name}").unwrap();
        assert_eq!(
            result,
            Some((
                "name".to_owned(),
                ComponentAttribute::Expression("user.profile.name".to_owned())
            ))
        );
    }

    #[test]
    fn test_expression_with_parentheses() {
        // Test expression with function calls - onclick should be detected as EventHandler
        let result = parse_single_attribute("onclick={handle_click()}").unwrap();
        assert_eq!(
            result,
            Some((
                "onclick".to_owned(),
                ComponentAttribute::EventHandler("handle_click()".to_owned())
            ))
        );
    }

    #[test]
    fn test_expression_with_operators() {
        // Test expression with various operators
        let result = parse_single_attribute("computed={a + b * c - d}").unwrap();
        assert_eq!(
            result,
            Some((
                "computed".to_owned(),
                ComponentAttribute::Expression("a + b * c - d".to_owned())
            ))
        );
    }

    #[test]
    fn test_unquoted_literal_attribute() {
        // Test unquoted value - should return as Literal
        let result = parse_single_attribute("disabled=true").unwrap();
        assert_eq!(
            result,
            Some((
                "disabled".to_owned(),
                ComponentAttribute::Literal("true".to_owned())
            ))
        );
    }

    #[test]
    fn test_unquoted_literal_number() {
        // Test unquoted number
        let result = parse_single_attribute("tabindex=0").unwrap();
        assert_eq!(
            result,
            Some((
                "tabindex".to_owned(),
                ComponentAttribute::Literal("0".to_owned())
            ))
        );
    }

    #[test]
    fn test_whitespace_handling_in_key() {
        // Test that whitespace around key is trimmed
        let result = parse_single_attribute("  class  =\"value\"").unwrap();
        assert_eq!(
            result,
            Some((
                "class".to_owned(),
                ComponentAttribute::Literal("value".to_owned())
            ))
        );
    }

    #[test]
    fn test_whitespace_handling_in_value() {
        // Test that whitespace around value is trimmed
        let result = parse_single_attribute("class=  \"value\"  ").unwrap();
        assert_eq!(
            result,
            Some((
                "class".to_owned(),
                ComponentAttribute::Literal("value".to_owned())
            ))
        );
    }

    #[test]
    fn test_empty_key() {
        // Test edge case with empty key
        let result = parse_single_attribute("=\"value\"").unwrap();
        assert_eq!(
            result,
            Some((
                "".to_owned(),
                ComponentAttribute::Literal("value".to_owned())
            ))
        );
    }

    #[test]
    fn test_empty_value() {
        // Test edge case with empty value
        let result = parse_single_attribute("key=").unwrap();
        assert_eq!(
            result,
            Some(("key".to_owned(), ComponentAttribute::Literal("".to_owned())))
        );
    }

    #[test]
    fn test_braces_without_content() {
        // Test empty braces - should be treated as Variable (empty string is alphanumeric)
        let result = parse_single_attribute("empty={}").unwrap();
        assert_eq!(
            result,
            Some((
                "empty".to_owned(),
                ComponentAttribute::Variable("".to_owned())
            ))
        );
    }

    #[test]
    fn test_single_quote_literal() {
        // Test that single quotes don't get special treatment (only double quotes do)
        let result = parse_single_attribute("class='primary'").unwrap();
        assert_eq!(
            result,
            Some((
                "class".to_owned(),
                ComponentAttribute::Literal("'primary'".to_owned())
            ))
        );
    }

    #[test]
    fn test_mixed_quotes_in_braces() {
        // Test expression with quotes inside braces
        let result = parse_single_attribute("format={format!(\"Hello {}!\", name)}").unwrap();
        assert_eq!(
            result,
            Some((
                "format".to_owned(),
                ComponentAttribute::Expression("format!(\"Hello {}!\", name)".to_owned())
            ))
        );
    }

    #[test]
    fn test_special_characters_in_expression() {
        // Test expression with various special characters
        let result = parse_single_attribute("complex={items[0].value @ timestamp}").unwrap();
        assert_eq!(
            result,
            Some((
                "complex".to_owned(),
                ComponentAttribute::Expression("items[0].value @ timestamp".to_owned())
            ))
        );
    }

    #[test]
    fn test_quoted_string_with_equals() {
        // Test quoted string that contains equals sign
        let result = parse_single_attribute("data=\"key=value\"").unwrap();
        assert_eq!(
            result,
            Some((
                "data".to_owned(),
                ComponentAttribute::Literal("key=value".to_owned())
            ))
        );
    }

    #[test]
    fn test_braces_at_start_only() {
        // Test value that starts with { but doesn't end with }
        let result = parse_single_attribute("value={incomplete").unwrap();
        assert_eq!(
            result,
            Some((
                "value".to_owned(),
                ComponentAttribute::Literal("{incomplete".to_owned())
            ))
        );
    }

    #[test]
    fn test_braces_at_end_only() {
        // Test value that ends with } but doesn't start with {
        let result = parse_single_attribute("value=incomplete}").unwrap();
        assert_eq!(
            result,
            Some((
                "value".to_owned(),
                ComponentAttribute::Literal("incomplete}".to_owned())
            ))
        );
    }

    #[test]
    fn test_multiple_equals_signs() {
        // Test that only the first equals sign is used as separator
        let result = parse_single_attribute("equation=x=y+z").unwrap();
        assert_eq!(
            result,
            Some((
                "equation".to_owned(),
                ComponentAttribute::Literal("x=y+z".to_owned())
            ))
        );
    }
}
