use std::str::Chars;

use crate::tmpl::IfBlock;

use super::parse_directive_params::parse_directive_params;
use super::process_chars_until::process_chars_until;

pub(crate) fn parse_conditional_directive(
    chars: &mut std::iter::Peekable<Chars<'_>>,
) -> Vec<IfBlock> {
    let mut if_blocks = Vec::new();
    let directive_params = parse_directive_params(chars);

    let (block, exit) = process_chars_until(
        chars,
        Some(&[
            "{#endif}", "{#else}", "{#elseif", // No closing '}', bc there're params to parse
        ]),
    );

    if_blocks.push(IfBlock {
        condition: directive_params,
        children: block,
    });

    if exit == "{#endif}" {
        return if_blocks;
    } else if exit == "{#else}" {
        let (block, _) = process_chars_until(chars, Some(&["{#endif}"]));

        if_blocks.push(IfBlock {
            condition: "true".to_owned(),
            children: block,
        });
    } else if exit == "{#elseif" {
        if_blocks.extend(parse_conditional_directive(chars));
    }

    panic!("Expected {{#endif}}");
}
