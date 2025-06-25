use std::str::Chars;
use syn::Result;

/// Parse variable content from character iterator during template processing.
///
/// This function is essential for the Apex template macro system as it extracts
/// variable expressions from template strings that contain interpolated variables
/// in the format `{variable_name}` or `{complex.expression}`.
///
/// ## Why this function is needed:
/// - Templates often contain variable interpolations like `{user.name}` or `{count + 1}`
/// - The parser needs to extract the content between braces while handling nested expressions
/// - Proper brace matching ensures complex expressions with nested objects/arrays are parsed correctly
///
/// ## What it does:
/// - Reads characters from the iterator until it finds the matching closing brace `}`
/// - Tracks brace depth to handle nested expressions (e.g., `{user.data.{nested}}`)
/// - Returns the trimmed variable content as a string, or None if the content is empty/whitespace
/// - Consumes the closing brace from the iterator before returning
///
/// ## Examples of parsed content:
/// - `{name}` → returns "name"
/// - `{user.email}` → returns "user.email"
/// - `{items.{index}}` → returns "items.{index}" (preserves nesting for later processing)
/// - `{  }` → returns None (empty/whitespace content)
///
/// # Arguments
/// * `chars` - Mutable peekable iterator over template string characters, positioned after opening `{`
///
/// # Returns
/// * `Ok(Some(String))` - Variable content if non-empty after trimming
/// * `Ok(None)` - If the variable content is empty or only whitespace
/// * `Err` - Currently not used but allows for future error handling
pub(crate) fn parse_variable_content(
    chars: &mut std::iter::Peekable<Chars<'_>>,
) -> Result<Option<String>> {
    let mut var_str = String::new();
    let mut brace_depth = 1;

    // Read until matching '}'
    while let Some(&ch) = chars.peek() {
        if ch == '}' {
            brace_depth -= 1;
            if brace_depth == 0 {
                chars.next(); // consume '}'
                break;
            }
        } else if ch == '{' {
            brace_depth += 1;
        }

        var_str.push(chars.next().unwrap());
    }

    if var_str.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(var_str.trim().to_owned()))
}

