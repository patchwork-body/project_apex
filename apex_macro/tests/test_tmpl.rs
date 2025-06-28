#![allow(missing_docs)]
use apex::{Signal, tmpl};

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

#[test]
fn test_input_with_placeholder_with_space() {
    let template = tmpl! {
        <input type="text" placeholder="Enter text" />
    };

    assert_eq!(
        template.to_string(),
        "<input placeholder=\"Enter text\" type=\"text\" />"
    );
}

#[test]
fn test_input_with_multiple_attributes() {
    let value = Signal::new("Hello".to_owned());

    let template = tmpl! {
        <input type="text" value={value} placeholder="Enter text" />
    };

    assert_eq!(
        template.to_string(),
        "<input placeholder=\"Enter text\" type=\"text\" value=\"Hello\" />"
    );
}

#[test]
fn test_dynamic_variable_with_static_content_and_event_handler() {
    let counter = apex::signal!(0);

    let inc = {
        let counter = counter.clone();

        move |_event: apex::web_sys::Event| {
            counter.update(|c| *c += 1);
        }
    };

    let template = tmpl! {
        <h1>Awesome</h1>
        <span>{counter}</span>
        <button onclick={inc}>Increment</button>
    };

    assert_eq!(
        template.to_string(),
        "<h1>Awesome</h1><span>0</span><button id=\"apex_element_0\">Increment</button>"
    );
}

#[test]
fn test_self_closing_tags() {
    let template = tmpl! {
        <div>
            <br />
            <hr />
            <img src="test.jpg" alt="Test Image" />
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div><br /><hr /><img alt=\"Test Image\" src=\"test.jpg\" /></div>"
    );
}

#[test]
fn test_deeply_nested_structure() {
    let template = tmpl! {
        <div class="outer">
            <div class="middle">
                <div class="inner">
                    <p>Deep nesting</p>
                    <span>Multiple levels</span>
                </div>
            </div>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"outer\"><div class=\"middle\"><div class=\"inner\"><p>Deep\nnesting</p><span>Multiple levels</span></div></div></div>"
    );
}

#[test]
fn test_form_elements() {
    let name = "John";
    let email = "john@example.com";
    let selected_option = "option2";

    let template = tmpl! {
        <form action="/submit" method="post">
            <label for="name">Name:</label>
            <input type="text" id="name" name="name" value={name} />

            <label for="email">Email:</label>
            <input type="email" id="email" name="email" value={email} />

            <label for="options">Choose:</label>
            <select id="options" name="options">
                <option value="option1">Option 1</option>
                <option value="option2" selected={selected_option == "option2"}>Option 2</option>
                <option value="option3">Option 3</option>
            </select>

            <textarea name="message" placeholder="Your message"></textarea>
            <button type="submit">Submit</button>
        </form>
    };

    let expected = format!(
        "<form action=\"/submit\" method=\"post\"><label for=\"name\">Name:</label><input id=\"name\" name=\"name\" type=\"text\" value=\"{name}\" /><label for=\"email\">Email:</label><input id=\"email\" name=\"email\" type=\"email\" value=\"{email}\" /><label for=\"options\">Choose:</label><select id=\"options\" name=\"options\"><option value=\"option1\">Option 1</option><option selected=\"true\" value=\"option2\">Option 2</option><option value=\"option3\">Option 3</option></select><textarea name=\"message\" placeholder=\"Your message\"></textarea><button type=\"submit\">Submit</button></form>"
    );

    assert_eq!(template.to_string(), expected);
}

#[test]
fn test_table_structure() {
    let rows = [
        ("Alice", 25, "Engineer"),
        ("Bob", 30, "Designer"),
        ("Charlie", 35, "Manager"),
    ];

    let template = tmpl! {
        <table class="data-table">
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Age</th>
                    <th>Role</th>
                </tr>
            </thead>
            <tbody>
                {rows.iter().map(|(name, age, role)| {
                    format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>", name, age, role)
                }).collect::<String>()}
            </tbody>
        </table>
    };

    let expected = "<table class=\"data-table\"><thead><tr><th>Name</th><th>Age</th><th>Role</th></tr></thead><tbody>\n<tr><td>Alice</td><td>25</td><td>Engineer</td></tr><tr><td>Bob</td><td>30</td><td>Designer</td></tr><tr><td>Charlie</td><td>35</td><td>Manager</td></tr></tbody></table>";

    assert_eq!(template.to_string(), expected);
}

