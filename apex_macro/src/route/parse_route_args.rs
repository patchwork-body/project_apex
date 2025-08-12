use proc_macro::TokenStream;
use syn::{
    Ident, LitStr, Meta, Result,
    parse::{Parse, ParseStream},
};

/// Arguments parsed from the #[route(...)] macro
#[derive(Debug, Default)]
pub(crate) struct RouteArgs {
    pub component: Option<Ident>,
    pub path: Option<LitStr>,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut route_args = RouteArgs::default();

        if input.is_empty() {
            return Ok(route_args);
        }

        let parsed_args =
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated(input)?;

        for meta in parsed_args {
            match meta {
                Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("component") {
                        if let syn::Expr::Path(expr_path) = &name_value.value {
                            if let Some(ident) = expr_path.path.get_ident() {
                                route_args.component = Some(ident.clone());
                            }
                        }
                    } else if name_value.path.is_ident("path") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = &name_value.value
                        {
                            route_args.path = Some(s.clone());
                        }
                    }
                }
                Meta::Path(path) => {
                    // Handle simple identifiers without values if needed
                    if let Some(ident) = path.get_ident() {
                        // Could handle flags like #[route(async)] here
                        let _ = ident;
                    }
                }
                Meta::List(_) => {
                    // Handle nested attributes if needed in the future
                }
            }
        }

        Ok(route_args)
    }
}

/// Parse arguments from the route macro attribute
/// Supports syntax like: #[route(component = HomeComponent, path = "/home")]
pub(crate) fn parse_route_args(args: TokenStream) -> RouteArgs {
    if args.is_empty() {
        return RouteArgs::default();
    }

    syn::parse(args).unwrap_or_else(|err| {
        panic!("Failed to parse route arguments: {err}");
    })
}
