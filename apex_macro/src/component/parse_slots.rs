use syn::{Expr, FnArg, ItemFn, Pat, PatIdent};

/// A component slot extracted from function parameters
pub(crate) struct ComponentSlot {
    pub _name: PatIdent,
    pub _default: Option<Expr>,
}

/// Parse slots from function parameters that have #[slot] attribute
pub(crate) fn parse_slots(input: &ItemFn) -> Vec<ComponentSlot> {
    let mut slots = Vec::new();

    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Check if parameter has #[slot] attribute and extract default value
            let mut has_slot_attr = false;
            let mut default_value = None;

            for attr in &pat_type.attrs {
                if attr
                    .path()
                    .get_ident()
                    .map(|ident| ident == "slot")
                    .unwrap_or(false)
                {
                    has_slot_attr = true;

                    // Parse attribute arguments for default value
                    if let Ok(args) = attr.parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                    ) {
                        for meta in args {
                            if let syn::Meta::NameValue(name_value) = meta {
                                if name_value.path.is_ident("default") {
                                    if let syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Str(lit_str),
                                        ..
                                    }) = &name_value.value
                                    {
                                        // Parse the string as an expression
                                        if let Ok(expr) = syn::parse_str::<Expr>(&lit_str.value()) {
                                            default_value = Some(expr);
                                        }
                                    } else {
                                        // Direct expression
                                        default_value = Some(name_value.value);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if has_slot_attr {
                // Extract the parameter name and type
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    slots.push(ComponentSlot {
                        _name: pat_ident.clone(),
                        _default: default_value,
                    });
                }
            }
        }
    }

    slots
}
