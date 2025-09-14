use std::str::Chars;

fn is_valid_slot_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

pub(crate) fn parse_slot_name(chars: &mut std::iter::Peekable<Chars<'_>>) -> String {
    let mut slot_name = String::new();

    if chars.peek() == Some(&'<') {
        chars.next(); // consume '<'
    } else {
        panic!("Unexpected character: {:?}, expected '<'", chars.peek());
    }

    if chars.peek() == Some(&'/') {
        chars.next(); // consume '/'
    }

    if chars.peek() == Some(&'#') {
        chars.next(); // consume '#'
    } else {
        panic!("Unexpected character: {:?}, expected '#'", chars.peek());
    }

    while let Some(ch) = chars.next() {
        if ch == '>' {
            break;
        }

        if ch == '/' && chars.peek() == Some(&'>') {
            chars.next(); // consume '>'
            break;
        }

        if ch != ' ' && !is_valid_slot_name_char(ch) {
            panic!(
                "Invalid slot name character: {ch}, slot name must contain only alphanumeric or '_'"
            );
        }

        slot_name.push(ch);
    }

    slot_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slot_name() {
        let mut chars = "<#header>".chars().peekable();
        assert_eq!(parse_slot_name(&mut chars), "header");
    }

    #[test]
    fn test_parse_slot_name_closing_tag() {
        let mut chars = "</#header>".chars().peekable();
        assert_eq!(parse_slot_name(&mut chars), "header");
    }

    #[test]
    #[should_panic(expected = "Unexpected character: Some('#'), expected '<'")]
    fn test_parse_slot_name_without_opening_tag_should_panic() {
        parse_slot_name(&mut "#header>".chars().peekable());
    }

    #[test]
    #[should_panic(expected = "Unexpected character: Some('h'), expected '#'")]
    fn test_parse_slot_name_without_hash_should_panic() {
        parse_slot_name(&mut "</header>".chars().peekable());
    }

    #[test]
    #[should_panic(
        expected = "Invalid slot name character: #, slot name must contain only alphanumeric or '_'"
    )]
    fn test_parse_slot_name_with_double_hash_should_panic() {
        parse_slot_name(&mut "<##header>".chars().peekable());
    }
}
