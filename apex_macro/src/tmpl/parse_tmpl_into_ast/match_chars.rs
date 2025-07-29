use std::str::Chars;

pub(crate) fn match_chars(chars: &mut std::iter::Peekable<Chars<'_>>, end_of_block: &str) -> bool {
    let mut end_of_block_chars = end_of_block.chars().peekable();

    if chars.peek() == end_of_block_chars.peek() {
        let mut lookahead = chars.clone();
        let mut matched = true;

        while end_of_block_chars.peek().is_some() {
            if end_of_block_chars.peek() != lookahead.peek() {
                matched = false;
                break;
            }

            lookahead.next();
            end_of_block_chars.next();
        }

        if matched {
            // Consume the characters from the original iterator
            for _ in 0..end_of_block.len() {
                chars.next();
            }

            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_closing_element_tag() {
        let mut chars = "</div>".chars().peekable();
        assert!(match_chars(&mut chars, "</div>"));
        assert_eq!(chars.peek(), None);
    }

    #[test]
    fn test_match_closing_slot_tag() {
        let mut chars = "</#header>".chars().peekable();
        assert!(match_chars(&mut chars, "</#header>"));
        assert_eq!(chars.peek(), None);
    }

    #[test]
    fn test_match_closing_directive_tag() {
        let mut chars = "<#endif>".chars().peekable();
        assert!(match_chars(&mut chars, "<#endif>"));
        assert_eq!(chars.peek(), None);
    }
}
