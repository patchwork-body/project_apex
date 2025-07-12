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
                // element_counter += 1;
                // let element_id = format!("apex_element_{element_counter}");
                // let tag_name = tag.clone();

                // // Collect attribute setting code
                // let mut attr_code = Vec::new();
                // for (attr_name, attr_value) in attributes {
                //     if !attr_name.starts_with("on") {
                //         match attr_value {
                //             ComponentAttribute::Literal(value) => {
                //                 let attr_name_str = attr_name.clone();
                //                 let attr_value_str = value.clone();
                //                 attr_code.push(quote! {
                //                     let _ = new_element.set_attribute(#attr_name_str, #attr_value_str);
                //                 });
                //             }
                //             ComponentAttribute::Expression(expr) => {
                //                 if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                //                     let attr_name_str = attr_name.clone();
                //                     attr_code.push(quote! {
                //                         let expr_value = #expr_tokens;
                //                         let _ = new_element.set_attribute(#attr_name_str, &expr_value.to_string());
                //                     });
                //                 }
                //             }
                //             ComponentAttribute::EventHandler(_) => {} // Skip event handlers here
                //         }
                //     }
                // }

                // // Generate complete element creation code
                // let element_creation = quote! {
                //     {
                //         use apex::web_sys::*;
                //         let window = apex::web_sys::window().expect("no global `window` exists");
                //         let document = window.document().expect("should have a document on window");
                //         let new_element = document.create_element(#tag_name).expect("Failed to create element");
                //         new_element.set_id(#element_id);
                //         #(#attr_code)*
                //         let _ = element.append_child(&new_element);
                //     }
                // };

                // result.push(element_creation);

                // // Generate event listeners
                // let event_listeners = generate_event_listeners(&element_id, attributes)?;
                // for listener in event_listeners {
                //     result.push(listener);
                // }

                // // Process children
                // if !children.is_empty() {
                //     let child_functions = render_ast(children)?;
                //     for child_fn in child_functions {
                //         result.push(quote! {
                //             {
                //                 use apex::web_sys::*;
                //                 let window = apex::web_sys::window().expect("no global `window` exists");
                //                 let document = window.document().expect("should have a document on window");
                //                 if let Some(child_element) = document.get_element_by_id(#element_id) {
                //                     let element = &child_element;
                //                     #child_fn
                //                 }
                //             }
                //         });
                //     }
                // }
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
