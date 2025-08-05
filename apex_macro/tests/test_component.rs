// #![allow(missing_docs)]

// mod helpers;

// use apex::prelude::*;
// use helpers::mount_tmpl;
// use std::rc::Rc;
// use wasm_bindgen_test::*;

// wasm_bindgen_test_configure!(run_in_browser);

// #[wasm_bindgen_test]
// pub fn test_component_macro() {
//     use apex_macro::component;

//     #[component]
//     fn simple_counter() -> Html {
//         tmpl! {
//             <div class="counter">
//                 <h1>Counter Component</h1>
//                 <p>Count: 0</p>
//             </div>
//         }
//     }

//     // The macro should have generated a SimpleCounter struct
//     let tmpl = tmpl! {
//         <SimpleCounter />
//     };

//     let (_, get_html) = mount_tmpl(tmpl);

//     assert_eq!(
//         get_html(),
//         "<div class=\"counter\"><h1>Counter Component</h1><p>Count: 0</p></div>"
//     );
// }

// #[wasm_bindgen_test]
// pub fn test_component_macro_with_interpolation() {
//     use apex_macro::component;

//     #[component]
//     fn greeting_card() -> Html {
//         let name = "Alice";
//         let message = "Welcome!";

//         tmpl! {
//             <div class="greeting">
//                 <h2>{message}</h2>
//                 <p>Hello, {name}!</p>
//             </div>
//         }
//     }

//     let tmpl = tmpl! {
//         <GreetingCard />
//     };

//     let (_, get_html) = mount_tmpl(tmpl);

//     assert_eq!(
//         get_html(),
//         "<div class=\"greeting\"><h2>Welcome!</h2><p>Hello, Alice!</p></div>"
//     );
// }

// #[wasm_bindgen_test]
// pub fn test_component_macro_with_props() {
//     use apex_macro::component;

//     #[component]
//     fn user_card(#[prop] name: String, #[prop] age: u32) -> Html {
//         tmpl! {
//             <div class="user-card">
//                 <h3>{&name}</h3>
//                 <p>Age: {age}</p>
//             </div>
//         }
//     }

//     let user_name = "Bob".to_owned();
//     let user_age = 25;

//     let tmpl = tmpl! {
//         <UserCard name={user_name.clone()} age={user_age} />
//     };

//     let (_, get_html) = mount_tmpl(tmpl);

//     assert_eq!(
//         get_html(),
//         "<div class=\"user-card\"><h3>Bob</h3><p>Age: 25</p></div>"
//     );
// }

// #[wasm_bindgen_test]
// pub fn test_component_macro_with_signal_prop() {
//     use apex::signal::Signal;
//     use apex_macro::component;

//     #[component]
//     fn counter(#[prop] value: Signal<u32>) -> Html {
//         tmpl! {
//             <button>Count: {value.get()}</button>
//         }
//     }

//     let count = Signal::new(42);
//     let count_clone = count.clone();

//     let tmpl = tmpl! {
//         <Counter value={count.clone()} />
//     };

//     let (_, get_html) = mount_tmpl(tmpl);

//     assert_eq!(get_html(), "<button>Count: 42</button>");

//     count_clone.set(100);
//     assert_eq!(get_html(), "<button>Count: 100</button>");
// }

// #[wasm_bindgen_test]
// pub fn test_component_macro_with_signal_as_prop_and_handler() {
//     use apex::signal::Signal;
//     use apex::wasm_bindgen::JsCast;
//     use apex_macro::component;

//     #[component]
//     fn counter(#[prop] value: Signal<u32>) -> Html {
//         let inc = action!(value => {
//             value.update(|v| v + 1);
//         });

//         tmpl! {
//             <button onclick={inc}>Count: {value.get()}</button>
//         }
//     }

//     let count = Signal::new(0);
//     let count_clone = count.clone();

//     let tmpl = tmpl! {
//         <Counter value={count.clone()} />
//     };

//     let (id, get_html) = mount_tmpl(tmpl);

//     assert_eq!(get_html(), "<button>Count: 0</button>");

