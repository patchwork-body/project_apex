use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static SIGNAL_TRACKER: RefCell<SignalTracker> = RefCell::new(SignalTracker::new());
    static EFFECT_TRACKER: RefCell<EffectTracker> = RefCell::new(EffectTracker::new());
}

struct SignalTracker {
    signal_elements: HashMap<u64, String>, // signal_id -> element_id
    signal_text_nodes: HashMap<u64, (String, u32)>, // signal_id -> (element_id, text_node_index)
    next_signal_id: u64,
    signal_effects: HashMap<u64, Vec<u64>>, // signal_id -> effect_ids
}

impl SignalTracker {
    fn new() -> Self {
        Self {
            signal_elements: HashMap::new(),
            signal_text_nodes: HashMap::new(),
            next_signal_id: 0,
            signal_effects: HashMap::new(),
        }
    }

    fn register_signal_element(&mut self, signal_id: u64, element_id: String) {
        self.signal_elements.insert(signal_id, element_id);
    }

    fn register_signal_text_node(
        &mut self,
        signal_id: u64,
        element_id: String,
        text_node_index: u32,
    ) {
        self.signal_text_nodes
            .insert(signal_id, (element_id, text_node_index));
    }

    fn register_signal_effect(&mut self, signal_id: u64, effect_id: u64) {
        self.signal_effects
            .entry(signal_id)
            .or_insert_with(Vec::new)
            .push(effect_id);
    }

    fn notify_signal_changed(&self, signal_id: u64, _new_value: String) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            use web_sys::*;

            // Check if this is a text node update first (more specific)
            if let Some((element_id, text_node_index)) = self.signal_text_nodes.get(&signal_id) {
                let window = web_sys::window().expect("no global `window` exists");
                let document = window.document().expect("should have a document on window");
                if let Some(element) = document.get_element_by_id(element_id) {
                    if let Some(text_node) = element.child_nodes().item(*text_node_index as u32) {
                        text_node.set_text_content(Some(&_new_value));
                        console::log_1(
                            &format!(
                                "[APEX REACTIVITY] Updated text node {}[{}] with value: {}",
                                element_id, text_node_index, _new_value
                            )
                            .into(),
                        );
                    } else {
                        console::log_1(
                            &format!(
                                "[APEX ERROR] Text node {}[{}] not found in DOM",
                                element_id, text_node_index
                            )
                            .into(),
                        );
                    }
                } else {
                    console::log_1(
                        &format!(
                            "[APEX ERROR] Element '{}' not found in DOM for text node update",
                            element_id
                        )
                        .into(),
                    );
                }
            }
            // Fall back to full element update (legacy behavior)
            else if let Some(element_id) = self.signal_elements.get(&signal_id) {
                let window = web_sys::window().expect("no global `window` exists");
                let document = window.document().expect("should have a document on window");
                if let Some(element) = document.get_element_by_id(element_id) {
                    element.set_inner_html(&_new_value);
                    console::log_1(
                        &format!(
                            "[APEX REACTIVITY] Updated signal element '{}' with value: {}",
                            element_id, _new_value
                        )
                        .into(),
                    );
                } else {
                    console::log_1(
                        &format!(
                            "[APEX ERROR] Signal element '{}' not found in DOM",
                            element_id
                        )
                        .into(),
                    );
                }
            }
        }

        // Trigger all effects that depend on this signal
        if let Some(effect_ids) = self.signal_effects.get(&signal_id) {
            EFFECT_TRACKER.with(|tracker| {
                let mut tracker_ref = tracker.borrow_mut();
                for &effect_id in effect_ids {
                    tracker_ref.trigger_effect(effect_id);
                }
            });
        }
    }

    fn get_next_signal_id(&mut self) -> u64 {
        let id = self.next_signal_id;
        self.next_signal_id += 1;
        id
    }
}

struct EffectTracker {
    effects: HashMap<u64, Box<dyn FnMut()>>,
    next_effect_id: u64,
    current_effect: Option<u64>,
}

impl EffectTracker {
    fn new() -> Self {
        Self {
            effects: HashMap::new(),
            next_effect_id: 0,
            current_effect: None,
        }
    }

    fn create_effect<F>(&mut self, mut effect_fn: F) -> u64
    where
        F: FnMut() + 'static,
    {
        let effect_id = self.next_effect_id;
        self.next_effect_id += 1;

        // Run the effect once to establish dependencies
        self.current_effect = Some(effect_id);
        effect_fn();
        self.current_effect = None;

        // Store the effect for future triggering
        self.effects.insert(effect_id, Box::new(effect_fn));
        effect_id
    }

