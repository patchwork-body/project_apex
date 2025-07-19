use crate::tmpl::{Attribute, TmplAst};
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
                // Remove newlines and normalize whitespace, but preserve meaningful spaces
                let mut processed_text = text.replace(['\n', '\r'], " ");

                // Replace multiple spaces with single space
                while processed_text.contains("  ") {
                    processed_text = processed_text.replace("  ", " ");
                }

                // Only skip if the text is purely whitespace
                if processed_text.trim().is_empty() {
                    continue;
                }

                // Only trim if the text consists of more than just a few spaces and contains substantial content
                // This handles cases like "\n    Hello, world!\n    " but preserves " + " and " ! "
                let trimmed = processed_text.trim();
                let text = if trimmed.len() > 5 && processed_text.len() > trimmed.len() + 1 {
                    // Only trim if we have substantial content (>5 chars) and any whitespace difference
                    trimmed.to_owned()
                } else {
                    processed_text
                };

                if text.is_empty() {
                    continue;
                }

                result.push(quote! {
                    {
                        use apex::web_sys::*;

                        let window = window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        let text_node = document.create_text_node(#text);

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
                        Attribute::Literal(val) => Some(quote! {
                            new_element.set_attribute(#k, #val).expect("Failed to set attribute");
                        }),
                        Attribute::Expression(expr) => {
                            if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                Some(quote! {
                                    new_element.set_attribute(#k, &{ let v = #expr_tokens; v.to_string() }).expect("Failed to set dynamic attribute");
                                })
                            } else {
                                None
                            }
                        },
                        Attribute::Signal(expr) => {
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
                        Attribute::EventListener(_) => None,
                    }
                }).collect::<Vec<_>>();

                let event_listeners = attributes.iter().filter_map(|(k, v)| {
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

                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");

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

            TmplAst::Component {
                name,
                attributes,
                children,
            } => {
                let component_name = syn::Ident::new(name, proc_macro2::Span::call_site());

                if children.is_empty() && attributes.is_empty() {
                    // Component without children or attributes - use the original signature
                    result.push(quote! {
                        {
                            let component_instance = #component_name;
                            let component_html = #component_name::render(&component_instance);

                            component_html.mount(Some(&element));
                        }
                    });
                } else {
                    // Component with attributes and/or children

                    // Generate attributes struct if attributes exist
                    let attrs_code = if !attributes.is_empty() {
                        let mut attr_fields = Vec::new();
                        for (key, value) in attributes {
                            let field_name = syn::Ident::new(key, proc_macro2::Span::call_site());
                            match value {
                                Attribute::Literal(literal) => {
                                    attr_fields.push(quote! {
                                        #field_name: #literal.to_string()
                                    });
                                }
                                Attribute::Expression(expr) => {
                                    let expr_tokens: proc_macro2::TokenStream =
                                        expr.parse().unwrap();
                                    attr_fields.push(quote! {
                                        #field_name: #expr_tokens.to_string()
                                    });
                                }
                                Attribute::Signal(signal) => {
                                    let signal_tokens: proc_macro2::TokenStream =
                                        signal.parse().unwrap();
                                    attr_fields.push(quote! {
                                        #field_name: #signal_tokens.to_string()
                                    });
                                }
                                Attribute::EventListener(_) => {}
                            }
                        }

                        quote! {
                            let attrs = Attrs {
                                #(#attr_fields),*
                            };
                        }
                    } else {
                        quote! {}
                    };

                    // Generate children Html if children exist
                    let children_code = if !children.is_empty() {
                        // Special handling for text content - trim whitespace
                        let mut processed_children = Vec::new();

                        for child in children {
                            match child {
                                TmplAst::Text(text) => {
                                    let trimmed_text = text.trim();
                                    if !trimmed_text.is_empty() {
                                        processed_children
                                            .push(TmplAst::Text(trimmed_text.to_owned()));
                                    }
                                }
                                _ => {
                                    processed_children.push(child.clone());
                                }
                            }
                        }

                        let child_fns = render_ast(&processed_children)?;

                        quote! {
                            let children_html = apex::Html::new(|element| {
                                #(#child_fns)*
                            });
                        }
                    } else {
                        quote! {}
                    };

                    // Generate the render call based on what we have
                    let render_call = match (!attributes.is_empty(), !children.is_empty()) {
                        (true, true) => quote! {
                            let component_html = #component_name::render(&component_instance, attrs, children_html);
                        },
                        (true, false) => quote! {
                            let component_html = #component_name::render(&component_instance, attrs);
                        },
                        (false, true) => quote! {
                            let component_html = #component_name::render(&component_instance, children_html);
                        },
                        (false, false) => quote! {
                            let component_html = #component_name::render(&component_instance);
                        },
                    };

                    result.push(quote! {
                        {
                            let component_instance = #component_name;

                            #attrs_code
                            #children_code
                            #render_call

                            component_html.mount(Some(&element));
                        }
                    });
                }
            }
        }
    }

    Ok(result)
}
