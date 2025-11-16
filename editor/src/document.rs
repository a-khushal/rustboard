use crate::elements::{Ellipse, Rectangle};
use crate::geometry::Point;

pub struct Document {
    rectangles: Vec<Rectangle>,
    ellipses: Vec<Ellipse>,
    next_id: u64,
}

impl Document {
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
            ellipses: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_rectangle(&mut self, position: Point, width: f64, height: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.rectangles.push(Rectangle::new(id, position, width, height));
        id
    }

    pub fn get_rectangles(&self) -> &[Rectangle] {
        &self.rectangles
    }

    pub fn move_rectangle(&mut self, id: u64, new_position: Point) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            rect.position = new_position;
        }
    }

    pub fn resize_rectangle(&mut self, id: u64, width: f64, height: f64) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            rect.width = width;
            rect.height = height;
        }
    }

    pub fn delete_rectangle(&mut self, id: u64) {
        self.rectangles.retain(|r| r.id != id);
    }

    pub fn add_ellipse(&mut self, position: Point, radius_x: f64, radius_y: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.ellipses.push(Ellipse::new(id, position, radius_x, radius_y));
        id
    }

    pub fn get_ellipses(&self) -> &[Ellipse] {
        &self.ellipses
    }

    pub fn move_ellipse(&mut self, id: u64, new_position: Point) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            ellipse.position = new_position;
        }
    }

    pub fn resize_ellipse(&mut self, id: u64, radius_x: f64, radius_y: f64) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            ellipse.radius_x = radius_x;
            ellipse.radius_y = radius_y;
        }
    }

    pub fn delete_ellipse(&mut self, id: u64) {
        self.ellipses.retain(|e| e.id != id);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