//     let window = web_sys::window().expect("no global `window` exists");
//     let document = window.document().expect("no global `document` exists");
//     let container = document.get_element_by_id(&id).expect("no element with id");
//     let button = container
//         .query_selector("button")
//         .expect("no button found")
//         .unwrap();

//     let button = button
//         .dyn_into::<web_sys::HtmlButtonElement>()
//         .expect("not a button");
//     button.click();

//     assert_eq!(count_clone.get(), 1);
//     assert_eq!(get_html(), "<button>Count: 1</button>");
// }

// #[wasm_bindgen_test]
// pub fn test_component_macro_with_optional_props() {
//     use apex_macro::component;

//     #[component]
//     fn greeting(
//         #[prop] name: String,
//         #[prop(default = "Hello".to_owned())] prefix: String,
//     ) -> Html {
//         tmpl! {
//             <div class="greeting">
//                 <p>{&prefix} {&name}!</p>
//             </div>
//         }
//     }

//     // Test with all props provided
//     let tmpl1 = tmpl! {
//         <Greeting name={"Alice".to_owned()} prefix={"Hi".to_owned()} />
//     };
//     let (_, get_html1) = mount_tmpl(tmpl1);

//     assert_eq!(
//         get_html1(),
//         "<div class=\"greeting\"><p>Hi Alice!</p></div>"
//     );

//     // Test with only required prop
//     let tmpl2 = tmpl! {
//         <Greeting name={"Bob".to_owned()} />
//     };
//     let (_, get_html2) = mount_tmpl(tmpl2);

//     assert_eq!(
//         get_html2(),
//         "<div class=\"greeting\"><p>Hello Bob!</p></div>"
//     );
// }

// #[wasm_bindgen_test]
// pub fn test_component_with_closure_prop() {
//     use apex::signal::Signal;
//     use apex::wasm_bindgen::JsCast;
//     use apex_macro::component;

//     #[component]
//     fn counter(#[prop] value: Signal<u32>, #[prop] on_inc: Rc<dyn Fn(web_sys::Event)>) -> Html {
//         tmpl! {
//             <button onclick={on_inc}>Count: {value.get()}</button>
//         }
//     }

//     #[component]
//     fn app() -> Html {
//         let count = signal!(0);

//         let inc = {
//             let count = count.clone();

//             Rc::new(move |_event: web_sys::Event| {
//                 count.update(|v| v + 1);
//             })
//         };

//         tmpl! {
//             <Counter value={count.clone()} on_inc={inc.clone()} />
//         }
//     }

//     let tmpl = tmpl! {
//         <App />
//     };

//     let (id, get_html) = mount_tmpl(tmpl);
//     assert_eq!(get_html(), "<button>Count: 0</button>");

//     let window = web_sys::window().expect("no global `window` exists");
//     let document = window.document().expect("no global `document` exists");
//     let container = document.get_element_by_id(&id).expect("no element with id");
//     let button = container
//         .query_selector("button")
//         .expect("no button found")
//         .unwrap();

//     let button = button
//         .dyn_into::<web_sys::HtmlButtonElement>()
//         .expect("not a button");
//     button.click();

//     assert_eq!(get_html(), "<button>Count: 1</button>");
// }

// // #[wasm_bindgen_test]
// // pub fn test_component_with_slots() {
// //     use apex_macro::component;

// //     #[component]
// //     fn card_layout(
// //         #[prop] title: String,
// //         #[slot] header: Html,
// //         #[slot] content: Html,
// //         #[slot(default = tmpl! { <p>Default footer</p> })] footer: Html,
// //     ) -> Html {
// //         tmpl! {
// //             <div class="card">
// //                 <div class="card-header">
// //                     {@header}
// //                 </div>
// //                 <h2>{&title}</h2>
// //                 <div class="card-content">
// //                     {@content}
// //                 </div>
// //                 <div class="card-footer">
// //                     {@footer}
// //                 </div>
// //             </div>
// //         }
// //     }

