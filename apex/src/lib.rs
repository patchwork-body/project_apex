#![allow(missing_docs)]
use std::{collections::HashMap, rc::Rc};
use web_sys::{Comment, Element, Text};

pub mod prelude;

pub use bytes;
pub use js_sys;
pub use wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
pub use web_sys;
use web_sys::window;

pub mod action;
pub mod signal;
pub mod template;

/// Trait that defines the view layer for components
///
/// Components must implement this trait to provide their HTML rendering logic
pub trait View {
    /// Render the component to Html
    ///
    /// This method should return the complete HTML representation of the component
    fn render(&self) -> String;
}

type MountCallback = Rc<Closure<dyn Fn(web_sys::Element) -> js_sys::Function>>;
type UnmountCallback = Rc<Closure<dyn Fn()>>;

/// Represents rendered HTML content
///
/// This type wraps HTML strings and provides a safe way to handle HTML content
#[derive(Debug, Clone)]
pub struct Html {
    mount_callback: MountCallback,
    unmount_callback: Option<UnmountCallback>,
}

impl Default for Html {
    fn default() -> Self {
        Html::new(|_| {
            let callback: Closure<dyn Fn()> = Closure::new(Box::new(|| {}) as Box<dyn Fn()>);
            callback.into_js_value().dyn_into().unwrap()
        })
    }
}

impl Html {
    /// Create Html with a callback function for dynamic content generation
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(web_sys::Element) -> js_sys::Function + 'static,
    {
        Html {
            mount_callback: Rc::new(Closure::wrap(
                Box::new(callback) as Box<dyn Fn(web_sys::Element) -> js_sys::Function>
            )),
            unmount_callback: None,
        }
    }

    /// Mount the HTML into a DOM element
    ///
    /// # Arguments
    /// * `target` - Optional target element (defaults to document body)
    ///
    /// # Returns
    /// * `Result<js_sys::Function, wasm_bindgen::JsValue>` - Ok with callback function if successful, Err with JS error if failed
    pub fn mount(
        &mut self,
        target: Option<&web_sys::Element>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        use web_sys::{Element, window};

        let target_element: Element = if let Some(element) = target {
            element.clone()
        } else {
            let window = window().ok_or("No global window object")?;
            let document = window.document().ok_or("No document object")?;
            document.body().ok_or("No body element")?.into()
        };

        let func: &js_sys::Function = self.mount_callback.as_ref().as_ref().unchecked_ref();
        let result = func.call1(&wasm_bindgen::JsValue::NULL, &target_element.clone().into())?;

        // The result should be a JavaScript function
        let js_function: js_sys::Function = result.dyn_into()?;

        self.unmount_callback = Some(Rc::new(Closure::new(Box::new(move || {
            let _ = js_function.call0(&wasm_bindgen::JsValue::NULL);
        }))));

        Ok(())
    }
}

impl TryFrom<String> for Html {
    type Error = ();

    fn try_from(content: String) -> Result<Self, Self::Error> {
        Ok(Html {
            mount_callback: Rc::new(Closure::new(Box::new(move |element: web_sys::Element| {
                element.set_inner_html(&content);
                let callback: Closure<dyn Fn()> = Closure::new(Box::new(|| {}) as Box<dyn Fn()>);
                callback.into_js_value().dyn_into().unwrap()
            }))),
            unmount_callback: None,
        })
    }
}

impl From<&str> for Html {
    fn from(content: &str) -> Self {
        let owned_content = content.to_string();

        Html {
            mount_callback: Rc::new(Closure::new(Box::new(move |element: web_sys::Element| {
                element.set_inner_html(&owned_content);
                let callback: Closure<dyn Fn()> = Closure::new(Box::new(|| {}) as Box<dyn Fn()>);
                callback.into_js_value().dyn_into().unwrap()
            }))),
            unmount_callback: None,
        }
    }
}

impl std::fmt::Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Ok(temp_element) = document.create_element("div") {
                    let func: &js_sys::Function =
                        self.mount_callback.as_ref().as_ref().unchecked_ref();

                    if func
                        .call1(&wasm_bindgen::JsValue::NULL, &temp_element.clone().into())
                        .is_ok()
                    {
                        return write!(f, "{}", temp_element.inner_html());
                    }
                }
            }
        }

        write!(f, "")
    }
}

/// Universal Apex application that works on both server and client
#[derive(Default)]
pub struct Apex {}

impl Apex {
    /// Create a new Apex application
    pub fn new() -> Self {
        Self {}
    }

    /// Hydrate the client-side application with a component
    pub fn hydrate(self, mut html: Html) -> Result<(), wasm_bindgen::JsValue> {
        let window = window().ok_or("No global window object")?;
        let document = window.document().ok_or("No document object")?;

        let body = document.body().ok_or("No body element")?;
        html.mount(Some(&body))?;

        Ok(())
    }

    pub fn hydrate2(
        f: impl Fn(&HashMap<String, web_sys::Text>, &HashMap<String, web_sys::Element>),
    ) {
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
                    let next_node = comment.next_sibling().expect("next node not found") else {
                        continue;
                    };

                    let text_node = next_node.dyn_ref::<Text>().expect("text node not found");
                    expressions_map.insert(comment_id.clone(), text_node.clone());

                    let next_node = next_node.next_sibling().expect("next node not found") else {
                        continue;
                    };

                    let end_comment = next_node
                        .dyn_ref::<Comment>()
                        .expect("end comment node not found");

                    nodes_to_remove.push(comment.clone());
                    nodes_to_remove.push(end_comment.clone());
                } else if comment_type == "@element" {
                    let next_node = comment.next_sibling().expect("next node not found") else {
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

        f(&expressions_map, &elements_map);
    }
}
