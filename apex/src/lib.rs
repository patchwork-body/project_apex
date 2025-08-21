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
                let _ = history.push_state_with_url(&js_sys::Object::new(), "", Some(&path));

                web_sys::console::log_1(&format!("Navigated to: {path}").into());
            }
        }) as Box<dyn FnMut(_)>);

        let _ = document.add_event_listener_with_callback(
            "apex:navigate",
            navigate_callback.as_ref().unchecked_ref(),
        );

        navigate_callback.forget();

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
