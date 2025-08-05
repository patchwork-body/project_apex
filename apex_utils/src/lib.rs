#![allow(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};

// Global counters for runtime ID generation
static RUNTIME_TEXT_NODE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static RUNTIME_ELEMENT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_text_node_counter() -> usize {
    RUNTIME_TEXT_NODE_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn next_element_counter() -> usize {
    RUNTIME_ELEMENT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn reset_counters() {
    RUNTIME_TEXT_NODE_COUNTER.store(0, Ordering::SeqCst);
    RUNTIME_ELEMENT_COUNTER.store(0, Ordering::SeqCst);
}
