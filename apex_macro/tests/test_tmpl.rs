#![allow(missing_docs)]
#![allow(unused)]

use apex::{Html, signal, tmpl, web_sys};
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

fn mount_tmpl(tmpl: Html) -> impl Fn() -> String {
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

    move || target.inner_html()
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text() {
    let tmpl = tmpl! { Hello, world! };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello, world!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation() {
    let name = "world";
    let tmpl = tmpl! { Hello, {name}! };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello, world!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation_only() {
    let name = "world";
    let tmpl = tmpl! { {name} };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "world");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation_only_and_whitespace() {
    let name = "world";
    let tmpl = tmpl! { { name } };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "world");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_two_interpolations() {
    let first_name = "John";
    let second_name = "Doe";

    let tmpl = tmpl! { Hello { first_name }, and welcome { second_name }! };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello John, and welcome Doe!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_empty_div() {
    let tmpl = tmpl! { <div></div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div></div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_text() {
    let tmpl = tmpl! { <div>Hello, world!</div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_text_and_interpolation() {
    let name = "world";
    let tmpl = tmpl! { <div>Hello, {name}!</div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_attrs() {
    let tmpl = tmpl! { <div class="container">Hello, world!</div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={class}>Hello, world!</div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_interpolation_in_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={format!("{}-{}", class, 1)}></div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container-1\"></div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_attrs_and_text() {
    let class = "container";
    let name = "world";
    let tmpl = tmpl! { <div class={class}>Hello, {name}!</div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_nested_elements() {
    let tmpl = tmpl! { <div>
        <div>Hello, world!</div>
        <div>Hello, world!</div>
    </div> };

    let get_html = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div><div>Hello, world!</div><div>Hello, world!</div></div>"
    );
}

#[wasm_bindgen_test]
fn test_tmpl_renders_several_elements_on_the_same_level() {
    let tmpl = tmpl! {
        <div>Hello, world 1!</div>
        <div>Hello, world 2!</div>
    };

    let get_html = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div>Hello, world 1!</div><div>Hello, world 2!</div>"
    );
}

#[wasm_bindgen_test]
fn test_tmpl_renders_signal() {
    let counter = signal!(0);
    let counter_clone = counter.clone();
    let tmpl = tmpl! { <div>{$counter}</div> };
    let get_html = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>0</div>");

    counter_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1</div>");
}
