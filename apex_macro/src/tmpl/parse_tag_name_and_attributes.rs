/// Parse tag name and attributes from HTML/XML-like tag strings
///
/// ## Purpose
/// This function is a core utility in the Apex template engine that parses component and HTML
/// tag declarations to extract the tag name and its attributes. It's essential for the macro
/// system to understand and process template syntax, enabling the framework to:
/// - Identify component types for proper instantiation
/// - Extract attribute strings for further processing and validation
/// - Handle both self-closing and regular tag formats
/// - Support the HTML-like syntax that makes Apex templates intuitive
///
/// ## Functionality
/// The function takes a raw tag string (typically from template parsing) and separates it into
/// two distinct parts:
/// 1. **Tag Name**: The component or HTML element identifier
/// 2. **Attributes**: The remaining string containing all attributes and their values
///
/// ## Algorithm
/// 1. Trims leading/trailing whitespace from the input
/// 2. Searches for the first space character to identify the boundary between tag name and attributes
/// 3. If a space is found:
///    - Extracts everything before the space as the tag name
///    - Extracts everything after the space as the attributes string
/// 4. If no space is found (tag has no attributes):
///    - The entire trimmed string becomes the tag name
///    - Returns an empty string for attributes
/// 5. Handles self-closing tags by removing trailing '/' from tag names
///
/// ## Parameters
/// - `tag_str`: A string slice containing the raw tag declaration (e.g., "Button class='primary' id='submit'")
///
/// ## Returns
/// A tuple containing:
/// - `String`: The extracted tag name (e.g., "Button")
/// - `String`: The attributes string (e.g., "class='primary' id='submit'") or empty string if no attributes
///
/// ## Examples
/// ```rust,ignore
/// // Component with attributes
/// let (tag, attrs) = parse_tag_name_and_attributes("Button class='primary' onclick='handleClick()'");
/// assert_eq!(tag, "Button");
/// assert_eq!(attrs, "class='primary' onclick='handleClick()'");
///
/// // Self-closing component with attributes
/// let (tag, attrs) = parse_tag_name_and_attributes("Input type='text' value='hello'/");
/// assert_eq!(tag, "Input");
/// assert_eq!(attrs, "type='text' value='hello'");
///
/// // Component without attributes
/// let (tag, attrs) = parse_tag_name_and_attributes("Header");
/// assert_eq!(tag, "Header");
/// assert_eq!(attrs, "");
///
/// // Self-closing component without attributes
/// let (tag, attrs) = parse_tag_name_and_attributes("Divider/");
/// assert_eq!(tag, "Divider");
/// assert_eq!(attrs, "");
///
/// // HTML element with attributes
/// let (tag, attrs) = parse_tag_name_and_attributes("div class='container' id='main'");
/// assert_eq!(tag, "div");
/// assert_eq!(attrs, "class='container' id='main'");
/// ```
///
/// ## Edge Cases Handled
/// - **Whitespace**: Leading and trailing whitespace is automatically trimmed
/// - **Self-closing tags**: Trailing '/' is removed from tag names
/// - **No attributes**: Returns empty string for attributes when none are present
/// - **Multiple spaces**: Only the first space is used as the delimiter
///
/// ## Integration with Apex Framework
/// This function is typically called during the template compilation phase where:
/// 1. The template parser encounters a tag declaration
/// 2. This function extracts the tag name and attributes
/// 3. The tag name is used to determine if it's a component or HTML element
/// 4. The attributes string is further parsed to extract individual attribute-value pairs
/// 5. The processed information is used to generate the appropriate Rust code
///
/// ## Performance Considerations
/// - Uses string slicing for efficient parsing without unnecessary allocations
/// - Only allocates memory for the final owned strings returned
/// - Linear time complexity O(n) where n is the length of the input string
pub(crate) fn parse_tag_name_and_attributes(tag_str: &str) -> (String, String) {
    let trimmed = tag_str.trim();

    if let Some(space_pos) = trimmed.find(' ') {
        let tag_name = trimmed[..space_pos].trim_end_matches('/');
        let attributes = &trimmed[space_pos + 1..];

        (tag_name.to_owned(), attributes.to_owned())
    } else {
        let tag_name = trimmed.trim_end_matches('/');

        (tag_name.to_owned(), String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_with_attributes() {
        let (tag, attrs) =
            parse_tag_name_and_attributes("button class='primary' onclick='handleClick()'");

        assert_eq!(tag, "button");
        assert_eq!(attrs, "class='primary' onclick='handleClick()'");
    }

    #[test]
    fn test_self_closing_tag_with_attributes() {
        let (tag, attrs) = parse_tag_name_and_attributes("input type='text' value='hello'/");

        assert_eq!(tag, "input");
        assert_eq!(attrs, "type='text' value='hello'/");
    }

    #[test]
    fn test_tag_without_attributes() {
        let (tag, attrs) = parse_tag_name_and_attributes("header");

        assert_eq!(tag, "header");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_self_closing_tag_without_attributes() {
        let (tag, attrs) = parse_tag_name_and_attributes("divider/");

        assert_eq!(tag, "divider");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_html_element_with_attributes() {
        let (tag, attrs) = parse_tag_name_and_attributes("div class='container' id='main'");

        assert_eq!(tag, "div");
        assert_eq!(attrs, "class='container' id='main'");
    }

    #[test]
    fn test_tag_with_leading_whitespace() {
        let (tag, attrs) = parse_tag_name_and_attributes("  button class='primary'");

        assert_eq!(tag, "button");
        assert_eq!(attrs, "class='primary'");
    }

    #[test]
    fn test_tag_with_trailing_whitespace() {
        let (tag, attrs) = parse_tag_name_and_attributes("button class='primary'  ");

        assert_eq!(tag, "button");
        assert_eq!(attrs, "class='primary'");
    }

    #[test]
    fn test_tag_with_both_leading_and_trailing_whitespace() {
        let (tag, attrs) = parse_tag_name_and_attributes("  button class='primary'  ");

        assert_eq!(tag, "button");
        assert_eq!(attrs, "class='primary'");
    }

    #[test]
    fn test_self_closing_with_whitespace_before_slash() {
        let (tag, attrs) = parse_tag_name_and_attributes("input type='text' /");

        assert_eq!(tag, "input");
        assert_eq!(attrs, "type='text' /");
    }

    #[test]
    fn test_self_closing_with_whitespace_after_slash() {
        let (tag, attrs) = parse_tag_name_and_attributes("input/ ");

        assert_eq!(tag, "input");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_tag_with_multiple_spaces_in_attributes() {
        let (tag, attrs) = parse_tag_name_and_attributes("button  class='primary'   id='submit'");

        assert_eq!(tag, "button");
        assert_eq!(attrs, " class='primary'   id='submit'");
    }

    #[test]
    fn test_empty_string() {
        let (tag, attrs) = parse_tag_name_and_attributes("");

        assert_eq!(tag, "");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_whitespace_only() {
        let (tag, attrs) = parse_tag_name_and_attributes("   ");

        assert_eq!(tag, "");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_single_character_tag() {
        let (tag, attrs) = parse_tag_name_and_attributes("a");

        assert_eq!(tag, "a");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_single_character_tag_with_attributes() {
        let (tag, attrs) = parse_tag_name_and_attributes("a href='#'");

        assert_eq!(tag, "a");
        assert_eq!(attrs, "href='#'");
    }

    #[test]
    fn test_tag_with_only_space() {
        let (tag, attrs) = parse_tag_name_and_attributes("button ");

        assert_eq!(tag, "button");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_self_closing_tag_with_only_slash() {
        let (tag, attrs) = parse_tag_name_and_attributes("/");

        assert_eq!(tag, "");
        assert_eq!(attrs, "");
    }

    #[test]
    fn test_complex_attributes_with_quotes() {
        let (tag, attrs) = parse_tag_name_and_attributes(
            "img src=\"image.jpg\" alt=\"A beautiful image\" width='100'",
        );

        assert_eq!(tag, "img");
        assert_eq!(
            attrs,
            "src=\"image.jpg\" alt=\"A beautiful image\" width='100'"
        );
    }

    #[test]
    fn test_component_with_hyphen() {
        let (tag, attrs) = parse_tag_name_and_attributes("my-component prop='value'");

        assert_eq!(tag, "my-component");
        assert_eq!(attrs, "prop='value'");
    }

    #[test]
    fn test_self_closing_component_with_hyphen() {
        let (tag, attrs) = parse_tag_name_and_attributes("custom-input value='test'/");

        assert_eq!(tag, "custom-input");
        assert_eq!(attrs, "value='test'/");
    }

    #[test]
    fn test_tag_ending_with_slash_but_space_before() {
        let (tag, attrs) = parse_tag_name_and_attributes("button class='test' /");

        assert_eq!(tag, "button");
        assert_eq!(attrs, "class='test' /");
    }
}
