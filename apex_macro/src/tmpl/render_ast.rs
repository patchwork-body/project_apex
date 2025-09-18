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

pub(crate) fn collect_variables_from_ast(ast_nodes: &[TmplAst]) -> Vec<Ident> {
    let mut visitor = IdentifierVisitor::new();

    fn visit_ast_node(node: &TmplAst, visitor: &mut IdentifierVisitor) {
        match node {
            TmplAst::Expression(expr) => {
                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                    visitor.visit_expr(&expr_tokens);
                }
            }
            TmplAst::Element {
                children,
                attributes,
                ..
            } => {
                for child in children {
                    visit_ast_node(child, visitor);
                }

                for attr in attributes.values() {
                    match attr {
                        Attribute::Expression(expr) | Attribute::EventListener(expr) => {
                            if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                visitor.visit_expr(&expr_tokens);
                            }
                        }
                        _ => {}
                    }
                }
            }
            TmplAst::SlotInterpolation {
                default_children, ..
            } => {
                if let Some(children) = default_children {
                    for child in children {
                        visit_ast_node(child, visitor);
                    }
                }
            }
            TmplAst::Slot { children, .. } => {
                for child in children {
                    visit_ast_node(child, visitor);
                }
            }
            TmplAst::ConditionalDirective(if_blocks) => {
                for if_block in if_blocks {
                    for child in &if_block.children {
                        visit_ast_node(child, visitor);
                    }
                }
            }
            TmplAst::Text(_) | TmplAst::Outlet => {}
        }
    }

    for node in ast_nodes {
        visit_ast_node(node, &mut visitor);
    }

    visitor.identifiers
}

impl<'ast> Visit<'ast> for IdentifierVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(ident) = path.get_ident() {
            let ident_str = ident.to_string();

            // Skip well-known macros and builtins that shouldn't be captured/cloned
            // This prevents generating `let format = format.clone();` which breaks `format!` macro calls.
            const SKIP: &[&str] = &[
                "format",
                "vec",
                "println",
                "eprintln",
                "write",
                "writeln",
                "format_args",
            ];
            if SKIP.contains(&ident_str.as_str()) {
                return;
            }

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

fn trim_whitespace_around_slots(content: &[TmplAst]) -> Vec<TmplAst> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < content.len() {
        match &content[i] {
            TmplAst::SlotInterpolation { .. } => {
                // Trim whitespace before the slot
                if let Some(TmplAst::Text(text)) = result.last_mut() {
                    *text = text.trim_end().to_owned();
                    if text.is_empty() {
                        result.pop();
                    }
                }

                // Add the slot
                result.push(content[i].clone());

                // Skip whitespace after the slot
                if i + 1 < content.len() {
                    if let TmplAst::Text(text) = &content[i + 1] {
                        if text.trim().is_empty() {
                            i += 1; // Skip the whitespace text node
                        } else {
                            // Add the text node with leading whitespace trimmed
                            i += 1;
                            result.push(TmplAst::Text(text.trim_start().to_owned()));
                        }
                    }
                }
            }
            _ => {
                result.push(content[i].clone());
            }
        }
        i += 1;
    }

    result
}