    fn trigger_effect(&mut self, effect_id: u64) {
        if let Some(effect) = self.effects.get_mut(&effect_id) {
            #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
            {
                web_sys::console::log_1(
                    &format!("[APEX EFFECT] Triggering effect {}", effect_id).into(),
                );
            }
            effect();
        }
    }

    fn get_current_effect(&self) -> Option<u64> {
        self.current_effect
    }
}

/// A reactive signal that holds a value and can notify when changed
pub struct Signal<T: Clone> {
    inner: Rc<RefCell<T>>,
    signal_id: u64,
}

impl<T: Clone + std::fmt::Debug> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("inner", &self.inner)
            .field("signal_id", &self.signal_id)
            .finish()
    }
}

impl<T: Clone> Signal<T> {
    /// Create a new signal with an initial value
    pub fn new(initial_value: T) -> Self {
        let signal_id = SIGNAL_TRACKER.with(|tracker| tracker.borrow_mut().get_next_signal_id());

        Signal {
            inner: Rc::new(RefCell::new(initial_value)),
            signal_id,
        }
    }

    /// Get the current value of the signal and track dependency
    pub fn get(&self) -> T {
        // Track this signal as a dependency if we're inside an effect
        EFFECT_TRACKER.with(|effect_tracker| {
            if let Some(current_effect) = effect_tracker.borrow().get_current_effect() {
                SIGNAL_TRACKER.with(|signal_tracker| {
                    signal_tracker
                        .borrow_mut()
                        .register_signal_effect(self.signal_id, current_effect);
                });
            }
        });

        self.inner.borrow().clone()
    }

    /// Set a new value
    pub fn set(&self, new_value: T)
    where
        T: ToString,
    {
        *self.inner.borrow_mut() = new_value.clone();
        self.notify_change(new_value.to_string());
    }

    /// Update the value using a closure
    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
        T: ToString,
    {
        {
            let mut value = self.inner.borrow_mut();
            updater(&mut *value);
        }
        let new_value = self.get();
        self.notify_change(new_value.to_string());
    }

    /// Get the signal ID for tracking
    pub fn get_signal_id(&self) -> u64 {
        self.signal_id
    }

    /// Register this signal with a DOM element for automatic updates
    pub fn register_element(&self, element_id: String) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            SIGNAL_TRACKER.with(|tracker| {
                tracker
                    .borrow_mut()
                    .register_signal_element(self.get_signal_id(), element_id);
            });
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
        }
    }

    /// Register this signal with a specific text node for automatic updates
    pub fn register_text_node(&self, element_id: String, text_node_index: u32) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            SIGNAL_TRACKER.with(|tracker| {
                tracker.borrow_mut().register_signal_text_node(
                    self.get_signal_id(),
                    element_id,
                    text_node_index,
                );
            });
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
            let _ = text_node_index;
        }
    }

    /// Notify that this signal has changed
    fn notify_change(&self, new_value: String) {
        SIGNAL_TRACKER.with(|tracker| {
            tracker
                .borrow()
                .notify_signal_changed(self.get_signal_id(), new_value);
        });
    }
}

impl<T: Clone> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            inner: Rc::clone(&self.inner),
            signal_id: self.signal_id, // Same signal ID for clones
        }
    }
}

impl<T: Clone + std::fmt::Display> std::fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// An effect that runs when its signal dependencies change
pub struct Effect {
    effect_id: u64,
}

impl Effect {
    /// Create a new effect that runs when its signal dependencies change
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: FnMut() + 'static,
    {
        let effect_id =
            EFFECT_TRACKER.with(|tracker| tracker.borrow_mut().create_effect(effect_fn));

        Effect { effect_id }
    }

    /// Create an effect that updates a DOM element
    #[cfg(feature = "hydrate")]
    pub fn create_dom_effect<F>(element_id: String, mut update_fn: F) -> Self
    where
        F: FnMut() -> String + 'static,
    {
        let effect_fn = move || {
            use web_sys::*;
            let new_value = update_fn();

            let window = web_sys::window().expect("no global `window` exists");
            let document = window.document().expect("should have a document on window");
            if let Some(element) = document.get_element_by_id(&element_id) {
                element.set_inner_html(&new_value);
                console::log_1(
                    &format!(
                        "[APEX EFFECT] Updated DOM element '{}' with value: {}",
                        element_id, new_value
                    )
                    .into(),
                );
            } else {
                console::log_1(
                    &format!(
                        "[APEX ERROR] Effect target element '{}' not found in DOM",
                        element_id
                    )
                    .into(),
                );
            }
        };

        Self::new(effect_fn)
    }

    /// Get the effect ID
    pub fn get_effect_id(&self) -> u64 {
        self.effect_id
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

    /// Get the signal ID if this is a signal
    fn signal_id(&self) -> Option<u64> {
        None
    }

    /// Register this reactive value with a DOM element for automatic updates
    fn register_element(&self, _element_id: String) {
        // Default implementation does nothing for non-reactive values
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
        }
    }

