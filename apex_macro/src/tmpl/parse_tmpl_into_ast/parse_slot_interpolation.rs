use crate::tmpl::TmplAst;
use std::str::Chars;

use super::process_chars_until::process_chars_until;

pub(crate) fn parse_slot_interpolation(chars: &mut std::iter::Peekable<Chars<'_>>) -> TmplAst {
    // At this point, we've already determined this is a slot interpolation starting with <#slot

    // Consume "<#slot"
    chars.next(); // consume '<'
    chars.next(); // consume '#'
    chars.next(); // consume 's'
    chars.next(); // consume 'l'
    chars.next(); // consume 'o'
    chars.next(); // consume 't'

    // Skip whitespace
    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
        chars.next();
    }

    let mut slot_name = None;

    // Check if there's a slot name after "slot"
    if chars.peek() != Some(&'>') && chars.peek() != Some(&'/') {
        let mut name = String::new();

        // Parse slot name until we hit '>' or '/>' or whitespace
        while let Some(ch) = chars.peek() {
            if *ch == '>' || *ch == '/' || ch.is_whitespace() {
                break;
            }
            name.push(chars.next().unwrap());
        }

        if !name.is_empty() {
            slot_name = Some(name);
        }
    }

    // Skip any remaining whitespace
    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
        chars.next();
    }

    // Check if it's self-closing
    let is_self_closing = if chars.peek() == Some(&'/') {
        chars.next(); // consume '/'
        if chars.peek() == Some(&'>') {
            chars.next(); // consume '>'
            true
        } else {
            panic!("Expected '>' after '/' in self-closing slot tag");
        }
    } else if chars.peek() == Some(&'>') {
        chars.next(); // consume '>'
        false
    } else {
        panic!("Expected '>' or '/>' in slot tag");
    };

    let default_children = if is_self_closing {
        // For self-closing slots, we need to check if there's content after it
        // that should be treated as default children
        // We'll return None here and let the caller handle this special case
        None
    } else {
        // Parse children until we find the closing tag
        let closing_tag = "</#slot>";
        let (children, _) = process_chars_until(chars, Some(&[closing_tag]));

        if children.is_empty() {
            None
        } else {
            Some(children)
        }
    };

    TmplAst::SlotInterpolation {
        slot_name,
        default_children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::TmplAst;

    #[test]
    fn test_self_closing_unnamed_slot() {
        let mut chars = "<#slot />".chars().peekable();
        let ast = parse_slot_interpolation(&mut chars);

        assert_eq!(
            ast,
            TmplAst::SlotInterpolation {
                slot_name: None,
                default_children: None,
            }
        );
    }

    #[test]
    fn test_self_closing_named_slot() {
        let mut chars = "<#slot my_slot />".chars().peekable();
        let ast = parse_slot_interpolation(&mut chars);

        assert_eq!(
            ast,
            TmplAst::SlotInterpolation {
                slot_name: Some("my_slot".to_owned()),
                default_children: None,
            }
        );
    }

    #[test]
    fn test_slot_with_default_children() {
        let mut chars = "<#slot>Hello, world!</#slot>".chars().peekable();
        let ast = parse_slot_interpolation(&mut chars);

        assert_eq!(
            ast,
            TmplAst::SlotInterpolation {
                slot_name: None,
                default_children: Some(vec![TmplAst::Text("Hello, world!".to_owned())]),
            }
        );
    }
}
