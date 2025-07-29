use crate::tmpl::TmplAst;

mod match_chars;
mod parse_element_opening_tag;
mod parse_slot_name;
mod process_chars_until;

use process_chars_until::*;

pub(crate) fn parse_tmpl_into_ast(input: &str) -> Vec<TmplAst> {
    // Normalize input: remove line breaks and reduce all whitespace to a single space
    let input = input
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned();

    let mut chars = input.chars().peekable();

    process_chars_until(&mut chars, None)
}
