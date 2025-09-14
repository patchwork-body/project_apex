use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::fmt;

thread_local! {
    static EFFECTS: RefCell<std::collections::HashMap<usize, Box<dyn Fn()>>> = RefCell::new(std::collections::HashMap::new());
    static EFFECT_COUNTER: RefCell<usize> = const { RefCell::new(0) };
    static CURRENT_EFFECT: RefCell<Option<usize>> = const { RefCell::new(None) };
}

/// A reactive signal holding a value of type T.
#[derive(Clone)]
pub struct Signal<T: 'static + Clone> {
    value: Rc<RefCell<T>>,
    listeners: Rc<RefCell<HashSet<usize>>>,
}

impl<T: 'static + Clone> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            listeners: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn get(&self) -> T {
        // Auto-subscribe the current effect if one is running
        CURRENT_EFFECT.with(|current| {
            if let Some(effect_id) = *current.borrow() {
                self.subscribe_effect(effect_id);
            }
        });

        self.value.borrow().clone()
    }

    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        self.notify();
    }

    pub fn update<F: FnOnce(&T) -> T>(&self, f: F) {
        let new_value = f(&self.value.borrow());
        *self.value.borrow_mut() = new_value;
        self.notify();
    }

    fn notify(&self) {
        // Call all registered effects
        let listeners = self.listeners.borrow().clone();

        for id in listeners {
            EFFECTS.with(|effects| {
                if let Some(effect) = effects.borrow().get(&id) {
                    (effect)();
                }
            });
        }
    }

    pub fn subscribe_effect(&self, effect_id: usize) {
        self.listeners.borrow_mut().insert(effect_id);
    }

    pub fn derive<U: 'static + Clone, F: Fn(&T) -> U + 'static>(&self, f: F) -> Signal<U> {
        let derived = Signal::new(f(&self.value.borrow()));
        let this = self.clone();
        let derived_clone = derived.clone();

        let effect_id = effect(move || {
            let value = f(&this.get());
            derived_clone.set(value);
        });

        run_tracked_effect(effect_id, || {
            run_effect_by_id(effect_id);
        });

        derived
    }
}

impl From<&str> for Signal<String> {
    fn from(s: &str) -> Self {
        Signal::new(s.to_owned())
    }
}

impl From<String> for Signal<String> {
    fn from(s: String) -> Self {
        Signal::new(s)
    }
}

impl<T: Clone + fmt::Display + 'static> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

/// Register an effect that runs whenever any of the accessed signals change.
pub fn effect<F: Fn() + 'static>(f: F) -> usize {
    // Assign a unique id to this effect
    let id = EFFECT_COUNTER.with(|counter| {
        let mut c = counter.borrow_mut();
        *c += 1;
        *c
    });

    EFFECTS.with(|effects| {
        effects.borrow_mut().insert(id, Box::new(f));
    });

    id // Return the effect ID so it can be used with subscribe_effect
}

/// Run an effect with automatic signal subscription tracking
pub fn run_tracked_effect<F: Fn()>(effect_id: usize, f: F) {
    // Set the current effect context
    CURRENT_EFFECT.with(|current| {
        *current.borrow_mut() = Some(effect_id);
    });

    // Run the effect function (this will auto-subscribe to any signals accessed)
    f();

    // Clear the current effect context
    CURRENT_EFFECT.with(|current| {
        *current.borrow_mut() = None;
    });
}

/// Helper function to run an effect by ID (used by the effect! macro)
pub fn run_effect_by_id(effect_id: usize) {
    EFFECTS.with(|effects| {
        if let Some(effect) = effects.borrow().get(&effect_id) {
            (effect)();
        }
    });
}

/// Macro for ergonomic signal creation: signal!(value)
#[macro_export]
macro_rules! signal {
    ($val:expr) => {
        $crate::signal::Signal::new($val)
    };
}

/// Macro for creating effects with automatic signal subscription
#[macro_export]
macro_rules! effect {
    ($body:expr) => {{
        // Create the effect function
        let effect_fn = move || $body;

        // Register the effect and get its ID
        let effect_id = $crate::signal::effect(effect_fn);

        // Run the effect once with tracking to establish initial subscriptions
        $crate::signal::run_tracked_effect(effect_id, || {
            $crate::signal::run_effect_by_id(effect_id);
        });
    }};
}

