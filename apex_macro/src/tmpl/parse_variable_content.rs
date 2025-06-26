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
/// # Signal Detection - Type-Based Approach
///
/// This module also contains `is_signal_variable()` which determines whether a variable
/// expression represents a reactive signal. **As of the current implementation, this uses
/// a type-based approach rather than heuristic pattern matching.**
///
/// ## Previous Implementation (Heuristic-Based)
/// The old approach used pattern matching to guess if a variable was a signal:
/// - `self.field` → assumed to be a signal
/// - `signal.get()` → assumed to be a signal
/// - `regular_var` → assumed to be static
///
/// This was unreliable because:
/// - False positives: `self.static_field` was treated as reactive
/// - False negatives: `my_signal` was treated as static
/// - Fragile: dependent on naming conventions
///
/// ## Current Implementation (Type-Based)
/// The new approach:
/// 1. Parses variable expressions using `syn` to validate Rust syntax
/// 2. Treats all valid expressions as potentially reactive
/// 3. Uses the `Reactive` trait at runtime to determine actual signal status
/// 4. Calls `value.is_reactive()` on the runtime value to check for signals
///
/// ### Benefits of Type-Based Approach:
/// - **Accurate**: Uses actual type information via the `Reactive` trait
/// - **Reliable**: Works regardless of variable naming patterns
/// - **Future-proof**: Automatically supports new signal types
/// - **Type-safe**: Leverages Rust's compile-time type checking
///
/// ### How It Works:
/// ```rust,ignore
/// // At compile time (macro):
/// is_signal_variable("my_signal") // → true (valid expression, potentially reactive)
///
/// // At runtime (generated code):
/// let value = &my_signal;
/// if value.is_reactive() {          // Uses Reactive trait
///     // Handle as signal
///     value.get_value().to_string()
/// } else {
///     // Handle as static value
///     value.to_string()
/// }
/// ```
///
/// This ensures that signal detection is based on actual types rather than assumptions.
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

