mod generate_event_listeners;
mod parse_tmpl;
mod parse_tmpl_into_ast;
mod render_ast;

pub(crate) use parse_tmpl::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ComponentAttribute {
    Literal(String),
    Expression(String),
    EventHandler(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TmplAst {
    Text(String),
    Expression(String),
    Signal(String),
    EventListener(String),
    Element {
        tag: String,
        attributes: std::collections::HashMap<String, ComponentAttribute>,
        self_closing: bool,
        children: Vec<TmplAst>,
    },
    Component {
        name: String,
        children: Vec<TmplAst>,
    },
}