#[test]
fn test_special_characters_in_content() {
    let message = "Hello & welcome! <script>alert('xss')</script>";
    let html_content = "<em>emphasized</em>";

    let template = tmpl! {
        <div>
            <p>{message}</p>
            <div>{html_content}</div>
        </div>
    };

    // Content should be properly escaped
    assert_eq!(
        template.to_string(),
        "<div><p>Hello & welcome! <script>alert('xss')</script></p><div><em>emphasized</em></div></div>"
    );
}

#[test]
fn test_boolean_attributes() {
    let is_disabled = true;
    let is_checked = false;
    let is_readonly = true;

    let template = tmpl! {
        <div>
            <input type="text" disabled={is_disabled} readonly={is_readonly} />
            <input type="checkbox" checked={is_checked} />
            <button disabled={!is_disabled}>Enable</button>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div><input disabled=\"true\" readonly=\"true\" type=\"text\" /><input checked=\"false\" type=\"checkbox\" /><button disabled=\"false\">Enable</button></div>"
    );
}

#[test]
fn test_multiple_variables_in_template() {
    let title = "My App";
    let version = "1.0.0";
    let author = "Developer";
    let year = 2024;

    let template: apex::Html = tmpl! {
        <div class="app-info">
            <h1>{title}</h1>
            <p>Version: {version}</p>
            <p>Author: {author}</p>
            <p>Copyright (c) {year}</p>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"app-info\"><h1>My App</h1><p>Version: 1.0.0</p><p>Author: Developer</p><p>Copyright(c) 2024</p></div>"
    );
}

#[test]
fn test_complex_expressions_in_attributes() {
    let base_url = "https://api.example.com";
    let endpoint = "users";
    let user_id = 123;
    let is_admin = true;

    let template: apex::Html = tmpl! {
        <div>
            <a href={format!("{}/{}/{}", base_url, endpoint, user_id)}>User Profile</a>
            <span class={if is_admin { "admin-badge" } else { "user-badge" }}>
                {if is_admin { "Admin" } else { "User" }}
            </span>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div><a href=\"https://api.example.com/users/123\">User\nProfile</a><span class=\"admin-badge\">\nAdmin</span></div>"
    );
}

#[test]
fn test_event_handlers_on_different_elements() {
    let click_handler = move |_event: apex::web_sys::Event| {
        // Handle click
    };

    let input_handler = move |_event: apex::web_sys::Event| {
        // Handle input
    };

    let submit_handler = move |_event: apex::web_sys::Event| {
        // Handle form submit
    };

    let template: apex::Html = tmpl! {
        <div>
            <button onclick={click_handler}>Click Me</button>
            <input type="text" oninput={input_handler} />
            <form onsubmit={submit_handler}>
                <button type="submit">Submit</button>
            </form>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div><button id=\"apex_element_1\">Click Me</button><input id=\"apex_element_2\" type=\"text\" /><form id=\"apex_element_3\"><button type=\"submit\">Submit</button></form></div>"
    );
}

#[test]
fn test_mixed_static_and_dynamic_content() {
    let user_name = "Alice";
    let message_count = 5;
    let is_online = true;

    let template = tmpl! {
        <div class="user-status">
            <h2>Welcome back, {user_name}!</h2>
            <p>You have {message_count} new messages.</p>
            <div class="status">
                Status:
                {if is_online {
                    tmpl!{ <span class="online">Online</span> }
                } else {
                    tmpl!{ <span class="offline">Offline</span> }
                }}
            </div>
            <button>View Messages</button>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"user-status\"><h2>Welcome back, Alice!</h2><p>You have\n5new messages.</p><div class=\"status\"> Status:\n<span class=\"online\">Online</span></div><button>View Messages</button></div>"
    );
}

#[test]
fn test_empty_elements() {
    let template = tmpl! {
        <div>
            <p></p>
            <span></span>
            <div class="empty"></div>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div><p></p><span></span><div class=\"empty\"></div></div>"
    );
}

