use std::rc::Rc;

/// Type alias for action handlers created by the `action!` macro
pub type Action = Rc<dyn Fn(web_sys::Event)>;

/// Creates a no-op action that does nothing
pub fn noop_action() -> Action {
    Rc::new(|_event: web_sys::Event| {})
}
