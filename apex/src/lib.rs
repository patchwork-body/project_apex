#![allow(missing_docs)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};
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

pub struct Outlet {
    pub begin: Option<web_sys::Comment>,
    pub end: Option<web_sys::Comment>,
}

pub struct Apex {
    pub outlets: Rc<RefCell<HashMap<String, Outlet>>>,
}

impl Apex {
    pub fn new() -> Self {
        Self {
            outlets: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn hydrate<R>(&mut self, route: R)
    where
        R: router::ApexRoute + 'static,
    {
        let route = Rc::new(route);
        let window = web_sys::window().expect("window not found");
        let document = window.document().expect("document not found");

        let navigate_callback = {
            Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
                if let Some(path) = event.detail().as_string() {
                    let window = web_sys::window().expect("window not found");
                    let history = window.history().expect("history not found");
                    let document = window.document().expect("document not found");

                    web_sys::console::log_1(&format!("Navigating to: {path}").into());

                    let current_path = window.location().pathname().expect("pathname not found");
                    let exclude_path = crate::router::get_matched_path(&current_path, &path);

                    // Fetch the new page content
                    let fetch_promise =
                        window.fetch_with_str(&format!("{path}?exclude={exclude_path}&"));

                    // Handle the fetch response
                    let document_clone = document.clone();
                    let path_clone: String = path.clone();
                    let history_clone = history.clone();

                    let response_callback = {
                        let document_clone = document_clone.clone();
                        let path_clone = path_clone.clone();
                        let history_clone = history_clone.clone();

                        Closure::wrap(Box::new(move |response: wasm_bindgen::JsValue| {
                            let document = document_clone.clone();
                            let path = path_clone.clone();
                            let history = history_clone.clone();
                            let exclude_path = exclude_path.clone();

                            // Cast JsValue to Response
                            if let Ok(response) = response.dyn_into::<web_sys::Response>() {
                                let text_promise = response.text().unwrap();
                                let document_clone2 = document.clone();

                                let text_callback = {
                                    let document_clone2 = document_clone2.clone();
                                    let path = path.clone();
                                    let history = history.clone();

                                    Closure::wrap(Box::new(
                                        move |html_text: wasm_bindgen::JsValue| {
                                            if let Some(html_text) = html_text.as_string() {
                                                web_sys::console::log_1(
                                                    &format!("Exclude path: {exclude_path}").into(),
                                                );

                                                web_sys::console::log_1(
                                                    &format!("HTML text: {html_text}").into(),
                                                );

                                                // Update URL in browser history
                                                let _ = history.push_state_with_url(
                                                    &js_sys::Object::new(),
                                                    "",
                                                    Some(&path),
                                                );

                                                let event_init = web_sys::CustomEventInit::new();
                                                let detail = js_sys::Object::new();

                                                let _ = js_sys::Reflect::set(
                                                    &detail,
                                                    &"outlet_key".into(),
                                                    &exclude_path.to_string().into(),
                                                );

                                                let _ = js_sys::Reflect::set(
                                                    &detail,
                                                    &"outlet_content".into(),
                                                    &html_text.into(),
                                                );

                                                event_init.set_detail(&detail);

                                                // Trigger rehydration by dispatching a custom event
                                                if let Ok(custom_event) =
                                                    web_sys::CustomEvent::new_with_event_init_dict(
                                                        "apex:rehydrate",
                                                        &event_init,
                                                    )
                                                {
                                                    let _ = document_clone2
                                                        .dispatch_event(&custom_event);
                                                }
                                            }
                                        },
                                    )
                                        as Box<dyn FnMut(wasm_bindgen::JsValue)>)
                                };

                                let _ = text_promise.then(&text_callback);
                                text_callback.forget();
                            }
                        })
                            as Box<dyn FnMut(wasm_bindgen::JsValue)>)
                    };

                    let _ = fetch_promise.then(&response_callback);
                    response_callback.forget();
                }
            }) as Box<dyn FnMut(_)>)
        };

        // Add rehydrate event listener
        let rehydrate_callback = {
            let outlets = self.outlets.clone();
            let route = route.clone();

            Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
                let outlets = outlets.borrow_mut();
                let event_detail: wasm_bindgen::JsValue = event.detail();

                web_sys::console::log_1(&format!("Event detail: {:#?}", event_detail).into());

                let Ok(outlet_key) = js_sys::Reflect::get(&event_detail, &"outlet_key".into())
                else {
                    return;
                };

                let Ok(outlet_content) =
                    js_sys::Reflect::get(&event_detail, &"outlet_content".into())
                else {
                    return;
                };

                if let Some(outlet_key) = outlet_key.as_string()
                    && let Some(outlet_content) = outlet_content.as_string()
                {
                    let Some(outlet) = outlets.get(&outlet_key) else {
                        return;
                    };

                    let Some(begin) = &outlet.begin else {
                        return;
                    };

                    let Some(end) = &outlet.end else {
                        return;
                    };

                    // Replace content between begin and end with outlet_content
                    let window = web_sys::window().expect("window not found");
                    let document = window.document().expect("document not found");

                    // Create a temporary div to parse the outlet content
                    let temp_div = document
                        .create_element("div")
                        .expect("failed to create div");
                    temp_div.set_inner_html(&outlet_content);

                    // Remove all nodes between begin and end comments
                    let mut current_node = begin.next_sibling();
                    while let Some(node) = &current_node {
                        if node.is_same_node(Some(end)) {
                            break;
                        }

                        let next = node.next_sibling();
                        if let Some(parent) = node.parent_node() {
                            let _ = parent.remove_child(node);
                        }

                        current_node = next;
                    }

                    // Insert all nodes from temp_div before the end comment
                    if let Some(parent) = end.parent_node() {
                        while let Some(child) = temp_div.first_child() {
                            let _ = parent.insert_before(&child, Some(end));
                        }
                    }

                    let mut apex = Apex::new();
                    apex.hydrate_components(route.clone(), &outlet_key);
                }
            }) as Box<dyn FnMut(_)>)
        };

        let _ = document.add_event_listener_with_callback(
            "apex:rehydrate",
            rehydrate_callback.as_ref().unchecked_ref(),
        );

        rehydrate_callback.forget();

        let _ = document.add_event_listener_with_callback(
            "apex:navigate",
            navigate_callback.as_ref().unchecked_ref(),
        );

        navigate_callback.forget();

        self.hydrate_components(route, "");
    }

    pub fn hydrate_components(&mut self, route: Rc<dyn router::ApexRoute>, exclude_path: &str) {
        apex_utils::reset_counters();
        static SHOW_COMMENT: u32 = 128;

        let window = web_sys::window().expect("window not found");
        let document = window.document().expect("document not found");

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
                } else if comment_type == "@outlet-begin" {
                    self.outlets.borrow_mut().insert(
                        comment_id.clone(),
                        Outlet {
                            begin: Some(comment.clone()),
                            end: None,
                        },
                    );
                } else if comment_type == "@outlet-end"
                    && let Some(outlet) = self.outlets.borrow_mut().get_mut(comment_id)
                {
                    outlet.end = Some(comment.clone());
                }
            }
        }

        for node in nodes_to_remove {
            node.remove();
        }

        let location = window.location();
        let pathname = location.pathname().expect("pathname not found");

        route.hydrate_components(&pathname, exclude_path, &expressions_map, &elements_map);
    }
}

impl Default for Apex {
    fn default() -> Self {
        Self::new()
    }
}