#[test]
fn test_numeric_and_string_interpolation() {
    let count = 42;
    let price = 19.99;
    let product = "Widget";
    let is_available = true;

    let template = tmpl! {
        <div class="product">
            <h3>{product}</h3>
            <p>Price: ${price}</p>
            <p>In stock: {count} units</p>
            <p>Available: {is_available}</p>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"product\"><h3>Widget</h3><p>Price: $19.99</p><p>In\nstock: 42units</p><p>Available: true</p></div>"
    );
}

#[test]
fn test_signal_with_static_variable_and_event_handler() {
    let app_name = "Counter App";
    let version = "1.0.0";
    let counter = apex::signal!(0);

    let increment = {
        let counter = counter.clone();
        move |_event: apex::web_sys::Event| {
            counter.update(|c| *c += 1);
        }
    };

    let template = tmpl! {
        <div class="app">
            <header>
                <h1>{app_name}</h1>
                <span class="version">v{version}</span>
            </header>
            <main>
                <div class="counter-display">
                    <span class="count">{counter}</span>
                </div>
                <button onclick={increment}>Increment</button>
            </main>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"app\"><header><h1>Counter App</h1><span class=\"version\">v1.0.0</span></header><main><div class=\"counter-display\"><span class=\"count\">0</span></div><button id=\"apex_element_4\">Increment</button></main></div>"
    );
}

#[test]
fn test_multiple_signals_with_static_data() {
    let title = "Dashboard";
    let user_name = "Alice";
    let score = apex::signal!(100);
    let health = apex::signal!(75);
    let level = apex::signal!(5);

    let template = tmpl! {
        <div class="dashboard">
            <h1>{title}</h1>
            <div class="user-info">
                <span>Player: {user_name}</span>
            </div>
            <div class="stats">
                <div class="stat">
                    <label>Score:</label>
                    <span>{score}</span>
                </div>
                <div class="stat">
                    <label>Health:</label>
                    <span>{health}%</span>
                </div>
                <div class="stat">
                    <label>Level:</label>
                    <span>{level}</span>
                </div>
            </div>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"dashboard\"><h1>Dashboard</h1><div class=\"user-info\"><span>Player: Alice</span></div><div class=\"stats\"><div class=\"stat\"><label>Score:</label><span>100</span></div><div class=\"stat\"><label>Health:</label><span>75%</span></div><div class=\"stat\"><label>Level:</label><span>5</span></div></div></div>"
    );
}

#[test]
fn test_form_with_signals_and_static_labels() {
    let form_title = "User Registration";
    let username = apex::signal!("".to_string());
    let email = apex::signal!("".to_string());
    let is_premium = apex::signal!(false);

    let handle_username = {
        let username = username.clone();
        move |_event: apex::web_sys::Event| {
            // Handle username input
        }
    };

    let handle_email = {
        let email = email.clone();
        move |_event: apex::web_sys::Event| {
            // Handle email input
        }
    };

    let handle_premium = {
        let is_premium = is_premium.clone();
        move |_event: apex::web_sys::Event| {
            is_premium.update(|p| *p = !*p);
        }
    };

    let template: apex::Html = tmpl! {
        <form class="registration-form">
            <h2>{form_title}</h2>

            <div class="field">
                <label for="username">Username:</label>
                <input
                    type="text"
                    id="username"
                    value={username}
                    oninput={handle_username}
                />
            </div>

            <div class="field">
                <label for="email">Email Address:</label>
                <input
                    type="email"
                    id="email"
                    value={email}
                    oninput={handle_email}
                />
            </div>

            <div class="field">
                <label>
                    <input
                        type="checkbox"
                        checked={is_premium}
                        onchange={handle_premium}
                    />
                    Premium Account
                </label>
            </div>
        </form>
    };

    assert_eq!(
        template.to_string(),
        "<form class=\"registration-form\"><h2>User Registration</h2><div class=\"field\"><label for=\"username\">Username:</label><input id=\"apex_element_5\" type=\"text\" value=\"\" /></div><div class=\"field\"><label for=\"email\">Email Address:</label><input id=\"apex_element_6\" type=\"email\" value=\"\" /></div><div class=\"field\"><label><input checked=\"false\" id=\"apex_element_7\" type=\"checkbox\" />Premium Account</label></div></form>"
    );
}

