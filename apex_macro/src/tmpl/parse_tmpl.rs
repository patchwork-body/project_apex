use proc_macro::TokenStream;
use quote::quote;
use syn::Result;

use crate::tmpl::{generate_render_parts::*, parse_tmpl_structure::*};

/// Parse HTML-like template syntax for the Apex framework.
///
/// ## Purpose
///
/// This function serves as the **main entry point** and orchestrator for Apex's template macro system.
/// It transforms HTML-like template syntax from the `tmpl!` macro into executable Rust code that
/// generates `apex::Html` objects at runtime. This is the core function that bridges declarative
/// template syntax with type-safe, performant Rust code generation.
///
/// ## SSR/CSR Separation
///
/// The function now supports Server-Side Rendering (SSR) and Client-Side Rendering (CSR) separation:
/// - **Server-side**: Event handlers are omitted from HTML output for clean, static HTML
/// - **Client-side**: Event handlers are attached to DOM elements during hydration
///
/// This approach follows modern web framework patterns where:
/// 1. Server renders initial HTML without interactivity (for SEO and fast page loads)
/// 2. Client-side JavaScript "hydrates" the page by attaching event handlers
///
/// ## Functionality
///
/// The function performs a multi-stage transformation:
///
/// ### 1. Input Processing
/// - Converts the incoming `TokenStream` from the `tmpl!` macro to a parseable string
/// - Prepares the content for structured parsing
///
/// ### 2. Template Structure Parsing
/// - Calls `parse_tmpl_structure()` to parse HTML-like content into structured representations
/// - Handles multiple content types:
///   - **Text**: Plain text content between tags
///   - **Variables**: Rust expressions in `{expression}` syntax
///   - **HTML Elements**: Standard HTML tags like `<div>`, `<span>`, etc.
///   - **Custom Components**: Apex components identified by hyphenated names like `<my-component>`
///
/// ### 3. Code Generation
/// - Calls `generate_render_parts()` to transform parsed content into Rust code tokens
/// - Generates efficient string concatenation code that:
///   - Wraps text in `.to_string()` calls
///   - Converts static variables to `(expression).to_string()` calls
///   - Wraps dynamic variables in spans with unique IDs for DOM updates
///   - Creates component instantiation and rendering code
///   - Produces proper HTML tag generation code
///   - Conditionally registers event listeners based on render context
///   - Generates signal subscriptions for reactive DOM updates
///
/// ### 4. Output Assembly
/// - Combines generated code parts into final `apex::Html` creation
/// - Optimizes for single vs. multiple render parts
/// - Conditionally includes event listener registration based on render context
///
/// ## Template Transformation Examples
///
/// ### Simple Template
/// ```rust,ignore
/// tmpl! { <div>Hello World</div> }
/// ```
/// Becomes:
/// ```rust,ignore
/// apex::Html::new("<div>Hello World</div>".to_string())
/// ```
///
/// ### Template with Event Handlers (Server-side)
/// ```rust,ignore
/// tmpl! { <button onclick={handler}>Click me</button> }
/// ```
/// Server-side output:
/// ```rust,ignore
/// apex::Html::new("<button id=\"apex_element_0\">Click me</button>".to_string())
/// ```
///
/// ### Template with Event Handlers (Client-side)
/// ```rust,ignore
/// tmpl! { <button onclick={handler}>Click me</button> }
/// ```
/// Client-side output:
/// ```rust,ignore
/// {
///     let html = apex::Html::new("<button id=\"apex_element_0\">Click me</button>".to_string());
///     // Event listener registration code
///     { /* web_sys event binding */ }
///     html
/// }
/// ```
///
/// ### Template with Dynamic Variables (Client-side)
/// ```rust,ignore
/// tmpl! { <div>Count: {counter}</div> }
/// ```
/// Client-side output:
/// ```rust,ignore
/// {
///     let html = apex::Html::new(format!("<div>Count: <span id=\"apex_dynamic_var_0\">{}</span></div>", counter.get_value()));
///     // Signal updater registration code
///     { counter.subscribe(|new_value| { /* DOM update */ }) }
///     html
/// }
/// ```
///
/// ## Integration with Apex Architecture
///
/// This function is central to Apex's compile-time template processing approach, which provides:
///
/// - **Performance**: Templates are processed at compile time, eliminating runtime parsing overhead
/// - **Type Safety**: Generated code enables compile-time checking of template variables
/// - **Component Integration**: Seamless mixing of HTML elements and custom Apex components
/// - **SSR/CSR Support**: Proper separation of server and client rendering concerns
/// - **Developer Experience**: HTML-like syntax that's familiar and intuitive
///
/// ## Error Handling
///
/// Propagates errors from:
/// - Template structure parsing (malformed HTML-like syntax)
/// - Code generation (invalid Rust expressions, missing components)
/// - Attribute processing (malformed component attributes)
///
/// ## Parameters
///
/// * `input` - TokenStream from the `tmpl!` macro containing the raw template content
///
/// ## Returns
///
/// * `Ok(TokenStream)` - Generated Rust code that creates an `apex::Html` object
/// * `Err(syn::Error)` - Compilation error if template parsing or code generation fails
///
/// ## Usage Context
///
/// This function is exclusively called by the `tmpl!` procedural macro defined in `lib.rs`:
///
/// ```rust,ignore
/// #[proc_macro]
/// pub fn tmpl(input: TokenStream) -> TokenStream {
///     match parse_tmpl(input) {
///         Ok(tokens) => tokens.into(),
///         Err(err) => err.to_compile_error().into(),
///     }
/// }
/// ```
pub(crate) fn parse_tmpl(input: TokenStream) -> Result<proc_macro2::TokenStream> {
    let input_str = input.to_string();
    let parsed_content = parse_tmpl_structure(&input_str)?;
    let (html_parts, event_parts, updater_parts) = generate_render_parts(&parsed_content)?;

    let generation = quote! {
        {
            #[cfg(feature = "hydrate")]
            {
                // Defer registration until after DOM is updated
                use apex::wasm_bindgen::prelude::*;
                use apex::web_sys::*;

                let callback = Closure::wrap(Box::new(move || {
                    // Register event listeners after the HTML is inserted into the DOM
                    #(#event_parts)*

                    // Register signal updaters for reactive variables
                    #(#updater_parts)*
                }) as Box<dyn FnMut()>);

                // Call the callback to register event listeners and signal updaters
                let window = apex::web_sys::window().expect("no global `window` exists");
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    0
                );

                // Forget the callback to prevent it from being dropped prematurely
                callback.forget();
            }

            apex::Html::new([#(#html_parts),*].join(""))
        }
    };

    Ok(generation)
}
