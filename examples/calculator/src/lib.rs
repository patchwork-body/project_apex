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
                {symbol.get()}
            </span>
        </button>
    }
}

#[derive(PartialEq, Clone, Debug)]
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

impl Operator {
    fn apply(&self, left: f64, right: f64) -> f64 {
        match self {
            Operator::Add => left + right,
            Operator::Subtract => left - right,
            Operator::Multiply => left * right,
            Operator::Divide => left / right,
        }
    }
}

#[derive(Clone)]
struct Expression {
    left: Option<Box<Expression>>,
    right: String,
    operator: Option<Operator>,
    rank: u8,
    is_negative: bool,
}

impl Default for Expression {
    fn default() -> Self {
        Self {
            left: None,
            right: "0".to_owned(),
            operator: None,
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
        if self.right == "0" && symbol != "." {
            self.right = symbol;
        } else {
            let right = self.right.clone();
            self.right = format!("{right}{symbol}");
        };
    }

    fn set_operator(&mut self, operator: Operator) {
        if self.left.is_some()
            && self.left.as_ref().unwrap().operator.as_ref().unwrap() != &Operator::Subtract
            && operator == Operator::Subtract
            && self.right.is_empty()
        {
            self.negative_right_value();
        } else {
            match operator {
                Operator::Add | Operator::Subtract => {
                    self.rank = 1;
                }
                Operator::Multiply | Operator::Divide => {
                    self.rank = 2;
                }
            }

            if !self.right.is_empty() {
                self.operator = operator.into();
                self.left = Box::new(self.clone()).into();

                self.reset();
            }
        }
    }

    fn reset(&mut self) {
        self.rank = 0;
        self.operator = None;
        self.is_negative = false;
        self.right = "".to_owned();
    }

    fn negative_right_value(&mut self) {
        if self.right != "0" || !self.right.is_empty() {
            self.is_negative = !self.is_negative;
        }
    }

    fn remove_last_symbol(&mut self) {
        if self.right.is_empty() && self.left.is_some() {
            self.replace_right_with_left();

            if self.operator.is_some() {
                self.operator = None;
                self.rank = 0;
            }
        } else if self.right.len() == 1 {
            self.right = "0".to_owned();
        } else {
            self.right.pop();
        }
    }

    fn replace_right_with_left(&mut self) {
        if let Some(left) = self.left.as_mut() {
            self.is_negative = left.is_negative;
            self.right = left.right.clone();
            self.operator = left.operator.clone();
            self.rank = left.rank;
            self.left = left.left.clone();
        }
    }

    fn format_with_commas(value: &str) -> String {
        let (neg, value) = if let Some(stripped) = value.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, value)
        };

        let mut parts = value.splitn(2, '.');
        let int_part = parts.next().unwrap_or("");
        let frac_part = parts.next();
        let chars: Vec<char> = int_part.chars().rev().collect();

        let mut formatted = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                formatted.push(',');
            }
            formatted.push(*c);
        }

        let mut formatted: String = formatted.chars().rev().collect();
        if let Some(frac) = frac_part {
            formatted.push('.');
            formatted.push_str(frac);
        }

        if neg {
            formatted.insert(0, '-');
        }

        formatted
    }

    fn get_display_value(&self) -> String {
        if self.left.is_none() {
            Self::format_with_commas(&self.get_current_value())
        } else {
            let Some(left_expression) = self.left.as_ref() else {
                return "".to_owned();
            };

            let left_expression_display_value = left_expression.get_display_value();
            let current_value = Self::format_with_commas(&self.get_current_value());

            format!(
                "{}{}{}",
                left_expression_display_value,
                left_expression
                    .operator
                    .as_ref()
                    .map_or("".to_owned(), |op| op.to_string()),
                current_value
            )
        }
    }

    fn execute(&self) -> f64 {
        let parse_value = |val: &str, neg: bool| -> f64 {
            let mut v = val.parse::<f64>().unwrap_or(0.0);
            if neg {
                v = -v;
            }
            v
        };

        // Flatten the tree into values and operators
        let mut values = Vec::new();
        let mut operators = Vec::new();
        let mut node = Some(self);
        while let Some(n) = node {
            values.push(parse_value(&n.right, n.is_negative));
            if let Some(op) = &n.operator {
                operators.push((op.clone(), n.rank));
            }
            node = n.left.as_deref();
        }
        values.reverse();
        operators.reverse();

        // Precedence climbing evaluation
        while !operators.is_empty() && values.len() > 1 {
            // Find the highest precedence operator
            let max_rank = operators.iter().map(|(_, rank)| *rank).max().unwrap();
            let idx = operators
                .iter()
                .position(|(_, rank)| *rank == max_rank)
                .unwrap();
            let (op, _) = &operators[idx];
            let result = op.apply(values[idx], values[idx + 1]);
            values.splice(idx..=idx + 1, [result]);
            operators.remove(idx);
        }
        values[0]
    }
}

