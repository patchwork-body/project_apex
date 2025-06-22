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
}
