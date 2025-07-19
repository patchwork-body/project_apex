use syn::{FnArg, ItemFn, Pat, PatIdent, Type};

/// A component prop extracted from function parameters
pub(crate) struct ComponentProp {
    pub name: PatIdent,
    pub ty: Type,
}

/// Parse props from function parameters that have #[prop] attribute
pub(crate) fn parse_props(input: &ItemFn) -> Vec<ComponentProp> {
    let mut props = Vec::new();

    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Check if parameter has #[prop] attribute
            let has_prop_attr = pat_type.attrs.iter().any(|attr| {
                attr.path()
                    .get_ident()
                    .map(|ident| ident == "prop")
                    .unwrap_or(false)
            });

            if has_prop_attr {
                // Extract the parameter name and type
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    props.push(ComponentProp {
                        name: pat_ident.clone(),
                        ty: (*pat_type.ty).clone(),
                    });
                }
            }
        }
    }

    props
}
