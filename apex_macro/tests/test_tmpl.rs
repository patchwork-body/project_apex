#![allow(missing_docs)]
#![allow(unused)]

use apex::{Html, tmpl, web_sys};
use std::cell::RefCell;
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread_local;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

static INIT: Once = Once::new();

thread_local! {
    static COUNTER: RefCell<AtomicUsize> = const { RefCell::new(AtomicUsize::new(0)) };
}

fn get_unique_id() -> usize {
    COUNTER.with(|counter| {
        let id = counter.borrow().fetch_add(1, Ordering::SeqCst);
        id
    })
}

fn mount_tmpl(tmpl: Html) -> String {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("no global `document` exists");
    let body = document.body().expect("no global `body` exists");
    let target = document
        .create_element("div")
        .expect("no global `div` exists");

    let id = format!("test-container-{}", get_unique_id());
    target.set_id(&id);

    let _ = body.append_child(&target);

    tmpl.mount(Some(&format!("#{id}"))).unwrap();

    target.inner_html()
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text() {
    let tmpl = tmpl! { Hello, world! };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "Hello, world!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation() {
    let name = "world";
    let tmpl = tmpl! { Hello, {name}! };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "Hello, world!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation_only() {
    let name = "world";
    let tmpl = tmpl! { {name} };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "world");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation_only_and_whitespace() {
    let name = "world";
    let tmpl = tmpl! { { name } };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "world");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_two_interpolations() {
    let first_name = "John";
    let second_name = "Doe";

    let tmpl = tmpl! { Hello { first_name }, and welcome { second_name }! };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "Hello John, and welcome Doe!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_empty_div() {
    let tmpl = tmpl! { <div></div> };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "<div></div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_text() {
    let tmpl = tmpl! { <div>Hello, world!</div> };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_text_and_interpolation() {
    let name = "world";
    let tmpl = tmpl! { <div>Hello, {name}!</div> };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_attrs() {
    let tmpl = tmpl! { <div class="container">Hello, world!</div> };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={class}>Hello, world!</div> };
    let html = mount_tmpl(tmpl);

    assert_eq!(html, "<div class=\"container\">Hello, world!</div>");
}
