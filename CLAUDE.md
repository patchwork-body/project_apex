# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Apex is a modern Rust web framework that supports both server-side and WebAssembly (WASM) client-side development. It features declarative routing with the `#[route]` macro, component-based development with `#[component]` macro, and a LoaderResult pattern for handling data loading and exceptional behavior (redirects, errors).

## Architecture

The project uses a Cargo workspace with these core crates:

- **`apex/`** - Core framework with dual-target support (server via Hyper/Tokio, WASM client via web-sys/wasm-bindgen)
- **`apex_macro/`** - Procedural macros for `#[component]` and `tmpl!` template parsing
- **`apex_utils/`** - Shared utilities between crates
- **`examples/`** - Example applications demonstrating framework features

Key dependencies:

- Server-side: Hyper, Tokio for HTTP handling
- Client-side: wasm-bindgen, web-sys, js-sys for DOM manipulation
- Macro development: syn, quote, proc-macro2

## Development Commands

### Building and Testing

```bash
# Build entire workspace
cargo build

# Build for WASM target 
cargo build --target wasm32-unknown-unknown

# Run tests (server-side)
cargo test

# Run WASM tests for macros
cd apex_macro && cargo test --target wasm32-unknown-unknown

# Build specific examples
cd examples/calculator && cargo build --target wasm32-unknown-unknown
```

### Running Examples

Examples use Trunk for WASM build and serve:

```bash
# Install trunk if needed
cargo install trunk

# Run calculator example
cd examples/calculator && trunk serve

# Run asdf example
cd examples/asdf && trunk serve
```

Server binds to 127.0.0.1:8080 by default (configurable in Trunk.toml).

### Code Quality

Workspace enforces strict linting:

- `unsafe_code = "forbid"` - No unsafe code allowed
- Comprehensive Clippy lints for code quality
- Missing docs warnings enabled

## Key Patterns

### Component Macro Usage

```rust
#[component]
pub fn my_component(name: String, count: i32) -> impl IntoResponse {
    tmpl! {
        <div class="component">
            <h1>{name}</h1>
            <p>Count: {count}</p>
        </div>
    }
}
```

### Template Parsing

The `tmpl!` macro parses HTML-like syntax into DOM operations. It handles:

- Dynamic expressions with `{variable}` syntax
- Conditional directives
- Element attributes and text content
- Slot-based component composition

### WASM/Server Dual Targeting

Code must be compatible with both server (`not(target_arch = "wasm32")`) and WASM targets. Use feature flags and conditional compilation as needed.

## File Structure Notes

- `apex_macro/src/tmpl/parse_tmpl_into_ast/` contains the core template parsing logic
- `apex_macro/src/component/` handles component macro generation
- `apex/src/` contains the runtime framework code with separate server/client modules
- Examples in `examples/` demonstrate real-world usage patterns

## Important Context

The framework is in active development with ongoing work on:

- Loader type system integration
- Path parameter extraction (`:id` syntax)
- Data flow between loaders and components
- Template engine improvements

When working on macro code, be aware of the complex AST parsing in the `tmpl` module and ensure changes maintain compatibility with both server and WASM compilation targets.
