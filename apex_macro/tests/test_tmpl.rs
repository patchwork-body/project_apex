#![allow(missing_docs)]
use apex::tmpl;

#[test]
fn test_basic_div_with_text() {
    let template = tmpl! {
        <div>Hello</div>
    };

    assert_eq!(template.to_string(), "<div>Hello</div>");
}

#[test]
fn test_basic_div_with_text_and_attributes() {
    let template = tmpl! {
        <div class="main">Hello</div>
    };

    assert_eq!(template.to_string(), "<div class=\"main\">Hello</div>");
}

#[test]
fn test_button_with_text_and_attributes() {
    let template = tmpl! {
        <button class="main" id="main">Hello</button>
    };

    assert_eq!(
        template.to_string(),
        "<button class=\"main\" id=\"main\">Hello</button>"
    );
}

#[test]
fn test_input_with_variable_attributes() {
    let user_input = "user_input";

    let template = tmpl! {
        <input type="text" value={user_input} />
    };

    assert_eq!(
        template.to_string(),
        "<input type=\"text\" value=\"user_input\" />"
    );
}

#[test]
fn test_nested_divs() {
    let template = tmpl! {
        <div>
            <div>Nested</div>
        </div>
    };

    assert_eq!(template.to_string(), "<div><div>Nested</div></div>");
}

#[test]
fn test_attribute_expression() {
    let class_name = "container";
    let template = tmpl! {
        <div class={format!("main-{}", class_name)}>Content</div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"main-container\">Content</div>"
    );
}

#[test]
fn test_controlled_input() {
    let value = "Enter text";

    let template = tmpl! {
        <input type="text" value={value} />
    };

    assert_eq!(
        template.to_string(),
        "<input type=\"text\" value=\"Enter text\" />"
    );
}

#[test]
fn test_conditional_structure() {
    let title = "Page Title";
    let user_name = "John Doe";
    let is_logged_in = false;

    let template = tmpl! {
        <div class="container">
            <h1>{title}</h1>
            <p>Welcome, {user_name}!</p>
            {if is_logged_in {
                tmpl!{ <button>Logout</button> }
            } else {
                tmpl!{ <a href="/login">Login</a> }
            }}
        </div>
    };

    let expected = format!(
        "<div class=\"container\"><h1>{title}</h1><p>Welcome, {user_name}!</p>\n<a href=\"/login\">Login</a></div>"
    );

    assert_eq!(template.to_string(), expected);

    let is_logged_in = true;

    let template = tmpl! {
        <div class="container">
            <h1>{title}</h1>
            <p>Welcome, {user_name}!</p>
            {if is_logged_in {
                tmpl!{ <button>Logout</button> }
            } else {
                tmpl!{ <a href="/login">Login</a> }
            }}
        </div>
    };

    let expected = format!(
        "<div class=\"container\"><h1>{title}</h1><p>Welcome, {user_name}!</p>\n<button>Logout</button></div>"
    );

    assert_eq!(template.to_string(), expected);
}
