const DEFAULT_TEXT_FONT_SIZE: f64 = 16.0;

use crate::elements::{Arrow, Diamond, Ellipse, Group, Line, Rectangle};
use crate::geometry::Point;
use crate::Text;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DocumentSnapshot {
    rectangles: Vec<Rectangle>,
    ellipses: Vec<Ellipse>,
    lines: Vec<Line>,
    arrows: Vec<Arrow>,
    diamonds: Vec<Diamond>,
    texts: Vec<Text>,
    groups: Vec<Group>,
    next_id: u64,
}

pub struct Document {
    rectangles: Vec<Rectangle>,
    ellipses: Vec<Ellipse>,
    lines: Vec<Line>,
    arrows: Vec<Arrow>,
    diamonds: Vec<Diamond>,
    texts: Vec<Text>,
    groups: Vec<Group>,
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
            diamonds: Vec::new(),
            texts: Vec::new(),
            groups: Vec::new(),
            next_id: 0,
            history: Vec::new(),
            history_index: 0,
            max_history: 100,
        };
        doc.save_snapshot();
        doc
    }

    pub fn save_snapshot(&mut self) {
        let snapshot = DocumentSnapshot {
            rectangles: self.rectangles.clone(),
            ellipses: self.ellipses.clone(),
            lines: self.lines.clone(),
            arrows: self.arrows.clone(),
            diamonds: self.diamonds.clone(),
            texts: self.texts.clone(),
            groups: self.groups.clone(),
            next_id: self.next_id,
        };

        if self.history_index > 0 {
            let last_snapshot = &self.history[self.history_index - 1];
            if last_snapshot.rectangles == snapshot.rectangles
                && last_snapshot.ellipses == snapshot.ellipses
                && last_snapshot.lines == snapshot.lines
                && last_snapshot.arrows == snapshot.arrows
                && last_snapshot.diamonds == snapshot.diamonds
                && last_snapshot.texts == snapshot.texts
                && last_snapshot.groups == snapshot.groups
                && last_snapshot.next_id == snapshot.next_id
            {
                return;
            }
        }

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
        self.diamonds = snapshot.diamonds.clone();
        self.texts = snapshot.texts.clone();
        self.groups = snapshot.groups.clone();
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

    pub fn history_index(&self) -> usize {
        self.history_index
    }

    pub fn add_rectangle(&mut self, position: Point, width: f64, height: f64) -> u64 {
        let id = self.add_rectangle_without_snapshot(position, width, height);
        self.save_snapshot();
        id
    }

    pub fn add_rectangle_without_snapshot(
        &mut self,
        position: Point,
        width: f64,
        height: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.rectangles
            .push(Rectangle::new(id, position, width, height));
        id
    }

    pub fn get_rectangles(&self) -> &[Rectangle] {
        &self.rectangles
    }

    pub fn move_rectangle(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(rect) = self.rectangles.iter().find(|r| r.id == id) {
            if rect.position != new_position {
                if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
                    rect.position = new_position;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn resize_rectangle(&mut self, id: u64, width: f64, height: f64, save_history: bool) {
        if let Some(rect) = self.rectangles.iter().find(|r| r.id == id) {
            if rect.width != width || rect.height != height {
                if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
                    rect.width = width;
                    rect.height = height;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_rectangle(&mut self, id: u64) {
        let existed = self.delete_rectangle_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_rectangle_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.rectangles.iter().any(|r| r.id == id);
        self.rectangles.retain(|r| r.id != id);
        existed
    }

    pub fn set_rectangle_stroke_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            if rect.stroke_color != color {
                rect.stroke_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_rectangle_fill_color(&mut self, id: u64, color: Option<String>, save_history: bool) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            if rect.fill_color != color {
                rect.fill_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_rectangle_line_width(&mut self, id: u64, width: f64, save_history: bool) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            if (rect.line_width - width).abs() > f64::EPSILON {
                rect.line_width = width.max(0.1);
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_rectangle_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            if (rect.rotation_angle - angle).abs() > f64::EPSILON {
                rect.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn add_diamond(&mut self, position: Point, width: f64, height: f64) -> u64 {
        let id = self.add_diamond_without_snapshot(position, width, height);
        self.save_snapshot();
        id
    }

    pub fn add_diamond_without_snapshot(
        &mut self,
        position: Point,
        width: f64,
        height: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.diamonds
            .push(Diamond::new(id, position, width, height));
        id
    }

    pub fn get_diamonds(&self) -> &[Diamond] {
        &self.diamonds
    }

    pub fn move_diamond(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter().find(|d| d.id == id) {
            if diamond.position != new_position {
                if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
                    diamond.position = new_position;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn resize_diamond(&mut self, id: u64, width: f64, height: f64, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter().find(|d| d.id == id) {
            if diamond.width != width || diamond.height != height {
                if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
                    diamond.width = width;
                    diamond.height = height;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_diamond(&mut self, id: u64) {
        let existed = self.delete_diamond_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_diamond_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.diamonds.iter().any(|d| d.id == id);
        self.diamonds.retain(|d| d.id != id);
        existed
    }

    pub fn set_diamond_stroke_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            if diamond.stroke_color != color {
                diamond.stroke_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_diamond_fill_color(&mut self, id: u64, color: Option<String>, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            if diamond.fill_color != color {
                diamond.fill_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_diamond_line_width(&mut self, id: u64, width: f64, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            if (diamond.line_width - width).abs() > f64::EPSILON {
                diamond.line_width = width.max(0.1);
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_diamond_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            if (diamond.rotation_angle - angle).abs() > f64::EPSILON {
                diamond.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn add_ellipse(&mut self, position: Point, radius_x: f64, radius_y: f64) -> u64 {
        let id = self.add_ellipse_without_snapshot(position, radius_x, radius_y);
        self.save_snapshot();
        id
    }

    pub fn add_ellipse_without_snapshot(
        &mut self,
        position: Point,
        radius_x: f64,
        radius_y: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.ellipses
            .push(Ellipse::new(id, position, radius_x, radius_y));
        id
    }

    pub fn get_ellipses(&self) -> &[Ellipse] {
        &self.ellipses
    }

    pub fn move_ellipse(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter().find(|e| e.id == id) {
            if ellipse.position != new_position {
                if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
                    ellipse.position = new_position;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn resize_ellipse(&mut self, id: u64, radius_x: f64, radius_y: f64, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter().find(|e| e.id == id) {
            if ellipse.radius_x != radius_x || ellipse.radius_y != radius_y {
                if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
                    ellipse.radius_x = radius_x;
                    ellipse.radius_y = radius_y;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_ellipse(&mut self, id: u64) {
        let existed = self.delete_ellipse_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_ellipse_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.ellipses.iter().any(|e| e.id == id);
        self.ellipses.retain(|e| e.id != id);
        existed
    }

    pub fn set_ellipse_stroke_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            if ellipse.stroke_color != color {
                ellipse.stroke_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_ellipse_fill_color(&mut self, id: u64, color: Option<String>, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            if ellipse.fill_color != color {
                ellipse.fill_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_ellipse_line_width(&mut self, id: u64, width: f64, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            if (ellipse.line_width - width).abs() > f64::EPSILON {
                ellipse.line_width = width.max(0.1);
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_ellipse_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            if (ellipse.rotation_angle - angle).abs() > f64::EPSILON {
                ellipse.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn add_line(&mut self, start: Point, end: Point) -> u64 {
        let id = self.add_line_without_snapshot(start, end);
        self.save_snapshot();
        id
    }

    pub fn add_line_without_snapshot(&mut self, start: Point, end: Point) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.lines.push(Line::new(id, start, end));
        id
    }

    pub fn get_lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn move_line(&mut self, id: u64, new_start: Point, new_end: Point, save_history: bool) {
        if let Some(line) = self.lines.iter().find(|l| l.id == id) {
            if line.start != new_start || line.end != new_end {
                if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
                    line.start = new_start;
                    line.end = new_end;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_line(&mut self, id: u64) {
        let existed = self.delete_line_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_line_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.lines.iter().any(|l| l.id == id);
        self.lines.retain(|l| l.id != id);
        existed
    }

    pub fn set_line_stroke_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            if line.stroke_color != color {
                line.stroke_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_line_line_width(&mut self, id: u64, width: f64, save_history: bool) {
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            if (line.line_width - width).abs() > f64::EPSILON {
                line.line_width = width.max(0.1);
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_line_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            if (line.rotation_angle - angle).abs() > f64::EPSILON {
                line.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn add_arrow(&mut self, start: Point, end: Point) -> u64 {
        let id = self.add_arrow_without_snapshot(start, end);
        self.save_snapshot();
        id
    }

    pub fn add_arrow_without_snapshot(&mut self, start: Point, end: Point) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.arrows.push(Arrow::new(id, start, end));
        id
    }

    pub fn get_arrows(&self) -> &[Arrow] {
        &self.arrows
    }

    pub fn move_arrow(&mut self, id: u64, new_start: Point, new_end: Point, save_history: bool) {
        if let Some(arrow) = self.arrows.iter().find(|a| a.id == id) {
            if arrow.start != new_start || arrow.end != new_end {
                if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
                    arrow.start = new_start;
                    arrow.end = new_end;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_arrow(&mut self, id: u64) {
        let existed = self.delete_arrow_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_arrow_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.arrows.iter().any(|a| a.id == id);
        self.arrows.retain(|a| a.id != id);
        existed
    }

    pub fn set_arrow_stroke_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            if arrow.stroke_color != color {
                arrow.stroke_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_arrow_line_width(&mut self, id: u64, width: f64, save_history: bool) {
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            if (arrow.line_width - width).abs() > f64::EPSILON {
                arrow.line_width = width.max(0.1);
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_arrow_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            if (arrow.rotation_angle - angle).abs() > f64::EPSILON {
                arrow.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn add_text(&mut self, position: Point, text: String) -> u64 {
        self.add_text_with_size(position, text, DEFAULT_TEXT_FONT_SIZE, true)
    }

    pub fn add_text_without_snapshot(&mut self, position: Point, text: String) -> u64 {
        self.add_text_with_size(position, text, DEFAULT_TEXT_FONT_SIZE, false)
    }

    pub fn add_text_with_size_without_snapshot(&mut self, position: Point, text: String, font_size: f64) -> u64 {
        self.add_text_with_size(position, text, font_size, false)
    }

    fn add_text_with_size(&mut self, position: Point, text: String, font_size: f64, save_snapshot: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.texts.push(Text::new(id, position, text, font_size));
        if save_snapshot {
            self.save_snapshot();
        }
        id
    }

    pub fn get_texts(&self) -> &[Text] {
        &self.texts
    }

    pub fn move_text(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(text) = self.texts.iter().find(|t| t.id == id) {
            if text.position != new_position {
                if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
                    text.position = new_position;
                }
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn update_text(&mut self, id: u64, new_text: String, save_history: bool) {
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            if text.text != new_text {
                text.text = new_text;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_text(&mut self, id: u64) {
        let existed = self.delete_text_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_text_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.texts.iter().any(|t| t.id == id);
        self.texts.retain(|t| t.id != id);
        existed
    }

    pub fn resize_text(&mut self, id: u64, new_font_size: f64, save_history: bool) {
        let clamped = new_font_size.max(4.0);
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            if (text.font_size - clamped).abs() > f64::EPSILON {
                text.font_size = clamped;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn resize_text_without_snapshot(&mut self, id: u64, new_font_size: f64) {
        let clamped = new_font_size.max(4.0);
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            text.font_size = clamped;
        }
    }

    pub fn set_text_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            if text.text_color != color {
                text.text_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_text_box_width(&mut self, id: u64, width: Option<f64>, save_history: bool) {
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            let normalized = width.and_then(|w| {
                if w.is_finite() && w > 0.0 {
                    Some(w)
                } else {
                    None
                }
            });
            if text.box_width != normalized {
                text.box_width = normalized;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_text_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            if (text.rotation_angle - angle).abs() > f64::EPSILON {
                text.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn group_elements(&mut self, element_ids: Vec<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let group = Group::new(id, element_ids);
        self.groups.push(group);
        self.save_snapshot();
        id
    }

    pub fn ungroup_elements(&mut self, group_id: u64) -> Vec<u64> {
        if let Some(index) = self.groups.iter().position(|g| g.id == group_id) {
            let group = self.groups.remove(index);
            self.save_snapshot();
            return group.element_ids;
        }
        Vec::new()
    }

    pub fn get_groups(&self) -> &[Group] {
        &self.groups
    }

    pub fn bring_shape_to_front(&mut self, id: u64) {
        let max_z = self.get_max_z_index();
        let new_z = max_z + 1;
        
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            rect.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            ellipse.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            diamond.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            line.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            arrow.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            text.z_index = new_z;
            self.save_snapshot();
        }
    }

    pub fn bring_shape_forward(&mut self, id: u64) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            rect.z_index += 1;
            self.save_snapshot();
            return;
        }
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            ellipse.z_index += 1;
            self.save_snapshot();
            return;
        }
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            diamond.z_index += 1;
            self.save_snapshot();
            return;
        }
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            line.z_index += 1;
            self.save_snapshot();
            return;
        }
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            arrow.z_index += 1;
            self.save_snapshot();
            return;
        }
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            text.z_index += 1;
            self.save_snapshot();
        }
    }

    pub fn send_shape_backward(&mut self, id: u64) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            rect.z_index -= 1;
            self.save_snapshot();
            return;
        }
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            ellipse.z_index -= 1;
            self.save_snapshot();
            return;
        }
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            diamond.z_index -= 1;
            self.save_snapshot();
            return;
        }
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            line.z_index -= 1;
            self.save_snapshot();
            return;
        }
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            arrow.z_index -= 1;
            self.save_snapshot();
            return;
        }
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            text.z_index -= 1;
            self.save_snapshot();
        }
    }

    pub fn send_shape_to_back(&mut self, id: u64) {
        let min_z = self.get_min_z_index();
        let new_z = min_z - 1;
        
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            rect.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            ellipse.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            diamond.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            line.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            arrow.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(text) = self.texts.iter_mut().find(|t| t.id == id) {
            text.z_index = new_z;
            self.save_snapshot();
        }
    }

    fn get_max_z_index(&self) -> i32 {
        let mut max_z = 0i32;
        for rect in &self.rectangles {
            max_z = max_z.max(rect.z_index);
        }
        for ellipse in &self.ellipses {
            max_z = max_z.max(ellipse.z_index);
        }
        for diamond in &self.diamonds {
            max_z = max_z.max(diamond.z_index);
        }
        for line in &self.lines {
            max_z = max_z.max(line.z_index);
        }
        for arrow in &self.arrows {
            max_z = max_z.max(arrow.z_index);
        }
        for text in &self.texts {
            max_z = max_z.max(text.z_index);
        }
        max_z
    }

    fn get_min_z_index(&self) -> i32 {
        let mut min_z = 0i32;
        for rect in &self.rectangles {
            min_z = min_z.min(rect.z_index);
        }
        for ellipse in &self.ellipses {
            min_z = min_z.min(ellipse.z_index);
        }
        for diamond in &self.diamonds {
            min_z = min_z.min(diamond.z_index);
        }
        for line in &self.lines {
            min_z = min_z.min(line.z_index);
        }
        for arrow in &self.arrows {
            min_z = min_z.min(arrow.z_index);
        }
        for text in &self.texts {
            min_z = min_z.min(text.z_index);
        }
        min_z
    }


    pub fn serialize(&self) -> String {
        let snapshot = DocumentSnapshot {
            rectangles: self.rectangles.clone(),
            ellipses: self.ellipses.clone(),
            lines: self.lines.clone(),
            arrows: self.arrows.clone(),
            diamonds: self.diamonds.clone(),
            texts: self.texts.clone(),
            groups: self.groups.clone(),
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
