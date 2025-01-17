use serde::{Deserialize, Serialize};
use cfg_if::cfg_if;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TodoState {
    pub todos: Vec<String>,
}

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::JsValue;
        use gloo_utils::format::JsValueSerdeExt;
        
        pub fn save_todos(todos: &[String]) {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                // Convert Vec<String> to JsValue using gloo-utils
                let todos_json = JsValue::from_serde(todos).unwrap();
                let todos_str = todos_json.as_string().unwrap_or_else(|| serde_json::to_string(&todos).unwrap());
                storage.set_item("todos", &todos_str).unwrap();
            }
        }

        pub fn load_todos() -> Vec<String> {
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                if let Ok(Some(todos_json)) = storage.get_item("todos") {
                    // Convert JsValue back to Vec<String>
                    return serde_json::from_str::<Vec<String>>(&todos_json).unwrap();
                }
            }
            Vec::new()
        }
    } else {
        use std::fs;
        pub fn save_todos(todos: &[String]) {
            let data = TodoState { todos: todos.to_vec() };
            if let Ok(json) = serde_json::to_string(&data) {
                fs::write("todos.json", json).expect("Unable to write file");
            }
        }

        pub fn load_todos() -> Vec<String> {
            if let Ok(contents) = fs::read_to_string("todos.json") {
                if let Ok(data) = serde_json::from_str::<TodoState>(&contents) {
                    return data.todos;
                }
            }
            Vec::new()
        }
    }
}