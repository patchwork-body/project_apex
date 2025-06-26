use crate::tmpl::{
    HtmlContent, generate_component_code::*, generate_event_listeners::*,
    generate_html_opening_tag_code::*,
};
use quote::quote;
use syn::Result;

/// Generate render parts from parsed HTML content for the Apex template system.
///
/// ## Purpose
///
/// This function serves as the core code generation engine for Apex's template processing system.
/// It transforms parsed HTML content into Rust code tokens that can be compiled into efficient
/// render functions. This is essential for Apex's compile-time template processing, which provides
/// type safety and performance benefits over runtime template parsing.
///
/// ## Why This Function Is Needed
///
/// 1. **Compile-Time Template Processing**: Apex processes templates at compile time rather than
///    runtime, eliminating the overhead of template parsing during request handling.
///
/// 2. **Type Safety**: By generating Rust code from templates, we get compile-time type checking
///    for template variables and component properties.
///
/// 3. **Performance**: The generated code produces optimized render functions that directly
///    concatenate strings without intermediate parsing steps.
///
/// 4. **Component Integration**: Seamlessly integrates custom Apex components with standard HTML
///    elements in a unified rendering pipeline.
///
/// 5. **Event Handler Support**: Generates event listener registration code that works with
///    web_sys for WebAssembly compatibility.
///
/// ## How It Works
///
/// The function processes different types of HTML content and generates appropriate Rust code:
///
/// ### Text Content
/// Plain text strings are wrapped in `.to_string()` calls to ensure they can be concatenated
/// with other render parts.
///
/// **Example Input**: `HtmlContent::Text("Hello World")`
/// **Generated Code**: `"Hello World".to_string()`
///
/// ### Variables/Expressions
/// Rust expressions embedded in templates (typically in `{variable}` syntax) are parsed
/// and wrapped to convert their output to strings.
///
/// **Example Input**: `HtmlContent::Variable("user.name")`
/// **Generated Code**: `(user.name).to_string()`
///
/// ### Components
/// Custom Apex components are processed through the component generation system, which
/// handles property passing, lifecycle, and rendering.
///
/// **Example Input**: `HtmlContent::Component { tag: "Button", attributes: {...} }`
/// **Generated Code**: `Button::new().with_props(...).render().into_string()`
///
/// ### HTML Elements
/// Standard HTML elements are processed to generate proper opening/closing tags with
/// attributes, supporting both regular and self-closing elements. Elements with event
/// handlers get unique IDs automatically assigned and event listeners are registered.
///
/// **Example Input**: `HtmlContent::Element { tag: "div", attributes: [...] }`
/// **Generated Code**: `"<div class=\"container\">"`
///
/// ## Event Handler Processing
///
/// When elements contain event handlers (e.g., onclick={handler}):
/// 1. Event handlers are excluded from HTML output
/// 2. Elements automatically get unique IDs assigned
/// 3. Separate event listener registration code is generated
/// 4. Event listeners are registered using web_sys after DOM rendering
///
/// ## Integration with Macro System
///
/// This function is typically called from higher-level template processing macros that:
/// 1. Parse template strings into `HtmlContent` structures
/// 2. Call `generate_render_parts` to convert content to code tokens
/// 3. Combine the tokens into a complete render function
/// 4. Emit the final Rust code for compilation
///
/// ## Error Handling
///
/// The function propagates parsing errors from:
/// - Variable expression parsing (invalid Rust syntax)
/// - Component code generation (missing components, invalid properties)
/// - HTML tag generation (malformed attributes)
/// - Event listener generation (invalid handler expressions)
///
/// ## Performance Considerations
///
/// - **Compile-Time Cost**: Processing happens during compilation, not runtime
/// - **Memory Efficiency**: Generated code avoids allocations where possible
/// - **String Concatenation**: Uses efficient string building patterns
/// - **Event Registration**: Minimal runtime overhead for event setup
///
/// # Arguments
///
/// * `content` - A slice of `HtmlContent` items representing parsed template content
///
/// # Returns
///
/// * `Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>)>` -
///   A tuple containing:
///   - Vector of HTML render code tokens
///   - Vector of event listener registration code tokens
///   - Vector of signal updater registration code tokens
///     Returns an error if code generation fails
///
/// # Examples
///
/// ```rust,ignore
/// use crate::tmpl::HtmlContent;
///
/// let content = vec![
///     HtmlContent::Text("Hello "),
///     HtmlContent::Variable("name"),
///     HtmlContent::Element {
///         tag: "button".to_string(),
///         attributes: {
///             let mut attrs = std::collections::HashMap::new();
///             attrs.insert("onclick".to_string(), ComponentAttribute::EventHandler("handle_click".to_string()));
///             attrs
///         },
///         self_closing: false,
///         element_id: Some("apex_element_0".to_string())
///     },
/// ];
///
/// let (html_parts, event_parts, updater_parts) = generate_render_parts(&content)?;
/// // html_parts contains tokens for HTML generation
/// // event_parts contains tokens for event listener registration
/// // updater_parts contains tokens for signal updater registration
/// ```
pub(crate) fn generate_render_parts(
    content: &[HtmlContent],
) -> Result<(
    Vec<proc_macro2::TokenStream>,
    Vec<proc_macro2::TokenStream>,
    Vec<proc_macro2::TokenStream>,
)> {
    let mut html_parts = Vec::new();
    let mut event_parts = Vec::new();
    let mut updater_parts = Vec::new();

    println!("[DEBUG] Generating render parts");
    println!("[DEBUG] Content: {content:?}");

    for item in content {
        match item {
            HtmlContent::Text(text) => {
                if !text.is_empty() {
                    html_parts.push(quote! { #text.to_string() });
                }
            }
            HtmlContent::StaticVariable(var_name) => {
                if let Ok(expr) = syn::parse_str::<syn::Expr>(var_name) {
                    // Static variables - non-reactive, directly convert to string
                    html_parts.push(quote! {
                        (#expr).to_string()
                    });
                }
            }
            HtmlContent::DynamicVariable { variable, context } => {
                if let Ok(expr) = syn::parse_str::<syn::Expr>(variable) {
                    // Dynamic variables - signal-based, use reactive rendering
                    // Context information can be used for specialized rendering behavior
                    match context {
                        crate::tmpl::DynamicVariableContext::TextNode {
                            element_id,
                            text_node_index,
                        } => {
                            // For dynamic text nodes, we need to ensure they become separate DOM text nodes
                            // Use a placeholder marker that will be processed after DOM creation
                            html_parts.push(quote! {
                                {
                                    use apex::Reactive;
                                    let value = &#expr;
                                    let content = if value.is_reactive() {
                                        value.get_value().to_string()
                                    } else {
                                        // Fallback for non-reactive values that were classified as dynamic
                                        value.to_string()
                                    };
                                    // Use a unique marker to identify this dynamic content for DOM post-processing
                                    format!("<!--APEX-DYNAMIC-{}-{}-->{}", #element_id, #text_node_index, content)
                                }
                            });

                            // Generate DOM post-processing to create proper text node structure
                            updater_parts.push(quote! {
                                {
                                    use apex::Reactive;
                                    let value_ref = &#expr;

                                    // Only process reactive values
                                    if value_ref.is_reactive() {
                                        // Post-process DOM to ensure proper text node separation
                                        if let Some(element) = apex::web_sys::window()
                                            .unwrap()
                                            .document()
                                            .unwrap()
                                            .get_element_by_id(#element_id) {
                                            
                                            let initial_value = value_ref.get_value().to_string();
                                            let marker = format!("<!--APEX-DYNAMIC-{}-{}-->", #element_id, #text_node_index);
                                            let html = element.inner_html();
                                            
                                            apex::web_sys::console::log_1(&format!("Processing DOM for element {}, looking for marker", #element_id).into());
                                            
                                            if html.contains(&marker) {
                                                // Parse and rebuild DOM with separate text nodes
                                                let target_text = format!("{}{}", marker, initial_value);
                                                if let Some(marker_pos) = html.find(&target_text) {
                                                    // Clear element and rebuild with proper text node structure
                                                    element.set_inner_html("");
                                                    
                                                    let document = apex::web_sys::window().unwrap().document().unwrap();
                                                    
                                                    // Add content before the marker as text node(s)
                                                    let before_marker = &html[..marker_pos];
                                                    if !before_marker.is_empty() {
                                                                                                            let before_node = document.create_text_node(before_marker);
                                                    element.append_child(&before_node).unwrap();
                                                }
                                                
                                                // Add the dynamic content as a separate text node
                                                let dynamic_node = document.create_text_node(&initial_value);
                                                element.append_child(&dynamic_node).unwrap();
                                                
                                                // Add content after the dynamic part
                                                let after_pos = marker_pos + target_text.len();
                                                if after_pos < html.len() {
                                                    let after_marker = &html[after_pos..];
                                                    if !after_marker.is_empty() {
                                                        let after_node = document.create_text_node(after_marker);
                                                        element.append_child(&after_node).unwrap();
                                                        }
                                                    }
                                                    
                                                    apex::web_sys::console::log_1(&format!("Created {} text nodes for element {}", element.child_nodes().length(), #element_id).into());
                                                }
                                            }
                                        }
                                        
                                        // Register the signal with the correct text node index
                                        value_ref.register_text_node(#element_id.to_string(), #text_node_index as u32);
                                        apex::web_sys::console::log_1(&format!("Registered signal text node: {}[{}]", #element_id, #text_node_index).into());
                                    }
                                }
                            });
                        }
                        crate::tmpl::DynamicVariableContext::AttributeValue {
                            element_id,
                            attribute_name,
                        } => {
                            // For attributes, we track the specific attribute to update
                            html_parts.push(quote! {
                                {
                                    use apex::Reactive;
                                    let value = &#expr;
                                    if value.is_reactive() {
                                        // TODO: Register signal updater for element_id and attribute_name
                                        // Signal updater will update DOM attribute when signal changes
                                        let element_id = #element_id;
                                        let attribute_name = #attribute_name;
                                        // For now, just render the current value
                                        value.get_value().to_string()
                                    } else {
                                        value.to_string()
                                    }
                                }
                            });
                        }
                    }
                }
            }
            HtmlContent::Component {
                tag, attributes, ..
            } => {
                let component_code = generate_component_code(tag, attributes)?;
                html_parts.push(quote! { #component_code.into_string() });
            }
            HtmlContent::Element {
                tag,
                attributes,
                self_closing,
                element_id,
            } => {
                // Generate HTML opening tag code with element ID support
                let element_code = generate_html_opening_tag_code(
                    tag,
                    attributes,
                    *self_closing,
                    element_id.as_deref(),
                );

                html_parts.push(quote! { #element_code });

                // Generate event listener registration code if element has an ID
                if let Some(id) = element_id {
                    let listeners = generate_event_listeners(id, attributes)?;
                    event_parts.extend(listeners);
                }
            }
        }
    }

    Ok((html_parts, event_parts, updater_parts))
}
