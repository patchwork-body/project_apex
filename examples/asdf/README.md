# Apex Counter Example

This example demonstrates both server-side rendering and client-side WASM functionality using the Apex framework.

## Features

- **Reactive Counter Component**: Interactive counter with increment/decrement buttons
- **Signal-based State Management**: Uses Apex signals for reactive state updates
- **Universal Rendering**: Works both server-side and client-side
- **WASM Integration**: Client-side interactivity powered by WebAssembly

## Running the Example

### Server-Side Rendering
```bash
# Run the server
cargo run --bin server

# Visit http://127.0.0.1:3000/counter for server-rendered page
# Visit http://127.0.0.1:3000/ for the main counter route
```

### Client-Side WASM

First, install trunk for building WASM applications:
```bash
cargo install trunk
```

Then build and serve the WASM version:
```bash
# Build and serve the WASM app
trunk serve

# Or build for production
trunk build --release
```

Visit http://127.0.0.1:8080 to see the interactive WASM version.

## Architecture

- `src/lib.rs` - Contains the Counter and CounterPage components
- `src/bin/server.rs` - Server-side entry point with routing
- `src/bin/client.rs` - Client-side WASM entry point
- `index.html` - HTML template for WASM builds
- `Cargo.toml` - Dependencies and build configuration

## Component Structure

### Counter Component
- Reactive state using `Signal<i32>`
- Increment/decrement event handlers
- Configurable name prop

### CounterPage Component
- Contains the Counter component
- Reactive page title
- Full HTML page structure with styling

## Development

The example uses conditional compilation to include different dependencies for server vs WASM builds:
- Server builds include tokio, axum, and other server dependencies
- WASM builds include wasm-bindgen and web-sys for browser APIs 