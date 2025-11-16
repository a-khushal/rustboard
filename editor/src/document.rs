use crate::elements::{Ellipse, Rectangle, Line, Arrow};
use crate::geometry::Point;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DocumentSnapshot {
    rectangles: Vec<Rectangle>,
    ellipses: Vec<Ellipse>,
    lines: Vec<Line>,
    arrows: Vec<Arrow>,
    next_id: u64,
}

pub struct Document {
    rectangles: Vec<Rectangle>,
    ellipses: Vec<Ellipse>,
    lines: Vec<Line>,
    arrows: Vec<Arrow>,
    next_id: u64,
    history: Vec<DocumentSnapshot>,
    history_index: usize,
    max_history: usize,
}

impl Document {
    pub fn new() -> Self {
        let mut doc = Self {
            rectangles: Vec::new(),
            ellipses: Vec::new(),
            lines: Vec::new(),
            arrows: Vec::new(),
            next_id: 0,
            history: Vec::new(),
            history_index: 0,
            max_history: 100,
        };
        doc.save_snapshot();
        doc
    }

    fn save_snapshot(&mut self) {
        let snapshot = DocumentSnapshot {
            rectangles: self.rectangles.clone(),
            ellipses: self.ellipses.clone(),
            lines: self.lines.clone(),
            arrows: self.arrows.clone(),
            next_id: self.next_id,
        };
        
        self.history.truncate(self.history_index);
        self.history.push(snapshot);
        self.history_index = self.history.len();
        
        if self.history.len() > self.max_history {
            self.history.remove(0);
            self.history_index -= 1;
        }
    }

    fn restore_snapshot(&mut self, snapshot: &DocumentSnapshot) {
        self.rectangles = snapshot.rectangles.clone();
        self.ellipses = snapshot.ellipses.clone();
        self.lines = snapshot.lines.clone();
        self.arrows = snapshot.arrows.clone();
        self.next_id = snapshot.next_id;
    }

    pub fn undo(&mut self) -> bool {
        if self.history_index <= 1 {
            return false;
        }
        self.history_index -= 1;
        let snapshot = self.history[self.history_index - 1].clone();
        self.restore_snapshot(&snapshot);
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.history_index >= self.history.len() {
            return false;
        }
        let snapshot = self.history[self.history_index].clone();
        self.restore_snapshot(&snapshot);
        self.history_index += 1;
        true
    }

    pub fn can_undo(&self) -> bool {
        self.history_index > 1
    }

    pub fn can_redo(&self) -> bool {
        self.history_index < self.history.len()
    }

    pub fn add_rectangle(&mut self, position: Point, width: f64, height: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.rectangles.push(Rectangle::new(id, position, width, height));
        self.save_snapshot();
        id
    }

    pub fn get_rectangles(&self) -> &[Rectangle] {
        &self.rectangles
    }

    pub fn move_rectangle(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(rect) = self.rectangles.iter().find(|r| r.id == id) {
            if rect.position != new_position {
                if save_history {
                    self.save_snapshot();
                }
                if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
                    rect.position = new_position;
                }
            }
        }
    }

    pub fn resize_rectangle(&mut self, id: u64, width: f64, height: f64, save_history: bool) {
        if let Some(rect) = self.rectangles.iter().find(|r| r.id == id) {
            if rect.width != width || rect.height != height {
                if save_history {
                    self.save_snapshot();
                }
                if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
                    rect.width = width;
                    rect.height = height;
                }
            }
        }
    }

    pub fn delete_rectangle(&mut self, id: u64) {
        let existed = self.rectangles.iter().any(|r| r.id == id);
            self.rectangles.retain(|r| r.id != id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn add_ellipse(&mut self, position: Point, radius_x: f64, radius_y: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.ellipses.push(Ellipse::new(id, position, radius_x, radius_y));
        self.save_snapshot();
        id
    }

    pub fn get_ellipses(&self) -> &[Ellipse] {
        &self.ellipses
    }

    pub fn move_ellipse(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter().find(|e| e.id == id) {
            if ellipse.position != new_position {
                if save_history {
                    self.save_snapshot();
                }
                if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
                    ellipse.position = new_position;
                }
            }
        }
    }

    pub fn resize_ellipse(&mut self, id: u64, radius_x: f64, radius_y: f64, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter().find(|e| e.id == id) {
            if ellipse.radius_x != radius_x || ellipse.radius_y != radius_y {
                if save_history {
                    self.save_snapshot();
                }
                if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
                    ellipse.radius_x = radius_x;
                    ellipse.radius_y = radius_y;
                }
            }
        }
    }

    pub fn delete_ellipse(&mut self, id: u64) {
        let existed = self.ellipses.iter().any(|e| e.id == id);
            self.ellipses.retain(|e| e.id != id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn add_line(&mut self, start: Point, end: Point) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.lines.push(Line::new(id, start, end));
        self.save_snapshot();
        id
    }

    pub fn get_lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn move_line(&mut self, id: u64, new_start: Point, new_end: Point, save_history: bool) {
        if let Some(line) = self.lines.iter().find(|l| l.id == id) {
            if line.start != new_start || line.end != new_end {
                if save_history {
                    self.save_snapshot();
                }
                if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
                    line.start = new_start;
                    line.end = new_end;
                }
            }
        }
    }

    pub fn delete_line(&mut self, id: u64) {
        let existed = self.lines.iter().any(|l| l.id == id);
        self.lines.retain(|l| l.id != id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn add_arrow(&mut self, start: Point, end: Point) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.arrows.push(Arrow::new(id, start, end));
        self.save_snapshot();
        id
    }

    pub fn get_arrows(&self) -> &[Arrow] {
        &self.arrows
    }

    pub fn move_arrow(&mut self, id: u64, new_start: Point, new_end: Point, save_history: bool) {
        if let Some(arrow) = self.arrows.iter().find(|a| a.id == id) {
            if arrow.start != new_start || arrow.end != new_end {
                if save_history {
                    self.save_snapshot();
                }
                if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
                    arrow.start = new_start;
                    arrow.end = new_end;
                }
            }
        }
    }

    pub fn delete_arrow(&mut self, id: u64) {
        let existed = self.arrows.iter().any(|a| a.id == id);
        self.arrows.retain(|a| a.id != id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn serialize(&self) -> String {
        let snapshot = DocumentSnapshot {
            rectangles: self.rectangles.clone(),
            ellipses: self.ellipses.clone(),
            lines: self.lines.clone(),
            arrows: self.arrows.clone(),
            next_id: self.next_id,
        };
        serde_json::to_string(&snapshot).unwrap_or_default()
    }

    pub fn deserialize(&mut self, data: &str) -> bool {
        match serde_json::from_str::<DocumentSnapshot>(data) {
            Ok(snapshot) => {
                self.restore_snapshot(&snapshot);
                self.history.clear();
                self.history_index = 0;
                self.save_snapshot();
                true
            }
            Err(_) => false,
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
