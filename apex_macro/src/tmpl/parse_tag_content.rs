use std::str::Chars;
use syn::Result;

use crate::tmpl::{
    HtmlContent, parse_component_attributes_from_str::*, parse_tag_name_and_attributes::*,
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
    let (tag_name, attributes_str) = parse_tag_name_and_attributes(&tag_str);
    let self_closing = tag_str.trim_end().ends_with('/');

    // Check if this is a component tag (PascalCase or kebab-case)
    if is_component(&tag_name) {
        let attributes = parse_component_attributes_from_str(&attributes_str, self_closing)?;

        Ok(Some(HtmlContent::Component {
            tag: tag_name.to_owned(),
            attributes,
        }))
    } else {
        // Regular HTML element
        let attributes =
            parse_component_attributes_from_str(&attributes_str, false).unwrap_or_default();

        Ok(Some(HtmlContent::Element {
            tag: tag_name.to_owned(),
            attributes,
            self_closing,
        }))
    }
}
