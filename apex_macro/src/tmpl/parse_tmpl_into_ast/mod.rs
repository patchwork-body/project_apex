use crate::tmpl::TmplAst;

mod is_pascal_case;
mod match_chars;
mod parse_conditional_directive;
mod parse_directive_name;
mod parse_directive_params;
mod parse_element_opening_tag;
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

    let mut chars = input.chars().peekable();

    let (ast, _) = process_chars_until(&mut chars, None);

    ast
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmpl::{Attribute, IfBlock, TmplAst};
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
                        name: "header".to_owned(),
                        children: vec![TmplAst::Element {
                            tag: "h2".to_owned(),
                            attributes: HashMap::new(),
                            is_component: false,
                            self_closing: false,
                            children: vec![TmplAst::Text("Profile Information".to_owned())],
                        },],
                    },
                    TmplAst::Slot {
                        name: "content".to_owned(),
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
                        name: "footer".to_owned(),
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
    fn conditional_template() {
        let input = r#"
            <div class="dashboard">
                {#if user.is_admin}
                    <div class="admin-panel">
                        <h3>Admin Controls</h3>
                        <button onclick={delete_user}>Delete User</button>
                    </div>
                {#endif}
                {#if user.has_notifications}
                    <div class="notifications">
                        <p>You have {notification_count} notifications</p>
                    </div>
                {#endif}
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
                    TmplAst::ConditionalDirective(vec![IfBlock {
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
                    TmplAst::ConditionalDirective(vec![IfBlock {
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
}
