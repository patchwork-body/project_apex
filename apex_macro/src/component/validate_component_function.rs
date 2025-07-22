use crate::component::is_html_type;
use syn::{FnArg, ItemFn, Pat, ReturnType};

/// Validate that the function has the correct signature for a component
pub(crate) fn validate_component_function(input: &ItemFn) {
    // Check that all parameters have #[prop] or #[slot] attribute
    for arg in &input.sig.inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                // Check if parameter has #[prop] or #[slot] attribute
                let has_prop_attr = pat_type.attrs.iter().any(|attr| {
                    attr.path()
                        .get_ident()
                        .map(|ident| ident == "prop")
                        .unwrap_or(false)
                });

                let has_slot_attr = pat_type.attrs.iter().any(|attr| {
                    attr.path()
                        .get_ident()
                        .map(|ident| ident == "slot")
                        .unwrap_or(false)
                });

                if !has_prop_attr && !has_slot_attr {
                    // Extract parameter name for better error message
                    let param_name = match &*pat_type.pat {
                        Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                        _ => "parameter".to_owned(),
                    };

                    panic!(
                        "Component parameter '{param_name}' must have #[prop] or #[slot] attribute"
                    );
                }
            }
            FnArg::Receiver(_) => {
                panic!("Component functions cannot have self parameter");
            }
        }
    }

    // Check that function returns Html
    match &input.sig.output {
        ReturnType::Type(_, ty) => {
            // Check if return type is Html (simple check, could be improved)
            if !is_html_type(ty) {
                panic!("Component functions must return Html");
            }
        }
        ReturnType::Default => {
            panic!("Component functions must have an explicit Html return type");
        }
    }
}
