use std::str::Chars;

use crate::tmpl::{Attribute, Attributes};

#[derive(PartialEq)]
enum ElementOpeningTagState {
    Void,
    ElementName,
    AttributeName,
    AttributeValue,
}

fn insert_attribute(
    element_attrs: &mut Attributes,
    attribute_name: &mut String,
    attribute_value: &mut Attribute,
) {
    if !attribute_name.is_empty() {
        element_attrs.insert(attribute_name.clone(), attribute_value.clone());
        *attribute_name = String::new();
        *attribute_value = Attribute::Empty;
    }
}

pub(crate) fn parse_element_opening_tag(
    chars: &mut std::iter::Peekable<Chars<'_>>,
) -> (String, Attributes, bool) {
    let mut state = ElementOpeningTagState::ElementName;
    let mut element_name = String::new();
    let mut element_attrs = Attributes::new();
    let mut attribute_name = String::new();
    let mut attribute_value = Attribute::Empty;
    let mut is_self_closing = false;
    let mut expression_nesting_level = 0;

    if chars.peek() == Some(&'<') {
        chars.next(); // consume '<'
    } else {
        panic!("Unexpected character: {:?}, expected '<'", chars.peek());
    }

    for ch in chars.by_ref() {
        if is_self_closing && ch != '>' {
            continue;
        }

        if ch == '>' {
            insert_attribute(
                &mut element_attrs,
                &mut attribute_name,
                &mut attribute_value,
            );

            break;
        } else if ch == '/' {
            if state == ElementOpeningTagState::AttributeValue {
                attribute_value.push(ch);
            } else {
                is_self_closing = true;
                insert_attribute(
                    &mut element_attrs,
                    &mut attribute_name,
                    &mut attribute_value,
                );
            }
        } else if ch == ' ' {
            match state {
                ElementOpeningTagState::Void => state = ElementOpeningTagState::AttributeName,
                ElementOpeningTagState::ElementName => {
                    state = ElementOpeningTagState::AttributeName;
                }
                ElementOpeningTagState::AttributeName => {
                    state = ElementOpeningTagState::AttributeValue;
                    insert_attribute(
                        &mut element_attrs,
                        &mut attribute_name,
                        &mut attribute_value,
                    );
                }
                ElementOpeningTagState::AttributeValue => {
                    attribute_value.push(ch);
                }
            }
        } else if ch == '=' {
            match state {
                ElementOpeningTagState::Void => {
                    continue;
                }
                ElementOpeningTagState::AttributeName => {
                    state = ElementOpeningTagState::AttributeValue;
                }
                ElementOpeningTagState::AttributeValue => {
                    attribute_value.push(ch);
                }
                ElementOpeningTagState::ElementName => {
                    panic!("Unexpected character: {ch}, is not allowed in element name");
                }
            }
        } else if ch == '"' {
            if state == ElementOpeningTagState::AttributeValue {
                match &attribute_value {
                    Attribute::Literal(_) => {
                        state = ElementOpeningTagState::Void;
                        insert_attribute(
                            &mut element_attrs,
                            &mut attribute_name,
                            &mut attribute_value,
                        );
                    }
                    Attribute::Empty => {
                        attribute_value = Attribute::Literal(String::new());
                    }
                    _ => {
                        attribute_value.push(ch);
                    }
                }
            }
        } else if ch == '{' {
            if state == ElementOpeningTagState::AttributeValue {
                match &attribute_value {
                    Attribute::Expression(_) | Attribute::EventListener(_) => {
                        expression_nesting_level += 1;
                        attribute_value.push(ch);
                    }
                    Attribute::Empty => {
                        if attribute_name.starts_with("on") {
                            attribute_value = Attribute::EventListener(String::new());
                        } else {
                            attribute_value = Attribute::Expression(String::new());
                        }
                    }
                    Attribute::Literal(_) => {
                        attribute_value.push(ch);
                    }
                }
            }
        } else if ch == '}' {
            if state == ElementOpeningTagState::AttributeValue {
                match &attribute_value {
                    Attribute::Expression(_) | Attribute::EventListener(_) => {
                        if expression_nesting_level == 0 {
                            state = ElementOpeningTagState::Void;
                            insert_attribute(
                                &mut element_attrs,
                                &mut attribute_name,
                                &mut attribute_value,
                            );
                        } else {
                            attribute_value.push(ch);
                            expression_nesting_level -= 1;
                        }
                    }
                    _ => {
                        attribute_value.push(ch);
                    }
                }
            }
        } else if state == ElementOpeningTagState::ElementName {
            element_name.push(ch);
        } else if state == ElementOpeningTagState::AttributeName {
            attribute_name.push(ch);
        } else if state == ElementOpeningTagState::AttributeValue {
            attribute_value.push(ch);
        } else {
            panic!("Unexpected character: {ch}");
        }
    }

    (element_name, element_attrs, is_self_closing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let mut chars = "<div>".chars().peekable();
        let (element_name, element_attrs, is_self_closing) = parse_element_opening_tag(&mut chars);

        assert_eq!(element_name, "div");
        assert_eq!(element_attrs, Attributes::new());
        assert!(!is_self_closing);
    }

    #[test]
    fn test_self_closing_tag() {
        let mut chars = "<div />".chars().peekable();
        let (element_name, element_attrs, is_self_closing) = parse_element_opening_tag(&mut chars);

        assert_eq!(element_name, "div");
        assert_eq!(element_attrs, Attributes::new());
        assert!(is_self_closing);
    }

    #[test]
    fn test_tag_with_one_literal_attribute() {
        let mut chars = "<div class=\"container\">".chars().peekable();
        let (element_name, element_attrs, is_self_closing) = parse_element_opening_tag(&mut chars);

        assert_eq!(element_name, "div");
        assert_eq!(
            element_attrs,
            Attributes::from([(
                "class".to_owned(),
                Attribute::Literal("container".to_owned())
            )])
        );
        assert!(!is_self_closing);
    }

    #[test]
    fn tag_with_one_literal_attribute_and_one_expression_attribute() {
        let mut chars = "<div class=\"container\" onclick={handle_click}>"
            .chars()
            .peekable();

        let (element_name, element_attrs, is_self_closing) = parse_element_opening_tag(&mut chars);

        assert_eq!(element_name, "div");
        assert_eq!(
            element_attrs,
            Attributes::from([
                (
                    "class".to_owned(),
                    Attribute::Literal("container".to_owned())
                ),
                (
                    "onclick".to_owned(),
                    Attribute::EventListener("handle_click".to_owned())
                )
            ])
        );
        assert!(!is_self_closing);
    }

    #[test]
    fn test_tag_with_path_attribute() {
        let mut chars = "<a href=\"/path\">".chars().peekable();
        let (element_name, element_attrs, is_self_closing) = parse_element_opening_tag(&mut chars);

        assert_eq!(element_name, "a");
        assert_eq!(
            element_attrs,
            Attributes::from([("href".to_owned(), Attribute::Literal("/path".to_owned()))])
        );
        assert!(!is_self_closing);
    }

    #[test]
    fn tag_with_expression_attribute() {
        let mut chars = "<div data-test={1 + 1}></div>".chars().peekable();
        let (element_name, element_attrs, is_self_closing) = parse_element_opening_tag(&mut chars);

        assert_eq!(element_name, "div");
        assert_eq!(
            element_attrs,
            Attributes::from([(
                "data-test".to_owned(),
                Attribute::Expression("1 + 1".to_owned())
            )])
        );
        assert!(!is_self_closing);
    }
}
