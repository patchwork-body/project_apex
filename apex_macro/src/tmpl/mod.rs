mod generate_component_code;
mod generate_event_listeners;
mod generate_html_opening_tag_code;
mod generate_render_parts;
mod parse_component_attributes_from_str;
mod parse_single_attribute;
mod parse_tag_content;
mod parse_tag_name_and_attributes;
mod parse_tmpl_structure;
mod parse_tmpl_with_context;
mod parse_variable_content;

pub(crate) use parse_tmpl_with_context::*;

/// HTML content types for structured parsing
#[derive(Debug, Clone)]
pub(crate) enum HtmlContent {
    Text(String),
    Variable(String),
    Component {
        tag: String,
        attributes: std::collections::HashMap<String, ComponentAttribute>,
    },
    Element {
        tag: String,
        attributes: std::collections::HashMap<String, ComponentAttribute>,
        self_closing: bool,
        element_id: Option<String>, // For event listener registration
    },
}

/// Component attribute types
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ComponentAttribute {
    Literal(String),
    Variable(String),
    Expression(String),
    EventHandler(String), // For event handlers like onclick={handler}
}
