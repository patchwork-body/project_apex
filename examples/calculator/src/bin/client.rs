#![allow(missing_docs)]

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use apex::prelude::*;
        use calculator::RootPageRoute;

        // Test route-based hydration
        apex::Apex::hydrate(RootPageRoute);

        // This works but we want route-based to work too:
        // apex::Apex::hydrate2(tmpl! {
        //     <Layout />
        //     <Calculator />
        // });
    }
}
