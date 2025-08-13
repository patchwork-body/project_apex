use std::collections::HashSet;

use crate::tmpl::{Attribute, TmplAst};
use quote::quote;
use syn::{Ident, visit::Visit};

struct IdentifierVisitor {
    identifiers: Vec<Ident>,
    seen: HashSet<String>,
}

impl IdentifierVisitor {
    fn new() -> Self {
        Self {
            identifiers: Vec::new(),
            seen: HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for IdentifierVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(ident) = path.get_ident() {
            let ident_str = ident.to_string();

            if self.seen.insert(ident_str) {
                self.identifiers.push(ident.clone());
            }
        }

        // Continue visiting nested paths
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_call(&mut self, method_call: &'ast syn::ExprCall) {
        // Visit the receiver (the object the method is called on)
        self.visit_expr(&method_call.func);

        // Don't visit method name itself, just the receiver and args
        for arg in &method_call.args {
            self.visit_expr(arg);
        }
    }
}

pub(crate) fn render_ast(
    content: &[TmplAst],
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut instructions = Vec::new();
    let mut expressions = Vec::new();

    for ast in content {
        match ast {
            TmplAst::Text(text) => {
                instructions.push(quote! {
                    buffer.push_str(#text);
                });
            }
            TmplAst::Expression(expr) => {
                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                    let mut visitor = IdentifierVisitor::new();
                    visitor.visit_expr(&expr_tokens);

                    let vars = visitor.identifiers;
                    let text_node_counter = quote! { apex::apex_utils::next_text_node_counter() };

                    expressions.push(quote! {
                        {
                            #(let #vars = #vars.clone();)*
                            let text_node_counter = #text_node_counter;
                            let text_node = expressions_map.get(&text_node_counter.to_string()).expect("text node not found").clone();

                            apex::effect!({
                                text_node.set_data(&(#expr_tokens).to_string());
                            });
                        }
                    });

                    instructions.push(quote! {
                        {
                            let text_node_counter = #text_node_counter;

                            buffer.push_str("<!-- @expr-text-begin:");
                            buffer.push_str(&text_node_counter.to_string());
                            buffer.push_str(" -->");
                            buffer.push_str(&(#expr_tokens).to_string());
                            buffer.push_str("<!-- @expr-text-end:");
                            buffer.push_str(&text_node_counter.to_string());
                            buffer.push_str(" -->");
                        }
                    });
                }
            }
            TmplAst::Element {
                tag,
                attributes,
                is_component,
                self_closing,
                children,
            } => {
                if *is_component {
                    let component_name = syn::Ident::new(tag, proc_macro2::Span::call_site());

                    // Collect slot children into a map: slot_name -> Html
                    // let mut slot_map = std::collections::HashMap::new();
                    // let mut non_slot_children = Vec::new();

                    // for child in children {
                    //     if let TmplAst::Slot { name, children } = child {
                    //         // Render the slot children into Html
                    //         let slot_child_fns = render_ast(children);

                    //         let slot_html = quote! {
                    //             apex::Html::new(|element| {
                    //                 #(#slot_child_fns)*
                    //             })
                    //         };

                    //         slot_map.insert(name.clone(), slot_html);
                    //     } else {
                    //         non_slot_children.push(child.clone());
                    //     }
                    // }

                    // Generate builder method calls for each attribute
                    let mut builder_chain = quote! { #component_name::builder() };

                    for (key, value) in attributes {
                        let method_name = syn::Ident::new(key, proc_macro2::Span::call_site());

                        let value_expr = match value {
                            Attribute::Empty => continue,
                            Attribute::Literal(literal) => {
                                quote! {
                                    #literal.into()
                                }
                            }
                            Attribute::Expression(expr) => {
                                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                    quote! { #expr_tokens }
                                } else {
                                    continue;
                                }
                            }
                            Attribute::EventListener(handler) => {
                                if let Ok(handler_tokens) = syn::parse_str::<syn::Expr>(handler) {
                                    quote! { #handler_tokens }
                                } else {
                                    continue;
                                }
                            }
                        };

                        builder_chain = quote! { #builder_chain.#method_name(#value_expr) };
                    }

                    // Add builder calls for slots
                    // for (slot_name, slot_html) in &slot_map {
                    //     let method_name =
                    //         syn::Ident::new(slot_name, proc_macro2::Span::call_site());
                    //     builder_chain = quote! { #builder_chain.#method_name(#slot_html) };
                    // }

                    // Generate children Html if non-slot children exist (for default slot)
                    // if !non_slot_children.is_empty() {
                    //     let child_fns = render_ast(&non_slot_children);

                    //     let children_html = quote! {
                    //         apex::Html::new(|element| {
                    //             #(#child_fns)*
                    //         })
                    //     };

                    //     builder_chain = quote! { #builder_chain.children(#children_html) };
                    // }

                    // Complete the builder chain with .build()

                    instructions.push(quote! {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let component_instance = #builder_chain.build();
                            let component_html = component_instance.render();

                            buffer.push_str(&component_html);
                        }
                    });

                    expressions.push(quote! {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let component_instance = #builder_chain.build();
                            let hydrate = component_instance.hydrate();

                            hydrate(expressions_map, elements_map)
                        }
                    });
                } else {
                    let tag_name = tag.clone();
                    let element_counter = quote! { apex::apex_utils::next_element_counter() };

                    let comment_element = if attributes.iter().any(|(_, v)| {
                        matches!(v, Attribute::EventListener(_) | Attribute::Expression(_))
                    }) {
                        quote! {
                            {
                                buffer.push_str("<!-- @element:");
                                buffer.push_str(&element_counter.to_string());
                                buffer.push_str(" -->")
                            }
                        }
                    } else {
                        quote! {}
                    };

                    // Sort attributes for consistent ordering in tests
                    let mut sorted_attributes: Vec<_> = attributes.iter().collect();
                    sorted_attributes.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

                    let attr_setters = sorted_attributes
                        .iter()
                        .filter_map(|(k, v)| match v {
                            Attribute::Empty | Attribute::EventListener(_) => None,
                            Attribute::Literal(val) => Some(quote! {
                                buffer.push_str(&(#k));
                                buffer.push_str("=\"");
                                buffer.push_str(&(#val));
                                buffer.push_str("\"");
                            }),
                            Attribute::Expression(expr) => {
                                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                    Some(quote! {
                                        buffer.push_str(&(#k));
                                        buffer.push_str("=\"");
                                        buffer.push_str(&(#expr_tokens).to_string());
                                        buffer.push_str("\"");
                                    })
                                } else {
                                    None
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    expressions.push(quote! {
                        let element_counter = #element_counter;
                    });

                    let attr_setters_expressions = sorted_attributes
                        .iter()
                        .filter_map(|(k, v)| match v {
                            Attribute::Expression(expr) => {
                                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                    let mut visitor = IdentifierVisitor::new();
                                    visitor.visit_expr(&expr_tokens);

                                    let vars = visitor.identifiers;

                                    Some(quote! {
                                        {
                                            #(let #vars = #vars.clone();)*
                                            let element = elements_map.get(&element_counter.to_string()).expect("element not found").clone();

                                            apex::effect!({
                                                element.set_attribute(#k, &(#expr_tokens).to_string());
                                            });
                                        }
                                    })
                                } else {
                                    None
                                }
                            },
                            _ => None,
                        })
                        .collect::<Vec<_>>();

                    expressions.extend(attr_setters_expressions);

                    let event_listeners = sorted_attributes.iter().filter_map(|(k, v)| {
                        match v {
                            Attribute::EventListener(handler) => {
                                // Extract event name from attribute (e.g., "onclick" -> "click")
                                let event_name = if k.starts_with("on") && k.len() > 2 {
                                    &k[2..] // Remove "on" prefix
                                } else {
                                    return None; // Skip invalid event names
                                };

                                if let Ok(handler_tokens) = syn::parse_str::<syn::Expr>(handler) {
                                    Some(quote! {
                                        {
                                            use apex::wasm_bindgen::prelude::*;
                                            use apex::web_sys::*;

                                            let handler_fn = (#handler_tokens).clone();
                                            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                                                handler_fn(event);
                                            }) as Box<dyn FnMut(web_sys::Event)>);

                                            let element = elements_map.get(&element_counter.to_string()).expect("element not found").clone();

                                            let _ = element.add_event_listener_with_callback(
                                                #event_name,
                                                closure.as_ref().unchecked_ref()
                                            );

                                            closure.forget(); // Prevent cleanup
                                        }
                                    })
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }).collect::<Vec<_>>();

                    expressions.extend(event_listeners);

                    let (children_instructions, children_expressions) = render_ast(children);

                    expressions.extend(children_expressions);

                    let open_tag = if attributes.is_empty() {
                        format!("<{tag_name}")
                    } else {
                        format!("<{tag_name} ")
                    };

                    if *self_closing {
                        instructions.push(quote! {
                            let element_counter = #element_counter;
                            #comment_element
                            buffer.push_str(&(#open_tag));
                            #(#attr_setters)*
                            buffer.push_str("/>");
                        });
                    } else {
                        let close_tag = format!("</{tag_name}>");

                        instructions.push(quote! {
                            let element_counter = #element_counter;
                            #comment_element
                            buffer.push_str(&(#open_tag));
                            #(#attr_setters)*
                            buffer.push_str(">");
                            #(#children_instructions)*
                            buffer.push_str(&(#close_tag));
                        });
                    }
                }
            }
            TmplAst::Outlet => {
                // Generate code to call the outlet helper function
                instructions.push(quote! {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Server-side: outlet content will be handled by the router
                        // For now, add a placeholder comment
                        buffer.push_str("<!-- @outlet -->");
                    }
                });

                expressions.push(quote! {
                    #[cfg(target_arch = "wasm32")]
                    {
                        // Client-side: outlet placeholder for future client-side routing
                        // This will be handled by the client-side router
                    }
                });
            }
            _ => {}
        }
    }

    (instructions, expressions)
}