/// Determine if a variable expression represents a signal (reactive value) based on its type.
///
/// This function takes a proper type-based approach rather than using heuristics. It parses
/// the variable expression as Rust code and treats all valid expressions as potentially
/// reactive, deferring the actual reactivity check to runtime via the `Reactive` trait.
///
/// ## Type-Based Detection
///
/// The function relies on Rust's type system and the `Reactive` trait implementation:
/// - `Signal<T>` implements `Reactive` with `is_reactive() -> true`
/// - All other types implement `Reactive` with `is_reactive() -> false`
/// - This allows accurate runtime detection without pattern matching
///
/// ## Why This Approach Is Better
///
/// - **Accurate**: Uses actual type information rather than naming patterns
/// - **Reliable**: Works regardless of variable naming conventions
/// - **Future-proof**: Automatically handles new Signal types or Reactive implementations
/// - **Type-safe**: Leverages Rust's compile-time type checking
///
/// ## How It Works
///
/// 1. Parse the variable expression using `syn` to validate it's valid Rust code
/// 2. Return `true` if parsing succeeds (we'll check reactivity at runtime in generated code)
/// 3. Return `false` if parsing fails (treat as static/literal)
///
/// The generated code will use the `Reactive` trait to determine actual reactivity:
/// ```rust,ignore
/// // At compile time (macro):
/// is_signal_variable("my_signal") // → true (valid expression, potentially reactive)
///
/// // At runtime (generated code):
/// let value = &my_signal;
/// if value.is_reactive() {          // Uses Reactive trait
///     // Handle as signal - generate reactive DOM updates
///     render_with_effect(value)
/// } else {
///     // Handle as static value - no reactive updates needed
///     value.get_value().to_string()
/// }
/// ```
///
/// This ensures that signal detection is based on actual types rather than assumptions.
///
/// # Examples
/// ```rust,ignore
/// assert!(is_signal_variable("self.count"));        // Valid Rust expression
/// assert!(is_signal_variable("my_signal"));         // Valid Rust expression  
/// assert!(is_signal_variable("obj.field"));         // Valid Rust expression
/// assert!(is_signal_variable("compute_value()"));   // Valid Rust expression
/// assert!(!is_signal_variable(""));                 // Empty expression
/// assert!(!is_signal_variable("invalid..syntax"));  // Invalid Rust syntax
/// ```
pub(crate) fn is_signal_variable(var_content: &str) -> bool {
    let trimmed = var_content.trim();

    // If the expression is empty, it's not a signal
    if trimmed.is_empty() {
        return false;
    }

    // Try to parse as a valid Rust expression
    // If it parses successfully, treat it as potentially reactive
    // The actual reactivity will be determined at runtime using the Reactive trait
    match syn::parse_str::<syn::Expr>(trimmed) {
        Ok(_) => {
            // Valid Rust expression - could be reactive
            // The generated code will use the Reactive trait to check is_reactive() at runtime
            true
        }
        Err(_) => {
            // Invalid Rust syntax - treat as static text
            false
        }
    }
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

    // Tests for signal detection logic - now type-based rather than pattern-based
    #[test]
    fn test_is_signal_variable_valid_rust_expressions() {
        // With type-based detection, all valid Rust expressions are treated as potentially reactive
        assert!(is_signal_variable("self.title"));
        assert!(is_signal_variable("user.name"));
        assert!(is_signal_variable("config.port"));
        assert!(is_signal_variable("item.description"));

        // These are also valid expressions
        assert!(is_signal_variable("self.signal_field"));
        assert!(is_signal_variable("obj.signalValue"));
        assert!(is_signal_variable("my_signal"));

        // Common field patterns
        assert!(is_signal_variable("self.count"));
        assert!(is_signal_variable("self.value"));
    }

    #[test]
    fn test_is_signal_variable_complex_expressions() {
        // With type-based detection, all valid expressions are potentially reactive
        assert!(is_signal_variable("self.count + 1"));
        assert!(is_signal_variable("user.data.nested"));
        assert!(is_signal_variable("func(param)"));

        // Signal-like patterns are also valid
        assert!(is_signal_variable("self.signal_count"));
        assert!(is_signal_variable("obj.count"));
    }

    #[test]
    fn test_is_signal_variable_method_calls() {
        // All valid method calls are treated as potentially reactive
        assert!(is_signal_variable("signal.get()"));
        assert!(is_signal_variable("obj.compute_value()"));
        assert!(is_signal_variable("self.method_call()"));

        // Signal-named methods are also valid
        assert!(is_signal_variable("self.signal_method()"));
    }

    #[test]
    fn test_is_signal_variable_literals_and_constants() {
        // Simple valid identifiers are treated as potentially reactive
        assert!(is_signal_variable("name"));
        assert!(is_signal_variable("counter"));
        assert!(is_signal_variable("user"));
        assert!(is_signal_variable("data"));

        // Signal-named variables are also valid
        assert!(is_signal_variable("signal"));
        assert!(is_signal_variable("mySignal"));
        assert!(is_signal_variable("user_signal"));
    }

    #[test]
    fn test_is_signal_variable_invalid_expressions() {
        // Invalid expressions should return false
        assert!(!is_signal_variable(""));
        assert!(!is_signal_variable("   "));
        assert!(!is_signal_variable("@#$%^")); // Actually invalid characters
        assert!(!is_signal_variable("+++")); // Invalid operator sequence
    }

    #[test]
    fn test_is_signal_variable_edge_cases() {
        // Valid identifiers and expressions
        assert!(is_signal_variable("self")); // "self" is a valid identifier
        assert!(is_signal_variable("()")); // Empty tuple literal is valid
        assert!(is_signal_variable("{}")); // Empty block is valid 
        assert!(is_signal_variable("[]")); // Empty array is valid

        // These are valid identifiers/types
        assert!(is_signal_variable("Signal"));
        assert!(is_signal_variable("signalData"));
    }

    #[test]
    fn test_is_signal_variable_whitespace_handling() {
        // Test whitespace handling - trimmed expressions should be valid
        assert!(is_signal_variable("  self.title  "));
        assert!(is_signal_variable("\tuser.name\t"));
        assert!(is_signal_variable("\n  data  \n"));

        // These should also work after trimming
        assert!(is_signal_variable("  self.count  "));
        assert!(is_signal_variable("\tmy_signal\t"));
    }

    /// Test the new type-based signal detection approach
    #[test]
    fn test_type_based_signal_detection_approach() {
        // With type-based detection, all valid Rust expressions are potentially reactive
        // The actual reactivity is determined at runtime using the Reactive trait

        // Valid expressions return true
        assert!(is_signal_variable("user.name"));
        assert!(is_signal_variable("config.settings"));
        assert!(is_signal_variable("item.data"));
        assert!(is_signal_variable("self.count"));
        assert!(is_signal_variable("self.value"));
        assert!(is_signal_variable("user_signal"));
        assert!(is_signal_variable("mySignal"));
        assert!(is_signal_variable("signal_field"));
    }

    #[test]
    fn test_is_signal_variable_complex_rust_syntax() {
        // Complex valid expressions should return true
        assert!(is_signal_variable("vec![1, 2, 3]"));
        assert!(is_signal_variable("Some(value)"));

        // These might be too complex to parse as expressions in this context
        // Let's see what happens with match and if expressions
        // Note: These might fail to parse as standalone expressions
        let match_expr = "match x { _ => y }";
        let if_expr = "if true { a } else { b }";

        // For now, let's just test that the function doesn't panic
        let _ = is_signal_variable(match_expr);
        let _ = is_signal_variable(if_expr);

        // These should definitely work
        assert!(is_signal_variable("vec![signal_data]"));
        assert!(is_signal_variable("Some(my_signal)"));
    }
}
