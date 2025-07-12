use crate::tmpl::generate_event_listeners::*;
use crate::tmpl::{ComponentAttribute, TmplAst};
use quote::quote;
use syn::Result;

pub(crate) fn render_ast(content: &[TmplAst]) -> Result<Vec<proc_macro2::TokenStream>> {
    let mut result = Vec::new();

    for item in content {
        println!("item: {item:?}");

        match item {
            TmplAst::Text(text) => {
                // Generate code to append text node to the element
                let text_content = text.clone();

                result.push(quote! {
                    {
                        use apex::web_sys::*;
                        let window = window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        let text_node = document.create_text_node(#text_content);

                        let _ = element.append_child(&text_node);
                    }
                });
            }

            TmplAst::Expression(expr) => {
                // Generate code to append expression result as text
                let expr_str = expr.clone();
                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(&expr_str) {
                    result.push(quote! {
                        {
                            use apex::web_sys::*;
                            let window = apex::web_sys::window().expect("no global `window` exists");
                            let document = window.document().expect("should have a document on window");
                            let expr_value = #expr_tokens;
                            let text_node = document.create_text_node(&expr_value.to_string());

                            let _ = element.append_child(&text_node);
                        }
                    });
                }
            }

            TmplAst::Element {
                tag,
                attributes,
                self_closing: _,
                children,
            } => {
                let tag_name = tag.clone();

                let attr_setters = attributes.iter().filter_map(|(k, v)| {
                    match v {
                        ComponentAttribute::Literal(val) => Some(quote! {
                            new_element.set_attribute(#k, #val).expect("Failed to set attribute");
                        }),
                        ComponentAttribute::Expression(expr) => {
                            if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                Some(quote! {
                                    new_element.set_attribute(#k, &{ let v = #expr_tokens; v.to_string() }).expect("Failed to set dynamic attribute");
                                })
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                }).collect::<Vec<_>>();

                let child_fns = render_ast(children)?;

                result.push(quote! {
                    {
                        use apex::web_sys::*;
                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        let new_element = document.create_element(#tag_name).expect("Failed to create element");

                        #(#attr_setters)*

                        let _ = element.append_child(&new_element);
                        let element = &new_element;

                        #(#child_fns)*
                    }
                });
            }

            TmplAst::Component { name, children } => {
                // // Handle custom components - for now, treat as div with class
                // element_counter += 1;
                // let element_id = format!("apex_component_{element_counter}");
                // let component_name = name.clone();

                // result.push(quote! {
                //     {
                //         use apex::web_sys::*;
                //         let window = apex::web_sys::window().expect("no global `window` exists");
                //         let document = window.document().expect("should have a document on window");
                //         let component_element = document.create_element("div").expect("Failed to create component element");
                //         component_element.set_id(#element_id);
                //         let _ = component_element.set_attribute("class", #component_name);
                //         let _ = element.append_child(&component_element);
                //     }
                // });

                // // Process component children
                // if !children.is_empty() {
                //     let child_functions = render_ast(children)?;
                //     for child_fn in child_functions {
                //         result.push(quote! {
                //             {
                //                 use apex::web_sys::*;
                //                 let window = apex::web_sys::window().expect("no global `window` exists");
                //                 let document = window.document().expect("should have a document on window");
                //                 if let Some(component_element) = document.get_element_by_id(#element_id) {
                //                     let element = &component_element;
                //                     #child_fn
                //                 }
                //             }
                //         });
                //     }
                // }
            }

            TmplAst::EventListener(_) => {
                // Event listeners are handled within elements
                // This case shouldn't occur in normal parsing
            }
        }
    }

    println!("result: {result:?}");

    Ok(result)
}
