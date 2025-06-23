use crate::tmpl::ComponentAttribute;
use quote::quote;
use syn::Result;

/// Generate event listener registration code for HTML elements
///
/// This function generates code that registers event listeners for HTML elements
/// using web_sys. Event handlers are extracted from attributes and registered
/// after the HTML is rendered.
///
/// # Arguments
/// * `element_id` - A unique identifier for the HTML element
/// * `attributes` - HashMap containing all attributes, including event handlers
///
/// # Returns
/// A TokenStream that generates JavaScript event listener registration code
pub(crate) fn generate_event_listeners(
    element_id: &str,
    attributes: &std::collections::HashMap<String, ComponentAttribute>,
) -> Result<Vec<proc_macro2::TokenStream>> {
    let mut event_registrations = Vec::new();

    for (attr_name, attr_value) in attributes {
        if let ComponentAttribute::EventHandler(handler) = attr_value {
            // Extract event name from attribute (e.g., "onclick" -> "click")
            let event_name = if attr_name.starts_with("on") && attr_name.len() > 2 {
                &attr_name[2..] // Remove "on" prefix
            } else {
                continue; // Skip invalid event names
            };

            // Generate event listener registration code
            let registration_code = if let Ok(handler_ident) = syn::parse_str::<syn::Ident>(handler)
            {
                // Handler is a simple identifier (function name)
                quote! {
                    {
                        use apex::wasm_bindgen::prelude::*;
                        use apex::web_sys::*;

                        // Get the element by ID
                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        if let Some(element) = document.get_element_by_id(#element_id) {
                            // Create a closure that calls the Rust function
                            let closure = Closure::wrap(Box::new(move |event: apex::web_sys::Event| {
                                #handler_ident(event);
                            }) as Box<dyn FnMut(_)>);

                            // Add event listener
                            element.add_event_listener_with_callback(#event_name, closure.as_ref().unchecked_ref())
                                .expect("failed to add event listener");

                            // Forget the closure to keep it alive
                            closure.forget();
                        }
                    }
                }
            } else if let Ok(handler_expr) = syn::parse_str::<syn::Expr>(handler) {
                // Handler is an expression (function call, closure, etc.)
                quote! {
                    {
                        use apex::wasm_bindgen::prelude::*;
                        use apex::web_sys::*;

                        // Get the element by ID
                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        if let Some(element) = document.get_element_by_id(#element_id) {
                            // Create a closure that calls the Rust expression
                            let closure = Closure::wrap(Box::new(move |event: apex::web_sys::Event| {
                                let handler = #handler_expr;
                                handler(event);
                            }) as Box<dyn FnMut(_)>);

                            // Add event listener
                            element.add_event_listener_with_callback(#event_name, closure.as_ref().unchecked_ref())
                                .expect("failed to add event listener");

                            // Forget the closure to keep it alive
                            closure.forget();
                        }
                    }
                }
            } else {
                // Fallback: treat as string literal (function name)
                quote! {
                    {
                        use apex::wasm_bindgen::prelude::*;
                        use apex::web_sys::*;

                        // Get the element by ID
                        let window = apex::web_sys::window().expect("no global `window` exists");
                        let document = window.document().expect("should have a document on window");
                        if let Some(element) = document.get_element_by_id(#element_id) {
                            // Note: This is a simplified fallback. In a real implementation,
                            // you might want to handle this differently.
                            console::log_1(&format!("Event handler '{}' for '{}' on element '{}'", #handler, #event_name, #element_id).into());
                        }
                    }
                }
            };

            event_registrations.push(registration_code);
        }
    }

    Ok(event_registrations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_generate_event_listeners_empty() {
        let attributes = HashMap::new();
        let listeners = generate_event_listeners("test_id", &attributes).unwrap();
        assert!(listeners.is_empty());
    }

    #[test]
    fn test_generate_event_listeners_with_onclick() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "onclick".to_owned(),
            ComponentAttribute::EventHandler("handle_click".to_owned()),
        );

        let listeners = generate_event_listeners("test_id", &attributes).unwrap();
        assert_eq!(listeners.len(), 1);

        let code = listeners[0].to_string();
        assert!(code.contains("click")); // Event name should be "click", not "onclick"
        assert!(code.contains("handle_click"));
        assert!(code.contains("test_id"));
    }

    #[test]
    fn test_generate_event_listeners_multiple_events() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "onclick".to_owned(),
            ComponentAttribute::EventHandler("handle_click".to_owned()),
        );
        attributes.insert(
            "onmouseover".to_owned(),
            ComponentAttribute::EventHandler("handle_mouseover".to_owned()),
        );

        let listeners = generate_event_listeners("test_id", &attributes).unwrap();
        assert_eq!(listeners.len(), 2);
    }

    #[test]
    fn test_generate_event_listeners_ignores_non_events() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "class".to_owned(),
            ComponentAttribute::Literal("button".to_owned()),
        );
        attributes.insert(
            "onclick".to_owned(),
            ComponentAttribute::EventHandler("handle_click".to_owned()),
        );

        let listeners = generate_event_listeners("test_id", &attributes).unwrap_or_default();
        assert_eq!(listeners.len(), 1); // Only the onclick should generate a listener
    }
}
