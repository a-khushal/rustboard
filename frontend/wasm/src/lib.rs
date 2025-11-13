use wasm_bindgen::prelude::*;
use rustboard_editor::{Document, Point};
use std::cell::RefCell;

#[wasm_bindgen]
pub struct EditorApi {
    document: RefCell<Document>,
}

#[wasm_bindgen]
impl EditorApi {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EditorApi {
        EditorApi {
            document: RefCell::new(Document::new()),
        }
    }

    #[wasm_bindgen]
    pub fn add_rectangle(&self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        self.document.borrow_mut().add_rectangle(Point::new(x, y), width, height)
    }

    #[wasm_bindgen]
    pub fn get_rectangles_count(&self) -> usize {
        self.document.borrow().get_rectangles().len()
    }
}

impl Default for EditorApi {
    fn default() -> Self {
        Self::new()
    }
}
