use crate::route::parse_route_args::RouteArgs;
use quote::quote;

pub(crate) fn generate_outlet_helpers(
    fn_name: &syn::Ident,
    route_path: &str,
    args: &RouteArgs,
) -> proc_macro2::TokenStream {
    let outlet_helper_name = syn::Ident::new(&format!("{fn_name}_outlet_matcher"), fn_name.span());

    if args.children.is_empty() {
        // No children, no outlet matching needed
        quote! {}
    } else {
        let children_route_names = &args.children;

        quote! {
            pub fn #outlet_helper_name(request_path: &str) -> Option<Box<dyn apex::router::ApexRoute>> {
                apex::router::outlet_match(#route_path, request_path, vec![
                    #(Box::new(#children_route_names) as Box<dyn apex::router::ApexRoute>),*
                ])
            }
        }
    }
}
