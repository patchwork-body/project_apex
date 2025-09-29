use crate::components::{Button, Card};
use apex::{prelude::*, web_sys};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct AboutLoaderData {
    pub name: String,
    pub age: u8,
}

#[route(component = About, path = "/about")]
pub fn about_page(_params: HashMap<String, String>) -> AboutLoaderData {
    AboutLoaderData {
        name: "Mike".to_owned(),
        age: 30,
    }
}

#[component]
pub fn about() {
    let loader_data = loader_data!(about_page);

    let loader_name = derive!(loader_data, {
        loader_data
            .get()
            .map_or("No data".to_owned(), |data| data.name)
    });

    let loader_age = derive!(loader_data, {
        loader_data
            .get()
            .map_or("No data".to_owned(), |data| data.age.to_string())
    });

    let inc_age = action!(loader_age @ web_sys::MouseEvent => |_| {
        loader_age.update(|age| {
            (age.parse::<u8>().unwrap_or(0) + 1).to_string()
        });
    });

    tmpl! {
        <div class="about">
            <span class="name">{loader_name.get()}</span>

            <Card>
                <#header>
                    <Button onclick={inc_age.clone()}>{loader_age.get()}</Button>
                </#header>

                <#content>
                    <p>Card content</p>
                </#content>

                <#footer>
                    <Button onclick={inc_age.clone()}>{loader_age.get()}</Button>
                </#footer>
            </Card>

            {#if loader_age.get().is_empty()}
                <span class="age">You are old</span>
            {:else if loader_age.get().parse::<u8>().unwrap_or(0) > 30}
                <span class="age">You are old</span>
            {:else}
                <span class="age">You are young</span>
            {/if}
        </div>
    }
}
