use wasm_bindgen::JsValue;

/// Get INIT_DATA from window object as JsValue
pub fn get_init_data() -> Option<JsValue> {
    let window = web_sys::window()?;
    let init_data = js_sys::Reflect::get(&window, &JsValue::from_str("INIT_DATA")).ok()?;

    if init_data.is_undefined() || init_data.is_null() {
        return None;
    }

    Some(init_data)
}

/// Get typed INIT_DATA for a specific route by key
/// This looks for data under INIT_DATA[route_name]
pub fn get_typed_route_data<T>(route_name: &str) -> Option<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let init_data = get_init_data()?;

    // Get the specific route's data from INIT_DATA
    let route_data = js_sys::Reflect::get(&init_data, &JsValue::from_str(route_name)).ok()?;

    if route_data.is_undefined() || route_data.is_null() {
        return None;
    }

    // Convert JsValue to JSON string, then deserialize
    let json_string = js_sys::JSON::stringify(&route_data).ok()?;
    let json_str = json_string.as_string()?;

    // Use serde_json to deserialize the route's data
    serde_json::from_str(&json_str).ok()
}
