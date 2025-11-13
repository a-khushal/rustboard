use crate::elements::Rectangle;
use crate::geometry::Point;

pub struct Document {
    rectangles: Vec<Rectangle>,
    next_id: u64,
}

impl Document {
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
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
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
