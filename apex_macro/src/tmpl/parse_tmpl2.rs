use std::{cell::RefCell, collections::HashMap};

use proc_macro::TokenStream;
use quote::quote;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{Comment, Element, Text};

use crate::tmpl::{parse_tmpl_into_ast::*, render_ast2::*};

struct HydrationManager {
    expressions_map: HashMap<String, web_sys::Text>,
    elements_map: HashMap<String, web_sys::Element>,
    initialized: bool,
}

impl HydrationManager {
    fn new() -> Self {
        Self {
            expressions_map: HashMap::new(),
            elements_map: HashMap::new(),
            initialized: false,
        }
    }

    fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        static SHOW_COMMENT: u32 = 128;

        let window = web_sys::window().expect("window not found");
        let document = window.document().expect("document not found");

        let tree_walker = document
            .create_tree_walker_with_what_to_show(
                &document.body().expect("body not found"),
                SHOW_COMMENT,
            )
            .expect("tree walker not found");

        let mut nodes_to_remove = Vec::new();

        while let Ok(Some(node)) = tree_walker.next_node() {
            if let Some(comment) = node.dyn_ref::<Comment>() {
                let data = comment.data();
                let parts: Vec<String> = data.split(":").map(|s| s.trim().to_string()).collect();

                if parts.len() < 2 {
                    continue;
                }

                let comment_type = &parts[0];
                let comment_id = &parts[1];

                if comment_type == "@expr-text-begin" {
                    let next_node = comment.next_sibling().expect("next node not found") else {
                        continue;
                    };

                    let text_node = next_node.dyn_ref::<Text>().expect("text node not found");
                    self.expressions_map
                        .insert(comment_id.clone(), text_node.clone());

                    let next_node = next_node.next_sibling().expect("next node not found") else {
                        continue;
                    };

                    let end_comment = next_node
                        .dyn_ref::<Comment>()
                        .expect("end comment node not found");

                    nodes_to_remove.push(comment.clone());
                    nodes_to_remove.push(end_comment.clone());
                } else if comment_type == "@element" {
                    let next_node = comment.next_sibling().expect("next node not found") else {
                        continue;
                    };

                    let element_node = next_node
                        .dyn_ref::<Element>()
                        .expect("element node not found");

                    self.elements_map
                        .insert(comment_id.clone(), element_node.clone());

                    nodes_to_remove.push(comment.clone());
                }
            }
        }

        for node in nodes_to_remove {
            node.remove();
        }
    }
}

thread_local! {
    static HYDRATION_MANAGER: RefCell<HydrationManager> = RefCell::new(HydrationManager::new());
}

pub(crate) fn parse_tmpl2(input: TokenStream) -> proc_macro2::TokenStream {
    let input_str = input.to_string();
    let parsed_content = parse_tmpl_into_ast(&input_str);
    let (render_instructions, hydration_expressions) = render_ast2(&parsed_content);

    quote! {
        {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut buffer = String::with_capacity(1024);
                #(#render_instructions)*

                buffer
            }

            #[cfg(target_arch = "wasm32")]
            {
                let hydrate = move |expressions_map: &std::collections::HashMap<String, web_sys::Text>, elements_map: &std::collections::HashMap<String, web_sys::Element>| {
                    #(#hydration_expressions)*
                };

                hydrate
            }
        }
    }
}
