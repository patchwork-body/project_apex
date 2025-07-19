mod parse_tmpl;
mod parse_tmpl_into_ast;
mod render_ast;

pub(crate) use parse_tmpl::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Attribute {
    Literal(String),
    Expression(String),
    Signal(String),
    EventListener(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TmplAst {
    Text(String),
    Expression(String),
    Signal(String),
    Element {
        tag: String,
        attributes: std::collections::HashMap<String, Attribute>,
        self_closing: bool,
        children: Vec<TmplAst>,
    },
    Component {
        name: String,
        attributes: std::collections::HashMap<String, Attribute>,
        children: Vec<TmplAst>,
    },
}