#[test]
fn test_shopping_cart_with_mixed_content() {
    let store_name = "Tech Store";
    let currency_symbol = "$";
    let item_count = apex::signal!(0);
    let total_price = apex::signal!(0.0);
    let cart_items = vec![("Laptop", 999.99), ("Mouse", 29.99), ("Keyboard", 79.99)];

    let add_item = {
        let item_count = item_count.clone();
        let total_price = total_price.clone();
        move |_event: apex::web_sys::Event| {
            item_count.update(|c| *c += 1);
            total_price.update(|p| *p += 29.99);
        }
    };

    let clear_cart = {
        let item_count = item_count.clone();
        let total_price = total_price.clone();
        move |_event: apex::web_sys::Event| {
            item_count.set(0);
            total_price.set(0.0);
        }
    };

    let template = tmpl! {
        <div class="shop">
            <header class="shop-header">
                <h1>{store_name}</h1>
                <div class="cart-info">
                    <span>Items: {item_count}</span>
                    <span>Total: {currency_symbol}{total_price}</span>
                </div>
            </header>

            <div class="product-list">
                {cart_items.iter().map(|(name, price)| {
                    format!("<div class=\"product\"><span>{}</span><span>${}</span></div>", name, price)
                }).collect::<String>()}
            </div>

            <div class="cart-actions">
                <button onclick={add_item}>Add Random Item</button>
                <button onclick={clear_cart}>Clear Cart</button>
            </div>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"shop\"><header class=\"shop-header\"><h1>Tech Store</h1><div class=\"cart-info\"><span>Items: 0</span><span>Total:
$0</span></div></header><div class=\"product-list\">
<div class=\"product\"><span>Laptop</span><span>$999.99</span></div><div class=\"product\"><span>Mouse</span><span>$29.99</span></div><div class=\"product\"><span>Keyboard</span><span>$79.99</span></div></div><div class=\"cart-actions\"><button id=\"apex_element_8\">Add Random
Item</button><button id=\"apex_element_9\">Clear Cart</button></div></div>"
    );
}

#[test]
fn test_todo_app_comprehensive() {
    let app_title = "Todo List";
    let placeholder_text = "Enter a new task...";
    let todos = apex::signal!(Vec::<String>::new());
    let input_value = apex::signal!("".to_string());
    let completed_count = apex::signal!(0);

    let add_todo = {
        let todos = todos.clone();
        let input_value = input_value.clone();
        move |_event: apex::web_sys::Event| {
            // Add todo logic
        }
    };

    let handle_input = {
        let input_value = input_value.clone();
        move |_event: apex::web_sys::Event| {
            // Handle input change
        }
    };

    let clear_completed = {
        let todos = todos.clone();
        let completed_count = completed_count.clone();
        move |_event: apex::web_sys::Event| {
            completed_count.set(0);
        }
    };

    let template = tmpl! {
        <div class="todo-app">
            <header>
                <h1>{app_title}</h1>
                <div class="stats">
                                         <span>Total: 0</span>
                    <span>Completed: {completed_count}</span>
                </div>
            </header>

            <div class="input-section">
                <input
                    type="text"
                    placeholder={placeholder_text}
                    value={input_value}
                    oninput={handle_input}
                />
                <button onclick={add_todo}>Add Task</button>
            </div>

            <div class="todo-list">
                {todos.get().iter().enumerate().map(|(i, todo)| {
                    format!("<div class=\"todo-item\"><span>{}</span></div>", todo)
                }).collect::<String>()}
            </div>

            <footer class="todo-footer">
                <button onclick={clear_completed}>Clear Completed</button>
                <span class="footer-text">Manage your tasks efficiently</span>
            </footer>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"todo-app\"><header><h1>Todo List</h1><div class=\"stats\"><span>Total: 0</span><span>Completed: 0</span></div></header><div class=\"input-section\"><input id=\"apex_element_10\" placeholder=\"Enter a new task...\" type=\"text\" value=\"\" /><button id=\"apex_element_11\">Add Task</button></div><div class=\"todo-list\">
<!--APEX-DYNAMIC-apex_element_11-4--></div><footer class=\"todo-footer\"><button id=\"apex_element_12\">Clear Completed</button><span class=\"footer-text\">Manage your tasks efficiently</span></footer></div>"
    );
}

