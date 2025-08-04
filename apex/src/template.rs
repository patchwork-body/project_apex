pub trait Template {
    #[cfg(not(target_arch = "wasm32"))]
    fn render(&self) -> String;

    #[cfg(target_arch = "wasm32")]
    fn hydrate(&self);
}
