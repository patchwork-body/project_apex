#![allow(missing_docs)]

use matchit::Router;
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::get_matched_path;

pub trait ApexClientRoute {
    fn path(&self) -> &'static str {
        "/"
    }
    fn hydrate_component(
        &self,
        expressions_map: &HashMap<String, web_sys::Text>,
        elements_map: &HashMap<String, web_sys::Element>,
    );
    fn children(&self) -> Vec<Box<dyn ApexClientRoute>> {
        Vec::new()
    }
}

struct Outlet {
    pub begin: Option<web_sys::Comment>,
    pub end: Option<web_sys::Comment>,
}

struct RouteChain {
    parent_pattern: Option<Vec<String>>,
    route: Box<dyn ApexClientRoute>,
    outlet: RefCell<Option<Outlet>>,
}

pub struct ApexClientRouter {
    router: Rc<RefCell<Router<RouteChain>>>,
}

impl ApexClientRouter {
    pub fn new(route: Box<dyn ApexClientRoute>) -> Self {
        let mut r = Self {
            router: Rc::new(RefCell::new(Router::new())),
        };

        r.mount_root_route(route);

        r
    }

    fn mount_root_route(&mut self, route: Box<dyn ApexClientRoute>) {
        self.mount_route(route, None);
        self.init();
        Self::cleanup_init_script();
    }

    fn mount_route(
        &mut self,
        route: Box<dyn ApexClientRoute>,
        parent_pattern: Option<Vec<String>>,
    ) {
        let path = route.path();
        let children = route.children();

        let route_chain = RouteChain {
            parent_pattern: parent_pattern.clone(),
            route,
            outlet: RefCell::new(None),
        };

        let mut parent_path = parent_pattern.unwrap_or_default();
        let mut route_path = String::new();

        for part in parent_path.iter() {
            route_path.push_str(part.trim_end_matches("/"));
        }

        route_path.push_str(path);

        if let Err(e) = self.router.borrow_mut().insert(&route_path, route_chain) {
            panic!("Failed to insert route '{path}': {e}");
        }

        parent_path.push(route_path);

        for child in children.into_iter() {
            self.mount_route(child, parent_path.clone().into());
        }
    }

