use std::str::Chars;

use crate::tmpl::TmplAst;

use super::is_pascal_case::is_pascal_case;
use super::match_chars::match_chars;
use super::parse_conditional_directive::parse_conditional_directive;
use super::parse_directive_name::parse_directive_name;
use super::parse_element_opening_tag::parse_element_opening_tag;
use super::parse_slot_interpolation::parse_slot_interpolation;
use super::parse_slot_name::parse_slot_name;

#[derive(PartialEq)]
enum ExpressionType {
    Ordinary,
    Directive,
    Slot,
}

#[derive(PartialEq)]
enum ProcessCharsUntilState {
    Unknown,
    AfterExpression,
    Text,
    Element,
    Expression,
}

pub(crate) fn process_chars_until(
    chars: &mut std::iter::Peekable<Chars<'_>>,
    end_of_block: Option<&[&str]>,
) -> (Vec<TmplAst>, String) {
    let mut ast = Vec::new();
    let mut text = String::new();
    let mut state = ProcessCharsUntilState::Unknown;
    let mut expression_type = ExpressionType::Ordinary;
    let mut matched_end_of_block = "".to_owned();
    let mut has_temp_whitespace = false;

    'outer: while chars.peek().is_some() {
        if chars.peek().is_none() {
            if let Some(end_of_block) = end_of_block {
                panic!("Expected end of block: {end_of_block:?}");
            }

            break;
        }

        if let Some(end_of_block) = end_of_block {
            for end in end_of_block {
                if match_chars(chars, end) {
                    matched_end_of_block = (*end).to_owned();
                    break 'outer;
                }
            }
        }

        match state {
            ProcessCharsUntilState::Unknown
            | ProcessCharsUntilState::Text
            | ProcessCharsUntilState::AfterExpression => {
                if state == ProcessCharsUntilState::Unknown
                    || state == ProcessCharsUntilState::AfterExpression
                {
                    // Skip whitespace between elements only when not inside a Text node
                    while chars.peek() == Some(&' ')
                        || chars.peek() == Some(&'\n')
                        || chars.peek() == Some(&'\r')
                        || chars.peek() == Some(&'\t')
                    {
                        if state == ProcessCharsUntilState::AfterExpression {
                            has_temp_whitespace = true;
                        }

                        chars.next();
                    }
                }

                if chars.peek() == Some(&'<') {
                    state = ProcessCharsUntilState::Element;

                    if !text.is_empty() {
                        ast.push(TmplAst::Text(text));
                        text = String::new();
                    }
                } else if chars.peek() == Some(&'{') {
                    // Don't add whitespace for directive expressions - handle it later
                    if !text.is_empty() {
                        ast.push(TmplAst::Text(text));
                        text = String::new();
                    }

                    state = ProcessCharsUntilState::Expression;
                } else {
                    if has_temp_whitespace && state == ProcessCharsUntilState::AfterExpression {
                        text.push(' ');
                        has_temp_whitespace = false;
                    }

                    state = ProcessCharsUntilState::Text;

                    let Some(ch) = chars.next() else {
                        break;
                    };

                    text.push(ch);
                }
            }
            ProcessCharsUntilState::Element => {
                let mut lookahead = chars.clone();
                lookahead.next(); // consume '<'

                // Check if it's a slot tag
                if lookahead.peek() == Some(&'#') {
                    // Check if it's slot interpolation (<#slot) or slot definition (<#slot_name)
                    let mut lookahead2 = lookahead.clone();
                    lookahead2.next(); // consume '#'

                    // Check if it starts with "slot"
                    let is_slot_interpolation = lookahead2.peek() == Some(&'s') && {
                        let mut temp = lookahead2.clone();

                        temp.next() == Some('s')
                            && temp.next() == Some('l')
                            && temp.next() == Some('o')
                            && temp.next() == Some('t')
                            && (temp.peek() == Some(&'>')
                                || temp.peek() == Some(&'/')
                                || temp.peek().is_some_and(|c| c.is_whitespace()))
                    };

                    if is_slot_interpolation {
                        if has_temp_whitespace {
                            ast.push(TmplAst::Text(" ".to_owned()));
                            has_temp_whitespace = false;
                        }

                        ast.push(parse_slot_interpolation(chars));
                        state = ProcessCharsUntilState::AfterExpression;
                    } else {
                        // It's a slot definition
                        let slot_name = parse_slot_name(chars);
                        let closing_tag = format!("</#{slot_name}>");

                        let (children, _) = process_chars_until(chars, Some(&[&closing_tag]));

                        ast.push(TmplAst::Slot {
                            name: Some(slot_name),
                            children,
                        });

                        state = ProcessCharsUntilState::AfterExpression;
                    }
                } else {
                    // Clear temp whitespace for regular elements since they don't preserve it
                    has_temp_whitespace = false;

                    let (element_name, element_attrs, is_self_closing) =
                        parse_element_opening_tag(chars);

                    let is_component = element_name
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_uppercase());

                    if !is_self_closing {
                        let closing_tag = format!("</{element_name}>");
                        let (children, _) = process_chars_until(chars, Some(&[&closing_tag]));
                        let is_component = is_pascal_case(&element_name);

                        ast.push(TmplAst::Element {
                            tag: element_name,
                            attributes: element_attrs,
                            is_component,
                            self_closing: is_self_closing,
                            children,
                        });

                        state = ProcessCharsUntilState::AfterExpression;
                    } else {
                        ast.push(TmplAst::Element {
                            tag: element_name,
                            attributes: element_attrs,
                            is_component,
                            self_closing: is_self_closing,
                            children: Vec::new(),
                        });

                        state = ProcessCharsUntilState::AfterExpression;
                    }
                }
            }
            ProcessCharsUntilState::Expression => {
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                }

                if chars.peek() == Some(&'@') {
                    chars.next(); // consume '@'
                    expression_type = ExpressionType::Slot;
                } else if chars.peek() == Some(&'#') {
                    chars.next(); // consume '#'
                    expression_type = ExpressionType::Directive;
                    // Clear temp whitespace for directives since they don't preserve it
                    has_temp_whitespace = false;

                    let directive_name = parse_directive_name(chars);

                    if directive_name == "if" {
                        ast.push(TmplAst::ConditionalDirective(parse_conditional_directive(
                            chars,
                        )));
                    } else if directive_name == "outlet" {
                        // For outlet directive, we need to consume the closing brace
                        // The directive_name parsing should have stopped at the '}'
                        if chars.peek() == Some(&'}') {
                            chars.next(); // consume the '}'
                        }
                        ast.push(TmplAst::Outlet);
                    }

                    state = ProcessCharsUntilState::Unknown;
                } else if chars.peek() == Some(&'}') {
                    chars.next(); // consume '}'

                    if !text.is_empty() {
                        match expression_type {
                            ExpressionType::Ordinary => {
                                // Add whitespace for regular expressions
                                if has_temp_whitespace {
                                    ast.push(TmplAst::Text(" ".to_owned()));
                                    has_temp_whitespace = false;
                                }
                                ast.push(TmplAst::Expression(text));
                            }
                            ExpressionType::Slot => {
                                // Add whitespace for slot expressions
                                if has_temp_whitespace {
                                    ast.push(TmplAst::Text(" ".to_owned()));
                                    has_temp_whitespace = false;
                                }
                                ast.push(TmplAst::SlotInterpolation {
                                    slot_name: Some(text),
                                    default_children: None,
                                });
                            }
                            ExpressionType::Directive => {
                                // Don't add whitespace for directives
                                has_temp_whitespace = false;
                            }
                        }

                        expression_type = ExpressionType::Ordinary;
                        text = String::new();
                    }

                    state = ProcessCharsUntilState::AfterExpression;
                } else {
                    let Some(ch) = chars.next() else {
                        panic!("Invalid expression syntax")
                    };

                    text.push(ch);
                }
            }
        }
    }

    text = text.trim_end().to_owned();

    if !text.is_empty() {
        ast.push(TmplAst::Text(text));
    }

    (ast, matched_end_of_block)
}

