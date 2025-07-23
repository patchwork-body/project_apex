/// Macro to create event handlers that capture signals and execute closures
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
    // Simplest case - single signal, auto-capture with same name, event as 'event'
    ($signal:ident => $body:block) => {
        {
            let $signal = $signal.clone();
            ::std::rc::Rc::new(move |event: web_sys::Event| $body)
        }
    };

    // Multiple signals, auto-capture with same names, event as 'event'
    ($($signal:ident),+ => $body:block) => {
        {
            $(let $signal = $signal.clone();)+
            ::std::rc::Rc::new(move |event: web_sys::Event| $body)
        }
    };

    // Single signal with custom variable name, event as 'event'
    ($signal:ident as $captured:ident => $body:block) => {
        {
            let $captured = $signal.clone();
            ::std::rc::Rc::new(move |event: web_sys::Event| $body)
        }
    };

    // Multiple signals with custom variable names, event as 'event'
    ($($signal:ident as $captured:ident),+ => $body:block) => {
        {
            $(let $captured = $signal.clone();)+
            ::std::rc::Rc::new(move |event: web_sys::Event| $body)
        }
    };

    // Single signal with custom event parameter name
    ($signal:ident => |$event_param:ident| $body:block) => {
        {
            let $signal = $signal.clone();
            ::std::rc::Rc::new(move |$event_param: web_sys::Event| $body)
        }
    };

    // Multiple signals with custom event parameter name
    ($($signal:ident),+ => |$event_param:ident| $body:block) => {
        {
            $(let $signal = $signal.clone();)+
            ::std::rc::Rc::new(move |$event_param: web_sys::Event| $body)
        }
    };

    // Single signal with custom names for both signal and event
    ($signal:ident as $captured:ident => |$event_param:ident| $body:block) => {
        {
            let $captured = $signal.clone();
            ::std::rc::Rc::new(move |$event_param: web_sys::Event| $body)
        }
    };

    // Multiple signals with custom names and custom event parameter
    ($($signal:ident as $captured:ident),+ => |$event_param:ident| $body:block) => {
        {
            $(let $captured = $signal.clone();)+
            ::std::rc::Rc::new(move |$event_param: web_sys::Event| $body)
        }
    };
}