    /// Register this reactive value with a specific text node for automatic updates
    fn register_text_node(&self, _element_id: String, _text_node_index: u32) {
        // Default implementation does nothing for non-reactive values
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
        }
    }

    /// Create an effect that tracks this reactive value and updates a DOM element
    fn create_effect(&self, _element_id: String) -> Option<Effect>
    where
        Self: Clone + 'static,
        Self::Value: ToString,
    {
        None // Default implementation for non-reactive values
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

    fn signal_id(&self) -> Option<u64> {
        Some(self.get_signal_id())
    }

    fn register_element(&self, element_id: String) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            Signal::register_element(self, element_id);
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
        }
    }

    fn register_text_node(&self, element_id: String, text_node_index: u32) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            Signal::register_text_node(self, element_id, text_node_index);
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
            let _ = text_node_index;
        }
    }

    fn create_effect(&self, element_id: String) -> Option<Effect>
    where
        Self: Clone + 'static,
        Self::Value: ToString,
    {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            let signal_clone = self.clone();
            Some(Effect::create_dom_effect(element_id, move || {
                signal_clone.get().to_string()
            }))
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
            None
        }
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

    fn signal_id(&self) -> Option<u64> {
        Some(self.get_signal_id())
    }

    fn register_element(&self, element_id: String) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            Signal::register_element(*self, element_id);
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
        }
    }

    fn register_text_node(&self, element_id: String, text_node_index: u32) {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            Signal::register_text_node(*self, element_id, text_node_index);
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
            let _ = text_node_index;
        }
    }

    fn create_effect(&self, element_id: String) -> Option<Effect>
    where
        Self: Clone + 'static,
        Self::Value: ToString,
    {
        #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
        {
            let signal_clone = (*self).clone();
            Some(Effect::create_dom_effect(element_id, move || {
                signal_clone.get().to_string()
            }))
        }
        #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
        {
            // No-op on non-WASM targets
            let _ = element_id;
            None
        }
    }
}

// Implement Reactive for common types
/// Render a value with effect wrapper if it's reactive
pub fn render_with_effect<T>(value: &T) -> String
where
    T: Reactive,
    T::Value: ToString,
{
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        if value.is_reactive() {
            if let Some(signal_id) = value.signal_id() {
                // Create an effect that will update this DOM element
                let effect_id = EFFECT_TRACKER.with(|tracker| {
                    tracker.borrow_mut().create_effect(move || {
                        // This effect will be triggered when the signal changes
                        // The DOM update logic is handled in signal notification
                    })
                });

                let element_id = format!("apex_effect_{}", effect_id);

                // Register the signal with this element
                SIGNAL_TRACKER.with(|tracker| {
                    tracker
                        .borrow_mut()
                        .register_signal_element(signal_id, element_id.clone());
                });

                // Return the initial value wrapped in an effect container
                let initial_value = value.get_value();
                format!(
                    "<span id=\"{}\">{}</span>",
                    element_id,
                    initial_value.to_string()
                )
            } else {
                // Reactive but no signal ID - just return the value
                value.get_value().to_string()
            }
        } else {
            // Not reactive - just return the value
            value.get_value().to_string()
        }
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    {
        // Server-side or non-WASM: just return the value as string
        value.get_value().to_string()
    }
}

impl Reactive for String {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.clone()
    }
}

impl Reactive for crate::Html {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.as_str().to_string()
    }
}

impl Reactive for &str {
    type Value = String;

    fn get_value(&self) -> Self::Value {
        self.to_string()
    }
}

impl Reactive for i32 {
    type Value = i32;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &i32 {
    type Value = i32;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

// Add implementations for all missing primitive types
impl Reactive for bool {
    type Value = bool;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &bool {
    type Value = bool;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for i8 {
    type Value = i8;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &i8 {
    type Value = i8;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for i16 {
    type Value = i16;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &i16 {
    type Value = i16;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for i64 {
    type Value = i64;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &i64 {
    type Value = i64;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for u8 {
    type Value = u8;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &u8 {
    type Value = u8;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for u16 {
    type Value = u16;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &u16 {
    type Value = u16;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for u32 {
    type Value = u32;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &u32 {
    type Value = u32;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for u64 {
    type Value = u64;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &u64 {
    type Value = u64;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for f32 {
    type Value = f32;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &f32 {
    type Value = f32;

    fn get_value(&self) -> Self::Value {
        **self
    }
}

impl Reactive for f64 {
    type Value = f64;

    fn get_value(&self) -> Self::Value {
        *self
    }
}

impl Reactive for &f64 {
    type Value = f64;

    fn get_value(&self) -> Self::Value {
        **self
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
