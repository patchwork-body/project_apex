mod generate_component;
mod is_html_type;
mod parse_props;
mod to_pascal_case;
mod validate_component_function;

pub(crate) use generate_component::generate_component;
use is_html_type::is_html_type;
use validate_component_function::validate_component_function;
