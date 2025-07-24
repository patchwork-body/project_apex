#![allow(missing_docs)]

use apex::wasm_bindgen::JsCast;
use std::{
    fmt::{self, Display},
    rc::Rc,
};
use wasm_bindgen::prelude::Closure;

use apex::prelude::*;

#[component]
pub fn button(
    #[prop] symbol: Signal<String>,
    #[prop(default = false)] wide: bool,
    #[prop(default = false)] primary: bool,
    #[prop(default = false)] secondary: bool,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onclick: Rc<dyn Fn(web_sys::Event)>,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onmousedown: Rc<dyn Fn(web_sys::Event)>,
    #[prop(default = Rc::new(|_event: web_sys::Event| {}))] onmouseup: Rc<dyn Fn(web_sys::Event)>,
) -> Html {
    let mut classes = vec!["button"];

    if wide {
        classes.push("button-wide");
    }

    if primary {
        classes.push("button-primary");
    }

    if secondary {
        classes.push("button-secondary");
    }

    tmpl! {
        <button type="button" class={classes.join(" ")} onclick={onclick} onmousedown={onmousedown} onmouseup={onmouseup}>
            <span class="button-symbol">
                {$symbol}
            </span>
        </button>
    }
}

#[derive(PartialEq)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl From<String> for Operator {
    fn from(value: String) -> Self {
        match value.as_str() {
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "×" => Operator::Multiply,
            "÷" => Operator::Divide,
            _ => panic!("Invalid operator: {value}"),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Subtract => write!(f, "-"),
            Operator::Multiply => write!(f, "×"),
            Operator::Divide => write!(f, "÷"),
        }
    }
}

#[derive(Clone)]
struct Expression {
    left: Option<Box<Expression>>,
    right: String,
    operator: String,
    rank: u8,
    is_negative: bool,
}

impl Default for Expression {
    fn default() -> Self {
        Self {
            left: None,
            right: "0".to_owned(),
            operator: "".to_owned(),
            rank: 0,
            is_negative: false,
        }
    }
}

impl Expression {
    fn get_current_value(&self) -> String {
        if self.is_negative {
            format!("-{}", self.right)
        } else {
            self.right.clone()
        }
    }

    fn update_right(&mut self, symbol: String) {
        if self.right == "0" {
            self.right = symbol;
        } else {
            let right = self.right.clone();
            self.right = format!("{right}{symbol}");
        };
    }

    fn set_operator(&mut self, operator: Operator) {
        if self.right.is_empty() && operator == Operator::Subtract {
            self.right = "-".to_owned();
        } else {
            self.operator = operator.to_string();
        }
    }

    fn negative_value(&mut self) {
        if self.right != "0" || !self.right.is_empty() {
            self.is_negative = !self.is_negative;
        }
    }

    fn remove_last_symbol(&mut self) {
        if self.right.len() == 1 {
            self.right = "0".to_owned();
        } else {
            self.right.pop();
        }
    }
}

#[component]
pub fn calculator() -> Html {
    let expression = signal!(Expression::default());

    let set_operator = action!(expression => |event| {
        let symbol = event.current_target().unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap().inner_text();

        expression.update(|v| {
            let mut v = v.clone();
            v.set_operator(Operator::from(symbol));
            v
        });
    });

    let add_symbol = action!(expression => |event| {
        let symbol = event.current_target().unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap().inner_text();

        expression.update(|v| {
            let mut v = v.clone();
            v.update_right(symbol);
            v
        });
    });

    let clear_symbol = expression.derive(|v| {
        if v.right == "0" {
            "AC".to_owned()
        } else {
            "<-".to_owned()
        }
    });

    let timeout_id = signal!(None::<i32>);

    let remove_last_symbol = action!(expression, timeout_id => |_event| {
        if let Some(id) = timeout_id.get() {
            web_sys::window().unwrap().clear_timeout_with_handle(id);
        }

        expression.update(|v| {
            let mut v = v.clone();
            v.remove_last_symbol();
            v
        });
    });

    let set_timeout_to_clear_value = action!(expression, timeout_id => |_event| {
        let expression = expression.clone();

        let closure = Closure::wrap(Box::new(move || {
            expression.set(Expression::default());
        }) as Box<dyn Fn()>);

        let id = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            )
            .unwrap();

        timeout_id.set(Some(id));

        closure.forget();
    });

    let negative_value = action!(expression => |_event| {
        expression.update(|v| {
            let mut v = v.clone();
            v.negative_value();
            v
        });
    });

    let display_value = derive!(expression, {
        let expression = expression.get();
        let current_value = expression.get_current_value();

        if expression.left.is_none() {
            format!("{}{}", current_value, expression.operator)
        } else {
            format!(
                "{}{}{}",
                expression.left.as_ref().unwrap().get_current_value(),
                expression.operator,
                current_value
            )
        }
    });

    tmpl! {
        <div class="calculator">
            <div class="display">
                {$display_value}
            </div>

            <div class="buttons">
                <Button secondary={true} symbol={clear_symbol.clone()} onmousedown={set_timeout_to_clear_value.clone()} onmouseup={remove_last_symbol.clone()} />
                <Button secondary={true} symbol="±" onclick={negative_value.clone()} />
                <Button secondary={true} symbol="%" onclick={set_operator.clone()} />
                <Button primary={true} symbol="÷" onclick={set_operator.clone()} />
                <Button symbol="7" onclick={add_symbol.clone()} />
                <Button symbol="8" onclick={add_symbol.clone()} />
                <Button symbol="9" onclick={add_symbol.clone()} />
                <Button primary={true} symbol="×" onclick={set_operator.clone()} />
                <Button symbol="4" onclick={add_symbol.clone()} />
                <Button symbol="5" onclick={add_symbol.clone()} />
                <Button symbol="6" onclick={add_symbol.clone()} />
                <Button primary={true} symbol="-" onclick={set_operator.clone()} />
                <Button symbol="1" onclick={add_symbol.clone()} />
                <Button symbol="2" onclick={add_symbol.clone()} />
                <Button symbol="3" onclick={add_symbol.clone()} />
                <Button primary={true} symbol="+" onclick={set_operator.clone()} />
                <Button wide={true} symbol="0" onclick={add_symbol.clone()} />
                <Button symbol="." onclick={add_symbol.clone()} />
                <Button primary={true} symbol="=" />
            </div>
        </div>
    }
}
