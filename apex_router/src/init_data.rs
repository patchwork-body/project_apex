use std::collections::HashMap;
use std::sync::Mutex;
use wasm_bindgen::JsValue;

lazy_static::lazy_static! {
    static ref ROUTE_DATA_COLLECTOR: Mutex<HashMap<String, serde_json::Value>> = Mutex::new(HashMap::new());
}

pub fn add_route_data<T: serde::Serialize>(
    route_name: &str,
    data: T,
) -> Result<(), serde_json::Error> {
    let json_value = serde_json::to_value(data)?;
    ROUTE_DATA_COLLECTOR
        .lock()
        .unwrap()
        .insert(route_name.to_owned(), json_value);

    Ok(())
}

pub fn get_and_clear_route_data() -> HashMap<String, serde_json::Value> {
    let mut collector = ROUTE_DATA_COLLECTOR.lock().unwrap();
    let data = collector.clone();
    collector.clear();

    data
}

pub fn generate_init_data_script() -> String {
    let data = get_and_clear_route_data();

    if data.is_empty() {
        String::new()
    } else {
        let json_data = serde_json::json!(data);
        format!(
            r#"<script id="apex-init-data">window.INIT_DATA = {};</script>"#,
            serde_json::to_string(&json_data).unwrap_or_else(|_| "{}".to_owned())
        )
    }
}

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