#[component]
pub fn calculator() -> Html {
    let expression = signal!(Expression::default());
    let prev_expression = signal!(None::<Expression>);

    let set_operator = action!(expression, prev_expression => |event| {
        let symbol = event.current_target().unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap().inner_text();

        prev_expression.set(None);

        expression.update(|v| {
            let mut v = v.clone();
            v.set_operator(Operator::from(symbol));
            v
        });
    });

    let add_symbol = action!(expression, prev_expression => |event| {
        let symbol = event.current_target().unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap().inner_text();

        if prev_expression.get().is_some() {
            prev_expression.set(None);

            expression.update(|v| {
                let mut v = v.clone();
                v.reset();
                v.update_right(symbol);
                v
            });
        } else {
            expression.update(|v| {
                let mut v = v.clone();
                v.update_right(symbol);
                v
            });
        }
    });

    let clear_symbol = derive!(expression, prev_expression, {
        if expression.get().right == "0" || prev_expression.get().is_some() {
            "AC".to_owned()
        } else {
            "<-".to_owned()
        }
    });

    let timeout_id = signal!(None::<i32>);

    let remove_last_symbol = action!(expression, prev_expression, timeout_id => |_event| {
        if let Some(id) = timeout_id.get() {
            web_sys::window().unwrap().clear_timeout_with_handle(id);
        }

        if prev_expression.get().is_some() {
            prev_expression.set(None);
            expression.set(Expression::default());
        } else {
            expression.update(|v| {
                let mut v = v.clone();
                v.remove_last_symbol();
                v
            });
        }
    });

    let set_timeout_to_clear_value = action!(expression, prev_expression, timeout_id => |_event| {
        let expression = expression.clone();
        let prev_expression = prev_expression.clone();

        let closure = Closure::wrap(Box::new(move || {
            prev_expression.set(None);
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
            v.negative_right_value();
            v
        });
    });

    let add_decimal_symbol = action!(expression => |_event| {
        expression.update(|v| {
            let mut v = v.clone();
            if v.right.contains(".") {
                return v;
            }

            v.update_right(".".to_owned());
            v
        });
    });

    let display_expression = derive!(expression, { expression.get().get_display_value() });

    let execute_expression = action!(expression, prev_expression => |_event| {
        prev_expression.set(Some(expression.get().clone()));
        let result = expression.get().execute();

        expression.set(Expression {
            right: result.to_string(),
            ..Default::default()
        });
    });

    let display_prev_expression = derive!(prev_expression, {
        prev_expression
            .get()
            .map_or("".to_owned(), |v| v.get_display_value())
    });

    tmpl! {
        <div class="calculator">
            <div class="display">
                {#if true}
                    <span class="prev-expression">{display_prev_expression.get()}</span>
                {#endif}

                <span class="current-expression">{display_expression.get()}</span>
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
                <Button symbol="." onclick={add_decimal_symbol.clone()} />
                <Button primary={true} symbol="=" onclick={execute_expression.clone()} />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_expression() {
        let expr = Expression::default();
        assert_eq!(expr.right, "0");
        assert!(expr.left.is_none());
        assert!(expr.operator.is_none());
        assert_eq!(expr.rank, 0);
        assert!(!expr.is_negative);
    }

    #[test]
    fn test_get_current_value_positive() {
        let expr = Expression {
            right: "123".to_owned(),
            is_negative: false,
            ..Default::default()
        };

        assert_eq!(expr.get_current_value(), "123");
    }

    #[test]
    fn test_get_current_value_negative() {
        let expr = Expression {
            right: "123".to_owned(),
            is_negative: true,
            ..Default::default()
        };
        assert_eq!(expr.get_current_value(), "-123");
    }

    #[test]
    fn test_update_right() {
        let mut expr = Expression::default();
        expr.update_right("4".to_owned());
        assert_eq!(expr.right, "4");
        expr.update_right("2".to_owned());
        assert_eq!(expr.right, "42");
    }

    #[test]
    fn test_set_operator_and_reset() {
        let mut expr = Expression {
            right: "5".to_owned(),
            ..Default::default()
        };

        expr.set_operator(Operator::Add);
        assert!(expr.left.is_some());
        assert_eq!(expr.left.as_ref().unwrap().operator, Some(Operator::Add));
        assert_eq!(expr.right, "");
    }

    #[test]
    fn test_negative_right_value() {
        let mut expr = Expression::default();
        expr.negative_right_value();
        assert!(expr.is_negative);
        expr.negative_right_value();
        assert!(!expr.is_negative);
    }

    #[test]
    fn test_remove_last_symbol() {
        let mut expr = Expression {
            right: "12".to_owned(),
            ..Default::default()
        };

        expr.remove_last_symbol();
        assert_eq!(expr.right, "1");
        expr.remove_last_symbol();
        assert_eq!(expr.right, "0");
    }

    #[test]
    fn test_replace_right_with_left() {
        let left = Expression {
            right: "99".to_owned(),
            is_negative: true,
            ..Default::default()
        };

        let mut expr = Expression {
            left: Some(Box::new(left)),
            right: "".to_owned(),
            operator: Some(Operator::Add),
            rank: 2,
            is_negative: false,
        };

        expr.replace_right_with_left();
        assert_eq!(expr.right, "99");
        assert!(expr.is_negative);
    }

    #[test]
    fn test_get_display_value_simple() {
        let expr = Expression {
            right: "42".to_owned(),
            ..Default::default()
        };

        assert_eq!(expr.get_display_value(), "42");
    }

    #[test]
    fn test_get_display_value_nested() {
        let left = Expression {
            right: "1".to_owned(),
            operator: Some(Operator::Add),
            ..Default::default()
        };

        let expr = Expression {
            left: Some(Box::new(left)),
            right: "2".to_owned(),
            operator: None,
            rank: 0,
            is_negative: false,
        };

        assert_eq!(expr.get_display_value(), "1+2");
    }

    #[test]
    fn test_format_with_commas() {
        assert_eq!(Expression::format_with_commas("123456"), "123,456");
        assert_eq!(Expression::format_with_commas("1234"), "1,234");
        assert_eq!(Expression::format_with_commas("12345678"), "12,345,678");
        assert_eq!(Expression::format_with_commas("-12345678"), "-12,345,678");
        assert_eq!(Expression::format_with_commas("123456.789"), "123,456.789");
        assert_eq!(
            Expression::format_with_commas("-123456.789"),
            "-123,456.789"
        );
    }

    #[test]
    fn test_execute_simple_value() {
        let expr = Expression {
            right: "42".to_owned(),
            is_negative: false,
            ..Default::default()
        };

        assert_eq!(expr.execute(), 42.0);
    }

    #[test]
    fn test_execute_negative_value() {
        let expr = Expression {
            right: "42".to_owned(),
            is_negative: true,
            ..Default::default()
        };

        assert_eq!(expr.execute(), -42.0);
    }

    #[test]
    fn test_execute_add() {
        let left = Expression {
            right: "2".to_owned(),
            operator: Some(Operator::Add),
            ..Default::default()
        };

        let expr = Expression {
            left: Some(Box::new(left)),
            right: "3".to_owned(),
            ..Default::default()
        };

        assert_eq!(expr.execute(), 5.0);
    }

    #[test]
    fn test_execute_subtract() {
        let left = Expression {
            right: "5".to_owned(),
            operator: Some(Operator::Subtract),
            ..Default::default()
        };

        let expr = Expression {
            left: Some(Box::new(left)),
            right: "3".to_owned(),
            ..Default::default()
        };

        assert_eq!(expr.execute(), 2.0);
    }

    #[test]
    fn test_execute_multiply() {
        let left = Expression {
            right: "4".to_owned(),
            operator: Some(Operator::Multiply),
            ..Default::default()
        };

        let expr = Expression {
            left: Some(Box::new(left)),
            right: "3".to_owned(),
            ..Default::default()
        };

        assert_eq!(expr.execute(), 12.0);
    }

    #[test]
    fn test_execute_divide() {
        let left = Expression {
            right: "12".to_owned(),
            operator: Some(Operator::Divide),
            ..Default::default()
        };

        let expr = Expression {
            left: Some(Box::new(left)),
            right: "3".to_owned(),
            ..Default::default()
        };

        assert_eq!(expr.execute(), 4.0);
    }

    #[test]
    fn test_execute_precedence() {
        // (2 + 3) * 4 = 20
        let left = Expression {
            right: "2".to_owned(),
            operator: Some(Operator::Add),
            ..Default::default()
        };

        let mid = Expression {
            left: Some(Box::new(left)),
            right: "3".to_owned(),
            operator: None,
            ..Default::default()
        };

        let expr = Expression {
            left: Some(Box::new(mid)),
            right: "4".to_owned(),
            operator: Some(Operator::Multiply),
            ..Default::default()
        };

        assert_eq!(expr.execute(), 20.0); // (2 + 3) * 4 = 20

        // 2 + 3 * 4 = 14
        let left = Expression {
            right: "3".to_owned(),
            operator: None,
            ..Default::default()
        };

        let mul = Expression {
            left: Some(Box::new(left)),
            right: "4".to_owned(),
            operator: Some(Operator::Multiply),
            ..Default::default()
        };

        let add = Expression {
            left: Some(Box::new(Expression {
                right: "2".to_owned(),
                operator: None,
                ..Default::default()
            })),
            right: mul.execute().to_string(),
            operator: Some(Operator::Add),
            ..Default::default()
        };

        assert_eq!(add.execute(), 14.0); // 2 + 3 * 4 = 14

        // 2 + (3 * 4) = 14
        let left = Expression {
            right: "3".to_owned(),
            operator: None,
            ..Default::default()
        };

        let mul = Expression {
            left: Some(Box::new(left)),
            right: "4".to_owned(),
            operator: Some(Operator::Multiply),
            ..Default::default()
        };

        let add = Expression {
            left: Some(Box::new(Expression {
                right: "2".to_owned(),
                operator: None,
                ..Default::default()
            })),
            right: mul.execute().to_string(),
            operator: Some(Operator::Add),
            ..Default::default()
        };

        assert_eq!(add.execute(), 14.0); // 2 + (3 * 4) = 14
    }
}
