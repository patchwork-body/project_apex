use std::str::Chars;
use syn::Result;

use crate::tmpl::{
    ComponentAttribute, HtmlContent, parse_component_attributes_from_str::*,
    parse_tag_name_and_attributes::*,
};

/// Check if a tag name represents a component (PascalCase only)
fn is_component(tag_name: &str) -> bool {
    if tag_name.is_empty() {
        return false;
    }

    // Only PascalCase components: Counter, SomeComponent, etc.
    // Components should be resolved from scope like variables
    let first_char = tag_name.chars().next().unwrap_or_default();
    first_char.is_ascii_uppercase()
}

/// Check if attributes contain any event handlers
fn has_event_handlers(attributes: &std::collections::HashMap<String, ComponentAttribute>) -> bool {
    attributes
        .iter()
        .any(|(_, attr)| matches!(attr, ComponentAttribute::EventHandler(_)))
}

/// Check if attributes contain any dynamic variables
fn has_dynamic_variables(
    attributes: &std::collections::HashMap<String, ComponentAttribute>,
) -> bool {
    attributes
        .iter()
        .any(|(_, attr)| matches!(attr, ComponentAttribute::DynamicVariable { .. }))
}

/// Generate a unique element ID for event listener registration
fn generate_element_id() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("apex_element_{id}")
}

/// Parse tag content from character iterator
pub(crate) fn parse_tag_content(
    chars: &mut std::iter::Peekable<Chars<'_>>,
) -> Result<Option<HtmlContent>> {
    let mut tag_str = String::new();
    let mut brace_depth = 0;

    // Read until '>' but handle nested braces properly
    while let Some(&ch) = chars.peek() {
        if ch == '>' && brace_depth == 0 {
            chars.next(); // consume '>'
            break;
        }
        let consumed_ch = chars.next().unwrap();
        if consumed_ch == '{' {
            brace_depth += 1;
        } else if consumed_ch == '}' {
            brace_depth -= 1;
        }
        tag_str.push(consumed_ch);
    }

    if tag_str.is_empty() {
        return Ok(None);
    }

    // Parse tag name and attributes more carefully
    // Normalize whitespace within the tag to handle multiline tags
    let normalized_tag_str = tag_str.split_whitespace().collect::<Vec<_>>().join(" ");
    let (tag_name, attributes_str) = parse_tag_name_and_attributes(&normalized_tag_str);
    let self_closing = normalized_tag_str.trim_end().ends_with('/');

    // Check if this is a component tag (PascalCase or kebab-case)
    if is_component(&tag_name) {
        let attributes = parse_component_attributes_from_str(&attributes_str, self_closing)?;

        Ok(Some(HtmlContent::Component {
            tag: tag_name.to_owned(),
            attributes,
        }))
    } else {
        // Regular HTML element
        // First parse attributes to see if we need an element ID
        let attributes =
            parse_component_attributes_from_str(&attributes_str, false).unwrap_or_default();

        // Check if we need element ID (for event handlers or dynamic variables)
        let needs_element_id =
            has_event_handlers(&attributes) || has_dynamic_variables(&attributes);

        let (final_attributes, element_id) = if needs_element_id {
            // Generate element ID and re-parse attributes with context
            let element_id = generate_element_id();
            let attributes_with_context = parse_component_attributes_from_str_with_context(
                &attributes_str,
                false,
                Some(&element_id),
            )
            .unwrap_or_default();
            (attributes_with_context, Some(element_id))
        } else {
            (attributes, None)
        };

        Ok(Some(HtmlContent::Element {
            tag: tag_name.to_owned(),
            attributes: final_attributes,
            self_closing,
            element_id,
        }))
    }
}
