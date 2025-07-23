use std::{collections::HashMap, str::Chars};

use crate::tmpl::{Attribute, TmplAst};

/// Detects if an expression contains signal usage (variables prefixed with $ sign)
fn is_signal_expression(expr: &str) -> bool {
    // Check for signal patterns like $variable
    expr.contains('$')
}

/// Detects if a tag name is a component (starts with uppercase letter)
fn is_component(tag_name: &str) -> bool {
    tag_name.chars().next().is_some_and(|c| c.is_uppercase())
}

/// Detects if a tag name is a slot (starts with #)
fn is_slot(tag_name: &str) -> bool {
    tag_name.starts_with('#')
}

/// Extracts slot name from tag name (removes # prefix)
fn extract_slot_name(tag_name: &str) -> String {
    tag_name[1..].trim().to_owned()
}

/// Detects if an expression is a slot interpolation (starts with @)
fn is_slot_interpolation(expr: &str) -> bool {
    expr.trim().starts_with('@')
}

/// Extracts slot name from slot interpolation (removes @ prefix)
fn extract_slot_interpolation_name(expr: &str) -> String {
    expr.trim()[1..].trim().to_owned()
}

/// Splits an expression into multiple parts, separating signals from literals
fn split_expression_into_parts(expr: &str) -> Vec<TmplAst> {
    // If the expression contains signals, treat the entire expression as a single signal
    if is_signal_expression(expr) {
        return vec![TmplAst::Signal(expr.to_owned())];
    }

    // If no signals, treat as regular expression
    vec![TmplAst::Expression(expr.to_owned())]
}

pub(crate) fn parse_tmpl_into_ast(input: &str) -> Vec<TmplAst> {
    // Normalize input: remove line breaks and reduce all whitespace to a single space
    let input = input
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned();

    let mut ast = Vec::new();
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace between elements
        while chars.peek() == Some(&' ')
            || chars.peek() == Some(&'\n')
            || chars.peek() == Some(&'\r')
            || chars.peek() == Some(&'\t')
        {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        if chars.peek() == Some(&'<') {
            if let Some(element) = parse_element(&mut chars) {
                ast.push(element);
            }
        } else {
            let content = parse_text_or_expression(&mut chars);
            ast.extend(content);
        }
    }

    ast
}

