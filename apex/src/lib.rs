#![allow(missing_docs)]
use std::rc::Rc;

pub mod prelude;

pub use bytes;
pub use js_sys;
pub use wasm_bindgen;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
pub use web_sys;

pub mod signal;

/// Trait that defines the view layer for components
///
/// Components must implement this trait to provide their HTML rendering logic
pub trait View {
    /// Render the component to Html
    ///
    /// This method should return the complete HTML representation of the component
    fn render(&self) -> Html;
}

type HtmlCallback = Rc<Closure<dyn Fn(web_sys::Element)>>;

/// Represents rendered HTML content
///
/// This type wraps HTML strings and provides a safe way to handle HTML content
#[derive(Debug, Clone)]
pub struct Html {
    callback: HtmlCallback,
}

impl Default for Html {
    fn default() -> Self {
        Html::new(|_| {})
    }
}

impl Html {
    /// Create Html with a callback function for dynamic content generation
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(web_sys::Element) + 'static,
    {
        Html {
            callback: Rc::new(Closure::wrap(
                Box::new(callback) as Box<dyn Fn(web_sys::Element)>
            )),
        }
    }

    /// Mount the HTML into a DOM element
    ///
    /// # Arguments
    /// * `target` - Optional target element (defaults to document body)
    ///
    /// # Returns
    /// * `Result<(), wasm_bindgen::JsValue>` - Ok if successful, Err with JS error if failed
    pub fn mount(&self, target: Option<&web_sys::Element>) -> Result<(), wasm_bindgen::JsValue> {
        use web_sys::{Element, window};

        let target_element: Element = if let Some(element) = target {
            element.clone()
        } else {
            let window = window().ok_or("No global window object")?;
            let document = window.document().ok_or("No document object")?;
            document.body().ok_or("No body element")?.into()
        };

        let func: &js_sys::Function = self.callback.as_ref().as_ref().unchecked_ref();
        func.call1(&wasm_bindgen::JsValue::NULL, &target_element.clone().into())?;

        Ok(())
    }

    /// Update the mounted HTML by re-executing the callback
    /// This is useful for reactive updates when state changes
    pub fn update(&self, target: Option<&web_sys::Element>) -> Result<(), wasm_bindgen::JsValue> {
        self.mount(target)
    }
}

impl TryFrom<String> for Html {
    type Error = ();

    fn try_from(content: String) -> Result<Self, Self::Error> {
        Ok(Html {
            callback: Rc::new(Closure::new(Box::new(move |element: web_sys::Element| {
                element.set_inner_html(&content);
            }))),
        })
    }
}

impl From<&str> for Html {
    fn from(content: &str) -> Self {
        let owned_content = content.to_string();
        Html {
            callback: Rc::new(Closure::new(Box::new(move |element: web_sys::Element| {
                element.set_inner_html(&owned_content);
            }))),
        }
    }
}

impl std::fmt::Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use web_sys::window;

        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Ok(temp_element) = document.create_element("div") {
                    let func: &js_sys::Function = self.callback.as_ref().as_ref().unchecked_ref();
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
    pub fn hydrate<T: View>(self, component: T) -> Result<(), wasm_bindgen::JsValue> {
        use web_sys::window;

        let window = window().ok_or("No global window object")?;
        let document = window.document().ok_or("No document object")?;

        let body = document.body().ok_or("No body element")?;

        let html = component.render();
        html.mount(Some(&body))?;

        Ok(())
    }
}
