use std::cell::RefCell;
use std::rc::Rc;

/// A reactive signal that holds a value and can notify when changed
#[derive(Debug)]
pub struct Signal<T: Clone> {
    inner: Rc<RefCell<T>>,
}

impl<T: Clone> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(initial_value: T) -> Self {
        Signal {
            inner: Rc::new(RefCell::new(initial_value)),
        }
    }

    /// Get the current value of the signal
    pub fn get(&self) -> T {
        self.inner.borrow().clone()
    }

    /// Set a new value
    pub fn set(&self, new_value: T) {
        *self.inner.borrow_mut() = new_value;
    }

    /// Update the value using a closure
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.inner.borrow_mut();
        updater(&mut *value);
    }
}

impl<T: Clone> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone + std::fmt::Display> std::fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// A computed signal that automatically updates when its dependencies change
pub struct Computed<T: Clone> {
    compute_fn: Rc<dyn Fn() -> T>,
}

impl<T: Clone> Computed<T> {
    /// Create a computed signal from a closure
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self {
            compute_fn: Rc::new(compute),
        }
    }

    /// Get the current computed value
    pub fn get(&self) -> T {
        (self.compute_fn)()
    }
}

/// An effect that runs when signals change
pub struct Effect {
    _effect_fn: Rc<dyn Fn()>,
}

impl Effect {
    /// Create an effect that runs when signals change
    pub fn new<F>(effect: F) -> Self
    where
        F: Fn() + 'static,
    {
        // Run the effect once immediately
        effect();

        Self {
            _effect_fn: Rc::new(effect),
        }
    }
}

/// Trait for values that can be used in templates with reactivity
pub trait Reactive {
    type Value: Clone;

    /// Get the current value
    fn get_value(&self) -> Self::Value;

    /// Check if this is a reactive signal (for template optimization)
    fn is_reactive(&self) -> bool {
        false
    }
}

impl<T: Clone> Reactive for Signal<T> {
    type Value = T;

    fn get_value(&self) -> Self::Value {
        self.get()
    }

    fn is_reactive(&self) -> bool {
        true
    }
}

impl<T: Clone> Reactive for &Signal<T> {
    type Value = T;

    fn get_value(&self) -> Self::Value {
        self.get()
    }

    fn is_reactive(&self) -> bool {
        true
    }
}

// Specific implementations for common types
impl Reactive for String {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.clone()
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &String {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        (*self).clone()
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &str {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.to_string()
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for i32 {
    type Value = i32;

    fn get_value(&self) -> Self::Value {
        *self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &i32 {
    type Value = i32;

    fn get_value(&self) -> Self::Value {
        **self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for u32 {
    type Value = u32;

    fn get_value(&self) -> Self::Value {
        *self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &u32 {
    type Value = u32;

    fn get_value(&self) -> Self::Value {
        **self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for i8 {
    type Value = i8;

    fn get_value(&self) -> Self::Value {
        *self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &i8 {
    type Value = i8;

    fn get_value(&self) -> Self::Value {
        **self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for f32 {
    type Value = f32;

    fn get_value(&self) -> Self::Value {
        *self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &f32 {
    type Value = f32;

    fn get_value(&self) -> Self::Value {
        **self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for f64 {
    type Value = f64;

    fn get_value(&self) -> Self::Value {
        *self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &f64 {
    type Value = f64;

    fn get_value(&self) -> Self::Value {
        **self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for bool {
    type Value = bool;

    fn get_value(&self) -> Self::Value {
        *self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &bool {
    type Value = bool;

    fn get_value(&self) -> Self::Value {
        **self
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for crate::Html {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.as_str().to_string()
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

impl Reactive for &crate::Html {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.as_str().to_string()
    }

    fn is_reactive(&self) -> bool {
        false
    }
}

/// Macro to create a signal with initial value
#[macro_export]
macro_rules! signal {
    ($value:expr) => {
        Signal::new($value)
    };
}

/// Macro to create an effect
#[macro_export]
macro_rules! effect {
    ($body:expr) => {
        Effect::new(|| $body)
    };
}
