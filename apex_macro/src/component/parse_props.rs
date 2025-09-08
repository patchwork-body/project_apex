use syn::{Expr, FnArg, ItemFn, Pat, PatIdent, Type};

/// A component prop extracted from function parameters
pub(crate) struct ComponentProp {
    pub name: PatIdent,
    pub ty: Type,
    pub default: Option<Expr>,
}

/// Parse props from function parameters that have #[prop] attribute
pub(crate) fn parse_props(input: &ItemFn) -> Vec<ComponentProp> {
    let mut props = Vec::new();

    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Check if parameter has #[prop] or #[server_context] attribute
            let mut has_prop_attr = false;
            let mut default_value = None;

            for attr in &pat_type.attrs {
                let attr_name = attr
                    .path()
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .unwrap_or_default();

                if attr_name.as_str() == "prop" {
                    has_prop_attr = true;

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

            if has_prop_attr {
                // Extract the parameter name and type
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    props.push(ComponentProp {
                        name: pat_ident.clone(),
                        ty: (*pat_type.ty).clone(),
                        default: default_value,
                    });
                }
            }
        }
    }

    props
}