#[cfg(test)]
mod tests {
    use crate::tmpl::{Attribute, Attributes, ConditionalBlock};

    use super::*;

    #[test]
    fn text() {
        let mut chars = "Hello, world!".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(ast, vec![TmplAst::Text("Hello, world!".to_owned())]);
    }

    #[test]
    fn single_element() {
        let mut chars = "<div></div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                is_component: false,
                attributes: Attributes::new(),
                self_closing: false,
                children: vec![],
            }]
        );
    }

    #[test]
    fn element_with_text() {
        let mut chars = "<div>Hello, world!</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }]
        );
    }

    #[test]
    fn element_with_text_that_contains_path() {
        let mut chars = "<div>Hello, <a href=\"/path\">world</a>!</div>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::Element {
                        tag: "a".to_owned(),
                        attributes: Attributes::from([(
                            "href".to_owned(),
                            Attribute::Literal("/path".to_owned()),
                        )]),
                        self_closing: false,
                        is_component: false,
                        children: vec![TmplAst::Text("world".to_owned())],
                    },
                    TmplAst::Text("!".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn element_with_text_and_whitespace() {
        let mut chars = "<div>  Hello, world!  </div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }]
        );
    }

    #[test]
    fn nested_elements() {
        let mut chars = "<div><p>Hello, world!</p></div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![TmplAst::Element {
                    tag: "p".to_owned(),
                    attributes: Attributes::new(),
                    self_closing: false,
                    is_component: false,
                    children: vec![TmplAst::Text("Hello, world!".to_owned())],
                }],
            }]
        );
    }

    #[test]
    fn slot_with_text() {
        let mut chars = "<#slot_name>Hello, world!</#slot_name>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Slot {
                name: Some("slot_name".to_owned()),
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }]
        );
    }

    #[test]
    fn slots_with_nested_elements() {
        let mut chars = "<#slot_name><p>Hello, world!</p></#slot_name>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Slot {
                name: Some("slot_name".to_owned()),
                children: vec![TmplAst::Element {
                    tag: "p".to_owned(),
                    attributes: Attributes::new(),
                    self_closing: false,
                    is_component: false,
                    children: vec![TmplAst::Text("Hello, world!".to_owned())],
                }],
            }],
        );
    }

    #[test]
    fn expression() {
        let mut chars = "{1 + 1}".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(ast, vec![TmplAst::Expression("1 + 1".to_owned())]);
    }

    #[test]
    fn expression_with_text() {
        let mut chars = "{1 + 1}Hello, world!".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![
                TmplAst::Expression("1 + 1".to_owned()),
                TmplAst::Text("Hello, world!".to_owned())
            ]
        );
    }

    #[test]
    fn expression_inside_element() {
        let mut chars = "<div>{1 + 1}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![TmplAst::Expression("1 + 1".to_owned())],
            }]
        );
    }

    #[test]
    fn expression_inside_element_with_text() {
        let mut chars = "<div>{1 + 1}Hello, world!</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text("Hello, world!".to_owned())
                ],
            }]
        );
    }

    #[test]
    fn expression_inside_element_with_attrs() {
        let mut chars = "<div id=\"container-id\" onclick={handle_click}>{1 + 1}</div>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::from([
                    (
                        "id".to_owned(),
                        Attribute::Literal("container-id".to_owned())
                    ),
                    (
                        "onclick".to_owned(),
                        Attribute::EventListener("handle_click".to_owned())
                    ),
                ]),
                self_closing: false,
                is_component: false,
                children: vec![TmplAst::Expression("1 + 1".to_owned())],
            }]
        );
    }

    #[test]
    fn expression_as_element_attribute() {
        let mut chars = "<div data-test={1 + 1}></div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::from([(
                    "data-test".to_owned(),
                    Attribute::Expression("1 + 1".to_owned())
                )]),
                self_closing: false,
                is_component: false,
                children: vec![],
            }]
        );
    }

    #[test]
    fn text_inside_element_with_nested_elements() {
        let mut chars = "<div>Hello, <span>world</span>!</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::Element {
                        tag: "span".to_owned(),
                        attributes: Attributes::new(),
                        self_closing: false,
                        is_component: false,
                        children: vec![TmplAst::Text("world".to_owned())],
                    },
                    TmplAst::Text("!".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn text_inside_element_with_nested_expression() {
        let mut chars = "<div>Hello, {1 + 1}!</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                self_closing: false,
                is_component: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text("!".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn single_component() {
        let mut chars = "<MyComponent></MyComponent>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "MyComponent".to_owned(),
                attributes: Attributes::new(),
                is_component: true,
                self_closing: false,
                children: vec![],
            }]
        );
    }

    #[test]
    fn component_with_text() {
        let mut chars = "<MyComponent>Hello, world!</MyComponent>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "MyComponent".to_owned(),
                attributes: Attributes::new(),
                is_component: true,
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }]
        );
    }

    #[test]
    fn self_closing_component() {
        let mut chars = "<MyComponent />".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "MyComponent".to_owned(),
                attributes: Attributes::new(),
                is_component: true,
                self_closing: true,
                children: vec![],
            }]
        );
    }

    #[test]
    fn component_with_attrs() {
        let mut chars = "<MyComponent id=\"container-id\" onclick={handle_click}></MyComponent>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "MyComponent".to_owned(),
                attributes: Attributes::from([
                    (
                        "id".to_owned(),
                        Attribute::Literal("container-id".to_owned())
                    ),
                    (
                        "onclick".to_owned(),
                        Attribute::EventListener("handle_click".to_owned())
                    ),
                ]),
                is_component: true,
                self_closing: false,
                children: vec![],
            }]
        );
    }

    #[test]
    fn component_with_nested_elements() {
        let mut chars = "<MyComponent><p>Hello, world!</p></MyComponent>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "MyComponent".to_owned(),
                attributes: Attributes::new(),
                is_component: true,
                self_closing: false,
                children: vec![TmplAst::Element {
                    tag: "p".to_owned(),
                    attributes: Attributes::new(),
                    is_component: false,
                    self_closing: false,
                    children: vec![TmplAst::Text("Hello, world!".to_owned())],
                }],
            }]
        );
    }

    #[test]
    fn unnamed_slot_interpolation() {
        let mut chars = "<div><#slot>{1 + 1}</#slot></div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::SlotInterpolation {
                    slot_name: None,
                    default_children: Some(vec![TmplAst::Expression("1 + 1".to_owned())]),
                }],
            }]
        );
    }

    #[test]
    fn slot_interpolation() {
        let mut chars = "<div><#slot slot_name /></div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::SlotInterpolation {
                    slot_name: Some("slot_name".to_owned()),
                    default_children: None,
                }],
            }]
        );
    }

    #[test]
    fn expression_with_slot_interpolation_and_whitespace() {
        let mut chars = "<div>{1 + 1} <#slot slot_name /> {2 + 2}</div>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::SlotInterpolation {
                        slot_name: Some("slot_name".to_owned()),
                        default_children: None,
                    },
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn slot_interpolation_with_default_children() {
        let mut chars = "<div>Hello, <#slot>John Doe</#slot>!</div>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::SlotInterpolation {
                        slot_name: None,
                        default_children: Some(vec![TmplAst::Text("John Doe".to_owned())]),
                    },
                    TmplAst::Text("!".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn two_expression_split_by_whitespace() {
        let mut chars = "<div>{1 + 1} {2 + 2}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn multiple_expressions_with_whitespace() {
        let mut chars = "<div>{1 + 1} {2 + 2} {3 + 3}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("3 + 3".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn expression_with_text_and_whitespace() {
        let mut chars = "<div>Hello {1 + 1} World {2 + 2}!</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Text("Hello ".to_owned()),
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" World ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                    TmplAst::Text("!".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn expression_with_multiple_whitespace_chars() {
        let mut chars = "<div>{1 + 1}  {2 + 2}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn expression_with_tab_and_newline() {
        let mut chars = "<div>{1 + 1}\t{2 + 2}\n{3 + 3}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("3 + 3".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn nested_elements_with_whitespace() {
        let mut chars = "<div><span>{1 + 1} {2 + 2}</span></div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::Element {
                    tag: "span".to_owned(),
                    attributes: Attributes::new(),
                    is_component: false,
                    self_closing: false,
                    children: vec![
                        TmplAst::Expression("1 + 1".to_owned()),
                        TmplAst::Text(" ".to_owned()),
                        TmplAst::Expression("2 + 2".to_owned()),
                    ],
                }],
            }]
        );
    }

    #[test]
    fn component_with_whitespace_between_expressions() {
        let mut chars = "<MyComponent>{1 + 1} {2 + 2}</MyComponent>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "MyComponent".to_owned(),
                attributes: Attributes::new(),
                is_component: true,
                self_closing: false,
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn slot_with_whitespace_between_expressions() {
        let mut chars = "<#slot_name>{1 + 1} {2 + 2}</#slot_name>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Slot {
                name: Some("slot_name".to_owned()),
                children: vec![
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn top_level_whitespace_skipping() {
        let mut chars = "  <div>Hello</div>  <span>World</span>  "
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![
                TmplAst::Element {
                    tag: "div".to_owned(),
                    attributes: Attributes::new(),
                    is_component: false,
                    self_closing: false,
                    children: vec![TmplAst::Text("Hello".to_owned())],
                },
                TmplAst::Element {
                    tag: "span".to_owned(),
                    attributes: Attributes::new(),
                    is_component: false,
                    self_closing: false,
                    children: vec![TmplAst::Text("World".to_owned())],
                },
            ]
        );
    }

    #[test]
    fn mixed_content_with_whitespace() {
        let mut chars = "<div>Text {1 + 1} More {2 + 2} End</div>"
            .chars()
            .peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Text("Text ".to_owned()),
                    TmplAst::Expression("1 + 1".to_owned()),
                    TmplAst::Text(" More ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                    TmplAst::Text(" End".to_owned()),
                ],
            }]
        );
    }
    #[test]
    fn conditional_directive() {
        let mut chars = "{#if true}Hello, world!{/if}".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                condition: "true".to_owned(),
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }])]
        );
    }

    #[test]
    fn conditional_directive_inside_element() {
        let mut chars = r#"
            <div>
                {#if 1 + 1 == 2}
                    Hello, world!
                {/if}
            </div>
        "#
        .chars()
        .peekable();

        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                    condition: "1 + 1 == 2".to_owned(),
                    children: vec![TmplAst::Text("Hello, world!".to_owned())],
                }])],
            }]
        );
    }

    #[test]
    fn several_conditional_directives() {
        let mut chars = r#"
            <div>
                {#if 1 + 1 == 2}
                    <span>
                        Hello, world 1!
                    </span>
                {/if}

                {#if 1 + 1 == 2}
                    <span>
                        Hello, world 2!
                    </span>
                {/if}
            </div>
        "#
        .chars()
        .peekable();

        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                        condition: "1 + 1 == 2".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: Attributes::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Hello, world 1!".to_owned())],
                        }],
                    }]),
                    TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                        condition: "1 + 1 == 2".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: Attributes::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Hello, world 2!".to_owned())],
                        }],
                    }])
                ],
            }]
        );
    }

    #[test]
    fn conditions_with_after_elements() {
        let mut chars = r#"
            <div>
                {#if true}
                    <span>Hello, world!</span>
                {/if}

                <span>Hello, world 2!</span>
            </div>
        "#
        .chars()
        .peekable();

        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                        condition: "true".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: Attributes::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Hello, world!".to_owned())],
                        }],
                    }]),
                    TmplAst::Element {
                        tag: "span".to_owned(),
                        attributes: Attributes::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![TmplAst::Text("Hello, world 2!".to_owned())],
                    },
                ],
            }]
        );
    }

    #[test]
    fn outlet_directive() {
        let mut chars = "{#outlet}".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(ast, vec![TmplAst::Outlet]);
    }

    #[test]
    fn outlet_directive_in_element() {
        let mut chars = "<div>{#outlet}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::Outlet],
            }]
        );
    }
}