fn parse_element(chars: &mut std::iter::Peekable<Chars<'_>>) -> Option<TmplAst> {
    // Consume '<'
    chars.next();

    // Parse tag name
    let mut tag_name = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == ' ' || ch == '>' || ch == '/' {
            break;
        }

        tag_name.push(chars.next().unwrap());
    }

    // Skip whitespace
    while chars.peek() == Some(&' ') {
        chars.next();
    }

    // Parse attributes
    let mut attributes = HashMap::new();

    while let Some(&ch) = chars.peek() {
        if ch == '>' || ch == '/' {
            break;
        }

        // Parse attribute name
        let mut attr_name = String::new();

        while let Some(&ch) = chars.peek() {
            if ch == '=' || ch == ' ' || ch == '>' || ch == '/' {
                break;
            }

            attr_name.push(chars.next().unwrap());
        }

        // Skip whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }

        if chars.peek() == Some(&'=') {
            chars.next(); // consume '='

            // Skip whitespace
            while chars.peek() == Some(&' ') {
                chars.next();
            }

            if chars.peek() == Some(&'"') {
                chars.next(); // consume opening quote
                let mut value = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next(); // consume closing quote
                        break;
                    }
                    value.push(chars.next().unwrap());
                }

                attributes.insert(attr_name, Attribute::Literal(value));
                continue;
            } else if chars.peek() == Some(&'{') {
                // Expression in braces
                chars.next(); // consume '{'

                let mut value = String::new();
                let mut brace_depth = 1;

                while let Some(&ch) = chars.peek() {
                    if ch == '{' {
                        brace_depth += 1;
                    } else if ch == '}' {
                        brace_depth -= 1;

                        if brace_depth == 0 {
                            chars.next(); // consume closing '}'
                            break;
                        }
                    }
                    value.push(chars.next().unwrap());
                }

                if attr_name.starts_with("on") {
                    attributes.insert(attr_name, Attribute::EventListener(value.trim().to_owned()));
                } else if is_signal_expression(&value) {
                    attributes.insert(attr_name, Attribute::Signal(value.trim().to_owned()));
                } else {
                    attributes.insert(attr_name, Attribute::Expression(value.trim().to_owned()));
                }

                continue;
            } else {
                // Unquoted value
                let mut value = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch == ' ' || ch == '>' || ch == '/' {
                        break;
                    }

                    value.push(chars.next().unwrap());
                }

                attributes.insert(attr_name, Attribute::Literal(value));
                continue;
            };
        }

        // Skip whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }
    }

    // Check if self-closing
    let self_closing = chars.peek() == Some(&'/');
    if self_closing {
        chars.next(); // consume '/'
    }

    // Consume '>'
    if chars.peek() == Some(&'>') {
        chars.next();
    }

    let mut children = Vec::new();

    if !self_closing {
        // Parse children until closing tag
        while chars.peek().is_some() {
            // Check for closing tag
            if chars.peek() == Some(&'<') {
                // Look ahead to see if this is a closing tag
                let mut lookahead = chars.clone();
                lookahead.next(); // consume '<'

                if lookahead.peek() == Some(&'/') {
                    // This is a closing tag
                    lookahead.next(); // consume '/'
                    let mut closing_tag = String::new();

                    while let Some(&ch) = lookahead.peek() {
                        if ch == '>' {
                            break;
                        }

                        closing_tag.push(lookahead.next().unwrap());
                    }

                    if closing_tag.trim() == tag_name {
                        // This is our closing tag, consume it
                        chars.next(); // '<'
                        chars.next(); // '/'

                        while chars.peek() != Some(&'>') && chars.peek().is_some() {
                            chars.next();
                        }

                        if chars.peek() == Some(&'>') {
                            chars.next(); // '>'
                        }

                        break;
                    }
                }

                // Not our closing tag, parse as child element
                if let Some(child) = parse_element(chars) {
                    children.push(child);
                }
            } else {
                // Parse text or expression
                let parsed_children = parse_text_or_expression(chars);
                children.extend(parsed_children);
            }
        }
    }

    // If this element is a slot (starts with #), treat it as a Slot node
    if is_slot(&tag_name) {
        let slot_name = extract_slot_name(&tag_name);
        return Some(TmplAst::Slot {
            name: slot_name,
            children,
        });
    }

    Some(if is_component(&tag_name) {
        TmplAst::Component {
            name: tag_name,
            attributes,
            children,
        }
    } else {
        TmplAst::Element {
            tag: tag_name,
            attributes,
            self_closing,
            children,
        }
    })
}

