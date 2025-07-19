#![allow(missing_docs)]

use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

use crate::{component::generate_component, tmpl::parse_tmpl};

mod component;
pub(crate) mod tmpl;

/// HTML macro that generates Html content from HTML-like syntax with component support
///
/// Usage:
/// ```rust,ignore
/// use apex::{tmpl, Html};
/// use apex::component;
///
/// #[component(tag = "my-counter")]
/// pub struct MyCounter {
///     name: String,
///     count: i32,
/// }
///
/// impl apex::View for MyCounter {
///     fn render(&self) -> Html {
///         tmpl! {
///             <div>
///                 <h1>Hello, {self.name}!</h1>
///                 <p>Count: {self.count}</p>
///             </div>
///         }
///     }
/// }
///
/// let name = "World";
/// let count = 42;
///
/// let html = tmpl! {
///     <div class="container">
///         <h1>Hello, {name}!</h1>
///         <p>Count: {count}</p>
///         <MyCounter name="Counter" count={count} />
///     </div>
/// };
/// ```
#[proc_macro]
pub fn tmpl(input: TokenStream) -> TokenStream {
    match parse_tmpl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Component macro that transforms a function into an Apex component
///
/// Usage without props:
/// ```rust,ignore
/// use apex::{tmpl, Html, component};
///
/// #[component]
/// fn counter() -> Html {
///     tmpl! {
///         <div>Hello World!</div>
///     }
/// }
///
/// // This generates:
/// // struct Counter;
/// //
/// // impl Counter {
/// //     pub fn render(&self) -> apex::Html {
/// //         tmpl! {
/// //             <div>Hello World!</div>
/// //         }
/// //     }
/// // }
/// ```
///
/// Usage with props:
/// ```rust,ignore
/// use apex::{tmpl, Html, component};
///
/// #[component]
/// fn user_card(
///     #[prop] name: &'static str,
///     #[prop] age: u32,
/// ) -> Html {
///     tmpl! {
///         <div class="user-card">
///             <h3>{name}</h3>
///             <p>Age: {age}</p>
///         </div>
///     }
/// }
///
/// // This generates:
/// // struct UserCard {
/// //     pub name: &'static str,
/// //     pub age: u32,
/// // }
/// //
/// // impl UserCard {
/// //     pub fn render(&self) -> apex::Html {
/// //         let name = self.name.clone();
/// //         let age = self.age.clone();
/// //         tmpl! {
/// //             <div class="user-card">
/// //                 <h3>{name}</h3>
/// //                 <p>Age: {age}</p>
/// //             </div>
/// //         }
/// //     }
/// // }
///
/// // Usage in templates:
/// tmpl! {
///     <UserCard name="Alice" age={30} />
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    match generate_component(input_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