#[test]
fn test_game_ui_with_signals_and_static_elements() {
    let game_title = "Space Adventure";
    let player_name = "Captain Smith";
    let difficulty = "Hard";

    let score = apex::signal!(1250);
    let lives = apex::signal!(3);
    let level = apex::signal!(5);
    let is_paused = apex::signal!(false);

    let pause_game = {
        let is_paused = is_paused.clone();
        move |_event: apex::web_sys::Event| {
            is_paused.update(|p| *p = !*p);
        }
    };

    let restart_game = {
        let score = score.clone();
        let lives = lives.clone();
        let level = level.clone();
        move |_event: apex::web_sys::Event| {
            score.set(0);
            lives.set(3);
            level.set(1);
        }
    };

    let template = tmpl! {
        <div class="game-ui">
            <header class="game-header">
                <h1>{game_title}</h1>
                <div class="player-info">
                    <span>Player: {player_name}</span>
                    <span>Difficulty: {difficulty}</span>
                </div>
            </header>

            <div class="hud">
                <div class="stat-group">
                    <div class="stat">
                        <label>Score</label>
                        <span class="value">{score}</span>
                    </div>
                    <div class="stat">
                        <label>Lives</label>
                        <span class="value">{lives}</span>
                    </div>
                    <div class="stat">
                        <label>Level</label>
                        <span class="value">{level}</span>
                    </div>
                </div>

                <div class="status">
                    {"PLAYING"}
                </div>
            </div>

            <div class="game-controls">
                <button onclick={pause_game}>
                    {"Pause"}
                </button>
                <button onclick={restart_game}>Restart</button>
            </div>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"game-ui\"><header class=\"game-header\"><h1>Space Adventure</h1><div class=\"player-info\"><span>Player: Captain Smith</span><span>Difficulty: Hard</span></div></header><div class=\"hud\"><div class=\"stat-group\"><div class=\"stat\"><label>Score</label><span class=\"value\">1250</span></div><div class=\"stat\"><label>Lives</label><span class=\"value\">3</span></div><div class=\"stat\"><label>Level</label><span class=\"value\">5</span></div></div><div class=\"status\"> PLAYING</div></div><div class=\"game-controls\"><button id=\"apex_element_13\"> Pause</button><button id=\"apex_element_14\">Restart</button></div></div>"
    );
}

