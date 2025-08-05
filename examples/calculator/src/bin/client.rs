#![allow(missing_docs)]

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use apex::prelude::*;
        use calculator::Calculator;

        apex::Apex::hydrate(tmpl! { <Calculator /> });
    }
}
