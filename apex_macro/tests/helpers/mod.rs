use apex::prelude::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread_local;

thread_local! {
    static COUNTER: RefCell<AtomicUsize> = const { RefCell::new(AtomicUsize::new(0)) };
}

fn get_unique_id() -> usize {
    COUNTER.with(|counter| {
        let id = counter.borrow().fetch_add(1, Ordering::SeqCst);
        id
    })
}

pub(crate) fn mount_tmpl(mut tmpl: Html) -> (String, impl Fn() -> String) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("no global `document` exists");
    let body = document.body().expect("no global `body` exists");
    let target = document
        .create_element("div")
        .expect("no global `div` exists");

    let id = format!("test-container-{}", get_unique_id());
    target.set_id(&id);

    let _ = body.append_child(&target);

    tmpl.mount(Some(&target)).unwrap();

    (id, move || target.inner_html())
}
