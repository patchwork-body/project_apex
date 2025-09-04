use quote::quote;

use crate::route::parse_route_args::RouteArgs;

pub(crate) fn generate_children_method<RouteTrait: quote::ToTokens>(
    args: &RouteArgs,
    route_trait_type: RouteTrait,
) -> proc_macro2::TokenStream {
    let children_route_names = &args.children;
    let children_inits = children_route_names.iter().map(|child| {
        quote! {
            Box::new(#child::new()) as Box<dyn #route_trait_type>
        }
    });

    quote! {
        fn children(&self) -> Vec<Box<dyn #route_trait_type>> {
            vec![#(#children_inits),*]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::Ident;

    fn create_ident(name: &str) -> Ident {
        Ident::new(name, proc_macro2::Span::call_site())
    }

    #[test]
    fn empty_children() {
        let args = RouteArgs {
            component: None,
            path: None,
            children: vec![],
        };

        let result = generate_children_method(&args, quote!(apex::apex_router::ApexRoute));
        let expected = quote! {
            fn children(&self) -> Vec<Box<dyn apex::apex_router::ApexRoute>> {
                vec![]
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn single_child() {
        let args = RouteArgs {
            component: None,
            path: None,
            children: vec![create_ident("HomeRoute")],
        };

        let result = generate_children_method(&args, quote!(apex::apex_router::ApexRoute));
        let expected = quote! {
            fn children(&self) -> Vec<Box<dyn apex::apex_router::ApexRoute>> {
                vec![Box::new(HomeRoute::new()) as Box<dyn apex::apex_router::ApexRoute>]
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn multiple_children() {
        let args = RouteArgs {
            component: None,
            path: None,
            children: vec![
                create_ident("HomeRoute"),
                create_ident("AboutRoute"),
                create_ident("ContactRoute"),
            ],
        };

        let result = generate_children_method(&args, quote!(apex::apex_router::ApexRoute));
        let expected = quote! {
            fn children(&self) -> Vec<Box<dyn apex::apex_router::ApexRoute>> {
                vec![
                    Box::new(HomeRoute::new()) as Box<dyn apex::apex_router::ApexRoute>,
                    Box::new(AboutRoute::new()) as Box<dyn apex::apex_router::ApexRoute>,
                    Box::new(ContactRoute::new()) as Box<dyn apex::apex_router::ApexRoute>
                ]
            }
        };

        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn preserves_component_and_path() {
        let args = RouteArgs {
            component: Some(create_ident("MyComponent")),
            path: Some(syn::LitStr::new("/test", proc_macro2::Span::call_site())),
            children: vec![create_ident("ChildRoute")],
        };

        let result = generate_children_method(&args, quote!(apex::apex_router::ApexRoute));
        let expected = quote! {
            fn children(&self) -> Vec<Box<dyn apex::apex_router::ApexRoute>> {
                vec![Box::new(ChildRoute::new()) as Box<dyn apex::apex_router::ApexRoute>]
            }
        };

        // No side effects, component and path are not used
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn code_compiles() {
        // Test that the generated TokenStream can be parsed back into valid Rust code
        let empty_args = RouteArgs {
            component: None,
            path: None,
            children: vec![],
        };

        let result =
            generate_children_method(&empty_args, quote!(apex::apex_router::ApexServerRoute));

        // This will panic if the generated code is not valid Rust
        let _: syn::ItemFn =
            syn::parse2(result).expect("Generated empty children code should be valid Rust");

        let args = RouteArgs {
            component: None,
            path: None,
            children: vec![create_ident("TestRoute")],
        };

        let result = generate_children_method(&args, quote!(apex::apex_router::ApexServerRoute));
        let _: syn::ItemFn = syn::parse2(result).expect("Generated code should be valid Rust");
    }

    #[test]
    fn method_signature_is_correct() {
        let args = RouteArgs {
            component: None,
            path: None,
            children: vec![create_ident("SomeRoute")],
        };

        let result = generate_children_method(&args, quote!(apex::apex_router::ApexRoute));

        // Parse the generated code and verify the method signature
        let parsed: syn::ItemFn = syn::parse2(result).expect("Should parse as function");

        // Check function name
        assert_eq!(parsed.sig.ident.to_string(), "children");

        // Check return type
        if let syn::ReturnType::Type(_, ty) = &parsed.sig.output {
            let type_str = quote!(#ty).to_string();
            assert!(type_str.contains("Vec < Box < dyn apex :: apex_router :: ApexRoute > >"));
        } else {
            panic!("Expected return type");
        }

        // Check that it takes &self
        assert_eq!(parsed.sig.inputs.len(), 1);

        if let syn::FnArg::Receiver(receiver) = &parsed.sig.inputs[0] {
            assert!(receiver.reference.is_some());
        } else {
            panic!("Expected &self parameter");
        }
    }
}