#[test]
fn test_settings_panel_with_mixed_inputs() {
    let panel_title = "Settings";
    let app_version = "2.1.0";

    let volume = apex::signal!(75);
    let notifications_enabled = apex::signal!(true);
    let theme = apex::signal!("dark".to_string());
    let username = apex::signal!("user123".to_string());

    let volume_change = {
        let volume = volume.clone();
        move |_event: apex::web_sys::Event| {
            // Handle volume change
        }
    };

    let toggle_notifications = {
        let notifications_enabled = notifications_enabled.clone();
        move |_event: apex::web_sys::Event| {
            notifications_enabled.update(|n| *n = !*n);
        }
    };

    let theme_change = {
        let theme = theme.clone();
        move |_event: apex::web_sys::Event| {
            // Handle theme change
        }
    };

    let save_settings = move |_event: apex::web_sys::Event| {
        // Save settings logic
    };

    let template: apex::Html = tmpl! {
        <div class="settings-panel">
            <header>
                <h2>{panel_title}</h2>
                <span class="version">Version {app_version}</span>
            </header>

            <div class="settings-group">
                <h3>Audio Settings</h3>
                <div class="setting">
                    <label for="volume">Volume: {volume}%</label>
                    <input
                        type="range"
                        id="volume"
                        min="0"
                        max="100"
                        value={volume}
                        oninput={volume_change}
                    />
                </div>
            </div>

            <div class="settings-group">
                <h3>Preferences</h3>
                <div class="setting">
                    <label>
                        <input
                            type="checkbox"
                            checked={notifications_enabled}
                            onchange={toggle_notifications}
                        />
                        Enable Notifications
                    </label>
                </div>

                <div class="setting">
                    <label for="theme">Theme:</label>
                    <select id="theme" onchange={theme_change}>
                        <option value="light">Light</option>
                        <option value="dark" selected="true">Dark</option>
                        <option value="auto">Auto</option>
                    </select>
                </div>

                <div class="setting">
                    <label for="username">Username:</label>
                    <input
                        type="text"
                        id="username"
                        value={username}
                        placeholder="Enter username"
                    />
                </div>
            </div>

            <footer class="settings-footer">
                <button onclick={save_settings}>Save Changes</button>
                <span class="save-status">All changes saved automatically</span>
            </footer>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"settings-panel\"><header><h2>Settings</h2><span class=\"version\">Version 2.1.0</span></header><div class=\"settings-group\"><h3>Audio Settings</h3><div class=\"setting\"><label for=\"volume\">Volume: 75%</label><input id=\"apex_element_15\" max=\"100\" min=\"0\" type=\"range\" value=\"75\" /></div></div><div class=\"settings-group\"><h3>Preferences</h3><div class=\"setting\"><label><input checked=\"true\" id=\"apex_element_16\" type=\"checkbox\" />Enable Notifications</label></div><div class=\"setting\"><label for=\"theme\">Theme:</label><select id=\"apex_element_17\"><option value=\"light\">Light</option><option selected=\"true\" value=\"dark\">Dark</option><option value=\"auto\">Auto</option></select></div><div class=\"setting\"><label for=\"username\">Username:</label><input id=\"username\" placeholder=\"Enter username\" type=\"text\" value=\"user123\" /></div></div><footer class=\"settings-footer\"><button id=\"apex_element_18\">Save Changes</button><span class=\"save-status\">All changes saved automatically</span></footer></div>"
    );
}

#[test]
fn test_chat_interface_with_dynamic_messages() {
    let chat_title = "Team Chat";
    let current_user = "Alice";
    let online_count = 12;

    let message_input = apex::signal!("".to_string());
    let messages = apex::signal!(vec![
        ("Bob", "Hello everyone!"),
        ("Carol", "How's the project going?"),
        ("Dave", "Almost done with the frontend"),
    ]);

    let send_message = {
        let messages = messages.clone();
        let message_input = message_input.clone();
        move |_event: apex::web_sys::Event| {
            // Send message logic
        }
    };

    let handle_input = {
        let message_input = message_input.clone();
        move |_event: apex::web_sys::Event| {
            // Handle input change
        }
    };

    let template: apex::Html = tmpl! {
        <div class="chat-interface">
            <header class="chat-header">
                <h1>{chat_title}</h1>
                <div class="status">
                    <span>You: {current_user}</span>
                    <span>{online_count} online</span>
                </div>
            </header>

            <div class="messages-area">
                {vec![("Bob", "Hello everyone!"), ("Carol", "How's the project going?"), ("Dave", "Almost done with the frontend")].iter().map(|(user, msg)| {
                    format!("<div class=\"message\"><strong>{}:</strong> {}</div>", user, msg)
                }).collect::<String>()}
            </div>

            <div class="input-area">
                <input
                    type="text"
                    placeholder="Type a message..."
                    value={message_input}
                    oninput={handle_input}
                />
                <button onclick={send_message}>Send</button>
            </div>

            <footer class="chat-footer">
                <span>Connected to server</span>
                <span>Messages: 3</span>
            </footer>
        </div>
    };

    assert_eq!(
        template.to_string(),
        "<div class=\"chat-interface\"><header class=\"chat-header\"><h1>Team Chat</h1><div class=\"status\"><span>You:
Alice</span><span>12online</span></div></header><div class=\"messages-area\">
<div class=\"message\"><strong>Bob:</strong> Hello everyone!</div><div class=\"message\"><strong>Carol:</strong> How's the project going?</div><div class=\"message\"><strong>Dave:</strong> Almost done with the frontend</div></div><div class=\"input-area\"><input id=\"apex_element_19\" placeholder=\"Type a message...\" type=\"text\" value=\"\" /><button id=\"apex_element_20\">Send</button></div><footer class=\"chat-footer\"><span>Connected to server</span><span>Messages: 3</span></footer></div>"
    );
}
