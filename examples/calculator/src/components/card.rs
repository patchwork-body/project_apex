use apex::prelude::*;

#[component]
pub fn card() {
    let hello_world = action!(@ apex::web_sys::MouseEvent => |_| {
        apex::web_sys::console::log_1(&"Hello, world!".into());
    });

    tmpl! {
        <div class="card">
            <#slot header>
                <h1>Card</h1>
            </#slot>
            <#slot content>
                <p>Card content</p>
            </#slot>
            <#slot>
                <button onclick={hello_world}>Hello world</button>
            </#slot>
            <#slot footer>
                <button onclick={hello_world}>Hello world</button>
            </#slot>
        </div>
    }
}
