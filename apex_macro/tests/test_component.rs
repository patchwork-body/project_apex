#![allow(missing_docs)]
#![allow(dead_code)]
use apex::{Html, Signal, View, component, tmpl};

#[test]
fn test_primitive_component() {
    #[component]
    struct MyCounter;

    impl View for MyCounter {
        fn render(&self) -> Html {
            tmpl! {
                <div>MyCounter</div>
            }
        }
    }

    assert_eq!(
        MyCounter::new().render().to_string(),
        "<div>MyCounter</div>"
    );
}

#[test]
fn test_component_with_attributes() {
    #[component]
    struct MyComponent {
        name: String,
        count: i32,
    }

    impl View for MyComponent {
        fn render(&self) -> Html {
            tmpl! {
                <h1>{self.name}</h1>
                <p>{self.count}</p>
            }
        }
    }

    let mut counter = MyComponent::new();
    counter.set_name("MyCounter".to_owned());
    counter.set_count(10);

    assert_eq!(counter.render().to_string(), "<h1>MyCounter</h1><p>10</p>");
}

#[test]
fn test_component_with_other_component() {
    #[component]
    struct Awesome;

    impl View for Awesome {
        fn render(&self) -> Html {
            tmpl! {
                <div>Awesome</div>
            }
        }
    }

    #[component]
    struct MyComponent;

    impl View for MyComponent {
        fn render(&self) -> Html {
            tmpl! {
                <Awesome />
                <div>My Counter</div>
            }
        }
    }

    assert_eq!(
        MyComponent::new().render().to_string(),
        "<div>Awesome</div><div>My Counter</div>"
    );
}

#[test]
fn test_component_with_boolean_attributes() {
    #[component]
    struct ToggleComponent {
        enabled: bool,
        visible: bool,
    }

    impl View for ToggleComponent {
        fn render(&self) -> Html {
            tmpl! {
                <div>
                    <span>Enabled: {self.enabled}</span>
                    <span>Visible: {self.visible}</span>
                </div>
            }
        }
    }

    let mut toggle = ToggleComponent::new();
    toggle.set_enabled(true);
    toggle.set_visible(false);

    assert_eq!(
        toggle.render().to_string(),
        "<div><span>Enabled: true</span><span>Visible:\nfalse</span></div>"
    );
}

#[test]
fn test_component_with_numeric_attributes() {
    #[component]
    struct StatsComponent {
        score: f64,
        level: u32,
        health: i8,
    }

    impl View for StatsComponent {
        fn render(&self) -> Html {
            tmpl! {
                <div class="stats">
                    <div>Score: {self.score}</div>
                    <div>Level: {self.level}</div>
                    <div>Health: {self.health}</div>
                </div>
            }
        }
    }

    let mut stats = StatsComponent::new();
    stats.set_score(95.5);
    stats.set_level(12);
    stats.set_health(85);

    assert_eq!(
        stats.render().to_string(),
        r#"<div class="stats"><div>Score: 95.5</div><div>Level:
12</div><div>Health: 85</div></div>"#
    );
}

#[test]
fn test_component_with_conditional_rendering() {
    #[component]
    struct ConditionalComponent {
        show_content: bool,
        message: String,
    }

    impl View for ConditionalComponent {
        fn render(&self) -> Html {
            let show_content = self.show_content;
            let message = self.message.clone();

            tmpl! {
                <div>
                    {if show_content {
                        tmpl! { <p>{message}</p> }.to_string()
                    } else {
                        tmpl! { <p>Content hidden</p> }.to_string()
                    }}
                </div>
            }
        }
    }

    let mut component = ConditionalComponent::new();
    component.set_show_content(false);
    component.set_message("Hello World".to_owned());

    assert_eq!(
        component.render().to_string(),
        "<div>\n<p>Content hidden</p></div>"
    );

    component.set_show_content(true);
    assert_eq!(
        component.render().to_string(),
        "<div>\n<p>Hello World</p></div>"
    );
}

