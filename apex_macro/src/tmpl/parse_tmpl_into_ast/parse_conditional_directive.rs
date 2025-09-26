use std::str::Chars;

use crate::tmpl::ConditionalBlock;

use super::parse_directive_params::parse_directive_params;
use super::process_chars_until::process_chars_until;

pub(crate) fn parse_conditional_directive(
    chars: &mut std::iter::Peekable<Chars<'_>>,
) -> Vec<ConditionalBlock> {
    let mut conditional_blocks = Vec::new();
    let directive_params = parse_directive_params(chars);

    let (block, exit) = process_chars_until(
        chars,
        Some(&[
            "{/if}",
            "{:else}",
            "{:else if", // No closing '}', bc there're params to parse
        ]),
    );

    // First block is always an "if" block
    conditional_blocks.push(ConditionalBlock::If {
        condition: directive_params,
        children: block,
    });

    if exit == "{/if}" {
        return conditional_blocks;
    } else if exit == "{:else}" {
        let (block, _) = process_chars_until(chars, Some(&["{/if}"]));

        conditional_blocks.push(ConditionalBlock::Else { children: block });
    } else if exit == "{:else if" {
        // Parse the else-if condition
        let else_if_condition = parse_directive_params(chars);

        let (block, exit) = process_chars_until(chars, Some(&["{/if}", "{:else}", "{:else if"]));

        conditional_blocks.push(ConditionalBlock::ElseIf {
            condition: else_if_condition,
            children: block,
        });

        // Recursively handle any remaining else-if or else blocks
        if exit != "{/if}" {
            // Put back the exit token for recursive parsing
            let remaining_blocks = if exit == "{:else}" {
                let (block, _) = process_chars_until(chars, Some(&["{/if}"]));
                vec![ConditionalBlock::Else { children: block }]
            } else if exit == "{:else if" {
                parse_conditional_directive_continuation(chars)
            } else {
                Vec::new()
            };

            conditional_blocks.extend(remaining_blocks);
        }
    }

    conditional_blocks
}

// Helper function to parse continuation of else-if/else blocks
fn parse_conditional_directive_continuation(
    chars: &mut std::iter::Peekable<Chars<'_>>,
) -> Vec<ConditionalBlock> {
    let mut blocks = Vec::new();
    let condition = parse_directive_params(chars);

    let (block, exit) = process_chars_until(chars, Some(&["{/if}", "{:else}", "{:else if"]));

    blocks.push(ConditionalBlock::ElseIf {
        condition,
        children: block,
    });

    if exit == "{:else}" {
        let (block, _) = process_chars_until(chars, Some(&["{/if}"]));
        blocks.push(ConditionalBlock::Else { children: block });
    } else if exit == "{:else if" {
        blocks.extend(parse_conditional_directive_continuation(chars));
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::TmplAst;

    #[test]
    fn test_parse_conditional_directive() {
        let mut chars = "true}Hello, world!{/if}".chars().peekable();
        let conditional_blocks = parse_conditional_directive(&mut chars);

        assert_eq!(
            conditional_blocks,
            vec![ConditionalBlock::If {
                condition: "true".to_owned(),
                children: vec![TmplAst::Text("Hello, world!".to_owned())],
            }]
        );
    }
}
