mod generate_component;
mod parse_props;
mod parse_slots;
pub(crate) mod to_pascal_case;
mod validate_component_function;

pub(crate) use generate_component::generate_component;
use validate_component_function::validate_component_function;
