use std::str::Chars;

pub(crate) fn parse_directive_params(chars: &mut std::iter::Peekable<Chars<'_>>) -> String {
    let mut params = String::new();

    for c in chars.by_ref() {
        if c == '}' {
            break;
        }

        params.push(c);
    }

    params.trim().to_string()
}
