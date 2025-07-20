#![allow(missing_docs)]
#![allow(unused)]

use apex::{Html, signal, tmpl, web_sys};
use std::sync::Once;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread_local;
use std::{cell::RefCell, rc::Rc};
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

fn mount_tmpl(tmpl: Html) -> (String, impl Fn() -> String) {
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

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text() {
    let tmpl = tmpl! { Hello, world! };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello, world!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation() {
    let name = "world";
    let tmpl = tmpl! { Hello, {name}! };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello, world!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation_only() {
    let name = "world";
    let tmpl = tmpl! { {name} };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "world");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_interpolation_only_and_whitespace() {
    let name = "world";
    let tmpl = tmpl! { { name } };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "world");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_plain_text_with_two_interpolations() {
    let first_name = "John";
    let second_name = "Doe";

    let tmpl = tmpl! { Hello { first_name }, and welcome { second_name }! };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "Hello John, and welcome Doe!");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_empty_div() {
    let tmpl = tmpl! { <div></div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div></div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_self_closing_element() {
    let tmpl = tmpl! { <br> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<br>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_text() {
    let tmpl = tmpl! { <div>Hello, world!</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_text_and_interpolation() {
    let name = "world";
    let tmpl = tmpl! { <div>Hello, {name}!</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_attrs() {
    let tmpl = tmpl! { <div class="container">Hello, world!</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={class}>Hello, world!</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_interpolation_in_attrs() {
    let class = "container";
    let tmpl = tmpl! { <div class={format!("{}-{}", class, 1)}></div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container-1\"></div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_div_with_dynamic_attrs_and_text() {
    let class = "container";
    let name = "world";
    let tmpl = tmpl! { <div class={class}>Hello, {name}!</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div class=\"container\">Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_self_closing_element_with_static_attrs() {
    let tmpl = tmpl! { <br class="test"> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<br class=\"test\">");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_self_closing_element_with_dynamic_attrs() {
    let class = "test";
    let counter = 1;
    let tmpl = tmpl! { <br class={format!("{class}-{counter}")}> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<br class=\"test-1\">");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_nested_elements() {
    let tmpl = tmpl! { <div>
        <div>Hello, world!</div>
        <div>Hello, world!</div>
    </div> };

    let (id, get_html) = mount_tmpl(tmpl);

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

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div>Hello, world 1!</div><div>Hello, world 2!</div>"
    );
}

#[wasm_bindgen_test]
fn test_tmpl_renders_trimmed_text_with_newlines() {
    let tmpl = tmpl! { <div>
        Hello, world!
    </div> };

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>Hello, world!</div>");
}

#[wasm_bindgen_test]
fn test_template_with_spaces_between_expressions() {
    let a = "Hello";
    let b = "World";

    let tmpl = tmpl! {
        <p>{a} {b}!</p>
    };

    let (id, get_html) = mount_tmpl(tmpl);
    assert_eq!(get_html(), "<p>Hello World!</p>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_signal() {
    let counter = signal!(0);
    let counter_clone = counter.clone();
    let tmpl = tmpl! { <div>{$counter}</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>0</div>");

    counter_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_signal_with_multiple_signals() {
    let counter1 = signal!(0);
    let counter2 = signal!(0);
    let counter1_clone = counter1.clone();
    let counter2_clone = counter2.clone();
    let tmpl = tmpl! { <div>{$counter1} + {$counter2} = {($counter1 + $counter2)}</div> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<div>0 + 0 = 0</div>");

    counter1_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1 + 0 = 1</div>");

    counter2_clone.set(1);
    let html = get_html();

    assert_eq!(html, "<div>1 + 1 = 2</div>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_element_with_event_listener() {
    let tmpl = tmpl! { <button onclick={() => {}}>Click me</button> };
    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<button>Click me</button>");
}

#[wasm_bindgen_test]
fn test_tmpl_renders_element_with_event_listener_with_signal() {
    use apex::wasm_bindgen::JsCast;

    let counter = signal!(0);

    let inc = {
        let counter = counter.clone();

        move || {
            counter.update(|counter| counter + 1);
        }
    };

    let counter_clone = counter.clone();

    let tmpl = tmpl! { <button onclick={inc}>Inc {$counter}</button> };
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
fn test_new_component_macro() {
    use apex_macro::component;

    #[component]
    fn simple_counter() -> Html {
        tmpl! {
            <div class="counter">
                <h1>Counter Component</h1>
                <p>Count: 0</p>
            </div>
        }
    }

    // The macro should have generated a SimpleCounter struct
    let tmpl = tmpl! {
        <SimpleCounter />
    };

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div class=\"counter\"><h1>Counter Component</h1><p>Count: 0</p></div>"
    );
}

#[wasm_bindgen_test]
fn test_component_macro_with_interpolation() {
    use apex_macro::component;

    #[component]
    fn greeting_card() -> Html {
        let name = "Alice";
        let message = "Welcome!";

        tmpl! {
            <div class="greeting">
                <h2>{message}</h2>
                <p>Hello, {name}!</p>
            </div>
        }
    }

    let tmpl = tmpl! {
        <GreetingCard />
    };

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div class=\"greeting\"><h2>Welcome!</h2><p>Hello, Alice!</p></div>"
    );
}

#[wasm_bindgen_test]
fn test_component_macro_with_props() {
    use apex_macro::component;

    #[component]
    fn user_card(#[prop] name: String, #[prop] age: u32) -> Html {
        tmpl! {
            <div class="user-card">
                <h3>{&name}</h3>
                <p>Age: {age}</p>
            </div>
        }
    }

    let user_name = "Bob".to_owned();
    let user_age = 25;

    let tmpl = tmpl! {
        <UserCard name={user_name.clone()} age={user_age} />
    };

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(
        get_html(),
        "<div class=\"user-card\"><h3>Bob</h3><p>Age: 25</p></div>"
    );
}

#[wasm_bindgen_test]
fn test_component_macro_with_signal_prop() {
    use apex::signal::Signal;
    use apex_macro::component;

    #[component]
    fn counter(#[prop] value: Signal<u32>) -> Html {
        tmpl! {
            <button>Count: {$value}</button>
        }
    }

    let count = Signal::new(42);
    let count_clone = count.clone();

    let tmpl = tmpl! {
        <Counter value={count.clone()} />
    };

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<button>Count: 42</button>");

    count_clone.set(100);
    assert_eq!(get_html(), "<button>Count: 100</button>");
}

#[wasm_bindgen_test]
fn test_component_macro_with_signal_as_prop_and_handler() {
    use apex::signal::Signal;
    use apex::wasm_bindgen::JsCast;
    use apex_macro::component;

    #[component]
    fn counter(#[prop] value: Signal<u32>) -> Html {
        let inc = {
            let value = value.clone();

            move || {
                value.update(|v| v + 1);
            }
        };

        tmpl! {
            <button onclick={inc}>Count: {$value}</button>
        }
    }

    let count = Signal::new(0);
    let count_clone = count.clone();

    let tmpl = tmpl! {
        <Counter value={count.clone()} />
    };

    let (id, get_html) = mount_tmpl(tmpl);

    assert_eq!(get_html(), "<button>Count: 0</button>");

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

    assert_eq!(count_clone.get(), 1);
    assert_eq!(get_html(), "<button>Count: 1</button>");
}

#[wasm_bindgen_test]
fn test_component_macro_with_optional_props() {
    use apex_macro::component;

    #[component]
    fn greeting(
        #[prop] name: String,
        #[prop(default = "Hello".to_owned())] prefix: String,
    ) -> Html {
        tmpl! {
            <div class="greeting">
                <p>{&prefix} {&name}!</p>
            </div>
        }
    }

    // Test with all props provided
    let tmpl1 = tmpl! {
        <Greeting name={"Alice".to_owned()} prefix={"Hi".to_owned()} />
    };
    let (id1, get_html1) = mount_tmpl(tmpl1);
    assert_eq!(
        get_html1(),
        "<div class=\"greeting\"><p>Hi Alice!</p></div>"
    );

    // Test with only required prop
    let tmpl2 = tmpl! {
        <Greeting name={"Bob".to_owned()} />
    };
    let (id2, get_html2) = mount_tmpl(tmpl2);
    assert_eq!(
        get_html2(),
        "<div class=\"greeting\"><p>Hello Bob!</p></div>"
    );
}

#[wasm_bindgen_test]
fn test_component_with_closure_prop() {
    use apex::signal::Signal;
    use apex::wasm_bindgen::JsCast;
    use apex_macro::component;

    #[component]
    fn counter(#[prop] value: Signal<u32>, #[prop] on_inc: Rc<dyn Fn()>) -> Html {
        tmpl! {
            <button onclick={on_inc}>Count: {$value}</button>
        }
    }

    #[component]
    fn app() -> Html {
        let count = signal!(0);

        let inc = {
            let count = count.clone();

            Rc::new(move || {
                count.update(|v| v + 1);
            })
        };

        tmpl! {
            <Counter value={count.clone()} on_inc={inc.clone()} />
        }
    }

    let tmpl = tmpl! {
        <App />
    };

    let (id, get_html) = mount_tmpl(tmpl);
    assert_eq!(get_html(), "<button>Count: 0</button>");

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

    assert_eq!(get_html(), "<button>Count: 1</button>");
}
