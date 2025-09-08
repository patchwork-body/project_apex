use crate::routes::about::AboutPageRoute;
use crate::routes::calculator::CalculatorPageRoute;
use apex::prelude::*;
use apex_components::Link;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LoaderData {
    pub name: String,
    pub age: u8,
}

#[route(component = Layout, path = "/{name}/{age}", children = [CalculatorPageRoute, AboutPageRoute])]
pub fn root_page(params: HashMap<String, String>) -> LoaderData {
    LoaderData {
        name: params
            .get("name")
            .unwrap_or(&"Anonymous".to_owned())
            .to_owned(),

        age: params
            .get("age")
            .unwrap_or(&"0".to_owned())
            .parse::<u8>()
            .unwrap_or(0),
    }
}

#[component]
pub fn layout() {
    let loader_data = loader_data!(root_page);

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

    tmpl! {
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>Calculator</title>
                <link rel="stylesheet" href="/static/styles.css" />
                <script type="module" src="/static/init.js"></script>
            </head>
            <body>
                <Link href={format!("/{}/{}/about", loader_name.get(), loader_age.get())} text="About" />
                <Link href={format!("/{}/{}/calculator", loader_name.get(), loader_age.get())} text="Calculator" />

                <hr />

                <span class="loader-data">{loader_name.get()}: {loader_age.get()}</span>
                {#outlet}
            </body>
        </html>
    }
}
