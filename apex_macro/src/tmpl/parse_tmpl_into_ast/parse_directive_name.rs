use std::str::Chars;

pub(crate) fn parse_directive_name(chars: &mut std::iter::Peekable<Chars<'_>>) -> String {
    let mut name = String::new();

    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '}' {
            break;
        }

        chars.next();
        name.push(c);
    }

    name
}