pub(crate) fn render_ast(
    content: &[TmplAst],
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut instructions = Vec::new();
    let mut expressions = Vec::new();

    // Trim whitespace around slot interpolations
    let content = trim_whitespace_around_slots(content);

    for ast in &content {
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
                            if let Some(text_node) = expressions_map.get(&text_node_counter.to_string()).cloned() {

                                apex::effect!({
                                    text_node.set_data(&(#expr_tokens).to_string());
                                });
                            }
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

                    // Generate builder method calls for each attribute
                    let mut builder_chain = quote! { #component_name::builder() };

                    for (key, value) in attributes {
                        let method_name = syn::Ident::new(key, proc_macro2::Span::call_site());

                        builder_chain = match value {
                            Attribute::Empty => continue,
                            Attribute::Literal(literal) => {
                                quote! {
                                    #builder_chain.#method_name(#literal.into())
                                }
                            }
                            Attribute::Expression(expr) => {
                                if let Ok(expr_tokens) = syn::parse_str::<syn::Expr>(expr) {
                                    quote! {
                                        #builder_chain.#method_name(#expr_tokens)
                                    }
                                } else {
                                    continue;
                                }
                            }
                            Attribute::EventListener(handler) => {
                                if let Ok(handler_tokens) = syn::parse_str::<syn::Expr>(handler) {
                                    quote! {
                                        #builder_chain.#method_name(#handler_tokens)
                                    }
                                } else {
                                    continue;
                                }
                            }
                        };
                    }

                    let mut render_slots_map = quote! { std::collections::HashMap::new() };
                    let mut hydrate_slots_map = quote! { std::collections::HashMap::new() };
                    let mut regular_children = Vec::new();
                    let mut all_slot_expressions = Vec::new();

                    for child in children {
                        if let TmplAst::Slot {
                            name: Some(slot_name),
                            children: slot_children,
                        } = child
                        {
                            let (slot_instructions, slot_expressions) = render_ast(slot_children);
                            let slot_vars = collect_variables_from_ast(slot_children);

                            if slot_vars.is_empty() {
                                // No variables to capture, create simple closures
                                render_slots_map = quote! {
                                    {
                                        let mut map = #render_slots_map;

                                        map.insert(#slot_name.to_string(), std::rc::Rc::new(Box::new(move |buffer: &mut String, data: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>| {
                                            #(#slot_instructions)*
                                        }) as Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>));

                                        map
                                    }
                                };
                                // Hydration closures for named slots
                                hydrate_slots_map = quote! {
                                    {
                                        let mut map = #hydrate_slots_map;

                                        map.insert(#slot_name.to_string(), std::rc::Rc::new(Box::new(move |
                                            expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                                            elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
                                        | {
                                            #(#slot_expressions)*
                                        }) as Box<dyn Fn(&std::collections::HashMap<String, apex::web_sys::Text>, &std::collections::HashMap<String, apex::web_sys::Element>) + 'static>));

                                        map
                                    }
                                };
                            } else {
                                // Variables need to be captured - each slot gets its own closure with cloned variables
                                render_slots_map = quote! {
                                    {
                                        let mut map = #render_slots_map;

                                        map.insert(#slot_name.to_string(), std::rc::Rc::new(Box::new({
                                            // Clone variables for this specific slot closure
                                            #(let #slot_vars = #slot_vars.clone();)*

                                            move |buffer: &mut String, data: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>| {
                                                #(#slot_instructions)*
                                            }
                                        }) as Box<dyn for<'a> Fn(&'a mut String, std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>) + 'static>));

                                        map
                                    }
                                };

                                // Hydration closures for named slots with captured vars
                                hydrate_slots_map = quote! {
                                    {
                                        let mut map = #hydrate_slots_map;

                                        map.insert(#slot_name.to_string(), std::rc::Rc::new(Box::new({
                                            #(let #slot_vars = #slot_vars.clone();)*

                                            move |
                                                expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                                                elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
                                            | {
                                                #(#slot_expressions)*
                                            }
                                        }) as Box<dyn Fn(&std::collections::HashMap<String, apex::web_sys::Text>, &std::collections::HashMap<String, apex::web_sys::Element>) + 'static>));

                                        map
                                    }
                                };
                            }

                            all_slot_expressions.extend(slot_expressions);
                        } else {
                            regular_children.push(child.clone());
                        }
                    }

                    // Handle regular children (unnamed slot)
                    let (children_instructions, children_expressions) =
                        render_ast(&regular_children);

                    if !regular_children.is_empty() {
                        let children_vars = collect_variables_from_ast(&regular_children);

                        if children_vars.is_empty() {
                            // No variables to capture, create a simple closure
                            builder_chain = quote! {
                                #builder_chain
                                    .render_children(Box::new(move |buffer: &mut String, data: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>| {
                                        #(#children_instructions)*
                                    }))
                                    .hydrate_children(Box::new(move |
                                        expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                                        elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
                                    | {
                                        #(#children_expressions)*
                                    }))
                            };
                        } else {
                            // Variables need to be captured
                            builder_chain = quote! {
                                #builder_chain
                                    .render_children(Box::new({
                                        #(let #children_vars = #children_vars.clone();)*

                                        move |buffer: &mut String, data: std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, serde_json::Value>>>| {
                                            #(#children_instructions)*
                                        }
                                    }))
                                    .hydrate_children(Box::new({
                                        #(let #children_vars = #children_vars.clone();)*

                                        move |
                                            expressions_map: &std::collections::HashMap<String, apex::web_sys::Text>,
                                            elements_map: &std::collections::HashMap<String, apex::web_sys::Element>
                                        | {
                                            #(#children_expressions)*
                                        }
                                    }))
                            };
                        }
                    }

                    builder_chain = quote! {
                        #builder_chain.named_slots(#render_slots_map).hydrate_named_slots(#hydrate_slots_map)
                    };

                    instructions.push(quote! {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let component_instance = #builder_chain.build();
                            // Data is passed from the route, so it might not exist if this component is rendered not in the route
                            let component_html = component_instance.render(data.clone());

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
                                                if let Some(element) = elements_map.get(&element_counter.to_string()).cloned() {
                                                    apex::effect!({
                                                        element.set_attribute(#k, &(#expr_tokens).to_string());
                                                    });
                                            } else {
                                                apex::web_sys::console::warn_1(&format!("Warning: element {} not found during hydration", element_counter.to_string()).into());
                                            }
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

                    // Extract event handlers for server-side to prevent unused warnings
                    // This only runs at compile time and generates no runtime overhead
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let event_handler_usages = sorted_attributes
                            .iter()
                            .filter_map(|(_k, v)| {
                                match v {
                                    Attribute::EventListener(handler) => {
                                        if let Ok(handler_tokens) =
                                            syn::parse_str::<syn::Expr>(handler)
                                        {
                                            // Extract identifiers to create a usage in server-side code
                                            let mut visitor = IdentifierVisitor::new();
                                            visitor.visit_expr(&handler_tokens);
                                            let vars = visitor.identifiers;

                                            if !vars.is_empty() {
                                                Some(quote! {
                                                    // Create a usage of event handler variables to prevent unused warnings
                                                    // This is a no-op that will be optimized away by LLVM
                                                    // It generates zero machine code in release builds
                                                    #[cfg(not(target_arch = "wasm32"))]
                                                    {
                                                        #(let _ = &#vars;)*
                                                    }
                                                })
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                }
                            })
                            .collect::<Vec<_>>();

                        // Add event handler usages to instructions (server-side)
                        instructions.extend(event_handler_usages);
                    }

                    let event_listeners = sorted_attributes.iter().filter_map(|(k, v)| {
                        match v {
                            Attribute::EventListener(handler) => {
                                // Extract event name from attribute (e.g., "onclick" -> "click")
                                let event_name = if k.starts_with("on") && k.len() > 2 {
                                    &k[2..] // Remove "on" prefix
                                } else {
                                    return None; // Skip invalid event names
                                };

                                let event_type: syn::Type = match event_name {
                                    "click" | "mousedown" | "mouseup" => {
                                        syn::parse_str("apex::web_sys::MouseEvent").unwrap()
                                    },
                                    _ => {
                                        syn::parse_str("apex::web_sys::Event").unwrap()
                                    }
                                };

                                if let Ok(handler_tokens) = syn::parse_str::<syn::Expr>(handler) {
                                    Some(quote! {
                                        {
                                            use apex::wasm_bindgen::prelude::*;
                                            use apex::web_sys::*;

                                            let handler_fn = (#handler_tokens).clone();
                                            let closure = Closure::wrap(Box::new(move |event: #event_type| {
                                                handler_fn(event);
                                            }) as Box<dyn FnMut(#event_type)>);

                                            if let Some(element) = elements_map.get(&element_counter.to_string()).cloned() {
                                                let _ = element.add_event_listener_with_callback(
                                                    #event_name,
                                                    closure.as_ref().unchecked_ref()
                                                );

                                                closure.forget(); // Prevent cleanup
                                            }
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
            TmplAst::SlotInterpolation {
                slot_name,
                default_children,
            } => {
                if let Some(slot_name) = slot_name {
                    // Handle named slots
                    if let Some(default_children) = default_children {
                        let (default_instructions, default_expressions) =
                            render_ast(default_children);

                        instructions.push(quote! {
                            if let Some(named_slots) = &named_slots {
                                if let Some(render_slot) = named_slots.get(#slot_name) {
                                    render_slot(&mut buffer, data.clone());
                                } else {
                                    // Render default children
                                    #(#default_instructions)*
                                }
                            } else {
                                // Render default children
                                #(#default_instructions)*
                            }
                        });
                        expressions.push(quote! {
                            if let Some(hydrate_named_slots) = &hydrate_named_slots {
                                if let Some(hydrate_slot) = hydrate_named_slots.get(#slot_name) {
                                    hydrate_slot(expressions_map, elements_map);
                                } else {
                                    #(#default_expressions)*
                                }
                            } else {
                                #(#default_expressions)*
                            }
                        });
                    } else {
                        instructions.push(quote! {
                            if let Some(named_slots) = &named_slots {
                                if let Some(render_slot) = named_slots.get(#slot_name) {
                                    render_slot(&mut buffer, data.clone());
                                }
                            }
                        });
                        expressions.push(quote! {
                            if let Some(hydrate_named_slots) = &hydrate_named_slots {
                                if let Some(hydrate_slot) = hydrate_named_slots.get(#slot_name) {
                                    hydrate_slot(expressions_map, elements_map);
                                }
                            }
                        });
                    }
                } else if let Some(default_children) = default_children {
                    let (default_instructions, default_expressions) = render_ast(default_children);

                    instructions.push(quote! {
                        if let Some(render_children) = render_children.clone() {
                            render_children(&mut buffer, data.clone());
                        } else {
                            #(#default_instructions)*
                        }
                    });

                    expressions.push(quote! {
                        if let Some(hydrate_children) = &hydrate_children {
                            hydrate_children(expressions_map, elements_map);
                        } else {
                            #(#default_expressions)*
                        }
                    });
                } else {
                    instructions.push(quote! {
                        if let Some(render_children) = render_children.clone() {
                            render_children(&mut buffer, data.clone());
                        }
                    });
                    expressions.push(quote! {
                        if let Some(hydrate_children) = &hydrate_children {
                            hydrate_children(expressions_map, elements_map);
                        }
                    });
                }
            }
            TmplAst::Outlet => {
                instructions.push(quote! {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        buffer.push_str("<!-- @outlet-begin --><!-- @outlet-end -->");
                    }
                });
            }
            _ => {}
        }
    }

    (instructions, expressions)
}
