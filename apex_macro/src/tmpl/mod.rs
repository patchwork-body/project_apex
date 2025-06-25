mod generate_component_code;
mod generate_event_listeners;
mod generate_html_opening_tag_code;
mod generate_render_parts;
mod parse_component_attributes_from_str;
mod parse_single_attribute;
mod parse_tag_content;
mod parse_tag_name_and_attributes;
mod parse_tmpl;
mod parse_tmpl_structure;
mod parse_variable_content;

pub(crate) use parse_tmpl::*;

/// Context where a dynamic variable is used, including element reference for updates
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DynamicVariableContext {
    /// Used as text content between HTML elements
    TextNode {
        /// ID of the parent element that contains this text node
        element_id: String,
        /// Index of the text node within the parent element (for multiple text nodes)
        text_node_index: usize,
    },
    /// Used as an attribute value in an HTML element
    AttributeValue {
        /// ID of the element that has this attribute
        element_id: String,
        /// Name of the attribute to update
        attribute_name: String,
    },
}

/// HTML content types for structured parsing
#[derive(Debug, Clone)]
pub(crate) enum HtmlContent {
    Text(String),
    /// Static variable - regular Rust expression/variable (non-reactive)
    StaticVariable(String),
    /// Dynamic variable - Signal-based reactive variable with context information
    DynamicVariable {
        variable: String,
        context: DynamicVariableContext,
    },
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
    /// Static variable - regular Rust expression/variable (non-reactive)
    StaticVariable(String),
    /// Dynamic variable - Signal-based reactive variable with context information
    DynamicVariable {
        variable: String,
        context: DynamicVariableContext,
    },
    Expression(String),
    EventHandler(String), // For event handlers like onclick={handler}
}
