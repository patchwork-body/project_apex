# Apex Web Framework

A modern Rust web framework with declarative routing and component-based development.

## Features

- **Declarative Routing**: Define routes using the `#[route]` macro
- **Component System**: Create reusable web components with the `#[component]` macro
- **LoaderResult Pattern**: User-defined loaders that return data or exceptional behavior (redirects, errors)
- **Type-Safe**: Built with Rust's type system for safety and performance
- **Async-First**: Built on top of Tokio and Hyper for high performance

## Quick Start

### Basic Route

```rust
use apex::{route, Apex, ApexRouter};

#[route(path = "/hello")]
pub async fn hello_route() {}
```

### Route with Component

```rust
use apex::{component, route};

#[component(tag = "user-profile")]
pub struct UserProfile {
    #[prop(default = 0)]
    pub user_id: i32,
    pub name: String,
    pub email: String,
}

#[route(
    path = "/profile",
    component = UserProfile
)]
pub async fn profile_route() {}
```

### Route with User-Defined Loader and Component

```rust
use apex::{LoaderResult, HttpRequest, route, component};

// User-defined loader function
async fn user_loader(
    req: HttpRequest,
    context: &AppContext
) -> LoaderResult<UserData> {
    // Load data from database, API, etc.
    match load_user_from_db().await {
        Ok(user) => LoaderResult::ok(user),
        Err(_) => LoaderResult::not_found(),
    }
}

#[route(
    path = "/users/:id",
    loader = user_loader,
    component = UserProfile
)]
pub async fn user_route() {}
```

## LoaderResult Pattern

The `LoaderResult<T>` enum allows loaders to return data or exceptional behavior:

```rust
pub enum LoaderResult<T> {
    Ok(T),                    // Success with data
    Redirect(String),         // Redirect to URL
    NotFound,                 // 404 Not Found
    ServerError(String),      // 500 Server Error
    Response(HttpResponse),   // Custom response
}
```

### Usage Examples

```rust
// Success with data
LoaderResult::ok(user_data)

// Redirect (e.g., authentication required)
LoaderResult::redirect("/login")

// Not found
LoaderResult::not_found()

// Server error
LoaderResult::server_error("Database connection failed")

// Custom response
LoaderResult::response(custom_response)
```

## Route Macro Reference

The `#[route]` macro supports:

- `path` (required): URL path with parameter support
- `loader` (optional): User-defined function returning `LoaderResult<T>`
- `component` (optional): Component struct from `#[component]` macro

### Generated Functions

The macro generates route handler functions that:

1. Call the user-defined loader (if specified)
2. Handle the `LoaderResult`:
   - If `Ok(data)`: render component with data
   - If `Redirect/NotFound/ServerError`: return appropriate HTTP response
3. Return the final `HttpResponse`

## Component Macro Reference

```rust
#[component(tag = "my-component")]
pub struct MyComponent {
    #[prop(default = 0)]
    counter: i32,
    name: String,
}
```

Generated features:

- `new()` constructor with defaults
- `tag_name()` returns the HTML tag
- `render()` returns HTML string
- Setter methods for each field
- `Debug`, `Clone`, `Default` traits

## Examples

See `examples/counter/` for a working demonstration:

```bash
cd examples/counter
cargo run
```

Visit:

- http://127.0.0.1:3000/simple - Simple route
- http://127.0.0.1:3000/counter - Component rendering

## Current Status & Roadmap

### ✅ Implemented

- Basic `#[route]` macro with path and component support
- `#[component]` macro with props and rendering
- `LoaderResult<T>` enum for exceptional behavior
- Generated route handlers compatible with `ApexRouter`

### 🚧 In Development

- User-defined loader integration (type system challenges)
- Path parameter extraction (`:id` syntax)
- Data flow from loaders to components
- Improved error handling and debugging

### 🎯 Future Features

- Nested routing
- Middleware support
- Template engine integration
- SSR/Client-side hydration
- WebSocket support

## Architecture

Built on:

- **Hyper** - HTTP server and client
- **Tokio** - Async runtime
- **Syn/Quote** - Procedural macros
- **HTTP crate** - HTTP types and utilities

Framework structure:

- `apex` - Core framework with routing and HTTP handling
- `apex_macro` - Procedural macros for declarative development
- Examples showing real-world usage patterns

## Contributing

The framework is in active development. Key areas for contribution:

- Loader type system improvements
- Path parameter parsing
- Component data binding
- Documentation and examples

## License

MIT License
