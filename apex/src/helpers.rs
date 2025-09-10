use std::rc::Rc;

/// Generic event handler used in component props
pub type EventHandler<E> = Rc<dyn Fn(E)>;

/// Creates a no-op event handler for any event type
pub fn noop_event<E>() -> EventHandler<E> {
    Rc::new(|_event: E| {})
}
