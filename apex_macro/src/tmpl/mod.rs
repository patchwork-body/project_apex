mod parse_tmpl;
mod parse_tmpl_into_ast;
mod render_ast;

pub(crate) use parse_tmpl::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Attribute {
    Empty,
    Literal(String),
    Expression(String),
    Signal(String),
    EventListener(String),
}

impl Attribute {
    pub(crate) fn clear(&mut self) {
        *self = Attribute::Empty;
    }

    pub(crate) fn push(&mut self, ch: char) {
        match self {
            Attribute::Empty => {}
            Attribute::Literal(s) => s.push(ch),
            Attribute::Expression(s) => s.push(ch),
            Attribute::Signal(s) => s.push(ch),
            Attribute::EventListener(s) => s.push(ch),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IfBlock {
    condition: String,
    children: Vec<TmplAst>,
}

pub(crate) type Attributes = std::collections::HashMap<String, Attribute>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TmplAst {
    Text(String),
    Expression(String),
    Signal(String),
    Element {
        tag: String,
        attributes: Attributes,
        is_component: bool,
        self_closing: bool,
        children: Vec<TmplAst>,
    },
    Component {
        name: String,
        attributes: Attributes,
        children: Vec<TmplAst>,
    },
    Slot {
        name: String,
        children: Vec<TmplAst>,
    },
    SlotInterpolation {
        slot_name: String,
    },
    Conditional {
        if_blocks: Vec<IfBlock>,
        else_block: Option<Vec<TmplAst>>,
    },
}