    fn init(&self) {
        let window = web_sys::window().expect("window not found");
        let document = window.document().expect("document not found");

        let navigate_callback = {
            Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
                if let Some(path) = event.detail().as_string() {
                    let window = web_sys::window().expect("window not found");
                    let history = window.history().expect("history not found");
                    let document = window.document().expect("document not found");
                    let current_path = window.location().pathname().expect("pathname not found");
                    let exclude_path = get_matched_path(&current_path, &path);

                    web_sys::console::log_1(&format!("Navigating to: {path} current_path: {current_path} exclude_path: {exclude_path}").into());

                    let fetch_promise = window
                        .fetch_with_str(&format!("{path}?has_exclude&exclude={exclude_path}&"));

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

                            if let Ok(response) = response.dyn_into::<web_sys::Response>() {
                                let text_promise = response.text().unwrap();
                                let document_clone2 = document.clone();

                                let text_callback = {
                                    let document_clone2 = document_clone2.clone();
                                    let path = path.clone();
                                    let history = history.clone();

                                    Closure::wrap(Box::new(
                                        move |json_text: wasm_bindgen::JsValue| {
                                            if let Some(json_str) = json_text.as_string() {
                                                if let Ok(json_obj) = js_sys::JSON::parse(&json_str)
                                                {
                                                    let html_value = js_sys::Reflect::get(
                                                        &json_obj,
                                                        &"html".into(),
                                                    )
                                                    .unwrap_or(wasm_bindgen::JsValue::from_str(""));

                                                    if let Ok(data_value) = js_sys::Reflect::get(
                                                        &json_obj,
                                                        &"data".into(),
                                                    ) {
                                                        let window = web_sys::window()
                                                            .expect("window not found");

                                                        // Get existing INIT_DATA or create new object
                                                        let existing_data = js_sys::Reflect::get(
                                                            &window,
                                                            &"INIT_DATA".into(),
                                                        )
                                                        .unwrap_or_else(|_| {
                                                            js_sys::Object::new().into()
                                                        });

                                                        // If existing_data is not an object, create a new one
                                                        let init_data = if existing_data.is_object()
                                                        {
                                                            existing_data
                                                        } else {
                                                            js_sys::Object::new().into()
                                                        };

                                                        // Merge new data into existing INIT_DATA
                                                        if let Some(data_obj) =
                                                            data_value.dyn_ref::<js_sys::Object>()
                                                        {
                                                            let entries =
                                                                js_sys::Object::entries(data_obj);
                                                            let length =
                                                                js_sys::Array::length(&entries);

                                                            for i in 0..length {
                                                                if let Some(entry) = entries
                                                                    .get(i)
                                                                    .dyn_ref::<js_sys::Array>(
                                                                ) {
                                                                    let key = entry.get(0);
                                                                    let value = entry.get(1);
                                                                    let _ = js_sys::Reflect::set(
                                                                        &init_data, &key, &value,
                                                                    );
                                                                }
                                                            }
                                                        }

                                                        let _ = js_sys::Reflect::set(
                                                            &window,
                                                            &"INIT_DATA".into(),
                                                            &init_data,
                                                        );
                                                    }

                                                    if let Some(html_text) = html_value.as_string()
                                                    {
                                                        web_sys::console::log_1(
                                                            &format!(
                                                                "Exclude path: {exclude_path}"
                                                            )
                                                            .into(),
                                                        );

                                                        let _ = history.push_state_with_url(
                                                            &js_sys::Object::new(),
                                                            "",
                                                            Some(&path),
                                                        );

                                                        let event_init =
                                                            web_sys::CustomEventInit::new();
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

        let _ = document.add_event_listener_with_callback(
            "apex:navigate",
            navigate_callback.as_ref().unchecked_ref(),
        );

        navigate_callback.forget();

        let rehydrate_callback = {
            let router = self.router.clone();

            Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
                let event_detail: wasm_bindgen::JsValue = event.detail();

                web_sys::console::log_1(&format!("Event detail: {event_detail:#?}").into());

                let Ok(outlet_key) = js_sys::Reflect::get(&event_detail, &"outlet_key".into())
                else {
                    return;
                };

                let Ok(outlet_content) =
                    js_sys::Reflect::get(&event_detail, &"outlet_content".into())
                else {
                    return;
                };

                web_sys::console::log_1(&format!("Outlet key: {outlet_key:#?}").into());

                if let Some(outlet_key) = outlet_key.as_string()
                    && let Some(outlet_content) = outlet_content.as_string()
                {
                    // Find the route and get its outlet
                    let router_borrow = router.borrow();
                    let Ok(outlet_match) = router_borrow.at(&outlet_key) else {
                        return;
                    };

                    let outlet_ref = outlet_match.value.outlet.borrow();
                    let Some(outlet) = outlet_ref.as_ref() else {
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

                    web_sys::console::log_1(&format!("HTML text: {outlet_content}").into());

                    // Remove all nodes between begin and end comments
                    let mut current_node = begin.next_sibling();
                    while let Some(node) = &current_node {
                        if node.is_same_node(Some(end)) {
                            web_sys::console::log_1(&"End comment found".to_owned().into());
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
                    } else {
                        web_sys::console::log_1(&"End comment not found".to_owned().into());
                    }

                    Self::hydrate_router(router.clone(), Some(outlet_key));
                }
            }) as Box<dyn FnMut(_)>)
        };

        let _ = document.add_event_listener_with_callback(
            "apex:rehydrate",
            rehydrate_callback.as_ref().unchecked_ref(),
        );

        rehydrate_callback.forget();

        Self::hydrate_router(self.router.clone(), None);
    }

    fn cleanup_init_script() {
        let window = web_sys::window().expect("window not found");
        let document = window.document().expect("document not found");

        if let Some(script) = document.get_element_by_id("apex-init-data") {
            script.remove();
        }
    }

    fn hydrate_router(router: Rc<RefCell<Router<RouteChain>>>, exclude_path: Option<String>) {
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
            if let Some(comment) = node.dyn_ref::<web_sys::Comment>() {
                let data = comment.data();
                let parts: Vec<String> = data.split(":").map(|s| s.trim().to_owned()).collect();

                if parts.len() < 2 {
                    continue;
                }

                let comment_type = &parts[0];
                let comment_id = &parts[1];

                if comment_type == "@expr-text-begin" {
                    let Some(next_node) = comment.next_sibling() else {
                        continue;
                    };

                    if let Some(text_node) = next_node.dyn_ref::<web_sys::Text>() {
                        expressions_map.insert(comment_id.clone(), text_node.clone());

                        let Some(next_node) = next_node.next_sibling() else {
                            continue;
                        };

                        let Some(end_comment) = next_node.dyn_ref::<web_sys::Comment>() else {
                            continue;
                        };

                        nodes_to_remove.push(comment.clone());
                        nodes_to_remove.push(end_comment.clone());
                    } else if let Some(end_comment) = next_node.dyn_ref::<web_sys::Comment>() {
                        let data = end_comment.data();
                        let parts: Vec<String> =
                            data.split(":").map(|s| s.trim().to_owned()).collect();

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
                        }
                    }
                } else if comment_type == "@element" {
                    let Some(next_node) = comment.next_sibling() else {
                        continue;
                    };

                    let element_node = next_node
                        .dyn_ref::<web_sys::Element>()
                        .expect("element node not found");

                    elements_map.insert(comment_id.clone(), element_node.clone());
                    nodes_to_remove.push(comment.clone());
                } else if comment_type == "@outlet-begin" {
                    let exclude_path = exclude_path.clone().unwrap_or_default();

                    if !exclude_path.is_empty()
                        && get_matched_path(comment_id, &exclude_path) == exclude_path
                    {
                        continue;
                    }

                    if let Ok(route_match) = router.borrow().at(comment_id) {
                        let mut outlet_ref = route_match.value.outlet.borrow_mut();

                        if outlet_ref.is_none() {
                            *outlet_ref = Some(Outlet {
                                begin: Some(comment.clone()),
                                end: None,
                            });
                        }
                    }
                } else if comment_type == "@outlet-end" {
                    let exclude_path = exclude_path.clone().unwrap_or_default();

                    if !exclude_path.is_empty()
                        && get_matched_path(comment_id, &exclude_path) == exclude_path
                    {
                        continue;
                    }

                    if let Ok(route_match) = router.borrow().at(comment_id) {
                        let mut outlet_ref = route_match.value.outlet.borrow_mut();

                        if let Some(outlet) = outlet_ref.as_mut() {
                            outlet.end = Some(comment.clone());
                        }
                    }
                }
            }
        }

        // for node in nodes_to_remove {
        //     node.remove();
        // }

        let location = window.location();
        let pathname = location.pathname().expect("pathname not found");

        if let Ok(route_matched) = router.borrow().at(&pathname) {
            if let Some(parent_patterns_chain) = route_matched.value.parent_pattern.as_ref() {
                for parent_pattern in parent_patterns_chain.iter() {
                    let matched_path = get_matched_path(parent_pattern, &pathname);

                    if exclude_path.as_ref() == Some(&matched_path) {
                        continue;
                    }

                    if let Ok(parent_route_match) = router
                        .borrow()
                        .at(&get_matched_path(parent_pattern, &pathname))
                    {
                        parent_route_match
                            .value
                            .route
                            .hydrate_component(&expressions_map, &elements_map);
                    }
                }
            }

            route_matched
                .value
                .route
                .hydrate_component(&expressions_map, &elements_map);
        }
    }
}

impl fmt::Debug for ApexClientRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApexClientRouter")
            .field("router", &"Rc<RefCell<Router<RouteChain>>> { ... }")
            .finish()
    }
}
