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

    // u
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

    // rectangle
    #[wasm_bindgen]
    pub fn add_rectangle(&self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        self.document.borrow_mut().add_rectangle(Point::new(x, y), width, height)
    }

    #[wasm_bindgen]
    pub fn add_rectangle_without_snapshot(&self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        self.document.borrow_mut().add_rectangle_without_snapshot(Point::new(x, y), width, height)
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
    pub fn delete_rectangle_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_rectangle_without_snapshot(id)
    }

    #[wasm_bindgen]
    pub fn history_index(&self) -> usize {
        self.document.borrow().history_index()
    }

    // diamond
    #[wasm_bindgen]
    pub fn add_diamond(&self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        self.document.borrow_mut().add_diamond(Point::new(x, y), width, height)
    }

    #[wasm_bindgen]
    pub fn add_diamond_without_snapshot(&self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        self.document.borrow_mut().add_diamond_without_snapshot(Point::new(x, y), width, height)
    }

    #[wasm_bindgen]
    pub fn get_diamonds(&self) -> JsValue {
        let diamonds = self.document.borrow().get_diamonds().to_vec();
        to_value(&diamonds).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_diamond(&self, id: u64, x: f64, y: f64, save_history: bool) {
        self.document.borrow_mut().move_diamond(id, Point::new(x, y), save_history);
    }

    #[wasm_bindgen]
    pub fn resize_diamond(&self, id: u64, width: f64, height: f64, save_history: bool) {
        self.document.borrow_mut().resize_diamond(id, width, height, save_history);
    }

    #[wasm_bindgen]
    pub fn delete_diamond(&self, id: u64) {
        self.document.borrow_mut().delete_diamond(id);
    }

    #[wasm_bindgen]
    pub fn delete_diamond_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_diamond_without_snapshot(id)
    }

    #[wasm_bindgen]
    pub fn add_ellipse(&self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> u64 {
        self.document.borrow_mut().add_ellipse(Point::new(x, y), radius_x, radius_y)
    }

    #[wasm_bindgen]
    pub fn add_ellipse_without_snapshot(&self, x: f64, y: f64, radius_x: f64, radius_y: f64) -> u64 {
        self.document.borrow_mut().add_ellipse_without_snapshot(Point::new(x, y), radius_x, radius_y)
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
    pub fn delete_ellipse_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_ellipse_without_snapshot(id)
    }

    // line
    #[wasm_bindgen]
    pub fn add_line(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> u64 {
        self.document.borrow_mut().add_line(Point::new(start_x, start_y), Point::new(end_x, end_y))
    }

    #[wasm_bindgen]
    pub fn add_line_without_snapshot(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> u64 {
        self.document.borrow_mut().add_line_without_snapshot(Point::new(start_x, start_y), Point::new(end_x, end_y))
    }

    #[wasm_bindgen]
    pub fn get_lines(&self) -> JsValue {
        let lines = self.document.borrow().get_lines().to_vec();
        to_value(&lines).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_line(&self, id: u64, start_x: f64, start_y: f64, end_x: f64, end_y: f64, save_history: bool) {
        self.document.borrow_mut().move_line(id, Point::new(start_x, start_y), Point::new(end_x, end_y), save_history);
    }

    #[wasm_bindgen]
    pub fn delete_line(&self, id: u64) {
        self.document.borrow_mut().delete_line(id);
    }

    #[wasm_bindgen]
    pub fn delete_line_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_line_without_snapshot(id)
    }

    // arrow
    #[wasm_bindgen]
    pub fn add_arrow(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> u64 {
        self.document.borrow_mut().add_arrow(Point::new(start_x, start_y), Point::new(end_x, end_y))
    }

    #[wasm_bindgen]
    pub fn add_arrow_without_snapshot(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> u64 {
        self.document.borrow_mut().add_arrow_without_snapshot(Point::new(start_x, start_y), Point::new(end_x, end_y))
    }

    #[wasm_bindgen]
    pub fn get_arrows(&self) -> JsValue {
        let arrows = self.document.borrow().get_arrows().to_vec();
        to_value(&arrows).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_arrow(&self, id: u64, start_x: f64, start_y: f64, end_x: f64, end_y: f64, save_history: bool) {
        self.document.borrow_mut().move_arrow(id, Point::new(start_x, start_y), Point::new(end_x, end_y), save_history);
    }

    #[wasm_bindgen]
    pub fn delete_arrow(&self, id: u64) {
        self.document.borrow_mut().delete_arrow(id);
    }

    #[wasm_bindgen]
    pub fn delete_arrow_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_arrow_without_snapshot(id)
    }

    #[wasm_bindgen]
    pub fn add_path(&self, points: JsValue) -> u64 {
        let points: Vec<Point> = serde_wasm_bindgen::from_value(points).unwrap();
        self.document.borrow_mut().add_path(points)
    }

    #[wasm_bindgen]
    pub fn add_path_without_snapshot(&self, points: JsValue) -> u64 {
        let points: Vec<Point> = serde_wasm_bindgen::from_value(points).unwrap();
        self.document.borrow_mut().add_path_without_snapshot(points)
    }

    #[wasm_bindgen]
    pub fn get_paths(&self) -> JsValue {
        let paths = self.document.borrow().get_paths().to_vec();
        to_value(&paths).unwrap()
    }

    #[wasm_bindgen]
    pub fn delete_path(&self, id: u64) {
        self.document.borrow_mut().delete_path(id);
    }

    #[wasm_bindgen]
    pub fn delete_path_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_path_without_snapshot(id)
    }

    #[wasm_bindgen]
    pub fn set_path_stroke_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_path_stroke_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_path_line_width(&self, id: u64, width: f64, save_history: bool) {
        self.document.borrow_mut().set_path_line_width(id, width, save_history);
    }

    #[wasm_bindgen]
    pub fn set_path_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_path_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_path_points(&self, id: u64, points: JsValue, save_history: bool) {
        let points: Vec<Point> = serde_wasm_bindgen::from_value(points).unwrap();
        self.document.borrow_mut().set_path_points(id, points, save_history);
    }

    #[wasm_bindgen]
    pub fn move_path(&self, id: u64, delta_x: f64, delta_y: f64, save_history: bool) {
        self.document.borrow_mut().move_path(id, delta_x, delta_y, save_history);
    }

    #[wasm_bindgen]
    pub fn resize_path(&self, id: u64, x: f64, y: f64, width: f64, height: f64, save_history: bool) {
        self.document.borrow_mut().resize_path(id, x, y, width, height, save_history);
    }

    #[wasm_bindgen]
    pub fn add_image(&self, x: f64, y: f64, width: f64, height: f64, image_data: String) -> u64 {
        self.document.borrow_mut().add_image(Point::new(x, y), width, height, image_data)
    }

    #[wasm_bindgen]
    pub fn add_image_without_snapshot(&self, x: f64, y: f64, width: f64, height: f64, image_data: String) -> u64 {
        self.document.borrow_mut().add_image_without_snapshot(Point::new(x, y), width, height, image_data)
    }

    #[wasm_bindgen]
    pub fn get_images(&self) -> JsValue {
        let images = self.document.borrow().get_images().to_vec();
        to_value(&images).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_image(&self, id: u64, x: f64, y: f64, save_history: bool) {
        self.document.borrow_mut().move_image(id, Point::new(x, y), save_history);
    }

    #[wasm_bindgen]
    pub fn resize_image(&self, id: u64, width: f64, height: f64, save_history: bool) {
        self.document.borrow_mut().resize_image(id, width, height, save_history);
    }

    #[wasm_bindgen]
    pub fn set_image_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_image_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn delete_image(&self, id: u64) {
        self.document.borrow_mut().delete_image(id);
    }

    #[wasm_bindgen]
    pub fn delete_image_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_image_without_snapshot(id)
    }

    // text
    #[wasm_bindgen]
    pub fn add_text(&self, x: f64, y: f64, width: f64, height: f64, content: String) -> u64 {
        self.document.borrow_mut().add_text(Point::new(x, y), width, height, content)
    }

    #[wasm_bindgen]
    pub fn add_text_without_snapshot(&self, x: f64, y: f64, width: f64, height: f64, content: String) -> u64 {
        self.document.borrow_mut().add_text_without_snapshot(Point::new(x, y), width, height, content)
    }

    #[wasm_bindgen]
    pub fn get_texts(&self) -> JsValue {
        let texts = self.document.borrow().get_texts().to_vec();
        to_value(&texts).unwrap()
    }

    #[wasm_bindgen]
    pub fn move_text(&self, id: u64, x: f64, y: f64, save_history: bool) {
        self.document.borrow_mut().move_text(id, Point::new(x, y), save_history);
    }

    #[wasm_bindgen]
    pub fn resize_text(&self, id: u64, width: f64, height: f64, save_history: bool) {
        self.document.borrow_mut().resize_text(id, width, height, save_history);
    }

    #[wasm_bindgen]
    pub fn delete_text(&self, id: u64) {
        self.document.borrow_mut().delete_text(id);
    }

    #[wasm_bindgen]
    pub fn delete_text_without_snapshot(&self, id: u64) -> bool {
        self.document.borrow_mut().delete_text_without_snapshot(id)
    }

    #[wasm_bindgen]
    pub fn set_text_content(&self, id: u64, content: String, save_history: bool) {
        self.document.borrow_mut().set_text_content(id, content, save_history);
    }

    #[wasm_bindgen]
    pub fn set_text_font_family(&self, id: u64, font_family: String, save_history: bool) {
        self.document.borrow_mut().set_text_font_family(id, font_family, save_history);
    }

    #[wasm_bindgen]
    pub fn set_text_font_size(&self, id: u64, font_size: f64, save_history: bool) {
        self.document.borrow_mut().set_text_font_size(id, font_size, save_history);
    }

    #[wasm_bindgen]
    pub fn set_text_font_weight(&self, id: u64, font_weight: String, save_history: bool) {
        self.document.borrow_mut().set_text_font_weight(id, font_weight, save_history);
    }

    #[wasm_bindgen]
    pub fn set_text_text_align(&self, id: u64, text_align: String, save_history: bool) {
        self.document.borrow_mut().set_text_text_align(id, text_align, save_history);
    }

    #[wasm_bindgen]
    pub fn set_text_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_text_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_text_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_text_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_rectangle_stroke_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_rectangle_stroke_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_rectangle_fill_color(&self, id: u64, color: Option<String>, save_history: bool) {
        self.document.borrow_mut().set_rectangle_fill_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_rectangle_line_width(&self, id: u64, width: f64, save_history: bool) {
        self.document.borrow_mut().set_rectangle_line_width(id, width, save_history);
    }

    #[wasm_bindgen]
    pub fn set_rectangle_dash_pattern(&self, id: u64, pattern: String, save_history: bool) {
        self.document.borrow_mut().set_rectangle_dash_pattern(id, pattern, save_history);
    }

    #[wasm_bindgen]
    pub fn set_rectangle_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_rectangle_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_rectangle_border_radius(&self, id: u64, radius: f64, save_history: bool) {
        self.document.borrow_mut().set_rectangle_border_radius(id, radius, save_history);
    }

    #[wasm_bindgen]
    pub fn set_ellipse_stroke_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_ellipse_stroke_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_ellipse_fill_color(&self, id: u64, color: Option<String>, save_history: bool) {
        self.document.borrow_mut().set_ellipse_fill_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_ellipse_line_width(&self, id: u64, width: f64, save_history: bool) {
        self.document.borrow_mut().set_ellipse_line_width(id, width, save_history);
    }

    #[wasm_bindgen]
    pub fn set_ellipse_dash_pattern(&self, id: u64, pattern: String, save_history: bool) {
        self.document.borrow_mut().set_ellipse_dash_pattern(id, pattern, save_history);
    }

    #[wasm_bindgen]
    pub fn set_ellipse_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_ellipse_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_diamond_stroke_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_diamond_stroke_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_diamond_fill_color(&self, id: u64, color: Option<String>, save_history: bool) {
        self.document.borrow_mut().set_diamond_fill_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_diamond_line_width(&self, id: u64, width: f64, save_history: bool) {
        self.document.borrow_mut().set_diamond_line_width(id, width, save_history);
    }

    #[wasm_bindgen]
    pub fn set_diamond_dash_pattern(&self, id: u64, pattern: String, save_history: bool) {
        self.document.borrow_mut().set_diamond_dash_pattern(id, pattern, save_history);
    }

    #[wasm_bindgen]
    pub fn set_diamond_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_diamond_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_diamond_border_radius(&self, id: u64, radius: f64, save_history: bool) {
        self.document.borrow_mut().set_diamond_border_radius(id, radius, save_history);
    }

    #[wasm_bindgen]
    pub fn set_line_stroke_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_line_stroke_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_line_line_width(&self, id: u64, width: f64, save_history: bool) {
        self.document.borrow_mut().set_line_line_width(id, width, save_history);
    }

    #[wasm_bindgen]
    pub fn set_line_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_line_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_line_dash_pattern(&self, id: u64, pattern: String, save_history: bool) {
        self.document.borrow_mut().set_line_dash_pattern(id, pattern, save_history);
    }

    #[wasm_bindgen]
    pub fn set_arrow_stroke_color(&self, id: u64, color: String, save_history: bool) {
        self.document.borrow_mut().set_arrow_stroke_color(id, color, save_history);
    }

    #[wasm_bindgen]
    pub fn set_arrow_line_width(&self, id: u64, width: f64, save_history: bool) {
        self.document.borrow_mut().set_arrow_line_width(id, width, save_history);
    }

    #[wasm_bindgen]
    pub fn set_arrow_rotation(&self, id: u64, angle: f64, save_history: bool) {
        self.document.borrow_mut().set_arrow_rotation(id, angle, save_history);
    }

    #[wasm_bindgen]
    pub fn set_arrow_dash_pattern(&self, id: u64, pattern: String, save_history: bool) {
        self.document.borrow_mut().set_arrow_dash_pattern(id, pattern, save_history);
    }


    #[wasm_bindgen]
    pub fn group_elements(&self, ids: JsValue) -> u64 {
        let ids: Vec<u64> = serde_wasm_bindgen::from_value(ids).unwrap();
        self.document.borrow_mut().group_elements(ids)
    }

    #[wasm_bindgen]
    pub fn ungroup_elements(&self, group_id: u64) -> JsValue {
        let ids = self.document.borrow_mut().ungroup_elements(group_id);
        to_value(&ids).unwrap()
    }

    #[wasm_bindgen]
    pub fn get_groups(&self) -> JsValue {
        let groups = self.document.borrow().get_groups().to_vec();
        to_value(&groups).unwrap()
    }

    #[wasm_bindgen]
    pub fn bring_shape_to_front(&self, id: u64) {
        self.document.borrow_mut().bring_shape_to_front(id);
    }

    #[wasm_bindgen]
    pub fn bring_shape_forward(&self, id: u64) {
        self.document.borrow_mut().bring_shape_forward(id);
    }

    #[wasm_bindgen]
    pub fn send_shape_backward(&self, id: u64) {
        self.document.borrow_mut().send_shape_backward(id);
    }

    #[wasm_bindgen]
    pub fn send_shape_to_back(&self, id: u64) {
        self.document.borrow_mut().send_shape_to_back(id);
    }

    #[wasm_bindgen]
    pub fn serialize(&self) -> String {
        self.document.borrow().serialize()
    }

    #[wasm_bindgen]
    pub fn deserialize(&self, data: &str) -> bool {
        self.document.borrow_mut().deserialize(data)
    }

    #[wasm_bindgen]
    pub fn save_snapshot(&self) {
        self.document.borrow_mut().save_snapshot();
    }

    #[wasm_bindgen]
    pub fn is_element_locked(&self, id: u64) -> bool {
        self.document.borrow().is_element_locked(id)
    }

    #[wasm_bindgen]
    pub fn set_element_locked(&self, id: u64, locked: bool, save_history: bool) {
        self.document.borrow_mut().set_element_locked(id, locked, save_history);
    }
}
