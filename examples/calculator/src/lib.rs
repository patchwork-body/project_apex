#![allow(missing_docs)]

use apex::prelude::*;

#[component]
pub fn Button(
    #[prop] symbol: String,
    #[prop(default = false)] wide: bool,
    #[prop(default = false)] primary: bool,
    #[prop(default = false)] secondary: bool,
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
        <button type="button" class={classes.join(" ")}>
            <span class="button-symbol">
                {&symbol}
            </span>
        </button>
    }
}

#[component]
pub fn calculator() -> Html {
    let value = signal!(0);

    tmpl! {
        <div class="calculator">
            <div class="display">
                {$value}
            </div>

            <div class="buttons">
                <Button secondary={true} symbol={if value.get() == 0 { "AC".to_string() } else { "<-".to_string() }} />
                <Button secondary={true} symbol="±" />
                <Button secondary={true} symbol="%" />
                <Button primary={true} symbol="÷" />
                <Button symbol="7" />
                <Button symbol="8" />
                <Button symbol="9" />
                <Button primary={true} symbol="×" />
                <Button symbol="4" />
                <Button symbol="5" />
                <Button symbol="6" />
                <Button primary={true} symbol="-" />
                <Button symbol="1" />
                <Button symbol="2" />
                <Button symbol="3" />
                <Button primary={true} symbol="+" />
                <Button wide={true} symbol="0" />
                <Button symbol="." />
                <Button primary={true} symbol="=" />
            </div>
        </div>
    }
}