#[test]
fn test_nested_components_with_attributes() {
    #[component]
    struct Button {
        text: String,
        disabled: bool,
    }

    impl View for Button {
        fn render(&self) -> Html {
            let text = self.text.clone();
            let disabled = self.disabled;

            tmpl! {
                <button disabled={disabled.to_string()}>{text}</button>
            }
        }
    }

    #[component]
    struct Card {
        title: String,
        content: String,
    }

    impl View for Card {
        fn render(&self) -> Html {
            let title = self.title.clone();
            let content = self.content.clone();

            tmpl! {
                <div class="card">
                    <h2>{title}</h2>
                    <p>{content}</p>
                    <Button />
                </div>
            }
        }
    }

    let mut card = Card::new();
    card.set_title("Test Card".to_owned());
    card.set_content("This is test content".to_owned());

    assert_eq!(
        card.render().to_string(),
        r#"<div class="card"><h2>Test Card</h2><p>This is test content</p><button disabled="false"></button></div>"#
    );
}

#[test]
fn test_component_with_multiple_string_attributes() {
    #[component]
    struct UserProfile {
        name: String,
        email: String,
        bio: String,
    }

    impl View for UserProfile {
        fn render(&self) -> Html {
            let name = self.name.clone();
            let email = self.email.clone();
            let bio = self.bio.clone();

            tmpl! {
                <div class="profile">
                    <h1>{name}</h1>
                    <p>Email: {email}</p>
                    <p>Bio: {bio}</p>
                </div>
            }
        }
    }

    let mut profile = UserProfile::new();
    profile.set_name("John Doe".to_owned());
    profile.set_email("john@example.com".to_owned());
    profile.set_bio("Software developer".to_owned());

    let expected = r#"<div class="profile"><h1>John Doe</h1><p>Email: john@example.com</p><p>Bio: Software developer</p></div>"#;
    assert_eq!(profile.render().to_string(), expected);
}

#[test]
fn test_component_with_mixed_types() {
    #[component]
    struct MixedComponent {
        id: i32,
        name: String,
        active: bool,
        score: f32,
    }

    impl View for MixedComponent {
        fn render(&self) -> Html {
            let id = self.id.clone();
            let name = self.name.clone();
            let active = self.active.clone();
            let score = self.score.clone();

            tmpl! {
                <div>
                    <span>ID: {id}</span>
                    <span>Name: {name}</span>
                    <span>Active: {active}</span>
                    <span>Score: {score}</span>
                </div>
            }
        }
    }

    let mut component = MixedComponent::new();
    component.set_id(123);
    component.set_name("Test".to_owned());
    component.set_active(true);
    component.set_score(98.7);

    assert_eq!(
        component.render().to_string(),
        "<div><span>ID: 123</span><span>Name: Test</span><span>Active: true</span><span>Score: 98.7</span></div>"
    );
}

#[test]
fn test_empty_component() {
    #[component]
    struct EmptyComponent;

    impl View for EmptyComponent {
        fn render(&self) -> Html {
            tmpl! {
                <div></div>
            }
        }
    }

    assert_eq!(EmptyComponent::new().render().to_string(), "<div></div>");
}

#[test]
fn test_component_default_values() {
    #[component]
    struct DefaultComponent {
        name: String,
        count: i32,
        active: bool,
    }

    impl View for DefaultComponent {
        fn render(&self) -> Html {
            let name = self.name.clone();
            let count = self.count.clone();
            let active = self.active.clone();

            tmpl! {
                <div>
                    <span>Name: {name}</span>
                    <span>Count: {count}</span>
                    <span>Active: {active}</span>
                </div>
            }
        }
    }

    let component = DefaultComponent::new();

    assert_eq!(
        component.render().to_string(),
        "<div><span>Name: </span><span>Count: 0</span><span>Active: false</span></div>"
    );
}

