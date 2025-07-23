/// Macro to create event handlers that capture signals and execute closures
///
/// Usage examples:
/// ```rust
/// // Simplest - auto-captures signal with same name
/// let my_action = action!(count => { count.update(|c| c + 1); });
///
/// // Multiple signals
/// let my_action = action!(count, other => { count.update(|c| c + 1); });
///
/// // Custom variable names
/// let my_action = action!(count as c => { c.update(|x| x + 1); });
/// ```
#[macro_export]
macro_rules! action {
    // Simplest case - single signal, auto-capture with same name
    ($signal:ident => $body:block) => {
        {
            let $signal = $signal.clone();
            ::std::rc::Rc::new(move || $body)
        }
    };

    // Multiple signals, auto-capture with same names
    ($($signal:ident),+ => $body:block) => {
        {
            $(let $signal = $signal.clone();)+
            ::std::rc::Rc::new(move || $body)
        }
    };

    // Single signal with custom variable name
    ($signal:ident as $captured:ident => $body:block) => {
        {
            let $captured = $signal.clone();
            ::std::rc::Rc::new(move || $body)
        }
    };

    // Multiple signals with custom variable names
    ($($signal:ident as $captured:ident),+ => $body:block) => {
        {
            $(let $captured = $signal.clone();)+
            ::std::rc::Rc::new(move || $body)
        }
    };

    // Legacy support - explicit closure parameter syntax
    ($signal:ident => |$captured:ident| $body:block) => {
        {
            let $captured = $signal.clone();
            ::std::rc::Rc::new(move || $body)
        }
    };

    // Legacy support - multiple signals with explicit parameters
    ($($signal:ident),+ => |$($captured:ident),+| $body:block) => {
        {
            $(let $captured = $signal.clone();)+
            ::std::rc::Rc::new(move || $body)
        }
    };
}
