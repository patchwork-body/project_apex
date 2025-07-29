use std::str::Chars;

pub(crate) fn parse_directive_name(chars: &mut std::iter::Peekable<Chars<'_>>) -> String {
    let mut name = String::new();

    for c in chars.by_ref() {
        if c == ' ' {
            break;
        }

        name.push(c);
    }

    name
}
