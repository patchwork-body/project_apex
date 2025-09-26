#![allow(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};

// Global counters for runtime ID generation
static RUNTIME_TEXT_NODE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static RUNTIME_ELEMENT_COUNTER: AtomicUsize = AtomicUsize::new(0);
static RUNTIME_CONDITIONAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_text_node_counter() -> usize {
    RUNTIME_TEXT_NODE_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn next_element_counter() -> usize {
    RUNTIME_ELEMENT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn next_conditional_counter() -> usize {
    RUNTIME_CONDITIONAL_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn reset_counters() {
    reset_text_node_counter(None);
    reset_element_counter(None);
    reset_conditional_counter(None);
}

pub fn get_text_node_counter() -> usize {
    RUNTIME_TEXT_NODE_COUNTER.load(Ordering::SeqCst)
}

pub fn get_element_counter() -> usize {
    RUNTIME_ELEMENT_COUNTER.load(Ordering::SeqCst)
}

pub fn get_conditional_counter() -> usize {
    RUNTIME_CONDITIONAL_COUNTER.load(Ordering::SeqCst)
}

pub fn reset_text_node_counter(value: Option<usize>) {
    RUNTIME_TEXT_NODE_COUNTER.store(value.unwrap_or(0), Ordering::SeqCst);
}

pub fn reset_element_counter(value: Option<usize>) {
    RUNTIME_ELEMENT_COUNTER.store(value.unwrap_or(0), Ordering::SeqCst);
}

pub fn reset_conditional_counter(value: Option<usize>) {
    RUNTIME_CONDITIONAL_COUNTER.store(value.unwrap_or(0), Ordering::SeqCst);
}
