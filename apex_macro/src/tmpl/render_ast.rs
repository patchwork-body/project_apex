use crate::tmpl::{Attribute, TmplAst};
use quote::quote;

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

/// Determines if a text node contains only whitespace
fn is_whitespace_only(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace())
}

/// Determines if an AST node is an element (not text/expression/signal)
fn is_element_node(node: &TmplAst) -> bool {
    matches!(node, TmplAst::Element { .. } | TmplAst::Component { .. })
}

/// Context for text node processing
struct TextContext {
    prev_is_element: bool,
    next_is_element: bool,
    is_first_child: bool,
    is_last_child: bool,
}

/// Process text content based on its context in the AST
fn process_text_content(text: &str, ctx: TextContext) -> Option<String> {
    // Normalize whitespace: replace newlines with spaces and collapse multiple spaces
    let mut normalized = text.replace(['\n', '\r'], " ");

    // Collapse multiple spaces into single spaces
    while normalized.contains("  ") {
        normalized = normalized.replace("  ", " ");
    }

    // If the text is only whitespace and is between elements, skip it
    if is_whitespace_only(&normalized) && ctx.prev_is_element && ctx.next_is_element {
        return None;
    }

    // Trim leading whitespace if previous node is an element or this is the first child
    if ctx.prev_is_element || ctx.is_first_child {
        normalized = normalized.trim_start().to_owned();
    }

    // Trim trailing whitespace if next node is an element or this is the last child
    if ctx.next_is_element || ctx.is_last_child {
        normalized = normalized.trim_end().to_owned();
    }

    // Skip empty text after trimming
    if normalized.is_empty() {
        return None;
    }

    Some(normalized)
}

pub(crate) fn render_ast(content: &[TmplAst]) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();

    // Process content with context awareness
    for (i, item) in content.iter().enumerate() {
        // Determine context for text processing
        let prev_is_element = i > 0 && is_element_node(&content[i - 1]);
        let next_is_element = i + 1 < content.len() && is_element_node(&content[i + 1]);

        match item {
            TmplAst::Text(text) => {
                if let Some(normalized_text) = process_text_content(
                    text,
                    TextContext {
                        prev_is_element,
                        next_is_element,
                        is_first_child: i == 0,
                        is_last_child: i == content.len() - 1,
                    },
                ) {
                    result.push(quote! {
                        {
                            use apex::web_sys::*;

                            let window = window().expect("no global `window` exists");
                            let document = window.document().expect("should have a document on window");
                            let text_node = document.create_text_node(#normalized_text);

                            let _ = element.append_child(&text_node);
                        }
                    });
                }
            }

            TmplAst::Expression(expr) => {
                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
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
                                        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                                            handler_fn(event);
                                        }) as Box<dyn FnMut(web_sys::Event)>);

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

                let child_fns = render_ast(children);

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

                // Collect slot children into a map: slot_name -> Html
                let mut slot_map = std::collections::HashMap::new();
                let mut non_slot_children = Vec::new();

                for child in children {
                    if let TmplAst::Slot { name, children } = child {
                        // Render the slot children into Html
                        let slot_child_fns = render_ast(children);

                        let slot_html = quote! {
                            apex::Html::new(|element| {
                                #(#slot_child_fns)*
                            })
                        };

                        slot_map.insert(name.clone(), slot_html);
                    } else {
                        non_slot_children.push(child.clone());
                    }
                }

                // Generate builder method calls for each attribute
                let mut builder_chain = quote! { #component_name::builder() };

                for (key, value) in attributes {
                    let method_name = syn::Ident::new(key, proc_macro2::Span::call_site());

                    let value_expr = match value {
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
                        Attribute::Signal(signal) => {
                            if let Ok(signal_tokens) = syn::parse_str::<syn::Expr>(signal) {
                                quote! { #signal_tokens }
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
                for (slot_name, slot_html) in &slot_map {
                    let method_name = syn::Ident::new(slot_name, proc_macro2::Span::call_site());
                    builder_chain = quote! { #builder_chain.#method_name(#slot_html) };
                }

                // Generate children Html if non-slot children exist (for default slot)
                if !non_slot_children.is_empty() {
                    let child_fns = render_ast(&non_slot_children);

                    let children_html = quote! {
                        apex::Html::new(|element| {
                            #(#child_fns)*
                        })
                    };

                    builder_chain = quote! { #builder_chain.children(#children_html) };
                }

                // Complete the builder chain with .build()
                let component_instance = quote! { #builder_chain.build() };

                result.push(quote! {
                    {
                        let component_instance = #component_instance;
                        let component_html = component_instance.render();
                        component_html.mount(Some(&element)).expect("Failed to mount component");
                    }
                });
            }
            TmplAst::SlotInterpolation { slot_name } => {
                let slot_name = syn::Ident::new(slot_name, proc_macro2::Span::call_site());

                result.push(quote! {
                    {
                        let slot_html = &#slot_name;
                        slot_html.mount(Some(&element)).expect("Failed to mount slot");
                    }
                });
            }
            TmplAst::Slot {
                name: _,
                children: _,
            } => {
                // Slots are not rendered directly, they are passed to components
            }
        }
    }

    result
}
