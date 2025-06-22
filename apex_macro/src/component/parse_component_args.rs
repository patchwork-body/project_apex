use proc_macro::TokenStream;
use syn::Result;

/// Component configuration parsed from macro arguments
#[derive(Debug)]
pub(crate) struct ComponentConfig {
    // No longer need tag and imports - derive from struct name
}

/// Parse the component macro arguments to extract configuration
pub(crate) fn parse_component_args(_args: TokenStream) -> Result<ComponentConfig> {
    // No arguments needed anymore - component name derived from struct name
    Ok(ComponentConfig {})
}
