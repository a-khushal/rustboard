use serde_wasm_bindgen::to_value;
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
    pub fn undo(&self) -> bool {
        self.document.borrow_mut().undo()
    }

    #[wasm_bindgen]
    pub fn redo(&self) -> bool {
        self.document.borrow_mut().redo()
    }

    #[wasm_bindgen]
    pub fn can_undo(&self) -> bool {
        self.document.borrow().can_undo()
    }

    #[wasm_bindgen]
    pub fn can_redo(&self) -> bool {
        self.document.borrow().can_redo()
    }

    #[wasm_bindgen]
    pub fn add_rectangle(&self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        self.document.borrow_mut().add_rectangle(Point::new(x, y), width, height)
    }

    #[wasm_bindgen]
    pub fn get_rectangles(&self) -> JsValue {
        let rectangles = self.document.borrow().get_rectangles().to_vec();
        to_value(&rectangles).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_rectangle(&self, id: u64, x: f64, y: f64, save_history: bool) {
        self.document.borrow_mut().move_rectangle(id, Point::new(x, y), save_history);
    }

    #[wasm_bindgen]
    pub fn resize_rectangle(&self, id: u64, width: f64, height: f64, save_history: bool) {
        self.document.borrow_mut().resize_rectangle(id, width, height, save_history);
    }

    #[wasm_bindgen]
    pub fn delete_rectangle(&self, id: u64) {
        self.document.borrow_mut().delete_rectangle(id);
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> u64 {
        self.document.borrow_mut().add_ellipse(Point::new(x, y), radius_x, radius_y)
    }

    #[wasm_bindgen]
    pub fn get_ellipses(&self) -> JsValue {
        let ellipses = self.document.borrow().get_ellipses().to_vec();
        to_value(&ellipses).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_ellipse(&self, id: u64, x: f64, y: f64, save_history: bool) {
        self.document.borrow_mut().move_ellipse(id, Point::new(x, y), save_history);
    }

    #[wasm_bindgen]
    pub fn resize_ellipse(&self, id: u64, radius_x: f64, radius_y: f64, save_history: bool) {
        self.document.borrow_mut().resize_ellipse(id, radius_x, radius_y, save_history);
    }

    #[wasm_bindgen]
    pub fn delete_ellipse(&self, id: u64) {
        self.document.borrow_mut().delete_ellipse(id);
    }

    #[wasm_bindgen]
    pub fn serialize(&self) -> String {
        self.document.borrow().serialize()
    }

    #[wasm_bindgen]
    pub fn deserialize(&self, data: &str) -> bool {
        self.document.borrow_mut().deserialize(data)
    }
}

impl Default for EditorApi {
    fn default() -> Self {
        Self::new()
    }
}
