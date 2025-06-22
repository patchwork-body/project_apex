# Apex Signals Example

This example demonstrates the signals-based reactive state management system in Apex, similar to what you'd find in Angular or Solid.js.

## Key Features

### 1. Reactive State with Signals

Components can define reactive state using the `#[signal]` attribute:

```rust
#[component]
pub struct Counter {
    #[signal]
    count: Signal<i32>,

    #[prop(default = "Counter")]
    name: String,
}
```

### 2. Automatic Template Subscriptions

Templates automatically detect signals and subscribe to them:

```rust
impl View for Counter {
    fn render(&self) -> Html {
        tmpl! {
            <div class="counter">
                <h1>Reactive {self.name}</h1>
                <p>Count: {self.count}</p>  // Automatically calls .get()
            </div>
        }
    }
}
```

### 3. Event Handlers Update Signals

Event handlers can update signal values, which automatically triggers re-rendering:

```rust
<button onclick={|_| {
    self.count.update(|c| *c += 1);  // Updates signal value
}}>Increment</button>

<button onclick={|_| {
    self.count.set(0);  // Sets signal to specific value
}}>Reset</button>
```

## Signal API

### Creating Signals

```rust
// In component definitions
#[signal]
my_state: Signal<i32>,

// Or manually
let signal = Signal::new(42);
let signal = signal!(42);  // Using macro
```

### Reading Signal Values

```rust
let value = signal.get();           // Get current value
let value = {signal};               // In templates - automatic
```

### Updating Signal Values

```rust
signal.set(new_value);              // Set to specific value
signal.update(|val| *val += 1);     // Update with closure
```

## Running the Example

```bash
cd examples/counter
cargo run
```

Then visit `http://127.0.0.1:3000/counter` to see the reactive counter in action.

## How It Works

1. **Signal Creation**: Signals are created with `Signal::new(initial_value)`
2. **Template Subscription**: The template macro automatically detects signal usage and calls `.get()`
3. **Change Propagation**: When signals change, they notify subscribers (future feature)
4. **Type Safety**: All signal operations are type-safe at compile time

## Future Enhancements

- [ ] Automatic dependency tracking for computed values
- [ ] Effect system for side effects on signal changes
- [ ] Better integration with DOM events
- [ ] Signal-based component lifecycle hooks