#[test]
fn test_component_tag_name() {
    #[component]
    struct CustomTagComponent;

    impl View for CustomTagComponent {
        fn render(&self) -> Html {
            tmpl! {
                <div>Custom Tag</div>
            }
        }
    }

    assert_eq!(CustomTagComponent::tag_name(), "CustomTagComponent");
}

#[test]
fn test_component_from_attributes() {
    use std::collections::HashMap;

    #[component]
    struct AttributeComponent {
        name: String,
        count: i32,
    }

    impl View for AttributeComponent {
        fn render(&self) -> Html {
            let name = self.name.clone();
            let count = self.count.clone();

            tmpl! {
                <div>
                    <span>{name}</span>
                    <span>{count}</span>
                </div>
            }
        }
    }

    let mut attrs = HashMap::new();
    attrs.insert("name".to_owned(), "Test".to_owned());
    attrs.insert("count".to_owned(), "42".to_owned());

    let component = AttributeComponent::from_attributes(&attrs);

    assert_eq!(
        component.render().to_string(),
        "<div><span>Test</span><span>42</span></div>"
    );
}

#[test]
fn test_component_with_signal() {
    #[component]
    struct SignalComponent {
        #[signal]
        count: Signal<i32>,
    }

    impl View for SignalComponent {
        fn render(&self) -> Html {
            let count = self.count.clone();

            tmpl! {
                <div>Count: {count}</div>
            }
        }
    }

    let component = SignalComponent::new();
    component.set_count(10);

    // Test signal rendering - on WASM targets with hydrate feature, signals are wrapped in effect spans
    // On non-WASM targets, signals are rendered as plain values
    let rendered = component.render().to_string();
    assert!(rendered.contains("Count:"));
    assert!(rendered.contains("10"));

    component.count.update(|count| *count += 1);
    let rendered = component.render().to_string();
    assert!(rendered.contains("Count:"));
    assert!(rendered.contains("11"));

    component.count.set(20);
    let rendered = component.render().to_string();
    assert!(rendered.contains("Count:"));
    assert!(rendered.contains("20"));
}

#[test]
fn test_component_input_with_signal() {
    #[component]
    struct InputComponent {
        #[signal]
        value: Signal<String>,
    }

    impl View for InputComponent {
        fn render(&self) -> Html {
            tmpl! {
                <input type="text" value={self.value} />
            }
        }
    }

    let component = InputComponent::new();

    // With effect-based reactivity, signals are wrapped in effect spans
    let rendered = component.render().to_string();
    assert!(rendered.contains("input"));
    assert!(rendered.contains("type=\"text\""));
    assert!(rendered.contains("value=\"\""));

    component.set_value("Hello".to_owned());

    let rendered = component.render().to_string();
    assert!(rendered.contains("input"));
    assert!(rendered.contains("type=\"text\""));
    assert!(rendered.contains("value=\"Hello\""));

    component.value.set("World".to_owned());
    let rendered = component.render().to_string();
    assert!(rendered.contains("input"));
    assert!(rendered.contains("type=\"text\""));
    assert!(rendered.contains("value=\"World\""));
}

#[test]
fn test_component_with_signal_and_multiple_attributes() {
    #[component]
    struct SignalComponent {
        #[signal]
        value: Signal<String>,
    }

    impl View for SignalComponent {
        fn render(&self) -> Html {
            let value = self.value.clone();

            tmpl! {
                <h1>{value}</h1>
                <input type="text" value={value} placeholder="Enter text" />
            }
        }
    }

    let component = SignalComponent::new();

    // With effect-based reactivity, signals are wrapped in effect spans
    let rendered = component.render().to_string();
    assert!(rendered.contains("<h1>"));
    assert!(rendered.contains("</h1>"));
    assert!(rendered.contains("input"));
    assert!(rendered.contains("placeholder=\"Enter text\""));
    assert!(rendered.contains("type=\"text\""));
    assert!(rendered.contains("value=\"\""));
}
