use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::{
    common::to_pascal_case,
    component::{parse_props::parse_props, validate_component_function},
};

/// Generate a component from a function
pub(crate) fn generate_component(input: ItemFn) -> TokenStream {
    // Validate the function signature
    validate_component_function(&input);

    // Extract function details
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let vis = &input.vis;

    // Parse props and slots from function parameters
    let props = parse_props(&input);

    // Convert function name to PascalCase for the struct
    let struct_name = syn::Ident::new(&to_pascal_case(&fn_name.to_string()), fn_name.span());
    let builder_name = syn::Ident::new(&format!("{struct_name}Builder"), fn_name.span());

    // Generate struct fields from props and slots
    let struct_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            let ty = &prop.ty;

            quote! {
                pub #name: #ty
            }
        })
        .chain(std::iter::once(quote! {
            pub render_children: Option<std::rc::Rc<Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>>>,
            pub named_slots: Option<std::collections::HashMap<String, std::rc::Rc<Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>>>>,
            pub named_slots_keys: Vec<String>,
            pub has_render_children: bool,
        }));

    // Generate builder struct fields (Option for all)
    let builder_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            let ty = &prop.ty;

            quote! {
                #name: Option<#ty>
            }
        })
        .chain(std::iter::once(quote! {
            render_children: Option<std::rc::Rc<Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>>>,
            named_slots: Option<std::collections::HashMap<String, std::rc::Rc<Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>>>>,
            named_slots_keys: Vec<String>,
            has_render_children: bool,
        }));

    // Generate builder setter methods
    let builder_setters = props.iter().map(|prop| {
        let name = &prop.name;
        let ty = &prop.ty;

        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
     }).chain(std::iter::once(quote! {
         pub fn render_children(mut self, value: Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>) -> Self {
             self.render_children = Some(std::rc::Rc::new(value));
             self.has_render_children = true;
             self
         }
     })).chain(std::iter::once(quote! {
         pub fn named_slots(mut self, value: std::collections::HashMap<String, std::rc::Rc<Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>>>) -> Self {
             self.named_slots_keys = value.keys().cloned().collect();
             self.named_slots = Some(value);
             self
         }
     }));

    // Generate builder default field values
    let builder_default_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            quote! { #name: None }
        })
        .chain(std::iter::once(quote! {
            render_children: None,
            named_slots: None,
            named_slots_keys: Vec::new(),
            has_render_children: bool::default(),
        }));

    // Generate builder build method
    let build_struct_fields = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            if let Some(default) = &prop.default {
                quote! {
                    #name: self.#name.unwrap_or_else(|| #default)
                }
            } else {
                let name_str = name.ident.to_string();
                quote! {
                    #name: self.#name.expect(&format!("Required prop '{}' not set", #name_str))
                }
            }
        })
        .chain(std::iter::once(quote! {
            render_children: self.render_children.clone(),
            named_slots: self.named_slots.clone(),
            named_slots_keys: self.named_slots_keys.clone(),
            has_render_children: self.has_render_children.clone(),
        }));

    let prop_bindings = props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            quote! {
                let #name = self.#name.clone();
            }
        })
        .chain(std::iter::once(quote! {
            #[cfg(not(target_arch = "wasm32"))]
            let render_children = self.render_children.clone();
            #[cfg(not(target_arch = "wasm32"))]
            let named_slots = self.named_slots.clone();

            #[cfg(target_arch = "wasm32")]
            let named_slots_keys = self.named_slots_keys.clone();
            #[cfg(target_arch = "wasm32")]
            let has_render_children = self.has_render_children.clone();
        }))
        .collect::<Vec<_>>();

    // Generate the component struct and impl
    let output = quote! {
        #vis struct #struct_name {
            #(#struct_fields),*
        }

        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#builder_default_fields),*
                }
            }

            #(#builder_setters)*

            pub fn build(self) -> #struct_name {
                #struct_name {
                    #(#build_struct_fields),*
                }
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl #struct_name {
            pub fn render(&self, data: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) -> String {
                #(#prop_bindings)*
                #fn_body
            }
        }

        #[cfg(target_arch = "wasm32")]
        impl #struct_name {
            pub fn hydrate(&self) -> Box<dyn FnOnce(&std::collections::HashMap<String, apex::web_sys::Text>, &std::collections::HashMap<String, apex::web_sys::Element>)> {
                #(#prop_bindings)*
                let template_fn = #fn_body;
                Box::new(template_fn)
            }
        }
    };

    output
}
