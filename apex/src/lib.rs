#![allow(missing_docs)]
use std::collections::HashMap;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Comment, Element, Text};

pub mod prelude;

pub use apex_utils;
pub use bytes;
pub use js_sys;
pub use wasm_bindgen;
pub use web_sys;

pub mod action;
pub mod navigation;
pub mod router;
pub mod server_context;
pub mod signal;

pub mod init_data;

// Re-export main types for easy access
pub use router::ApexRouter;

pub struct Apex;

impl Apex {
    pub fn hydrate<R>(route: R)
    where
        R: router::ApexRoute + 'static,
    {
        apex_utils::reset_counters();
        static SHOW_COMMENT: u32 = 128;

        let window = web_sys::window().expect("window not found");
        let document = window.document().expect("document not found");

        let navigate_callback = Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
            if let Some(path) = event.detail().as_string() {
                let window = web_sys::window().expect("window not found");
                let history = window.history().expect("history not found");
                let document = window.document().expect("document not found");

                web_sys::console::log_1(&format!("Navigating to: {path}").into());

                // Update URL in browser history
                let _ = history.push_state_with_url(&js_sys::Object::new(), "", Some(&path));

                // Fetch the new page content
                let fetch_promise = window.fetch_with_str(&path);

                // Handle the fetch response
                let document_clone = document.clone();
                let path_clone = path.clone();
                let response_callback =
                    Closure::wrap(Box::new(move |response: wasm_bindgen::JsValue| {
                        let document = document_clone.clone();
                        let path = path_clone.clone();

                        // Cast JsValue to Response
                        if let Ok(response) = response.dyn_into::<web_sys::Response>() {
                            let text_promise = response.text().unwrap();
                            let document_clone2 = document.clone();
                            let text_callback =
                                Closure::wrap(Box::new(move |html_text: wasm_bindgen::JsValue| {
                                    let document = document_clone2.clone();
                                    if let Some(html_text) = html_text.as_string() {
                                        web_sys::console::log_1(
                                            &format!("Fetched content for: {path}").into(),
                                        );

                                        // Simple approach: extract body content from HTML string
                                        // Look for <body> tags and extract content between them
                                        if let Some(body_start) = html_text.find("<body")
                                            && let Some(body_content_start) =
                                                html_text[body_start..].find('>')
                                        {
                                            let body_content_start =
                                                body_start + body_content_start + 1;
                                            if let Some(body_end) =
                                                html_text[body_content_start..].find("</body>")
                                            {
                                                let body_content = &html_text[body_content_start
                                                    ..body_content_start + body_end];

                                                // Replace the current page body content
                                                if let Some(current_body) = document.body() {
                                                    current_body.set_inner_html(body_content);
                                                }
                                            }
                                        }

                                        // Re-run hydration on the new content
                                        // Note: We would need access to the route here to call hydrate_components
                                        // This is a simplified version - in a full implementation you'd want to
                                        // store the route reference or re-fetch component mappings
                                    }
                                })
                                    as Box<dyn FnMut(wasm_bindgen::JsValue)>);

                            let _ = text_promise.then(&text_callback);
                            text_callback.forget();
                        }
                    })
                        as Box<dyn FnMut(wasm_bindgen::JsValue)>);

                let _ = fetch_promise.then(&response_callback);
                response_callback.forget();
            }
        }) as Box<dyn FnMut(_)>);

        let _ = document.add_event_listener_with_callback(
            "apex:navigate",
            navigate_callback.as_ref().unchecked_ref(),
        );

        navigate_callback.forget();

        // Add popstate event listener for browser back/forward navigation
        let window_clone = window.clone();
        let document_clone = document.clone();
        let popstate_callback = Closure::wrap(Box::new(move |_event: web_sys::PopStateEvent| {
            let window = window_clone.clone();
            let document = document_clone.clone();

            // Get the current path from location
            let location = window.location();
            let pathname = location.pathname().unwrap_or_else(|_| "/".to_string());

            web_sys::console::log_1(&format!("Popstate navigation to: {pathname}").into());

            // Fetch the content for the new path
            let fetch_promise = window.fetch_with_str(&pathname);

            // Handle the fetch response
            let document_clone2 = document.clone();
            let pathname_clone = pathname.clone();
            let response_callback =
                Closure::wrap(Box::new(move |response: wasm_bindgen::JsValue| {
                    let document = document_clone2.clone();
                    let path = pathname_clone.clone();

                    // Cast JsValue to Response
                    if let Ok(response) = response.dyn_into::<web_sys::Response>() {
                        let text_promise = response.text().unwrap();
                        let document_clone3 = document.clone();
                        let text_callback =
                            Closure::wrap(Box::new(move |html_text: wasm_bindgen::JsValue| {
                                let document = document_clone3.clone();
                                if let Some(html_text) = html_text.as_string() {
                                    web_sys::console::log_1(
                                        &format!("Fetched content for popstate: {path}").into(),
                                    );

                                    // Extract body content from HTML string
                                    if let Some(body_start) = html_text.find("<body")
                                        && let Some(body_content_start) =
                                            html_text[body_start..].find('>')
                                    {
                                        let body_content_start =
                                            body_start + body_content_start + 1;
                                        if let Some(body_end) =
                                            html_text[body_content_start..].find("</body>")
                                        {
                                            let body_content = &html_text
                                                [body_content_start..body_content_start + body_end];

                                            // Replace the current page body content
                                            if let Some(current_body) = document.body() {
                                                current_body.set_inner_html(body_content);
                                            }
                                        }
                                    }
                                }
                            })
                                as Box<dyn FnMut(wasm_bindgen::JsValue)>);

                        let _ = text_promise.then(&text_callback);
                        text_callback.forget();
                    }
                }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);

            let _ = fetch_promise.then(&response_callback);
            response_callback.forget();
        }) as Box<dyn FnMut(_)>);

        let _ = window.add_event_listener_with_callback(
            "popstate",
            popstate_callback.as_ref().unchecked_ref(),
        );

        popstate_callback.forget();

        // Start hydrating the page
        let tree_walker = document
            .create_tree_walker_with_what_to_show(
                &document.body().expect("body not found"),
                SHOW_COMMENT,
            )
            .expect("tree walker not found");

        let mut expressions_map: HashMap<String, web_sys::Text> = HashMap::new();
        let mut elements_map: HashMap<String, web_sys::Element> = HashMap::new();
        let mut nodes_to_remove = Vec::new();

        while let Ok(Some(node)) = tree_walker.next_node() {
            if let Some(comment) = node.dyn_ref::<Comment>() {
                let data = comment.data();
                let parts: Vec<String> = data.split(":").map(|s| s.trim().to_string()).collect();

                if parts.len() < 2 {
                    continue;
                }

                let comment_type = &parts[0];
                let comment_id = &parts[1];

                if comment_type == "@expr-text-begin" {
                    let Some(next_node) = comment.next_sibling() else {
                        continue;
                    };

                    if let Some(text_node) = next_node.dyn_ref::<Text>() {
                        expressions_map.insert(comment_id.clone(), text_node.clone());

                        let Some(next_node) = next_node.next_sibling() else {
                            continue;
                        };

                        let Some(end_comment) = next_node.dyn_ref::<Comment>() else {
                            continue;
                        };

                        nodes_to_remove.push(comment.clone());
                        nodes_to_remove.push(end_comment.clone());
                    } else if let Some(end_comment) = next_node.dyn_ref::<Comment>() {
                        let data = end_comment.data();
                        let parts: Vec<String> =
                            data.split(":").map(|s| s.trim().to_string()).collect();

                        if parts.len() < 2 {
                            continue;
                        }

                        let end_comment_type = &parts[0];
                        let end_comment_id = &parts[1];

                        if end_comment_type == "@expr-text-end" && end_comment_id == comment_id {
                            // Create an empty text node
                            let text_node = document.create_text_node("");

                            // Insert the text node before the end comment
                            if let Some(parent) = end_comment.parent_node() {
                                let _ = parent.insert_before(&text_node, Some(end_comment));
                            }

                            // Store the text node reference in expressions_map
                            expressions_map.insert(comment_id.clone(), text_node);

                            nodes_to_remove.push(comment.clone());
                            nodes_to_remove.push(end_comment.clone());

                            continue;
                        }
                    }
                } else if comment_type == "@element" {
                    let Some(next_node) = comment.next_sibling() else {
                        continue;
                    };

                    let element_node = next_node
                        .dyn_ref::<Element>()
                        .expect("element node not found");

                    elements_map.insert(comment_id.clone(), element_node.clone());
                    nodes_to_remove.push(comment.clone());
                }
            }
        }

        for node in nodes_to_remove {
            node.remove();
        }

        let location = window.location();
        let pathname = location.pathname().expect("pathname not found");

        route.hydrate_components(&pathname, &expressions_map, &elements_map);
    }
}
