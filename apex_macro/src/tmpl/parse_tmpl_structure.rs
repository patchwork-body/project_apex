use crate::tmpl::{HtmlContent, parse_tag_content::*, parse_variable_content::*};
use syn::Result;

/// Parse HTML-like template content into structured representation for the Apex framework.
///
/// ## Purpose
///
/// This function serves as the **primary parsing engine** for Apex's template macro system (`tmpl!`).
/// It transforms raw HTML-like template strings into a structured, type-safe representation that can
/// be processed by subsequent code generation phases. This is a critical component of Apex's compile-time
/// template processing pipeline, enabling the framework to provide both performance and type safety.
///
/// ## Why This Function Exists
///
/// Apex uses a **compile-time template processing approach** rather than runtime parsing for several key reasons:
/// 1. **Performance**: Templates are parsed once at compile time, not on every request
/// 2. **Type Safety**: Variable references and component usage are validated at compile time
/// 3. **Zero Runtime Cost**: No template parsing overhead during request handling
/// 4. **Early Error Detection**: Template syntax errors are caught during compilation
///
/// ## Functionality Overview
///
/// The function performs **character-by-character parsing** of template content, using a state machine
/// approach to handle different parsing contexts. It recognizes and processes three main content types:
///
/// ### 1. **Text Content** (`HtmlContent::Text`)
/// Plain text between tags and variable expressions:
/// ```ignore
/// Input:  "Hello World"
/// Output: HtmlContent::Text("Hello World")
/// ```
///
/// ### 2. **Static Variables** (`HtmlContent::StaticVariable`)
/// Regular Rust expressions (non-reactive):
/// ```ignore
/// Input:  "{user.name}" or "{count + 1}"
/// Output: HtmlContent::StaticVariable("user.name") or HtmlContent::StaticVariable("count + 1")
/// ```
///
/// ### 3. **Dynamic Variables** (`HtmlContent::DynamicVariable`)
/// Signal-based reactive variables:
/// ```ignore
/// Input:  "{self.count}" or "{self.title}"
/// Output: HtmlContent::DynamicVariable { variable: "self.count", context: TextNode { element_id: "apex_wrapper_0", text_node_index: 0 } }
/// ```
///
/// ### 3. **HTML Tags and Components** (via `parse_tag_content`)
/// Both standard HTML elements and custom Apex components:
/// ```ignore
/// Input:  "<div class='container'>" or "<MyComponent count={5} />"
/// Output: HtmlContent::Element{...} or HtmlContent::Component{...}
/// ```
///
/// ## Parsing Algorithm
///
/// The function uses a **character-by-character state machine** with the following logic:
///
/// 1. **Text Accumulation**: Characters are accumulated into `current_text` until a special delimiter is encountered
/// 2. **Tag Processing**: When `<` is found, accumulated text is saved and tag parsing begins
/// 3. **Variable Processing**: When `{` is found (outside tags), accumulated text is saved and variable parsing begins
/// 4. **Context Awareness**: The `inside_tag` flag prevents variable parsing within HTML tag attributes
/// 5. **Cleanup**: Any remaining accumulated text is saved at the end
///
/// ## Context-Aware Parsing
///
/// The parser maintains parsing context to handle complex scenarios:
/// - **Variable Exclusion in Tags**: `{variables}` inside `<tag attr="{var}">` are handled by tag parsing, not variable parsing
/// - **Nested Braces**: Complex expressions like `{items.{index}}` are handled by the variable parser
/// - **Whitespace Handling**: Leading/trailing whitespace in text nodes is trimmed for cleaner output
///
/// ## Integration with Apex Pipeline
///
/// This function operates as part of a larger template processing pipeline:
/// ```ignore
/// Raw Template String → parse_tmpl_structure() → generate_render_parts() → Rust Code Generation
/// ```
///
/// The structured output enables subsequent phases to:
/// - Generate type-safe Rust code for each content type
/// - Validate component usage and variable references
/// - Optimize rendering performance through compile-time analysis
///
/// ## Error Handling
///
/// - Returns `syn::Result` to integrate with procedural macro error reporting
/// - Delegates detailed parsing errors to specialized parsers (`parse_tag_content`, `parse_variable_content`)
/// - Gracefully handles malformed input by treating unrecognized patterns as text content
///
/// ## Examples
///
/// ### Simple Text and Variables
/// ```ignore
/// Input:  "Hello {name}!"
/// Output: [
///     HtmlContent::Text("Hello "),
///     HtmlContent::StaticVariable("name"),
///     HtmlContent::Text("!")
/// ]
/// ```
///
/// ### Mixed HTML and Components
/// ```ignore
/// Input:  "<div>Count: {count}</div><Counter value={count} />"
/// Output: [
///     HtmlContent::Element { tag: "div", ... },
///     HtmlContent::Text("Count: "),
///     HtmlContent::StaticVariable("count"),
///     HtmlContent::Element { tag: "/div", ... },
///     HtmlContent::Component { tag: "Counter", attributes: {...} }
/// ]
/// ```
///
/// ## Performance Characteristics
///
/// - **Linear Time Complexity**: O(n) where n is the length of the input string
/// - **Single Pass**: Content is parsed in one forward pass through the character stream
/// - **Memory Efficient**: Uses iterative parsing with minimal memory allocation
/// - **Compile-Time Only**: Zero runtime performance impact as parsing occurs at compile time
///
/// # Arguments
/// * `content` - Raw HTML-like template string from the `tmpl!` macro
///
/// # Returns
/// * `Ok(Vec<HtmlContent>)` - Successfully parsed structured representation
/// * `Err(syn::Error)` - Parsing error with diagnostic information for the macro system
pub(crate) fn parse_tmpl_structure(content: &str) -> Result<Vec<HtmlContent>> {
    let mut result = Vec::new();
    let mut chars = content.chars().peekable();
    let mut current_text = String::new();
    let mut inside_tag = false;
    let mut element_counter = 0;
    let mut text_node_counter = 0;
    let mut current_element_id: Option<String> = None;

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Save accumulated text
                if !current_text.is_empty() {
                    result.push(HtmlContent::Text(current_text.trim().to_owned()));
                    current_text.clear();
                    text_node_counter += 1;
                }

                // Parse tag
                if let Some(tag_content) = parse_tag_content(&mut chars)? {
                    // Update current element context for text nodes
                    if let HtmlContent::Element { element_id, .. } = &tag_content {
                        if let Some(elem_id) = element_id {
                            current_element_id = Some(elem_id.clone());
                            text_node_counter = 0; // Reset text node counter for new element
                        }
                    }
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
                    text_node_counter += 1;
                }

                // Parse variable and determine if it's static or dynamic
                if let Some(var_content) = parse_variable_content(&mut chars)? {
                    if is_signal_variable(&var_content) {
                        // Generate context with current element information
                        let context = if let Some(ref elem_id) = current_element_id {
                            crate::tmpl::DynamicVariableContext::TextNode {
                                element_id: elem_id.clone(),
                                text_node_index: text_node_counter,
                            }
                        } else {
                            // Create a wrapper element if no current element context
                            let wrapper_id = format!("apex_wrapper_{}", element_counter);
                            element_counter += 1;
                            crate::tmpl::DynamicVariableContext::TextNode {
                                element_id: wrapper_id,
                                text_node_index: 0,
                            }
                        };

                        result.push(HtmlContent::DynamicVariable {
                            variable: var_content,
                            context,
                        });
                        text_node_counter += 1;
                    } else {
                        result.push(HtmlContent::StaticVariable(var_content));
                        text_node_counter += 1;
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::{ComponentAttribute, HtmlContent};
    use std::collections::HashMap;

    /// Helper function to create a simple component for testing
    #[allow(dead_code)]
    fn create_test_component(
        tag: &str,
        attributes: HashMap<String, ComponentAttribute>,
    ) -> HtmlContent {
        HtmlContent::Component {
            tag: tag.to_owned(),
            attributes,
        }
    }

    /// Helper function to create a simple element for testing
    #[allow(dead_code)]
    fn create_test_element(
        tag: &str,
        attributes: HashMap<String, ComponentAttribute>,
        self_closing: bool,
    ) -> HtmlContent {
        HtmlContent::Element {
            tag: tag.to_owned(),
            attributes,
            self_closing,
            element_id: None,
        }
    }

    #[test]
    fn test_empty_string() {
        let result = parse_tmpl_structure("").unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let result = parse_tmpl_structure("   \n\t  ").unwrap();
        // Whitespace-only content should be trimmed and result in empty or no content
        if result.len() == 1 {
            match &result[0] {
                HtmlContent::Text(text) => assert!(
                    text.trim().is_empty(),
                    "Text should be empty after trimming"
                ),
                _ => panic!("Expected empty text content or no content"),
            }
        } else {
            assert_eq!(result.len(), 0);
        }
    }

    #[test]
    fn test_simple_text() {
        let result = parse_tmpl_structure("Hello World").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            HtmlContent::Text(text) => assert_eq!(text, "Hello World"),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_text_with_leading_trailing_whitespace() {
        let result = parse_tmpl_structure("  Hello World  ").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            HtmlContent::Text(text) => assert_eq!(text, "Hello World"),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_simple_variable() {
        let result = parse_tmpl_structure("{name}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "name"),
            _ => panic!("Expected StaticVariable content"),
        }
    }

    #[test]
    fn test_dotted_variable() {
        let result = parse_tmpl_structure("{user.email}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "user.email"),
            _ => panic!("Expected StaticVariable content for non-self variable"),
        }
    }

    #[test]
    fn test_complex_expression_variable() {
        let result = parse_tmpl_structure("{count + 1}").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "count + 1"),
            _ => panic!("Expected StaticVariable content"),
        }
    }

    #[test]
    fn test_text_and_variable_combination() {
        let result = parse_tmpl_structure("Hello {name}!").unwrap();
        assert_eq!(result.len(), 3);

        match &result[0] {
            HtmlContent::Text(text) => assert_eq!(text, "Hello "),
            _ => panic!("Expected Text content at index 0"),
        }

        match &result[1] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "name"),
            _ => panic!("Expected StaticVariable content at index 1"),
        }

        match &result[2] {
            HtmlContent::Text(text) => assert_eq!(text, "!"),
            _ => panic!("Expected Text content at index 2"),
        }
    }

    #[test]
    fn test_multiple_variables() {
        let result = parse_tmpl_structure("{greeting} {name}!").unwrap();
        assert_eq!(result.len(), 4);

        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "greeting"),
            _ => panic!("Expected StaticVariable content at index 0"),
        }

        match &result[1] {
            HtmlContent::Text(text) => assert_eq!(text, " "),
            _ => panic!("Expected Text content at index 1"),
        }

        match &result[2] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "name"),
            _ => panic!("Expected StaticVariable content at index 2"),
        }

        match &result[3] {
            HtmlContent::Text(text) => assert_eq!(text, "!"),
            _ => panic!("Expected Text content at index 3"),
        }
    }

    #[test]
    fn test_simple_html_tag() {
        let result = parse_tmpl_structure("<div>").unwrap();
        assert_eq!(result.len(), 1);

        // The actual tag parsing is handled by parse_tag_content,
        // so we just verify that something was parsed (not Text or Variable)
        match &result[0] {
            HtmlContent::Element { .. } | HtmlContent::Component { .. } => {
                // This is expected - the exact structure depends on parse_tag_content
            }
            _ => panic!("Expected Element or Component content"),
        }
    }

    #[test]
    fn test_html_with_text_content() {
        let result = parse_tmpl_structure("<div>Hello World</div>").unwrap();
        assert!(result.len() >= 2); // Should have at least the opening tag and text

        // Look for text content in the results
        let has_text = result
            .iter()
            .any(|item| matches!(item, HtmlContent::Text(text) if text == "Hello World"));
        assert!(has_text, "Should contain 'Hello World' text content");
    }

    #[test]
    fn test_html_with_variable_content() {
        let result = parse_tmpl_structure("<div>{content}</div>").unwrap();
        assert!(result.len() >= 2);

        // Should contain a variable
        let has_variable = result
            .iter()
            .any(|item| matches!(item, HtmlContent::StaticVariable(var) if var == "content"));
        assert!(has_variable, "Should contain 'content' static variable");
    }

    #[test]
    fn test_mixed_content_complex() {
        let result =
            parse_tmpl_structure("Hello {name}, your count is <span>{count}</span>!").unwrap();
        assert!(result.len() >= 4);

        // Verify we have both variables
        let has_name_var = result
            .iter()
            .any(|item| matches!(item, HtmlContent::StaticVariable(var) if var == "name"));
        let has_count_var = result
            .iter()
            .any(|item| matches!(item, HtmlContent::StaticVariable(var) if var == "count"));

        assert!(has_name_var, "Should contain 'name' static variable");
        assert!(has_count_var, "Should contain 'count' static variable");
    }

    #[test]
    fn test_nested_braces_in_variable() {
        let result = parse_tmpl_structure("{items.{index}}").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "items.{index}"),
            _ => panic!("Expected StaticVariable content with nested braces"),
        }
    }

    #[test]
    fn test_deeply_nested_braces() {
        let result = parse_tmpl_structure("{data.{user.{profile.{name}}}}").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "data.{user.{profile.{name}}}"),
            _ => panic!("Expected StaticVariable content with deeply nested braces"),
        }
    }

    #[test]
    fn test_empty_variable_braces() {
        let result = parse_tmpl_structure("{}").unwrap();
        // Empty variable content should not create a Variable entry
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_whitespace_only_variable() {
        let result = parse_tmpl_structure("{   }").unwrap();
        // Whitespace-only variable content should not create a Variable entry
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_variable_with_whitespace() {
        let result = parse_tmpl_structure("{  name  }").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "name"),
            _ => panic!("Expected StaticVariable content with trimmed whitespace"),
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let result = parse_tmpl_structure("<br/>").unwrap();
        assert_eq!(result.len(), 1);

        // Should parse as an element (exact structure depends on parse_tag_content)
        match &result[0] {
            HtmlContent::Element { .. } | HtmlContent::Component { .. } => {
                // Expected
            }
            _ => panic!("Expected Element or Component for self-closing tag"),
        }
    }

    #[test]
    fn test_component_syntax() {
        let result = parse_tmpl_structure("<MyComponent />").unwrap();
        assert_eq!(result.len(), 1);

        // Should parse as component or element
        match &result[0] {
            HtmlContent::Element { .. } | HtmlContent::Component { .. } => {
                // Expected
            }
            _ => panic!("Expected Element or Component for component syntax"),
        }
    }

    #[test]
    fn test_multiline_content() {
        let input = r#"<div>
            Hello {name}
            <span>Count: {count}</span>
        </div>"#;

        let result = parse_tmpl_structure(input).unwrap();
        assert!(!result.is_empty());

        // Should contain both variables
        let has_name = result
            .iter()
            .any(|item| matches!(item, HtmlContent::StaticVariable(var) if var == "name"));
        let has_count = result
            .iter()
            .any(|item| matches!(item, HtmlContent::StaticVariable(var) if var == "count"));

        assert!(has_name, "Should contain 'name' static variable");
        assert!(has_count, "Should contain 'count' static variable");
    }

    #[test]
    fn test_consecutive_variables() {
        let result = parse_tmpl_structure("{first}{second}").unwrap();
        assert_eq!(result.len(), 2);

        match &result[0] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "first"),
            _ => panic!("Expected first static variable"),
        }

        match &result[1] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "second"),
            _ => panic!("Expected second static variable"),
        }
    }

    #[test]
    fn test_consecutive_tags() {
        let result = parse_tmpl_structure("<div></div><span></span>").unwrap();
        assert!(result.len() >= 2);

        // Should have parsed multiple tags
        let tag_count = result
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    HtmlContent::Element { .. } | HtmlContent::Component { .. }
                )
            })
            .count();

        assert!(tag_count >= 2, "Should have parsed multiple tags");
    }

    #[test]
    fn test_special_characters_in_text() {
        let input = "Hello & Welcome! @user #hashtag";
        let result = parse_tmpl_structure(input).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::Text(text) => assert_eq!(text, input),
            _ => panic!("Expected text with special characters"),
        }
    }

    #[test]
    fn test_unicode_content() {
        let input = "Hello 世界 🌍 {name}";
        let result = parse_tmpl_structure(input).unwrap();
        assert_eq!(result.len(), 2);

        match &result[0] {
            HtmlContent::Text(text) => assert_eq!(text, "Hello 世界 🌍 "),
            _ => panic!("Expected unicode text content"),
        }

        match &result[1] {
            HtmlContent::StaticVariable(var) => assert_eq!(var, "name"),
            _ => panic!("Expected static variable after unicode text"),
        }
    }

    #[test]
    fn test_dynamic_variable_self_field() {
        let result = parse_tmpl_structure("{self.count}").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::DynamicVariable { variable, context } => {
                assert_eq!(variable, "self.count");
                match context {
                    crate::tmpl::DynamicVariableContext::TextNode {
                        element_id,
                        text_node_index,
                    } => {
                        assert!(element_id.starts_with("apex_wrapper_"));
                        assert_eq!(*text_node_index, 0);
                    }
                    _ => panic!("Expected TextNode context"),
                }
            }
            _ => panic!("Expected DynamicVariable for self field access"),
        }
    }

    #[test]
    fn test_dynamic_variable_self_expression() {
        let result = parse_tmpl_structure("{self.title + \" suffix\"}").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::DynamicVariable { variable, context } => {
                assert_eq!(variable, "self.title + \" suffix\"");
                match context {
                    crate::tmpl::DynamicVariableContext::TextNode {
                        element_id,
                        text_node_index,
                    } => {
                        assert!(element_id.starts_with("apex_wrapper_"));
                        assert_eq!(*text_node_index, 0);
                    }
                    _ => panic!("Expected TextNode context"),
                }
            }
            _ => panic!("Expected DynamicVariable for self expression"),
        }
    }

    #[test]
    fn test_signal_method_call() {
        let result = parse_tmpl_structure("{signal.get()}").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::DynamicVariable { variable, context } => {
                assert_eq!(variable, "signal.get()");
                match context {
                    crate::tmpl::DynamicVariableContext::TextNode {
                        element_id,
                        text_node_index,
                    } => {
                        assert!(element_id.starts_with("apex_wrapper_"));
                        assert_eq!(*text_node_index, 0);
                    }
                    _ => panic!("Expected TextNode context"),
                }
            }
            _ => panic!("Expected DynamicVariable for signal method call"),
        }
    }

    #[test]
    fn test_complex_real_world_template() {
        let input = r#"
            <div class="container">
                <h1>Welcome {user.name}!</h1>
                <p>You have {message_count} messages.</p>
                <MyComponent count={items.len()} />
                {if show_footer {
                    tmpl! { <footer>© 2024</footer> }.to_string()
                } else {
                    String::new()
                }}
            </div>
        "#;

        let result = parse_tmpl_structure(input).unwrap();
        assert!(!result.is_empty());

        // Verify key variables are parsed
        let variables: Vec<String> = result
            .iter()
            .filter_map(|item| match item {
                HtmlContent::StaticVariable(var) => Some(var.clone()),
                HtmlContent::DynamicVariable {
                    variable,
                    context: _,
                } => Some(variable.clone()),
                _ => None,
            })
            .collect();

        assert!(variables.contains(&"user.name".to_owned()));
        assert!(variables.contains(&"message_count".to_owned()));

        // Should have complex conditional expression
        let has_complex_expr = variables.iter().any(|var| var.contains("if show_footer"));
        assert!(
            has_complex_expr,
            "Should contain complex conditional expression"
        );
    }

    #[test]
    fn test_unmatched_opening_brace() {
        let result = parse_tmpl_structure("Hello {name").unwrap();
        assert!(!result.is_empty());

        // Should handle gracefully - either as text or variable
        // The exact behavior depends on parse_variable_content implementation
    }

    #[test]
    fn test_unmatched_closing_brace() {
        let result = parse_tmpl_structure("Hello name}").unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            HtmlContent::Text(text) => assert_eq!(text, "Hello name}"),
            _ => panic!("Expected text content for unmatched closing brace"),
        }
    }

    #[test]
    fn test_malformed_tag() {
        let result = parse_tmpl_structure("Hello <").unwrap();
        // Should handle gracefully without panicking - just verify it doesn't crash
        let _ = result.len(); // Just access the result to verify it parsed
    }

    #[test]
    fn test_performance_large_input() {
        // Test with larger input to ensure reasonable performance
        let large_input = "Hello {name}! ".repeat(1000);
        let result = parse_tmpl_structure(&large_input).unwrap();

        // Should parse all repetitions
        assert!(result.len() > 1000);

        // Should contain many variables
        let var_count = result
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    HtmlContent::StaticVariable(_) | HtmlContent::DynamicVariable { .. }
                )
            })
            .count();
        assert_eq!(var_count, 1000);
    }
}
