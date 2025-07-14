use crate::tmpl::{ComponentAttribute, TmplAst};
use quote::quote;
use syn::Result;

fn find_signals(expr: &str) -> Vec<String> {
    let mut signals = Vec::new();
    let mut chars = expr.chars();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut signal = String::new();

            for ch in chars.by_ref() {
                if ch.is_alphanumeric() || ch == '_' {
                    signal.push(ch);
                } else {
                    break;
                }
            }

            signals.push(signal);
        }
    }

    signals
        .into_iter()
        .filter(|signal| !signal.is_empty())
        .collect()
}

pub(crate) fn render_ast(content: &[TmplAst]) -> Result<Vec<proc_macro2::TokenStream>> {
    let mut result = Vec::new();

    for item in content {
        match item {
            TmplAst::Text(text) => {
                // Skip whitespace-only text nodes
                if text.trim().is_empty() {
                    continue;
                }

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
                        ComponentAttribute::Signal(expr) => {
                            let signals = find_signals(expr);

                            let signal_clones = signals
                                .iter()
                                .map(|signal| {
                                    let signal_ident = syn::Ident::new(signal, proc_macro2::Span::call_site());
                                    let clone_ident = syn::Ident::new(
                                        &format!("{signal}_clone"),
                                        proc_macro2::Span::call_site(),
                                    );
                                    quote! {
                                        let #clone_ident = #signal_ident.clone();
                                    }
                                })
                                .collect::<Vec<_>>();

                            let mut processed_expr = expr.clone();

                            for signal in &signals {
                                let signal_pattern = format!("${signal}");
                                let replacement = format!("{signal}_clone.get()");
                                processed_expr = processed_expr.replace(&signal_pattern, &replacement);
                            }

                            if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(&processed_expr) {
                                Some(quote! {
                                    {
                                        #(#signal_clones)*
                                        let attr_value = #expr_tokens;
                                        new_element.set_attribute(#k, &attr_value.to_string()).expect("Failed to set signal attribute");
                                    }

                                    {
                                        let element_clone = new_element.clone();
                                        let attr_name = #k;
                                        #(#signal_clones)*

                                        apex::effect!({
                                            let attr_value = #expr_tokens;
                                            let _ = element_clone.set_attribute(attr_name, &attr_value.to_string());
                                        });
                                    }
                                })
                            } else {
                                None
                            }
                        },
                        ComponentAttribute::EventListener(_) => None,
                    }
                }).collect::<Vec<_>>();

                let event_listeners = attributes.iter().filter_map(|(k, v)| {
                    match v {
                        ComponentAttribute::EventListener(handler) => {
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
                                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                                            handler_fn();
                                        }) as Box<dyn FnMut(_)>);

                                        let _ = new_element.add_event_listener_with_callback(
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

                let child_fns = render_ast(children)?;

                result.push(quote! {
                    {
                        use apex::web_sys::*;

                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        let new_element = document.create_element(#tag_name).expect("Failed to create element");

                        #(#attr_setters)*

                        let _ = element.append_child(&new_element);

                        #(#event_listeners)*

                        {
                            let element = &new_element;
                            #(#child_fns)*
                        }
                    }
                });
            }

            TmplAst::Signal(expr) => {
                // Remove $ prefixes from signal names and trim whitespace
                // let processed_expr = expr.replace("$", "").trim().to_string();

                // Collect all signals names from the expression, all signals should be marked with $ prefix, expression can contain multiple signals and other literals
                let signals = find_signals(expr);

                // Generate cloning statements for each signal
                let signal_clones = signals
                    .iter()
                    .map(|signal| {
                        let signal_ident = syn::Ident::new(signal, proc_macro2::Span::call_site());
                        let clone_ident = syn::Ident::new(
                            &format!("{signal}_clone"),
                            proc_macro2::Span::call_site(),
                        );
                        quote! {
                            let #clone_ident = #signal_ident.clone();
                        }
                    })
                    .collect::<Vec<_>>();

                // Replace $signal_name with signal_name_clone.get()
                let mut processed_expr = expr.clone();

                for signal in &signals {
                    let signal_pattern = format!("${signal}");
                    let replacement = format!("{signal}_clone.get()");

                    processed_expr = processed_expr.replace(&signal_pattern, &replacement);
                }

                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(&processed_expr) {
                    result.push(quote! {
                        use apex::web_sys::*;
                        use apex::*;

                        let text_node = window().expect("no global `window` exists")
                            .document().expect("should have a document on window")
                            .create_text_node("");

                        let _ = element.append_child(&text_node);

                        #(#signal_clones)*

                        // Create initial text node with current value
                        let expression_value = #expr_tokens;
                        let text_node = document.create_text_node(&expression_value.to_string());
                        let _ = element.append_child(&text_node);

                        // Clone text node for the effect
                        let text_node_clone = text_node.clone();

                        // Set up reactive effect for updates
                        apex::effect!({
                            let expression_value = #expr_tokens;
                            text_node_clone.set_data(&expression_value.to_string());
                        });
                    });
                } else {
                    // Fallback for invalid expressions
                    result.push(quote! {
                        use apex::web_sys::*;

                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        let text_node = document.create_text_node("");
                        let _ = element.append_child(&text_node);
                    });
                }
            }

            TmplAst::Component { name, children } => {
                let component_name = syn::Ident::new(name, proc_macro2::Span::call_site());

                if children.is_empty() {
                    // Component without children - use the original signature
                    result.push(quote! {
                        {
                            let component_instance = #component_name;
                            let component_html = #component_name::render(&component_instance);

                            component_html.mount(Some(&element));
                        }
                    });
                } else {
                    // Component with children - create a single Html object for children
                    // Special handling for text content - trim whitespace
                    let mut processed_children = Vec::new();

                    // TODO: Review more closely
                    for child in children {
                        match child {
                            TmplAst::Text(text) => {
                                let trimmed_text = text.trim();
                                if !trimmed_text.is_empty() {
                                    processed_children.push(TmplAst::Text(trimmed_text.to_owned()));
                                }
                            }
                            _ => {
                                processed_children.push(child.clone());
                            }
                        }
                    }

                    let child_fns = render_ast(&processed_children)?;

                    result.push(quote! {
                        {
                            let component_instance = #component_name;

                            // Create children Html object - render children directly
                            let children_html = apex::Html::new(|children_element| {
                                let element = children_element;
                                #(#child_fns)*
                            });

                            let component_html = #component_name::render(&component_instance, children_html);

                            component_html.mount(Some(&element));
                        }
                    });
                }
            }
        }
    }

    Ok(result)
}
