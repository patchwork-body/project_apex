# Outlet Template Example

This demonstrates the complete `{#outlet}` directive functionality in templates.

## Usage

```rust
use std::collections::HashMap;
use apex::prelude::*;

// Layout component with outlet
#[component]
pub fn app_layout() {
    tmpl! {
        <html>
            <head>
                <title>My App</title>
            </head>
            <body>
                <nav>
                    <a href="/">Home</a>
                    <a href="/about">About</a>
                </nav>
                <main>
                    {#outlet}
                </main>
                <footer>
                    <p>© 2024 My App</p>
                </footer>
            </body>
        </html>
    }
}

// About page component
#[component]
pub fn about_page() {
    tmpl! {
        <div>
            <h1>About Us</h1>
            <p>This is the about page content.</p>
        </div>
    }
}

// Root route with children (uses hierarchical routing)
#[route(component = AppLayout, path = "/", children = [AboutPageRouteRoute])]
pub fn root_page(_params: HashMap<String, String>) {
    // Root page loader logic
}

// Child route that renders in the outlet
#[route(component = AboutPage, path = "/about")]
pub fn about_page_route(_params: HashMap<String, String>) {
    // About page loader logic
}
```

## How It Works

### 1. Template Parsing
The `{#outlet}` directive is parsed into a new `TmplAst::Outlet` variant:

```rust
// In mod.rs
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TmplAst {
    // ... other variants
    Outlet,
}
```

### 2. Directive Recognition
The parser recognizes `{#outlet}` as a directive in `process_chars_until.rs`:

```rust
let directive_name = parse_directive_name(chars);

if directive_name == "if" {
    // Handle conditional directive
} else if directive_name == "outlet" {
    // Handle outlet directive
    if chars.peek() == Some(&'}') {
        chars.next(); // consume the '}'
    }
    ast.push(TmplAst::Outlet);
}
```

### 3. Code Generation
The outlet generates different code for server vs client in `render_ast.rs`:

```rust
TmplAst::Outlet => {
    // Server-side: Call outlet helper to get child content
    instructions.push(quote! {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(outlet_content) = get_outlet_content(
                &apex::server_context::get_current_request_path().unwrap_or_default()
            ) {
                buffer.push_str(&outlet_content);
            }
        }
    });

    // Client-side: Handle client routing
    expressions.push(quote! {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(outlet_content) = get_outlet_content(
                &apex::client_context::get_current_path().unwrap_or_default()
            ) {
                // Client-side outlet rendering
            }
        }
    });
}
```

### 4. Route Helpers
The `#[route]` macro generates outlet matching helpers:

```rust
// Generated for routes with children
pub fn root_page_outlet_matcher(request_path: &str) -> Option<Box<dyn ApexRoute>> {
    // Matches child routes for the given path
}

pub fn get_outlet_content(request_path: &str) -> Option<String> {
    // Renders the matched child route content
}
```

## Request Flow

When a request comes in for `/about`:

1. **Router**: Hierarchical matching finds root route `/` first (since it has children)
2. **Root Route**: Renders `AppLayout` component with `{#outlet}` directive
3. **Outlet Processing**: `{#outlet}` calls `get_outlet_content("/about")`
4. **Child Matching**: `root_page_outlet_matcher` finds `AboutPageRouteRoute` matches `/about`
5. **Child Rendering**: Renders `AboutPage` component content
6. **Injection**: Child content is injected into the outlet placeholder
7. **Final HTML**: Complete page with layout + child content

## Result

The final rendered HTML combines the layout structure with the child content:

```html
<html>
    <head>
        <title>My App</title>
    </head>
    <body>
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
        </nav>
        <main>
            <!-- Child content rendered here -->
            <div>
                <h1>About Us</h1>
                <p>This is the about page content.</p>
            </div>
        </main>
        <footer>
            <p>© 2024 My App</p>
        </footer>
    </body>
</html>
```

This enables powerful nested layouts with proper separation of concerns between layout and content components.
