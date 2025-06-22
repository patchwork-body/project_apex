use crate::tmpl::{HtmlContent, parse_tag_content::*, parse_variable_content::*};
use syn::Result;

/// Parse HTML content into structured representation
pub(crate) fn parse_tmpl_structure(content: &str) -> Result<Vec<HtmlContent>> {
    let mut result = Vec::new();
    let mut chars = content.chars().peekable();
    let mut current_text = String::new();
    let mut inside_tag = false;

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Save accumulated text
                if !current_text.is_empty() {
                    result.push(HtmlContent::Text(current_text.trim().to_owned()));
                    current_text.clear();
                }

                // Parse tag
                if let Some(tag_content) = parse_tag_content(&mut chars)? {
                    result.push(tag_content);
                }
                inside_tag = false;
            }
            '{' if !inside_tag => {
                // Only treat { as variable start when not inside a tag
                // Save accumulated text
                if !current_text.is_empty() {
                    result.push(HtmlContent::Text(current_text.clone()));
                    current_text.clear();
                }

                // Parse variable
                if let Some(var_content) = parse_variable_content(&mut chars)? {
                    result.push(HtmlContent::Variable(var_content));
                }
            }
            _ => {
                current_text.push(ch);
            }
        }
    }

    // Save remaining text
    if !current_text.is_empty() {
        result.push(HtmlContent::Text(current_text.trim().to_owned()));
    }

    Ok(result)
}