fn parse_text_or_expression(chars: &mut std::iter::Peekable<Chars<'_>>) -> Vec<TmplAst> {
    let mut result = Vec::new();
    let mut content = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '<' {
            break;
        } else if ch == '{' {
            // If we have accumulated text, add it to results
            if !content.is_empty() {
                result.push(TmplAst::Text(content.clone()));
                content.clear();
            }

            // Parse expression
            chars.next(); // consume '{'
            let mut expr = String::new();
            let mut brace_depth = 1;

            while let Some(&ch) = chars.peek() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        chars.next(); // consume closing '}'
                        break;
                    }
                }
                expr.push(chars.next().unwrap());
            }

            // Add the expression to results
            if is_signal_expression(&expr) {
                result.extend(split_expression_into_parts(&expr));
            } else if is_slot_interpolation(&expr) {
                let slot_name = extract_slot_interpolation_name(&expr);
                result.push(TmplAst::SlotInterpolation { slot_name });
            } else {
                result.push(TmplAst::Expression(expr));
            }
        } else {
            content.push(chars.next().unwrap());
        }
    }

    // Add any remaining text
    if !content.is_empty() {
        result.push(TmplAst::Text(content));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::TmplAst;
    use std::collections::HashMap;

    #[test]
    fn test_one_element() {
        let input = "<div>Hello, world!</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }
        );
    }

    #[test]
    fn test_one_element_with_attribute() {
        let input = "<div class=\"container\">Hello, world!</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);
        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    Attribute::Literal("container".to_owned())
                )]),
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }
        );
    }

    #[test]
    fn test_one_element_with_attribute_and_expression() {
        let input = "<div class=\"container\">Hello, {name}!</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);
        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    Attribute::Literal("container".to_owned())
                )]),
                self_closing: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::Expression("name".to_owned()),
                    TmplAst::Text("!".to_owned()),
                ],
            }
        );
    }

    #[test]
    fn test_one_element_with_attribute_and_expression_and_event_listener() {
        let input = "<div class=\"container\" onclick={handleClick}>Hello, {name}!</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);
        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([
                    (
                        "class".to_owned(),
                        Attribute::Literal("container".to_owned())
                    ),
                    (
                        "onclick".to_owned(),
                        Attribute::EventListener("handleClick".to_owned())
                    )
                ]),
                self_closing: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::Expression("name".to_owned()),
                    TmplAst::Text("!".to_owned()),
                ],
            }
        );
    }

    #[test]
    fn test_several_elements_on_the_same_level() {
        let input = "<div>Hello, world!</div><div>Hello, world!</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 2);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }
        );

        assert_eq!(
            ast[1],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }
        );
    }

    #[test]
    fn test_nested_elements() {
        let input = "<div><div>Hello, world!</div></div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::Element {
                    tag: "div".to_owned(),
                    attributes: HashMap::new(),
                    self_closing: false,
                    children: vec![TmplAst::Text("Hello, world!".to_owned())],
                }],
            }
        );
    }

    #[test]
    fn test_dynamic_attrs() {
        let input = "<div class={class}></div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    Attribute::Expression("class".to_owned())
                )]),
                self_closing: false,
                children: vec![],
            }
        );
    }

    #[test]
    fn test_signal_expression() {
        let input = "<div>Hello, {$name}!</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![
                    TmplAst::Text("Hello, ".to_owned()),
                    TmplAst::Signal("$name".to_owned()),
                    TmplAst::Text("!".to_owned()),
                ],
            }
        );
    }

    #[test]
    fn test_mixed_expressions() {
        let input = "<div>{$count} items: {message}</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![
                    TmplAst::Signal("$count".to_owned()),
                    TmplAst::Text(" items: ".to_owned()),
                    TmplAst::Expression("message".to_owned()),
                ],
            }
        );
    }

    #[test]
    fn test_complex_signal_expression() {
        let input = "<div>Counter: {1 + $counter - 2 + $another_counter}</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![
                    TmplAst::Text("Counter: ".to_owned()),
                    TmplAst::Signal("1 + $counter - 2 + $another_counter".to_owned()),
                ],
            }
        );
    }

    #[test]
    fn test_signal_expression_with_other_literals() {
        let input = "<div>{$count + 1}</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::Signal("$count + 1".to_owned()),],
            }
        );
    }

    #[test]
    fn test_dollar_prefixed_literal_will_not_ba_parsed_as_signals() {
        let input = "<div>$count</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::Text("$count".to_owned())],
            }
        );
    }

    #[test]
    fn test_slot_interpolation() {
        let input = "<div>{@slot_name}</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![TmplAst::SlotInterpolation {
                    slot_name: "slot_name".to_owned()
                }],
            }
        );
    }

    #[test]
    fn test_component() {
        let input = "<HelloWorld />";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Component {
                name: "HelloWorld".to_owned(),
                attributes: HashMap::new(),
                children: vec![],
            }
        );
    }

    #[test]
    fn test_counter() {
        let input = "<div><button onclick={inc}>Inc</button><p class=\"count\">{$count}</p><button onclick={dec}>Dec</button></div>";

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                self_closing: false,
                children: vec![
                    TmplAst::Element {
                        tag: "button".to_owned(),
                        attributes: HashMap::from([(
                            "onclick".to_owned(),
                            Attribute::EventListener("inc".to_owned())
                        )]),
                        self_closing: false,
                        children: vec![TmplAst::Text("Inc".to_owned()),],
                    },
                    TmplAst::Element {
                        tag: "p".to_owned(),
                        attributes: HashMap::from([(
                            "class".to_owned(),
                            Attribute::Literal("count".to_owned())
                        )]),
                        self_closing: false,
                        children: vec![TmplAst::Signal("$count".to_owned())],
                    },
                    TmplAst::Element {
                        tag: "button".to_owned(),
                        attributes: HashMap::from([(
                            "onclick".to_owned(),
                            Attribute::EventListener("dec".to_owned())
                        )]),
                        self_closing: false,
                        children: vec![TmplAst::Text("Dec".to_owned())],
                    },
                ],
            }
        );
    }
}
