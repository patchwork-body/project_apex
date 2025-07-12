use std::{collections::HashMap, str::Chars};
use syn::Result;

use crate::tmpl::{ComponentAttribute, TmplAst};

pub(crate) fn parse_tmpl_into_ast(input: &str) -> Result<Vec<TmplAst>> {
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
            if let Some(element) = parse_element(&mut chars)? {
                ast.push(element);
            }
        } else if let Some(content) = parse_text_or_expression(&mut chars)? {
            ast.push(content);
        }
    }

    Ok(ast)
}

fn parse_element(chars: &mut std::iter::Peekable<Chars<'_>>) -> Result<Option<TmplAst>> {
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

                attributes.insert(attr_name, ComponentAttribute::Literal(value));
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
                    attributes.insert(
                        attr_name,
                        ComponentAttribute::EventHandler(value.trim().to_owned()),
                    );
                } else {
                    attributes.insert(
                        attr_name,
                        ComponentAttribute::Expression(value.trim().to_owned()),
                    );
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

                attributes.insert(attr_name, ComponentAttribute::Literal(value));
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
                if let Some(child) = parse_element(chars)? {
                    children.push(child);
                }
            } else {
                // Parse text or expression
                if let Some(child) = parse_text_or_expression(chars)? {
                    children.push(child);
                }
            }
        }
    }

    Ok(Some(TmplAst::Element {
        tag: tag_name,
        attributes,
        self_closing,
        children,
    }))
}

fn parse_text_or_expression(chars: &mut std::iter::Peekable<Chars<'_>>) -> Result<Option<TmplAst>> {
    let mut content = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '<' {
            break;
        } else if ch == '{' {
            // If we have accumulated text, return it first
            if !content.is_empty() {
                return Ok(Some(TmplAst::Text(content)));
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

            return Ok(Some(TmplAst::Expression(expr)));
        } else {
            content.push(chars.next().unwrap());
        }
    }

    if content.is_empty() {
        Ok(None)
    } else {
        Ok(Some(TmplAst::Text(content)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::TmplAst;
    use std::collections::HashMap;

    #[test]
    fn test_one_element() {
        let input = "<div>Hello, world!</div>";
        let ast = parse_tmpl_into_ast(input).unwrap();

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
        let ast = parse_tmpl_into_ast(input).unwrap();

        assert_eq!(ast.len(), 1);
        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    ComponentAttribute::Literal("container".to_owned())
                )]),
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }
        );
    }

    #[test]
    fn test_one_element_with_attribute_and_expression() {
        let input = "<div class=\"container\">Hello, {name}!</div>";
        let ast = parse_tmpl_into_ast(input).unwrap();

        assert_eq!(ast.len(), 1);
        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    ComponentAttribute::Literal("container".to_owned())
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
        let ast = parse_tmpl_into_ast(input).unwrap();

        assert_eq!(ast.len(), 1);
        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([
                    (
                        "class".to_owned(),
                        ComponentAttribute::Literal("container".to_owned())
                    ),
                    (
                        "onclick".to_owned(),
                        ComponentAttribute::EventHandler("handleClick".to_owned())
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
        let ast = parse_tmpl_into_ast(input).unwrap();

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
        let ast = parse_tmpl_into_ast(input).unwrap();

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
        let ast = parse_tmpl_into_ast(input).unwrap();

        assert_eq!(ast.len(), 1);

        assert_eq!(
            ast[0],
            TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    ComponentAttribute::Expression("class".to_owned())
                )]),
                self_closing: false,
                children: vec![],
            }
        );
    }
}
