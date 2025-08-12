/// Server context system for passing data from route loaders to components
use std::any::{Any, TypeId};
use std::collections::HashMap;

thread_local! {
    static SERVER_CONTEXT: std::cell::RefCell<HashMap<TypeId, Box<dyn Any>>> = std::cell::RefCell::new(HashMap::new());
}

/// Set server context data for a specific type
pub fn set_server_context<T: 'static>(data: T) {
    SERVER_CONTEXT.with(|ctx| {
        ctx.borrow_mut().insert(TypeId::of::<T>(), Box::new(data));
    });
}

/// Get server context data for a specific type
pub fn get_server_context<T: 'static + Clone>() -> Option<T> {
    SERVER_CONTEXT.with(|ctx| {
        ctx.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|data| data.downcast_ref::<T>())
            .cloned()
    })
}

/// Clear server context (useful for testing)
pub fn clear_server_context() {
    SERVER_CONTEXT.with(|ctx| {
        ctx.borrow_mut().clear();
    });
}
