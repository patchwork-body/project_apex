// // //! Client-side navigation module for handling SPA-style routing
// // //!
// // //! This module provides the infrastructure for client-side navigation,
// // //! including history management, route transitions, and prefetching.

// use std::cell::RefCell;
// use std::collections::HashMap;
// use std::rc::Rc;

// // #[cfg(target_arch = "wasm32")]
// // use wasm_bindgen::JsCast;
// // #[cfg(target_arch = "wasm32")]
// // use wasm_bindgen::prelude::*;
// // #[cfg(target_arch = "wasm32")]
// // use web_sys::{Document, Event, HtmlElement, PopStateEvent, Window};

// // /// Navigation state for client-side routing
// // #[derive(Clone, Debug)]
// // pub struct NavigationState {
// //     pub current_path: String,
// //     pub previous_path: Option<String>,
// //     pub scroll_positions: HashMap<String, (f64, f64)>,
// // }

// // impl Default for NavigationState {
// //     fn default() -> Self {
// //         Self {
// //             current_path: "/".to_string(),
// //             previous_path: None,
// //             scroll_positions: HashMap::new(),
// //         }
// //     }
// // }

// // /// Navigation event types
// // #[derive(Debug, Clone)]
// // pub enum NavigationEvent {
// //     Navigate { path: String, replace: bool },
// //     Back,
// //     Forward,
// // }

// // /// Navigation handler trait
// // pub trait NavigationHandler: 'static {
// //     fn handle_navigation(&self, event: NavigationEvent) -> Result<(), String>;
// // }

// /// Client-side router for managing navigation
// pub struct ClientRouter {
//     state: Rc<RefCell<NavigationState>>,
//     handlers: Vec<Box<dyn NavigationHandler>>,
//     route_cache: Rc<RefCell<HashMap<String, String>>>, // Path -> HTML cache
//     prefetch_queue: Rc<RefCell<Vec<String>>>,
// }

// impl ClientRouter {
//     pub fn new() -> Self {
//         Self {
//             state: Rc::new(RefCell::new(NavigationState::default())),
//             handlers: Vec::new(),
//             route_cache: Rc::new(RefCell::new(HashMap::new())),
//             prefetch_queue: Rc::new(RefCell::new(Vec::new())),
//         }
//     }

//     /// Register a navigation handler
//     pub fn add_handler<H: NavigationHandler>(&mut self, handler: H) {
//         self.handlers.push(Box::new(handler));
//     }

//     /// Navigate to a new path
//     pub fn navigate(&self, path: &str, replace: bool) -> Result<(), String> {
//         let event = NavigationEvent::Navigate {
//             path: path.to_string(),
//             replace,
//         };

//         // Update state
//         let mut state = self.state.borrow_mut();
//         state.previous_path = Some(state.current_path.clone());
//         state.current_path = path.to_string();
//         drop(state);

//         // Notify handlers
//         for handler in &self.handlers {
//             handler.handle_navigation(event.clone())?;
//         }

//         Ok(())
//     }

//     /// Go back in history
//     pub fn back(&self) -> Result<(), String> {
//         let event = NavigationEvent::Back;

//         // Update state
//         let mut state = self.state.borrow_mut();
//         if let Some(prev) = state.previous_path.take() {
//             let current = state.current_path.clone();
//             state.current_path = prev;
//             state.previous_path = Some(current);
//         }
//         drop(state);

//         // Notify handlers
//         for handler in &self.handlers {
//             handler.handle_navigation(event.clone())?;
//         }

//         Ok(())
//     }

//     /// Go forward in history
//     pub fn forward(&self) -> Result<(), String> {
//         let event = NavigationEvent::Forward;

//         // Notify handlers
//         for handler in &self.handlers {
//             handler.handle_navigation(event.clone())?;
//         }

//         Ok(())
//     }

//     /// Prefetch a route for faster navigation
//     pub async fn prefetch(&self, path: &str) -> Result<(), String> {
//         // Check if already cached
//         if self.route_cache.borrow().contains_key(path) {
//             return Ok(());
//         }

//         // Add to prefetch queue if not already there
//         let mut queue = self.prefetch_queue.borrow_mut();
//         if !queue.contains(&path.to_string()) {
//             queue.push(path.to_string());
//         }

//         // In a real implementation, this would:
//         // 1. Fetch the route's HTML/JS from the server
//         // 2. Parse and cache the response
//         // 3. Potentially preload assets

//         Ok(())
//     }

//     /// Get current navigation state
//     pub fn get_state(&self) -> NavigationState {
//         self.state.borrow().clone()
//     }

//     /// Save scroll position for the current route
//     pub fn save_scroll_position(&self) {
//         #[cfg(target_arch = "wasm32")]
//         {
//             if let Some(window) = web_sys::window() {
//                 let x = window.scroll_x().unwrap_or(0.0);
//                 let y = window.scroll_y().unwrap_or(0.0);

//                 let mut state = self.state.borrow_mut();
//                 state
//                     .scroll_positions
//                     .insert(state.current_path.clone(), (x, y));
//             }
//         }
//     }

//     /// Restore scroll position for a route
//     pub fn restore_scroll_position(&self, _path: &str) {
//         #[cfg(target_arch = "wasm32")]
//         {
//             let state = self.state.borrow();
//             if let Some((x, y)) = state.scroll_positions.get(_path) {
//                 if let Some(window) = web_sys::window() {
//                     window.scroll_to_with_x_and_y(*x, *y);
//                 }
//             }
//         }
//     }
// }

// // impl Default for ClientRouter {
// //     fn default() -> Self {
// //         Self::new()
// //     }
// // }

