/// Client-side utilities for accessing INIT_DATA passed from the server
use wasm_bindgen::prelude::*;

/// Get INIT_DATA from window object as JsValue
pub fn get_init_data() -> Option<JsValue> {
    let window = web_sys::window()?;
    let init_data = js_sys::Reflect::get(&window, &JsValue::from_str("INIT_DATA")).ok()?;

    if init_data.is_undefined() || init_data.is_null() {
        return None;
    }

    Some(init_data)
}

/// Get typed INIT_DATA by deserializing the entire object to the specified type
/// This is used by generated route helper functions
pub fn get_typed_init_data<T>() -> Option<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let init_data = get_init_data()?;

    // Convert JsValue to JSON string, then deserialize
    let json_string = js_sys::JSON::stringify(&init_data).ok()?;
    let json_str = json_string.as_string()?;

    // Use serde_json to deserialize the full object
    serde_json::from_str(&json_str).ok()
}
