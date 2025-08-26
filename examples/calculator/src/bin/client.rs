#![allow(missing_docs)]

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use calculator::RootPageRoute;

        let mut apex = apex::Apex::new();
        apex.hydrate(RootPageRoute);
    }
}
