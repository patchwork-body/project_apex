use syn::Type;

/// Check if a type is Html (basic implementation)
pub(crate) fn is_html_type(ty: &Type) -> bool {
    // This is a simple check - in a real implementation, you might want
    // to handle more cases like fully qualified paths
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Html";
        }
    }

    false
}