/// Analyze a variable expression to determine if it's likely signal-based (dynamic)
///
/// This function uses heuristics to detect signal-based variables:
/// - Field access on `self` with signal-like patterns (e.g., `self.count`, `self.signal_field`)
/// - Direct signal references (e.g., `count` where count is a Signal)
/// - Method calls on signal-like objects
///
/// Note: This is a compile-time heuristic analysis. For precise detection,
/// we would need type information from the Rust compiler.
pub(crate) fn is_signal_variable(var_content: &str) -> bool {
    let trimmed = var_content.trim();

    // Pattern 1: Any expression containing self field access
    // Examples: self.count, self.title, self.field.method(), format!("{}", self.name)
    if trimmed.contains("self.") {
        return true; // Any expression containing self field access is potentially dynamic
    }

    // Pattern 2: Direct signal variable reference
    // This is harder to detect without type info, but we can check for common signal patterns
    if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') && !trimmed.is_empty() {
        // Simple identifier - could be a signal variable
        // For now, assume non-self identifiers are static unless proven otherwise
        return false;
    }

    // Pattern 3: Method calls on signals
    // Examples: signal.get(), signal.get_value(), self.count.get()
    if trimmed.contains(".get()") || trimmed.contains(".get_value()") {
        return true;
    }

    // Default: assume static for complex expressions without clear signal patterns
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a peekable char iterator from a string
    fn create_char_iter(s: &str) -> std::iter::Peekable<Chars<'_>> {
        s.chars().peekable()
    }

    #[test]
    fn test_simple_variable_name() {
        let mut chars = create_char_iter("name}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("name".to_owned()));
    }

    #[test]
    fn test_dotted_variable_name() {
        let mut chars = create_char_iter("user.email}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("user.email".to_owned()));
    }

    #[test]
    fn test_complex_expression() {
        let mut chars = create_char_iter("count + 1}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("count + 1".to_owned()));
    }

    #[test]
    fn test_empty_content() {
        let mut chars = create_char_iter("}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_whitespace_only_content() {
        let mut chars = create_char_iter("   }");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_whitespace_with_tabs_and_newlines() {
        let mut chars = create_char_iter(" \t\n }");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_content_with_leading_trailing_whitespace() {
        let mut chars = create_char_iter("  name  }");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("name".to_owned()));
    }

    #[test]
    fn test_single_level_nested_braces() {
        let mut chars = create_char_iter("items.{index}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("items.{index}".to_owned()));
    }

    #[test]
    fn test_multiple_level_nested_braces() {
        let mut chars = create_char_iter("data.{user.{profile.{name}}}}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("data.{user.{profile.{name}}}".to_owned()));
    }

    #[test]
    fn test_nested_braces_with_whitespace() {
        let mut chars = create_char_iter("  obj.{ nested.{ deep } }  }");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("obj.{ nested.{ deep } }".to_owned()));
    }

    #[test]
    fn test_multiple_closing_braces_at_same_level() {
        let mut chars = create_char_iter("test}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("test".to_owned()));
        // Verify the extra '}' is left in the iterator
        assert_eq!(chars.next(), Some('}'));
    }

    #[test]
    fn test_multiple_opening_braces() {
        let mut chars = create_char_iter("{{nested}}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("{{nested}}".to_owned()));
    }

    #[test]
    fn test_balanced_braces_complex_expression() {
        let mut chars = create_char_iter("arr[{index}].{field}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("arr[{index}].{field}".to_owned()));
    }

    #[test]
    fn test_empty_iterator() {
        let mut chars = create_char_iter("");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_no_closing_brace() {
        let mut chars = create_char_iter("name_without_closing_brace");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("name_without_closing_brace".to_owned()));
    }

    #[test]
    fn test_immediate_closing_brace() {
        let mut chars = create_char_iter("}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_nested_empty_braces() {
        let mut chars = create_char_iter("test.{}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("test.{}".to_owned()));
    }

    #[test]
    fn test_special_characters() {
        let mut chars = create_char_iter("user.name!@#$%^&*()_+-=[]|;':,.<>?/}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(
            result,
            Some("user.name!@#$%^&*()_+-=[]|;':,.<>?/".to_owned())
        );
    }

    #[test]
    fn test_unicode_characters() {
        let mut chars = create_char_iter("用户.姓名}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("用户.姓名".to_owned()));
    }

    #[test]
    fn test_brace_depth_tracking() {
        // Test that brace depth is properly tracked
        let mut chars = create_char_iter("a{b{c{d}e}f}g}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("a{b{c{d}e}f}g".to_owned()));
    }

    #[test]
    fn test_deeply_nested_braces() {
        // Test with many levels of nesting
        let mut chars = create_char_iter("level1{level2{level3{level4{level5}}}}}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(
            result,
            Some("level1{level2{level3{level4{level5}}}}".to_owned())
        );
    }

    #[test]
    fn test_mixed_content_with_braces() {
        let mut chars = create_char_iter("fn(param){return value;}()}");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("fn(param){return value;}()".to_owned()));
    }

    #[test]
    fn test_iterator_position_after_parsing() {
        let mut chars = create_char_iter("variable}remaining_content");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("variable".to_owned()));

        // Check that iterator is positioned correctly after the closing brace
        let remaining: String = chars.collect();
        assert_eq!(remaining, "remaining_content");
    }

    #[test]
    fn test_iterator_position_with_nested_braces() {
        let mut chars = create_char_iter("var{nested}}after");
        let result = parse_variable_content(&mut chars).unwrap();
        assert_eq!(result, Some("var{nested}".to_owned()));

        // Check remaining content
        let remaining: String = chars.collect();
        assert_eq!(remaining, "after");
    }

    // Tests for signal detection logic
    #[test]
    fn test_is_signal_variable_self_field() {
        assert!(is_signal_variable("self.count"));
        assert!(is_signal_variable("self.title"));
        assert!(is_signal_variable("self.is_visible"));
        assert!(is_signal_variable("self._private_field"));
    }

    #[test]
    fn test_is_signal_variable_self_complex() {
        assert!(is_signal_variable("self.count + 1"));
        assert!(is_signal_variable("self.title + \" suffix\""));
        assert!(is_signal_variable("format!(\"{}\", self.name)"));
        assert!(is_signal_variable("self.value * 2"));
        assert!(is_signal_variable("self.text.to_string()"));
    }

    #[test]
    fn test_is_signal_variable_method_calls() {
        assert!(is_signal_variable("signal.get()"));
        assert!(is_signal_variable("my_signal.get_value()"));
        assert!(is_signal_variable("  signal.get()  ")); // with whitespace
        assert!(is_signal_variable("user_signal.get()"));
    }

    #[test]
    fn test_is_signal_variable_static() {
        assert!(!is_signal_variable("name"));
        assert!(!is_signal_variable("user.email"));
        assert!(!is_signal_variable("items.len()"));
        assert!(!is_signal_variable("count + 1"));
        assert!(!is_signal_variable("format!(\"Hello {}\", name)"));
        assert!(!is_signal_variable("variable_name"));
        assert!(!is_signal_variable("obj.property"));
    }

    #[test]
    fn test_is_signal_variable_edge_cases() {
        assert!(!is_signal_variable(""));
        assert!(!is_signal_variable("   "));
        assert!(!is_signal_variable("123"));
        assert!(!is_signal_variable("true"));
        assert!(!is_signal_variable("\"string literal\""));
        assert!(!is_signal_variable("false"));
    }

    #[test]
    fn test_is_signal_variable_self_method_chain() {
        // self field access with method chains should be considered dynamic
        assert!(is_signal_variable("self.field.method()"));
        // But complex chains without self should be static
        assert!(!is_signal_variable("obj.field.method()"));
    }

    #[test]
    fn test_is_signal_variable_whitespace_handling() {
        assert!(is_signal_variable("  self.count  "));
        assert!(is_signal_variable("\tself.title\t"));
        assert!(is_signal_variable("\nself.value\n"));
        assert!(!is_signal_variable("  name  "));
    }
}
