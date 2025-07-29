use std::str::Chars;

use crate::tmpl::TmplAst;

use super::is_pascal_case::is_pascal_case;
use super::match_chars::match_chars;
use super::parse_conditional_directive::parse_conditional_directive;
use super::parse_directive_name::parse_directive_name;
use super::parse_element_opening_tag::parse_element_opening_tag;
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
            ProcessCharsUntilState::Unknown | ProcessCharsUntilState::Text => {
                if state == ProcessCharsUntilState::Unknown && end_of_block.is_none() {
                    // Skip whitespace between elements only when not inside an element
                    while chars.peek() == Some(&' ')
                        || chars.peek() == Some(&'\n')
                        || chars.peek() == Some(&'\r')
                        || chars.peek() == Some(&'\t')
                    {
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
                    state = ProcessCharsUntilState::Expression;

                    if !text.is_empty() {
                        ast.push(TmplAst::Text(text));
                        text = String::new();
                    }

                    chars.next(); // consume '{'
                } else {
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
                    let slot_name = parse_slot_name(chars);
                    let closing_tag = format!("</#{slot_name}>");

                    let (children, _) = process_chars_until(chars, Some(&[&closing_tag]));

                    ast.push(TmplAst::Slot {
                        name: slot_name,
                        children,
                    });
                } else {
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
                    } else {
                        ast.push(TmplAst::Element {
                            tag: element_name,
                            attributes: element_attrs,
                            is_component,
                            self_closing: is_self_closing,
                            children: Vec::new(),
                        });
                    }
                }

                state = ProcessCharsUntilState::Unknown;
            }
            ProcessCharsUntilState::Expression => {
                if chars.peek() == Some(&'@') {
                    chars.next(); // consume '@'
                    expression_type = ExpressionType::Slot;
                } else if chars.peek() == Some(&'#') {
                    chars.next(); // consume '#'
                    expression_type = ExpressionType::Directive;

                    let directive_name = parse_directive_name(chars);

                    if directive_name == "if" {
                        let if_blocks = parse_conditional_directive(chars);

                        ast.push(TmplAst::ConditionalDirective(if_blocks));
                    }
                } else if chars.peek() == Some(&'}') {
                    chars.next(); // consume '}'

                    if !text.is_empty() {
                        match expression_type {
                            ExpressionType::Ordinary => {
                                ast.push(TmplAst::Expression(text));
                            }
                            ExpressionType::Slot => {
                                ast.push(TmplAst::SlotInterpolation { slot_name: text });
                            }
                            ExpressionType::Directive => {}
                        }

                        expression_type = ExpressionType::Ordinary;
                        text = String::new();
                    }

                    state = ProcessCharsUntilState::Unknown;
                } else {
                    let Some(ch) = chars.next() else {
                        panic!("Invalid expression syntax")
                    };

                    text.push(ch);
                }
            }
        }
    }

    if !text.is_empty() {
        ast.push(TmplAst::Text(text));
    }

    (ast, matched_end_of_block)
}

#[cfg(test)]
mod tests {
    use crate::tmpl::{Attribute, Attributes};

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
        let mut chars = "<#slot>Hello, world!</#slot>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Slot {
                name: "slot".to_owned(),
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }]
        );
    }

    #[test]
    fn slots_with_nested_elements() {
        let mut chars = "<#slot><p>Hello, world!</p></#slot>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Slot {
                name: "slot".to_owned(),
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
    fn slot_interpolation() {
        let mut chars = "<div>{@slot_name}</div>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: Attributes::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::SlotInterpolation {
                    slot_name: "slot_name".to_owned(),
                }],
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
                    TmplAst::Text("  ".to_owned()),
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
                    TmplAst::Text("\t".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                    TmplAst::Text("\n".to_owned()),
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
        let mut chars = "<#slot>{1 + 1} {2 + 2}</#slot>".chars().peekable();
        let (ast, _) = process_chars_until(&mut chars, None);

        assert_eq!(
            ast,
            vec![TmplAst::Slot {
                name: "slot".to_owned(),
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
    fn expression_with_slot_interpolation_and_whitespace() {
        let mut chars = "<div>{1 + 1} {@slot_name} {2 + 2}</div>".chars().peekable();
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
                        slot_name: "slot_name".to_owned(),
                    },
                    TmplAst::Text(" ".to_owned()),
                    TmplAst::Expression("2 + 2".to_owned()),
                ],
            }]
        );
    }
}