// //     let tmpl = tmpl! {
// //         <CardLayout title="My Card">
// //             <#header>
// //                 <h1>Custom Header</h1>
// //             </#header>
// //             <#content>
// //                 <p>This is the main content</p>
// //             </#content>
// //         </CardLayout>
// //     };

// //     let (_, get_html) = mount_tmpl(tmpl);

// //     assert_eq!(
// //         get_html(),
// //         "<div class=\"card\"><div class=\"card-header\"><h1>Custom Header</h1></div><h2>My Card</h2><div class=\"card-content\"><p>This is the main content</p></div><div class=\"card-footer\"><p>Default footer</p></div></div>"
// //     );
// // }

// #[wasm_bindgen_test]
// pub fn test_counter_component() {
//     use apex::wasm_bindgen::JsCast;

//     #[component]
//     fn counter() -> Html {
//         let counter = signal!(0);

//         let inc = {
//             let counter = counter.clone();

//             Rc::new(move |_event: web_sys::Event| {
//                 counter.update(|counter| counter + 1);
//             })
//         };

//         let dec = {
//             let counter = counter.clone();

//             Rc::new(move |_event: web_sys::Event| {
//                 counter.update(|counter| counter - 1);
//             })
//         };

//         tmpl! {
//             <div>
//                 <button id="inc" onclick={inc}>Inc</button>
//                 <p class="counter">{counter.get()}</p>
//                 <button id="dec" onclick={dec}>Dec</button>
//             </div>
//         }
//     }

//     let tmpl = tmpl! { <Counter /> };
//     let (id, get_html) = mount_tmpl(tmpl);

//     assert_eq!(
//         get_html(),
//         "<div><button id=\"inc\">Inc</button><p class=\"counter\">0</p><button id=\"dec\">Dec</button></div>"
//     );

//     let window = web_sys::window().expect("no global `window` exists");
//     let document = window.document().expect("no global `document` exists");
//     let container = document.get_element_by_id(&id).expect("no element with id");
//     let button_inc = container
//         .query_selector("#inc")
//         .expect("no button with id inc found")
//         .unwrap();

//     let button_inc = button_inc
//         .dyn_into::<web_sys::HtmlButtonElement>()
//         .expect("not a button");
//     button_inc.click();

//     assert_eq!(
//         get_html(),
//         "<div><button id=\"inc\">Inc</button><p class=\"counter\">1</p><button id=\"dec\">Dec</button></div>"
//     );
// }

// #[wasm_bindgen_test]
// pub fn test_component_with_string_prop_that_passed_as_signal() {
//     use apex_macro::component;

//     #[component]
//     fn echo(#[prop] text: Signal<String>) -> Html {
//         tmpl! {
//             <span>{text.get()}</span>
//         }
//     }

//     let text = signal!("Hello".to_owned());
//     let text_clone = text.clone();

//     let tmpl = tmpl! {
//         <Echo text={text.clone()} />
//     };

//     let (_, get_html) = mount_tmpl(tmpl);

//     assert_eq!(get_html(), "<span>Hello</span>");

//     text_clone.set("World".to_owned());
//     assert_eq!(get_html(), "<span>World</span>");

//     text_clone.set("Hello".to_owned());
//     assert_eq!(get_html(), "<span>Hello</span>");
// }

// #[wasm_bindgen_test]
// pub fn test_component_with_signal_in_conditional_directive() {
//     use apex_macro::component;

//     #[component]
//     fn counter(#[prop] value: Signal<u32>) -> Html {
//         tmpl! {
//             <div>
//                 {#if value.get() > 0}
//                     <p>Value is greater than 0</p>
//                 {#endif}

//                 <p>Value: {value.get()}</p>
//             </div>
//         }
//     }

//     let value = signal!(1);

//     let tmpl = tmpl! {
//         <Counter value={value.clone()} />
//     };

//     let (_, get_html) = mount_tmpl(tmpl);

//     assert_eq!(
//         get_html(),
//         "<div><p>Value is greater than 0</p><p>Value: 1</p></div>"
//     );
// }
