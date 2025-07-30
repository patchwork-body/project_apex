#![allow(missing_docs)]

mod helpers;

use apex::prelude::*;
use helpers::mount_tmpl;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn plain_text() {
    let tmpl = tmpl! { Hello, world! };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello, world!");
}

#[wasm_bindgen_test]
pub fn text_with_interpolation() {
    let name = "world";
    let tmpl = tmpl! { Hello, {name}! };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello, world!");
}

#[wasm_bindgen_test]
pub fn interpolation_only() {
    let name = "world";
    let tmpl = tmpl! { {name} };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "world");
}

#[wasm_bindgen_test]
pub fn text_with_interpolation_only_and_whitespaces() {
    let name = "world";
    let tmpl = tmpl! { { name } };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "world");
}

#[wasm_bindgen_test]
pub fn text_with_two_interpolations() {
    let first_name = "John";
    let second_name = "Doe";

    let tmpl = tmpl! { Hello { first_name }, and welcome { second_name }! };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello John, and welcome Doe!");
}

#[wasm_bindgen_test]
pub fn empty_div() {
    let tmpl = tmpl! { <div></div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div></div>");
}

#[wasm_bindgen_test]
pub fn self_closing_element() {
    let tmpl = tmpl! { <br> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<br>");
}

#[wasm_bindgen_test]
pub fn div_with_text() {
    let tmpl = tmpl! { <div>Hello, world!</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
pub fn div_with_text_and_interpolation() {
    let name = "world";
    let tmpl = tmpl! { <div>Hello, {name}!</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
pub fn div_with_attrs() {
    let tmpl = tmpl! { <div class="container">Hello, world!</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
pub fn div_with_dynamic_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={class}>Hello, world!</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
pub fn div_with_dynamic_interpolation_in_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={format!("{}-{}", class, 1)}></div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container-1\"></div>");
}

#[wasm_bindgen_test]
pub fn div_with_dynamic_attrs_and_text() {
    let class = "container";
    let name = "world";
    let tmpl = tmpl! { <div class={class}>Hello, {name}!</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
pub fn self_closing_element_with_static_attrs() {
    let tmpl = tmpl! { <br class="test"> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<br class=\"test\">");
}

#[wasm_bindgen_test]
pub fn self_closing_element_with_dynamic_attrs() {
    let class = "test";
    let counter = 1;
    let tmpl = tmpl! { <br class={format!("{class}-{counter}")}> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<br class=\"test-1\">");
}

#[wasm_bindgen_test]
pub fn nested_elements() {
    let tmpl = tmpl! { <div>
        <div>Hello, world!</div>
        <div>Hello, world!</div>
    </div> };

    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div><div>Hello, world!</div><div>Hello, world!</div></div>"
    );
}

#[wasm_bindgen_test]
pub fn several_elements_on_the_same_level() {
    let tmpl = tmpl! {
        <div>Hello, world 1!</div>
        <div>Hello, world 2!</div>
    };

    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div>Hello, world 1!</div><div>Hello, world 2!</div>"
    );
}

#[wasm_bindgen_test]
pub fn trimmed_text_with_newlines() {
    let tmpl = tmpl! { <div>
        Hello, world!
    </div> };

    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
pub fn test_template_with_spaces_between_expressions() {
    let a = "Hello";
    let b = "World";

    let tmpl = tmpl! {
        <p>{a} {b}!</p>
    };

    let (_, get_html) = mount_tmpl(tmpl);
    assert_eq!(get_html(), "<p>Hello World!</p>");
}

#[wasm_bindgen_test]
pub fn simple_signal() {
    let counter = signal!(0);
    let counter_clone = counter.clone();
    let tmpl = tmpl! { <div>{counter.get()}</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>0</div>");

    counter_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1</div>");
}

#[wasm_bindgen_test]
pub fn test_tmpl_renders_signal_with_multiple_signals() {
    let counter1 = signal!(0);
    let counter2 = signal!(0);
    let counter1_clone = counter1.clone();
    let counter2_clone = counter2.clone();
    let tmpl = tmpl! { <div>{counter1.get()} + {counter2.get()} = {counter1.get() + counter2.get()}</div> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>0 + 0 = 0</div>");

    counter1_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1 + 0 = 1</div>");

    counter2_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1 + 1 = 2</div>");
}

#[wasm_bindgen_test]
pub fn test_tmpl_renders_element_with_event_listener() {
    let callback = |_: web_sys::Event| {};
    let tmpl = tmpl! { <button onclick={callback}>Click me</button> };
    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<button>Click me</button>");
}

#[wasm_bindgen_test]
pub fn test_tmpl_renders_element_with_event_listener_with_signal() {
    use apex::wasm_bindgen::JsCast;

    let counter = signal!(0);

    let inc = {
        let counter = counter.clone();

        move |_event: web_sys::Event| {
            counter.update(|counter| counter + 1);
        }
    };

    let counter_clone = counter.clone();

    let tmpl = tmpl! { <button onclick={inc}>Inc {counter.get()}</button> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<button>Inc 0</button>");

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("no global `document` exists");
    let container = document.get_element_by_id(&id).expect("no element with id");
    let button = container
        .query_selector("button")
        .expect("no button found")
        .unwrap();

    let button = button
        .dyn_into::<web_sys::HtmlButtonElement>()
        .expect("not a button");
    button.click();

    assert_eq!(counter_clone.get(), 1);
    assert_eq!(get_html(), "<button>Inc 1</button>");
}

#[wasm_bindgen_test]
pub fn conditional_directive() {
    let tmpl = tmpl! {
        <div>
            {#if true}
                <span>Hello, world!</span>
            {#endif}
        </div>
    };

    let (_, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div><span>Hello, world!</span><span>Some other text</span></div>"
    );
}