/// Macro for creating derived signals from other signals
#[macro_export]
macro_rules! derive {
    ( $($sig:ident),+ , $body:block ) => {{
        $(let $sig = $sig.clone();)+
        let derived_signal = Signal::new(Default::default());
        let derived_signal_clone = derived_signal.clone();

        $crate::effect!({
            derived_signal_clone.set($body);
        });

        derived_signal
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn signal_get_set_update() {
        let s = Signal::new(1);
        assert_eq!(s.get(), 1);

        s.set(42);
        assert_eq!(s.get(), 42);

        s.update(|prev| prev + 1);
        assert_eq!(s.get(), 43);
    }

    #[test]
    fn signal_effect_notification() {
        let s = signal!(0);
        let called = Rc::new(Cell::new(0));
        let called_clone = called.clone();

        // Register effect and subscribe
        let id = 1; // Simulate effect id

        EFFECTS.with(|effects| {
            effects.borrow_mut().insert(
                id,
                Box::new(move || {
                    called_clone.set(called_clone.get() + 1);
                }),
            );
        });

        s.subscribe_effect(id);

        // Trigger effect by setting signal
        s.set(10);
        assert_eq!(called.get(), 1);

        s.update(|prev| prev + 1);
        assert_eq!(called.get(), 2);
    }

    #[test]
    fn effect_function_usage() {
        let s = signal!(0);
        let called = Rc::new(Cell::new(0));
        let called_clone = called.clone();

        // Use the effect function properly
        let effect_id = effect(move || {
            called_clone.set(called_clone.get() + 1);
        });

        // Subscribe the effect to the signal
        s.subscribe_effect(effect_id);

        // Trigger effect by setting signal
        s.set(10);
        assert_eq!(called.get(), 1);

        s.update(|prev| prev + 1);
        assert_eq!(called.get(), 2);
    }

    #[test]
    fn effect_reads_signal_value() {
        let s = signal!(42);
        let captured_value = Rc::new(Cell::new(0));
        let captured_clone = captured_value.clone();
        let signal_clone = s.clone();

        // Create an effect that reads the signal value
        let effect_id = effect(move || {
            let current_value = signal_clone.get();
            captured_clone.set(current_value);
        });

        // Subscribe the effect to the signal
        s.subscribe_effect(effect_id);

        // Initially, the effect hasn't run yet
        assert_eq!(captured_value.get(), 0);

        // Trigger effect by setting signal - effect should capture the new value
        s.set(100);
        assert_eq!(captured_value.get(), 100);

        // Update signal again - effect should capture the updated value
        s.update(|prev| prev * 2);
        assert_eq!(captured_value.get(), 200);

        // Set to a different value
        s.set(50);
        assert_eq!(captured_value.get(), 50);
    }

    #[test]
    fn effect_macro_auto_subscribes() {
        let counter = signal!(42);
        let captured_value = Rc::new(Cell::new(0));
        let captured_clone = captured_value.clone();
        let counter_clone = counter.clone();

        // Use the effect! macro - it should automatically subscribe to the signal
        effect!({
            let current_value = counter_clone.get(); // This should auto-subscribe!
            captured_clone.set(current_value);
        });

        // The effect should have run once during initialization
        assert_eq!(captured_value.get(), 42);

        // When we change the signal, the effect should automatically run
        counter.set(100);
        assert_eq!(captured_value.get(), 100);

        // Update signal again - effect should run automatically
        counter.update(|prev| prev * 2);
        assert_eq!(captured_value.get(), 200);

        // Set to a different value
        counter.set(50);
        assert_eq!(captured_value.get(), 50);
    }

    #[test]
    fn effect_macro_multiple_signals() {
        let counter1 = signal!(10);
        let counter2 = signal!(20);
        let sum = Rc::new(Cell::new(0));
        let sum_clone = sum.clone();
        let counter1_clone = counter1.clone();
        let counter2_clone = counter2.clone();

        // Effect that depends on both signals
        effect!({
            let counter1_value = counter1_clone.get(); // Auto-subscribe to counter1
            let counter2_value = counter2_clone.get(); // Auto-subscribe to counter2
            sum_clone.set(counter1_value + counter2_value);
        });

        // Effect should run initially
        assert_eq!(sum.get(), 30); // 10 + 20

        // Changing either signal should trigger the effect
        counter1.set(15);
        assert_eq!(sum.get(), 35); // 15 + 20

        counter2.set(25);
        assert_eq!(sum.get(), 40); // 15 + 25

        // Update both signals
        counter1.update(|prev| prev * 2); // 15 * 2 = 30
        assert_eq!(sum.get(), 55); // 30 + 25

        counter2.update(|prev| prev - 5); // 25 - 5 = 20
        assert_eq!(sum.get(), 50); // 30 + 20
    }

    #[test]
    fn string_signal_get_set_update() {
        let s = signal!("Hello, world!".to_owned());
        assert_eq!(s.get(), "Hello, world!".to_owned());

        s.set("Hello, world! 2".to_owned());
        assert_eq!(s.get(), "Hello, world! 2".to_owned());

        s.update(|prev| format!("{} 3", prev));
        assert_eq!(s.get(), "Hello, world! 2 3".to_owned());
    }

    #[test]
    fn signal_derive() {
        let count = signal!(2);
        let double = count.derive(|v| v * 2);

        assert_eq!(double.get(), 4);

        count.set(10);
        assert_eq!(double.get(), 20);

        count.update(|v| v + 1);
        assert_eq!(double.get(), 22);
    }

    #[test]
    fn derive_macro_single_signal() {
        let base = signal!(10);

        let double = derive!(base, { base.get() * 2 });
        assert_eq!(double.get(), 20);

        base.set(7);
        assert_eq!(double.get(), 14);

        base.update(|v| v + 1);
        assert_eq!(double.get(), 16);
    }

    #[test]
    fn derive_macro_multiple_signals() {
        let a = signal!(3);
        let b = signal!(4);

        let sum = derive!(a, b, { a.get() + b.get() });
        assert_eq!(sum.get(), 7);

        a.set(10);
        assert_eq!(sum.get(), 14);

        b.set(20);
        assert_eq!(sum.get(), 30);

        a.update(|v| v * 2);
        assert_eq!(sum.get(), 40);
    }

    #[test]
    fn derive_macro_string_signal() {
        let s = signal!("foo".to_owned());

        let upper = derive!(s, { s.get().to_uppercase() });
        assert_eq!(upper.get(), "FOO");

        s.set("Bar".to_owned());
        assert_eq!(upper.get(), "BAR");
    }
}
