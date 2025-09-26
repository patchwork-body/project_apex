use crate::tmpl::TmplAst;
use regex::Regex;

mod is_pascal_case;
mod match_chars;
mod parse_conditional_directive;
mod parse_directive_name;
mod parse_directive_params;
mod parse_element_opening_tag;
mod parse_slot_interpolation;
mod parse_slot_name;
mod process_chars_until;

use process_chars_until::*;

pub(crate) fn parse_tmpl_into_ast(input: &str) -> Vec<TmplAst> {
    // Normalize input: remove line breaks and reduce all whitespace to a single space
    let input = input
        .replace(['\n', '\r'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned();

    // Use regex to trim whitespace from expressions in curly braces
    let re = Regex::new(r"\{([^}]*)\}").unwrap();
    let input = re
        .replace_all(&input, |caps: &regex::Captures<'_>| {
            let expression = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            format!("{{{expression}}}")
        })
        .to_string();

    let mut chars = input.chars().peekable();
    let (ast, _) = process_chars_until(&mut chars, None);

    ast
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::{Attribute, ConditionalBlock, TmplAst};
    use std::collections::HashMap;

    #[test]
    fn whitespace_normalization() {
        let input = "  <div>  Hello  </div>  <span>  World  </span>  ";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![
                TmplAst::Element {
                    tag: "div".to_owned(),
                    attributes: HashMap::new(),
                    is_component: false,
                    self_closing: false,
                    children: vec![TmplAst::Text("Hello".to_owned())],
                },
                TmplAst::Element {
                    tag: "span".to_owned(),
                    attributes: HashMap::new(),
                    is_component: false,
                    self_closing: false,
                    children: vec![TmplAst::Text("World".to_owned())],
                },
            ]
        );
    }

    #[test]
    fn newline_removal() {
        let input = "<div>\nHello,\nworld!\n</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::Text("Hello, world!".to_owned())], // Newlines are replaced with spaces
            }]
        );
    }

    #[test]
    fn empty_input() {
        let input = "";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast, vec![]);
    }

    #[test]
    fn whitespace_only_input() {
        let input = "   \n\t  ";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(ast, vec![]);
    }

    #[test]
    fn formatted_template() {
        let input = r#"
            <div class="container">
                <h1>Welcome, {user.name}!</h1>
                <p>
                    You have {message_count} new messages.
                </p>
                <button onclick={handle_click}>
                    Click me
                </button>
            </div>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    Attribute::Literal("container".to_owned())
                )]),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Element {
                        tag: "h1".to_owned(),
                        attributes: HashMap::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![
                            TmplAst::Text("Welcome, ".to_owned()),
                            TmplAst::Expression("user.name".to_owned()),
                            TmplAst::Text("!".to_owned()),
                        ],
                    },
                    TmplAst::Element {
                        tag: "p".to_owned(),
                        attributes: HashMap::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![
                            TmplAst::Text("You have ".to_owned()),
                            TmplAst::Expression("message_count".to_owned()),
                            TmplAst::Text(" new messages.".to_owned()),
                        ],
                    },
                    TmplAst::Element {
                        tag: "button".to_owned(),
                        attributes: HashMap::from([(
                            "onclick".to_owned(),
                            Attribute::EventListener("handle_click".to_owned())
                        )]),
                        is_component: false,
                        self_closing: false,
                        children: vec![TmplAst::Text("Click me".to_owned())],
                    },
                ],
            }]
        );
    }

    #[test]
    fn component_template() {
        let input = r#"
            <UserProfile user={current_user}>
                <#header>
                    <h2>Profile Information</h2>
                </#header>
                <#content>
                    <p>Email: {current_user.email}</p>
                    <p>Member since: {current_user.join_date}</p>
                </#content>
                <#footer>
                    <button onclick={edit_profile}>Edit Profile</button>
                </#footer>
            </UserProfile>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "UserProfile".to_owned(),
                attributes: HashMap::from([(
                    "user".to_owned(),
                    Attribute::Expression("current_user".to_owned())
                )]),
                is_component: true,
                self_closing: false,
                children: vec![
                    TmplAst::Slot {
                        name: Some("header".to_owned()),
                        children: vec![TmplAst::Element {
                            tag: "h2".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Profile Information".to_owned())],
                        },],
                    },
                    TmplAst::Slot {
                        name: Some("content".to_owned()),
                        children: vec![
                            TmplAst::Element {
                                tag: "p".to_owned(),
                                attributes: HashMap::new(),
                                is_component: false,
                                self_closing: false,
                                children: vec![
                                    TmplAst::Text("Email: ".to_owned()),
                                    TmplAst::Expression("current_user.email".to_owned()),
                                ],
                            },
                            TmplAst::Element {
                                tag: "p".to_owned(),
                                attributes: HashMap::new(),
                                is_component: false,
                                self_closing: false,
                                children: vec![
                                    TmplAst::Text("Member since: ".to_owned()),
                                    TmplAst::Expression("current_user.join_date".to_owned()),
                                ],
                            },
                        ],
                    },
                    TmplAst::Slot {
                        name: Some("footer".to_owned()),
                        children: vec![TmplAst::Element {
                            tag: "button".to_owned(),
                            attributes: HashMap::from([(
                                "onclick".to_owned(),
                                Attribute::EventListener("edit_profile".to_owned())
                            )]),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Edit Profile".to_owned())],
                        },],
                    },
                ],
            }]
        );
    }

    #[test]
    fn conditional_template_with_whitespace() {
        let input = r#"
            <div>
                {#if true}
                    <span>Hello, world!</span>
                {/if}
            </div>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                    condition: "true".to_owned(),
                    children: vec![TmplAst::Element {
                        tag: "span".to_owned(),
                        attributes: HashMap::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![TmplAst::Text("Hello, world!".to_owned())],
                    },],
                }])],
            }]
        );
    }

    #[test]
    fn conditional_template() {
        let input = r#"
            <div class="dashboard">
                {#if user.is_admin}
                    <div class="admin-panel">
                        <h3>Admin Controls</h3>
                        <button onclick={delete_user}>Delete User</button>
                    </div>
                {/if}

                {#if user.has_notifications}
                    <div class="notifications">
                        <p>You have {notification_count} notifications</p>
                    </div>
                {/if}
            </div>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::from([(
                    "class".to_owned(),
                    Attribute::Literal("dashboard".to_owned())
                )]),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                        condition: "user.is_admin".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "div".to_owned(),
                            attributes: HashMap::from([(
                                "class".to_owned(),
                                Attribute::Literal("admin-panel".to_owned())
                            )]),
                            is_component: false,
                            self_closing: false,
                            children: vec![
                                TmplAst::Element {
                                    tag: "h3".to_owned(),
                                    attributes: HashMap::new(),
                                    is_component: false,
                                    self_closing: false,
                                    children: vec![TmplAst::Text("Admin Controls".to_owned())],
                                },
                                TmplAst::Element {
                                    tag: "button".to_owned(),
                                    attributes: HashMap::from([(
                                        "onclick".to_owned(),
                                        Attribute::EventListener("delete_user".to_owned())
                                    )]),
                                    is_component: false,
                                    self_closing: false,
                                    children: vec![TmplAst::Text("Delete User".to_owned())],
                                },
                            ],
                        },],
                    }]),
                    TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                        condition: "user.has_notifications".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "div".to_owned(),
                            attributes: HashMap::from([(
                                "class".to_owned(),
                                Attribute::Literal("notifications".to_owned())
                            )]),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Element {
                                tag: "p".to_owned(),
                                attributes: HashMap::new(),
                                is_component: false,
                                self_closing: false,
                                children: vec![
                                    TmplAst::Text("You have ".to_owned()),
                                    TmplAst::Expression("notification_count".to_owned()),
                                    TmplAst::Text(" notifications".to_owned()),
                                ],
                            },],
                        },],
                    }]),
                ],
            }]
        );
    }

    #[test]
    fn trimming_whitespace_in_directives() {
        let input = "<div>{#if true}<span>Hello, world!</span>{/if}</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::ConditionalDirective(vec![ConditionalBlock::If {
                    condition: "true".to_owned(),
                    children: vec![TmplAst::Element {
                        tag: "span".to_owned(),
                        attributes: HashMap::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![TmplAst::Text("Hello, world!".to_owned())],
                    },],
                }])],
            },]
        );
    }

    #[test]
    fn expression_whitespace_trimming() {
        let input = "<div> { user.name } </div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::Expression("user.name".to_owned())],
            }]
        );
    }

    #[test]
    fn outlet_directive() {
        let input = "<div>{#outlet}</div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::Outlet],
            }]
        );
    }

    #[test]
    fn outlet_directive_with_whitespace() {
        let input = "<div> {#outlet} </div>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::Outlet],
            }]
        );
    }

    #[test]
    fn layout_with_outlet() {
        let input = r#"
            <html>
                <head>
                    <title>My App</title>
                </head>
                <body>
                    <nav>Navigation</nav>
                    <main>
                        {#outlet}
                    </main>
                </body>
            </html>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "html".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![
                    TmplAst::Element {
                        tag: "head".to_owned(),
                        attributes: HashMap::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![TmplAst::Element {
                            tag: "title".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("My App".to_owned())],
                        }],
                    },
                    TmplAst::Element {
                        tag: "body".to_owned(),
                        attributes: HashMap::new(),
                        is_component: false,
                        self_closing: false,
                        children: vec![
                            TmplAst::Element {
                                tag: "nav".to_owned(),
                                attributes: HashMap::new(),
                                is_component: false,
                                self_closing: false,
                                children: vec![TmplAst::Text("Navigation".to_owned())],
                            },
                            TmplAst::Element {
                                tag: "main".to_owned(),
                                attributes: HashMap::new(),
                                is_component: false,
                                self_closing: false,
                                children: vec![TmplAst::Outlet],
                            },
                        ],
                    },
                ],
            }]
        );
    }

    #[test]
    fn conditional_with_else() {
        let input = r#"
            <div>
                {#if user.is_authenticated}
                    <span>Welcome back!</span>
                {:else}
                    <span>Please log in</span>
                {/if}
            </div>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::ConditionalDirective(vec![
                    ConditionalBlock::If {
                        condition: "user.is_authenticated".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Welcome back!".to_owned())],
                        }],
                    },
                    ConditionalBlock::Else {
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Please log in".to_owned())],
                        }],
                    },
                ])],
            }]
        );
    }

    #[test]
    fn conditional_with_else_if() {
        let input = r#"
            <div>
                {#if user.role == "admin"}
                    <span>Admin Panel</span>
                {:else if user.role == "moderator"}
                    <span>Moderator Tools</span>
                {:else}
                    <span>User Dashboard</span>
                {/if}
            </div>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::ConditionalDirective(vec![
                    ConditionalBlock::If {
                        condition: "user.role == \"admin\"".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Admin Panel".to_owned())],
                        }],
                    },
                    ConditionalBlock::ElseIf {
                        condition: "user.role == \"moderator\"".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Moderator Tools".to_owned())],
                        }],
                    },
                    ConditionalBlock::Else {
                        children: vec![TmplAst::Element {
                            tag: "span".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("User Dashboard".to_owned())],
                        }],
                    },
                ])],
            }]
        );
    }

    #[test]
    fn nested_conditionals_svelte_style() {
        let input = r#"
            <div>
                {#if user.is_authenticated}
                    <div class="user-area">
                        {#if user.has_avatar}
                            <img src={user.avatar} alt="Avatar" />
                        {:else}
                            <div class="default-avatar">👤</div>
                        {/if}
                        <span>{user.name}</span>
                    </div>
                {:else}
                    <button onclick={show_login}>Log In</button>
                {/if}
            </div>
        "#;

        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::Element {
                tag: "div".to_owned(),
                attributes: HashMap::new(),
                is_component: false,
                self_closing: false,
                children: vec![TmplAst::ConditionalDirective(vec![
                    ConditionalBlock::If {
                        condition: "user.is_authenticated".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "div".to_owned(),
                            attributes: HashMap::from([(
                                "class".to_owned(),
                                Attribute::Literal("user-area".to_owned())
                            )]),
                            is_component: false,
                            self_closing: false,
                            children: vec![
                                TmplAst::ConditionalDirective(vec![
                                    ConditionalBlock::If {
                                        condition: "user.has_avatar".to_owned(),
                                        children: vec![TmplAst::Element {
                                            tag: "img".to_owned(),
                                            attributes: HashMap::from([
                                                (
                                                    "src".to_owned(),
                                                    Attribute::Expression("user.avatar".to_owned())
                                                ),
                                                (
                                                    "alt".to_owned(),
                                                    Attribute::Literal("Avatar".to_owned())
                                                ),
                                            ]),
                                            is_component: false,
                                            self_closing: true,
                                            children: vec![],
                                        }],
                                    },
                                    ConditionalBlock::Else {
                                        children: vec![TmplAst::Element {
                                            tag: "div".to_owned(),
                                            attributes: HashMap::from([(
                                                "class".to_owned(),
                                                Attribute::Literal("default-avatar".to_owned())
                                            )]),
                                            is_component: false,
                                            self_closing: false,
                                            children: vec![TmplAst::Text("👤".to_owned())],
                                        }],
                                    },
                                ]),
                                TmplAst::Element {
                                    tag: "span".to_owned(),
                                    attributes: HashMap::new(),
                                    is_component: false,
                                    self_closing: false,
                                    children: vec![TmplAst::Expression("user.name".to_owned())],
                                },
                            ],
                        }],
                    },
                    ConditionalBlock::Else {
                        children: vec![TmplAst::Element {
                            tag: "button".to_owned(),
                            attributes: HashMap::from([(
                                "onclick".to_owned(),
                                Attribute::EventListener("show_login".to_owned())
                            )]),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Log In".to_owned())],
                        }],
                    },
                ])],
            }]
        );
    }

    #[test]
    fn unnamed_slot_interpolation() {
        let input = "<#slot>Hello, world!</#slot>";
        let ast = parse_tmpl_into_ast(input);

        assert_eq!(
            ast,
            vec![TmplAst::SlotInterpolation {
                slot_name: None,
                default_children: Some(vec![TmplAst::Text("Hello, world!".to_owned())]),
            }]
        );
    }
}
