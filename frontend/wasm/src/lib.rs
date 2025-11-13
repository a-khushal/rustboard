use wasm_bindgen::prelude::*;
use rustboard_editor;

#[wasm_bindgen]
pub struct EditorApi {}

#[wasm_bindgen]
impl EditorApi {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EditorApi {
        EditorApi {}
    }

    #[wasm_bindgen]
    pub fn greet(&self, name: &str) -> String {
        rustboard_editor::greet(name)
    }

    #[wasm_bindgen]
    pub fn add(&self, a: i32, b: i32) -> i32 {
        rustboard_editor::add(a, b)
    }
}

impl Default for EditorApi {
    fn default() -> Self {
        Self::new()
    }
}