// // /// Initialize client-side navigation
// // #[cfg(target_arch = "wasm32")]
// // pub fn init_client_navigation() -> Result<ClientRouter, JsValue> {
// //     let router = ClientRouter::new();
// //     let router_rc = Rc::new(RefCell::new(router));

// //     // Set up popstate handler for browser back/forward
// //     let window = web_sys::window().expect("no global `window` exists");
// //     let router_clone = router_rc.clone();

// //     let popstate_callback = Closure::wrap(Box::new(move |event: PopStateEvent| {
// //         if let Some(location) = web_sys::window().and_then(|w| w.location().pathname().ok()) {
// //             let _ = router_clone.borrow().navigate(&location, true);
// //         }
// //     }) as Box<dyn FnMut(_)>);

// //     window
// //         .add_event_listener_with_callback("popstate", popstate_callback.as_ref().unchecked_ref())?;
// //     popstate_callback.forget();

// //     // Set up custom navigation event listener
// //     let document = window.document().expect("should have a document on window");
// //     let router_clone = router_rc.clone();

// //     let navigate_callback = Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
// //         if let Ok(path) = event.detail().as_string() {
// //             let _ = router_clone.borrow().navigate(&path, false);
// //         }
// //     }) as Box<dyn FnMut(_)>);

// //     document.add_event_listener_with_callback(
// //         "apex:navigate",
// //         navigate_callback.as_ref().unchecked_ref(),
// //     )?;
// //     navigate_callback.forget();

// //     // Set up link click interceptor for all links
// //     let router_clone = router_rc.clone();
// //     let click_callback = Closure::wrap(Box::new(move |event: Event| {
// //         if let Some(target) = event.target() {
// //             if let Some(element) = target.dyn_ref::<HtmlElement>() {
// //                 // Check if it's a link or inside a link
// //                 let mut current = Some(element.clone());
// //                 while let Some(elem) = current {
// //                     if elem.tag_name() == "A" {
// //                         if let Some(anchor) = elem.dyn_ref::<web_sys::HtmlAnchorElement>() {
// //                             let href = anchor.href();

// //                             // Check if it's an internal link
// //                             if let Ok(location) = window.location().origin() {
// //                                 if href.starts_with(&location) {
// //                                     event.prevent_default();
// //                                     let path = href.replace(&location, "");
// //                                     let _ = router_clone.borrow().navigate(&path, false);
// //                                     break;
// //                                 }
// //                             }
// //                         }
// //                     }
// //                     current = elem.parent_element();
// //                 }
// //             }
// //         }
// //     }) as Box<dyn FnMut(_)>);

// //     document.add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())?;
// //     click_callback.forget();

// //     // Return the router
// //     Ok(Rc::try_unwrap(router_rc).unwrap().into_inner())
// // }

// // /// Server-side stub for init_client_navigation
// // #[cfg(not(target_arch = "wasm32"))]
// // pub fn init_client_navigation() -> Result<ClientRouter, String> {
// //     // On server-side, return a basic router that doesn't do anything
// //     Ok(ClientRouter::new())
// // }

// // /// Helper to check if a link is active
// // pub fn is_active_link(href: &str, current_path: &str, exact: bool) -> bool {
// //     if exact {
// //         href == current_path
// //     } else {
// //         current_path.starts_with(href)
// //     }
// // }

// // /// Utility to parse query parameters from a path
// // pub fn parse_query_params(path: &str) -> HashMap<String, String> {
// //     let mut params = HashMap::new();

// //     if let Some(query_start) = path.find('?') {
// //         let query = &path[query_start + 1..];
// //         for pair in query.split('&') {
// //             if let Some(eq_pos) = pair.find('=') {
// //                 let key = &pair[..eq_pos];
// //                 let value = &pair[eq_pos + 1..];
// //                 params.insert(key.to_string(), value.to_string());
// //             }
// //         }
// //     }

// //     params
// // }

// // /// Utility to build a path with query parameters
// // pub fn build_path_with_params(base: &str, params: &HashMap<String, String>) -> String {
// //     if params.is_empty() {
// //         base.to_string()
// //     } else {
// //         let query: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
// //         format!("{}?{}", base, query.join("&"))
// //     }
// // }

// // #[cfg(test)]
// // mod tests {
// //     use super::*;

// //     #[test]
// //     fn test_navigation_state() {
// //         let state = NavigationState::default();
// //         assert_eq!(state.current_path, "/");
// //         assert!(state.previous_path.is_none());
// //         assert!(state.scroll_positions.is_empty());
// //     }

// //     #[test]
// //     fn test_query_params_parsing() {
// //         let params = parse_query_params("/search?q=rust&limit=10");
// //         assert_eq!(params.get("q"), Some(&"rust".to_string()));
// //         assert_eq!(params.get("limit"), Some(&"10".to_string()));

// //         let empty = parse_query_params("/home");
// //         assert!(empty.is_empty());
// //     }

// //     #[test]
// //     fn test_build_path_with_params() {
// //         let mut params = HashMap::new();
// //         params.insert("q".to_string(), "rust".to_string());
// //         params.insert("page".to_string(), "2".to_string());

// //         let path = build_path_with_params("/search", &params);
// //         assert!(path.contains("q=rust"));
// //         assert!(path.contains("page=2"));
// //         assert!(path.starts_with("/search?"));
// //     }

// //     #[test]
// //     fn test_is_active_link() {
// //         assert!(is_active_link("/home", "/home", true));
// //         assert!(!is_active_link("/home", "/home/profile", true));

// //         assert!(is_active_link("/home", "/home", false));
// //         assert!(is_active_link("/home", "/home/profile", false));
// //         assert!(!is_active_link("/about", "/home", false));
// //     }
// // }
