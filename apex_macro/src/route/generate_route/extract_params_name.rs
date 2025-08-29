use syn::{FnArg, ItemFn, Pat};

pub(crate) fn extract_params_name(input: &ItemFn) -> &syn::Ident {
    if let Some(FnArg::Typed(pat_type)) = input.sig.inputs.first() {
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            return &pat_ident.ident;
        }
    }

    panic!("Could not extract parameter name from route function");
}
