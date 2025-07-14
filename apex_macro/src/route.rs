use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Result};

/// Configuration for a route
#[derive(Debug)]
pub(crate) struct RouteConfig {
    path: String,
    component: Option<String>,
}

/// Parse the route macro arguments to extract configuration
pub(crate) fn parse_route_args(args: TokenStream) -> Result<RouteConfig> {
    let args_str = args.to_string();

    let mut config = RouteConfig {
        path: String::new(),
        component: None,
    };

    // Parse path
    if let Some(path_pos) = args_str.find("path =") {
        let path_start = path_pos + 6;
        let rest = &args_str[path_start..].trim();

        if let Some(quote_start) = rest.find('"') {
            let quote_content = &rest[quote_start + 1..];
            if let Some(quote_end) = quote_content.find('"') {
                config.path = quote_content[..quote_end].to_string();
            }
        }
    }

    // Parse component
    if let Some(component_pos) = args_str.find("component =") {
        let component_start = component_pos + 11;
        let rest = &args_str[component_start..].trim();

        // Extract identifier until comma or end
        let component_end = rest.find(',').unwrap_or(rest.len());
        config.component = Some(rest[..component_end].trim().to_owned());
    }

    if config.path.is_empty() {
        return Err(syn::Error::new_spanned(
            proc_macro2::TokenStream::from(args),
            "Path is required for route macro",
        ));
    }

    Ok(config)
}

/// Generate the route handler implementation
pub(crate) fn generate_route(
    input: &ItemFn,
    config: &RouteConfig,
) -> Result<proc_macro2::TokenStream> {
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;

    // Generate the route handler function based on configuration
    let handler_impl = match &config.component {
        Some(component) => {
            let component_ident = syn::parse_str::<syn::Ident>(component)?;

            quote! {
                #fn_vis fn #fn_name<C>(
                    req: apex::HttpRequest,
                    context: &C
                ) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = apex::HttpResponse> + Send + '_>>
                where
                    C: std::marker::Send + std::marker::Sync + Clone + 'static
                {
                    std::boxed::Box::pin(async move {
                        use apex::http::{Response, StatusCode};
                        use apex::http_body_util::Full;
                        use apex::bytes::Bytes;

                        let component = #component_ident::new();
                        let html = apex::View::render(&component);

                        Response::builder()
                            .status(StatusCode::OK)
                            .header("content-type", "text/html; charset=utf-8")
                            .body(Full::new(Bytes::from(html.into_string())))
                            .unwrap()
                    })
                }
            }
        }
        None => {
            quote! {
                #fn_vis fn #fn_name<C>(
                    _req: apex::HttpRequest,
                    _context: &C
                ) -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = apex::HttpResponse> + Send + '_>>
                where
                    C: std::marker::Send + std::marker::Sync + Clone + 'static
                {
                    std::boxed::Box::pin(async move {
                        use http::{Response, StatusCode};
                        use http_body_util::Full;
                        use bytes::Bytes;

                        // For API endpoints, return simple JSON
                        let loader_result = LoaderResult::ok("api_data");
                        loader_result.into_response(|_data| {
                            format!("{{\"data\": \"loaded\"}}")
                        })
                    })
                }
            }
        }
    };

    Ok(handler_impl)
}
