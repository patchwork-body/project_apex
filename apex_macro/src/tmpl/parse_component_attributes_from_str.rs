use crate::tmpl::{ComponentAttribute, parse_single_attribute::*};
use syn::Result;

/// Parse component attributes with production-ready handling from string
pub(crate) fn parse_component_attributes_from_str(
    attributes_str: &str,
    _self_closing: bool,
) -> Result<std::collections::HashMap<String, ComponentAttribute>> {
    let mut attributes = std::collections::HashMap::new();

    if attributes_str.trim().is_empty() {
        return Ok(attributes);
    }

    // Parse attributes from the string, handling {variable} expressions properly
    let chars = attributes_str.chars().peekable();
    let mut current_attr = String::new();
    let mut brace_depth = 0;

    for ch in chars {
        if ch == ' ' && brace_depth == 0 && !current_attr.trim().is_empty() {
            // Process the current attribute only when we're not inside braces
            if let Some(attr) = parse_single_attribute(&current_attr)? {
                attributes.insert(attr.0, attr.1);
            }
            current_attr.clear();
        } else if ch == '{' {
            // Handle {expression} - read until matching } with proper nesting
            brace_depth += 1;
            current_attr.push(ch);
        } else if ch == '}' && brace_depth > 0 {
            brace_depth -= 1;
            current_attr.push(ch);
        } else {
            current_attr.push(ch);
        }
    }

    // Process the last attribute
    if !current_attr.trim().is_empty() {
        if let Some(attr) = parse_single_attribute(&current_attr)? {
            attributes.insert(attr.0, attr.1);
        }
    }

    Ok(attributes)
}
