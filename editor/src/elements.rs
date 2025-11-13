use crate::geometry::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Rectangle {
    pub id: u64,
    pub position: Point,
    pub width: f64,
    pub height: f64,
}

impl Rectangle {
    pub fn new(id: u64, position: Point, width: f64, height: f64) -> Self {
        Self {
            id,
            position,
            width,
            height,
        }
    }
}
