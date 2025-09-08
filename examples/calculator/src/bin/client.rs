#![allow(missing_docs)]

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        use calculator::routes::root::RootPageRoute;
        apex::apex_router::ApexClientRouter::new(Box::new(RootPageRoute));
    }
}
