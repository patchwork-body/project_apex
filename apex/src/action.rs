/// Macro to create event handlers that capture signals and execute closures
///
/// Returns an `Action` type (alias for `Rc<dyn Fn(apex::web_sys::Event)>`) that can be used
/// in component props for event handlers.
///
/// Usage examples:
/// ```rust
/// use apex::prelude::*;
///
/// let count = signal!(0);
///
/// // Simplest - auto-captures signal with same name, event available as 'event'
/// let my_action = action!(count => |event| {
///     event.prevent_default();
///     count.update(|c| c + 1);
/// });
///
/// let other = signal!("Hello".to_owned());
///
/// // Multiple signals
/// let my_action = action!(count, other => {
///     count.update(|c| c + 1);
/// });
///
/// // Custom variable names
/// let my_action = action!(count as c => {
///     c.update(|x| x + 1);
/// });
///
/// // Custom event variable name
/// let my_action = action!(count => |e| {
///     e.prevent_default();
///     count.update(|c| c + 1);
/// });
/// ```
#[macro_export]
macro_rules! action {
    // helper: bind signals with optional alias
    (@bind $sig:ident as $alias:ident) => { let $alias = $sig.clone(); };
    (@bind $sig:ident) => { let $sig = $sig.clone(); };

    // Explicit return type, ignore event with wildcard
    ( $( $sig:ident $(as $alias:ident)? ),* @ $ret_ty:ty $(; $( $cap_ident:ident = $cap_expr:expr ),+ )? => |_| $body:block ) => {{
        $( $crate::action!(@bind $sig $(as $alias)?); )*
        $( $( let $cap_ident = $cap_expr; )+ )?
        ::std::rc::Rc::new(move |_ignored: $ret_ty| $body)
    }};

    // Default Event return type, ignore event with wildcard
    ( $( $sig:ident $(as $alias:ident)? ),* $(; $( $cap_ident:ident = $cap_expr:expr ),+ )? => |_| $body:block ) => {{
        $( $crate::action!(@bind $sig $(as $alias)?); )*
        $( $( let $cap_ident = $cap_expr; )+ )?
        ::std::rc::Rc::new(move |_ignored: $crate::web_sys::Event| $body)
    }};

    // General form with explicit event param; optional:
    // - multiple signals with optional aliases
    // - return type with `@ EventTy` (closure takes EventTy)
    // - extra captures `; name = expr, ...`
    // - event param type annotation (casts when return type is Event)
    ( $( $sig:ident $(as $alias:ident)? ),* @ $ret_ty:ty $(; $( $cap_ident:ident = $cap_expr:expr ),+ )? => | $event_param:ident $( : $event_ty:ty )? | $body:block ) => {{
        $( $crate::action!(@bind $sig $(as $alias)?); )*
        $( $( let $cap_ident = $cap_expr; )+ )?
        ::std::rc::Rc::new(move |$event_param: $ret_ty| {
            $( let $event_param: $event_ty = $crate::wasm_bindgen::JsCast::unchecked_into($event_param); )?
            $body
        })
    }};

    // Same as above, default return type apex::web_sys::Event
    ( $( $sig:ident $(as $alias:ident)? ),* $(; $( $cap_ident:ident = $cap_expr:expr ),+ )? => | $event_param:ident $( : $event_ty:ty )? | $body:block ) => {{
        $( $crate::action!(@bind $sig $(as $alias)?); )*
        $( $( let $cap_ident = $cap_expr; )+ )?
        ::std::rc::Rc::new(move |$event_param: $crate::web_sys::Event| {
            $( let $event_param: $event_ty = $crate::wasm_bindgen::JsCast::unchecked_into($event_param); )?
            $body
        })
    }};

    // No explicit event param: default param name `event`, default return type Event
    ( $( $sig:ident $(as $alias:ident)? ),* $(; $( $cap_ident:ident = $cap_expr:expr ),+ )? => $body:block ) => {{
        $( $crate::action!(@bind $sig $(as $alias)?); )*
        $( $( let $cap_ident = $cap_expr; )+ )?
        ::std::rc::Rc::new(move |event: $crate::web_sys::Event| $body)
    }};
}
