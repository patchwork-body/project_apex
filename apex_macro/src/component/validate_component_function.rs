use syn::{FnArg, ItemFn, Pat, Result, ReturnType};

use crate::component::is_html_type;

/// Validate that the function has the correct signature for a component
pub(crate) fn validate_component_function(input: &ItemFn) -> Result<()> {
    // Check that all parameters have #[prop] attribute
    for arg in &input.sig.inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                // Check if parameter has #[prop] attribute
                let has_prop_attr = pat_type.attrs.iter().any(|attr| {
                    attr.path()
                        .get_ident()
                        .map(|ident| ident == "prop")
                        .unwrap_or(false)
                });

                if !has_prop_attr {
                    // Extract parameter name for better error message
                    let param_name = match &*pat_type.pat {
                        Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                        _ => "parameter".to_owned(),
                    };

                    return Err(syn::Error::new_spanned(
                        pat_type,
                        format!("Component parameter '{param_name}' must have #[prop] attribute"),
                    ));
                }
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    arg,
                    "Component functions cannot have self parameter",
                ));
            }
        }
    }

    // Check that function returns Html
    match &input.sig.output {
        ReturnType::Type(_, ty) => {
            // Check if return type is Html (simple check, could be improved)
            if !is_html_type(ty) {
                return Err(syn::Error::new_spanned(
                    ty,
                    "Component functions must return Html",
                ));
            }
        }
        ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &input.sig,
                "Component functions must have an explicit Html return type",
            ));
        }
    }

    Ok(())
}
