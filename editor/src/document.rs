const DEFAULT_TEXT_FONT_SIZE: f64 = 16.0;

use crate::elements::{Arrow, Diamond, Ellipse, Group, Image, Line, Rectangle, Path};
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
    paths: Vec<Path>,
    images: Vec<Image>,
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
    paths: Vec<Path>,
    images: Vec<Image>,
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
            paths: Vec::new(),
            images: Vec::new(),
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
            paths: self.paths.clone(),
            images: self.images.clone(),
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
                && last_snapshot.paths == snapshot.paths
                && last_snapshot.images == snapshot.images
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
        self.paths = snapshot.paths.clone();
        self.images = snapshot.images.clone();
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
        let mut rect = Rectangle::new(id, position, width, height);
        rect.z_index = self.get_max_z_index() + 1;
        self.rectangles.push(rect);
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

    pub fn set_rectangle_dash_pattern(&mut self, id: u64, pattern: String, save_history: bool) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            if rect.dash_pattern != pattern {
                rect.dash_pattern = pattern;
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

    pub fn set_rectangle_border_radius(&mut self, id: u64, radius: f64, save_history: bool) {
        if let Some(rect) = self.rectangles.iter_mut().find(|r| r.id == id) {
            if (rect.border_radius - radius).abs() > f64::EPSILON {
                rect.border_radius = radius.max(0.0);
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
        let mut diamond = Diamond::new(id, position, width, height);
        diamond.z_index = self.get_max_z_index() + 1;
        self.diamonds.push(diamond);
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

    pub fn set_diamond_dash_pattern(&mut self, id: u64, pattern: String, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            if diamond.dash_pattern != pattern {
                diamond.dash_pattern = pattern;
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

    pub fn set_diamond_border_radius(&mut self, id: u64, radius: f64, save_history: bool) {
        if let Some(diamond) = self.diamonds.iter_mut().find(|d| d.id == id) {
            if (diamond.border_radius - radius).abs() > f64::EPSILON {
                diamond.border_radius = radius.max(0.0);
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
        let mut ellipse = Ellipse::new(id, position, radius_x, radius_y);
        ellipse.z_index = self.get_max_z_index() + 1;
        self.ellipses.push(ellipse);
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

    pub fn set_ellipse_dash_pattern(&mut self, id: u64, pattern: String, save_history: bool) {
        if let Some(ellipse) = self.ellipses.iter_mut().find(|e| e.id == id) {
            if ellipse.dash_pattern != pattern {
                ellipse.dash_pattern = pattern;
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
        let mut line = Line::new(id, start, end);
        line.z_index = self.get_max_z_index() + 1;
        self.lines.push(line);
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

    pub fn set_line_dash_pattern(&mut self, id: u64, pattern: String, save_history: bool) {
        if let Some(line) = self.lines.iter_mut().find(|l| l.id == id) {
            if line.dash_pattern != pattern {
                line.dash_pattern = pattern;
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
        let mut arrow = Arrow::new(id, start, end);
        arrow.z_index = self.get_max_z_index() + 1;
        self.arrows.push(arrow);
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

    pub fn set_arrow_dash_pattern(&mut self, id: u64, pattern: String, save_history: bool) {
        if let Some(arrow) = self.arrows.iter_mut().find(|a| a.id == id) {
            if arrow.dash_pattern != pattern {
                arrow.dash_pattern = pattern;
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
        let mut text_elem = Text::new(id, position, text, font_size);
        text_elem.z_index = self.get_max_z_index() + 1;
        self.texts.push(text_elem);
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

    pub fn add_path(&mut self, points: Vec<Point>) -> u64 {
        let id = self.add_path_without_snapshot(points);
        self.save_snapshot();
        id
    }

    pub fn add_path_without_snapshot(&mut self, points: Vec<Point>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut path = Path::new(id, points);
        path.z_index = self.get_max_z_index() + 1;
        self.paths.push(path);
        id
    }

    pub fn get_paths(&self) -> &[Path] {
        &self.paths
    }

    pub fn delete_path(&mut self, id: u64) {
        let existed = self.delete_path_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_path_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.paths.iter().any(|p| p.id == id);
        self.paths.retain(|p| p.id != id);
        existed
    }

    pub fn set_path_stroke_color(&mut self, id: u64, color: String, save_history: bool) {
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            if path.stroke_color != color {
                path.stroke_color = color;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_path_line_width(&mut self, id: u64, width: f64, save_history: bool) {
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            if (path.line_width - width).abs() > f64::EPSILON {
                path.line_width = width.max(0.1);
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn move_path(&mut self, id: u64, delta_x: f64, delta_y: f64, save_history: bool) {
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            for point in &mut path.points {
                point.x += delta_x;
                point.y += delta_y;
            }
            if save_history {
                self.save_snapshot();
            }
        }
    }

    pub fn resize_path(&mut self, id: u64, new_x: f64, new_y: f64, new_width: f64, new_height: f64, save_history: bool) {
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            if path.points.is_empty() {
                return;
            }

            let mut min_x = path.points[0].x;
            let mut min_y = path.points[0].y;
            let mut max_x = path.points[0].x;
            let mut max_y = path.points[0].y;

            for point in &path.points {
                min_x = min_x.min(point.x);
                min_y = min_y.min(point.y);
                max_x = max_x.max(point.x);
                max_y = max_y.max(point.y);
            }

            let old_width = max_x - min_x;
            let old_height = max_y - min_y;

            // Avoid division by zero
            if old_width.abs() < f64::EPSILON || old_height.abs() < f64::EPSILON {
                return;
            }

            // Calculate scale factors
            let scale_x = new_width / old_width;
            let scale_y = new_height / old_height;

            // Scale and translate all points
            for point in &mut path.points {
                let relative_x = point.x - min_x;
                let relative_y = point.y - min_y;
                point.x = new_x + relative_x * scale_x;
                point.y = new_y + relative_y * scale_y;
            }

            if save_history {
                self.save_snapshot();
            }
        }
    }

    pub fn set_path_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            if (path.rotation_angle - angle).abs() > f64::EPSILON {
                path.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn set_path_points(&mut self, id: u64, points: Vec<Point>, save_history: bool) {
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            if path.points != points {
                path.points = points;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn add_image(&mut self, position: Point, width: f64, height: f64, image_data: String) -> u64 {
        let id = self.add_image_without_snapshot(position, width, height, image_data);
        self.save_snapshot();
        id
    }

    pub fn add_image_without_snapshot(&mut self, position: Point, width: f64, height: f64, image_data: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut image = Image::new(id, position, width, height, image_data);
        image.z_index = self.get_max_z_index() + 1;
        self.images.push(image);
        id
    }

    pub fn get_images(&self) -> &[Image] {
        &self.images
    }

    pub fn move_image(&mut self, id: u64, new_position: Point, save_history: bool) {
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            image.position = new_position;
            if save_history {
                self.save_snapshot();
            }
        }
    }

    pub fn resize_image(&mut self, id: u64, width: f64, height: f64, save_history: bool) {
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            image.width = width.max(1.0);
            image.height = height.max(1.0);
            if save_history {
                self.save_snapshot();
            }
        }
    }

    pub fn set_image_rotation(&mut self, id: u64, angle: f64, save_history: bool) {
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            if (image.rotation_angle - angle).abs() > f64::EPSILON {
                image.rotation_angle = angle;
                if save_history {
                    self.save_snapshot();
                }
            }
        }
    }

    pub fn delete_image(&mut self, id: u64) {
        let existed = self.delete_image_without_snapshot(id);
        if existed {
            self.save_snapshot();
        }
    }

    pub fn delete_image_without_snapshot(&mut self, id: u64) -> bool {
        let existed = self.images.iter().any(|i| i.id == id);
        self.images.retain(|i| i.id != id);
        existed
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
            return;
        }
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            path.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            image.z_index = new_z;
            self.save_snapshot();
        }
    }

    pub fn bring_shape_forward(&mut self, id: u64) {
        let current_z = self.get_shape_z_index(id);
        if current_z.is_none() {
            return;
        }
        let current_z = current_z.unwrap();
        let max_z = self.get_max_z_index();
        
        if current_z >= max_z {
            return;
        }
        
        let mut next_z: Option<i32> = None;
        for rect in &self.rectangles {
            if rect.id != id && rect.z_index > current_z {
                next_z = Some(next_z.map_or(rect.z_index, |z: i32| z.min(rect.z_index)));
            }
        }
        for ellipse in &self.ellipses {
            if ellipse.id != id && ellipse.z_index > current_z {
                next_z = Some(next_z.map_or(ellipse.z_index, |z: i32| z.min(ellipse.z_index)));
            }
        }
        for diamond in &self.diamonds {
            if diamond.id != id && diamond.z_index > current_z {
                next_z = Some(next_z.map_or(diamond.z_index, |z: i32| z.min(diamond.z_index)));
            }
        }
        for line in &self.lines {
            if line.id != id && line.z_index > current_z {
                next_z = Some(next_z.map_or(line.z_index, |z: i32| z.min(line.z_index)));
            }
        }
        for arrow in &self.arrows {
            if arrow.id != id && arrow.z_index > current_z {
                next_z = Some(next_z.map_or(arrow.z_index, |z: i32| z.min(arrow.z_index)));
            }
        }
        for text in &self.texts {
            if text.id != id && text.z_index > current_z {
                next_z = Some(next_z.map_or(text.z_index, |z: i32| z.min(text.z_index)));
            }
        }
        for path in &self.paths {
            if path.id != id && path.z_index > current_z {
                next_z = Some(next_z.map_or(path.z_index, |z: i32| z.min(path.z_index)));
            }
        }
        for image in &self.images {
            if image.id != id && image.z_index > current_z {
                next_z = Some(next_z.map_or(image.z_index, |z: i32| z.min(image.z_index)));
            }
        }
        
        let new_z = next_z.unwrap_or(current_z + 1);
        
        for rect in &mut self.rectangles {
            if rect.id != id && rect.z_index == new_z {
                rect.z_index = current_z;
                break;
            }
        }
        for ellipse in &mut self.ellipses {
            if ellipse.id != id && ellipse.z_index == new_z {
                ellipse.z_index = current_z;
                break;
            }
        }
        for diamond in &mut self.diamonds {
            if diamond.id != id && diamond.z_index == new_z {
                diamond.z_index = current_z;
                break;
            }
        }
        for line in &mut self.lines {
            if line.id != id && line.z_index == new_z {
                line.z_index = current_z;
                break;
            }
        }
        for arrow in &mut self.arrows {
            if arrow.id != id && arrow.z_index == new_z {
                arrow.z_index = current_z;
                break;
            }
        }
        for text in &mut self.texts {
            if text.id != id && text.z_index == new_z {
                text.z_index = current_z;
                break;
            }
        }
        for path in &mut self.paths {
            if path.id != id && path.z_index == new_z {
                path.z_index = current_z;
                break;
            }
        }
        for image in &mut self.images {
            if image.id != id && image.z_index == new_z {
                image.z_index = current_z;
                break;
            }
        }
        
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
            return;
        }
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            path.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            image.z_index = new_z;
            self.save_snapshot();
        }
    }

    pub fn send_shape_backward(&mut self, id: u64) {
        let current_z = self.get_shape_z_index(id);
        if current_z.is_none() {
            return;
        }
        let current_z = current_z.unwrap();
        
        if current_z <= 0 {
            return;
        }
        
        let mut prev_z: Option<i32> = None;
        for rect in &self.rectangles {
            if rect.id != id && rect.z_index < current_z {
                prev_z = Some(prev_z.map_or(rect.z_index, |z: i32| z.max(rect.z_index)));
            }
        }
        for ellipse in &self.ellipses {
            if ellipse.id != id && ellipse.z_index < current_z {
                prev_z = Some(prev_z.map_or(ellipse.z_index, |z: i32| z.max(ellipse.z_index)));
            }
        }
        for diamond in &self.diamonds {
            if diamond.id != id && diamond.z_index < current_z {
                prev_z = Some(prev_z.map_or(diamond.z_index, |z: i32| z.max(diamond.z_index)));
            }
        }
        for line in &self.lines {
            if line.id != id && line.z_index < current_z {
                prev_z = Some(prev_z.map_or(line.z_index, |z: i32| z.max(line.z_index)));
            }
        }
        for arrow in &self.arrows {
            if arrow.id != id && arrow.z_index < current_z {
                prev_z = Some(prev_z.map_or(arrow.z_index, |z: i32| z.max(arrow.z_index)));
            }
        }
        for text in &self.texts {
            if text.id != id && text.z_index < current_z {
                prev_z = Some(prev_z.map_or(text.z_index, |z: i32| z.max(text.z_index)));
            }
        }
        for path in &self.paths {
            if path.id != id && path.z_index < current_z {
                prev_z = Some(prev_z.map_or(path.z_index, |z: i32| z.max(path.z_index)));
            }
        }
        for image in &self.images {
            if image.id != id && image.z_index < current_z {
                prev_z = Some(prev_z.map_or(image.z_index, |z: i32| z.max(image.z_index)));
            }
        }
        
        let new_z = prev_z.unwrap_or(current_z - 1);
        
        for rect in &mut self.rectangles {
            if rect.id != id && rect.z_index == new_z {
                rect.z_index = current_z;
                break;
            }
        }
        for ellipse in &mut self.ellipses {
            if ellipse.id != id && ellipse.z_index == new_z {
                ellipse.z_index = current_z;
                break;
            }
        }
        for diamond in &mut self.diamonds {
            if diamond.id != id && diamond.z_index == new_z {
                diamond.z_index = current_z;
                break;
            }
        }
        for line in &mut self.lines {
            if line.id != id && line.z_index == new_z {
                line.z_index = current_z;
                break;
            }
        }
        for arrow in &mut self.arrows {
            if arrow.id != id && arrow.z_index == new_z {
                arrow.z_index = current_z;
                break;
            }
        }
        for text in &mut self.texts {
            if text.id != id && text.z_index == new_z {
                text.z_index = current_z;
                break;
            }
        }
        for path in &mut self.paths {
            if path.id != id && path.z_index == new_z {
                path.z_index = current_z;
                break;
            }
        }
        for image in &mut self.images {
            if image.id != id && image.z_index == new_z {
                image.z_index = current_z;
                break;
            }
        }
        
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
            return;
        }
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            path.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            image.z_index = new_z;
            self.save_snapshot();
        }
    }

    pub fn send_shape_to_back(&mut self, id: u64) {
        let new_z = 0;
        
        let current_z = self.get_shape_z_index(id);
        if current_z.is_none() || current_z.unwrap() == 0 {
            return;
        }
        
        for rect in &mut self.rectangles {
            if rect.id != id && rect.z_index < current_z.unwrap() {
                rect.z_index += 1;
            }
        }
        for ellipse in &mut self.ellipses {
            if ellipse.id != id && ellipse.z_index < current_z.unwrap() {
                ellipse.z_index += 1;
            }
        }
        for diamond in &mut self.diamonds {
            if diamond.id != id && diamond.z_index < current_z.unwrap() {
                diamond.z_index += 1;
            }
        }
        for line in &mut self.lines {
            if line.id != id && line.z_index < current_z.unwrap() {
                line.z_index += 1;
            }
        }
        for arrow in &mut self.arrows {
            if arrow.id != id && arrow.z_index < current_z.unwrap() {
                arrow.z_index += 1;
            }
        }
        for text in &mut self.texts {
            if text.id != id && text.z_index < current_z.unwrap() {
                text.z_index += 1;
            }
        }
        for path in &mut self.paths {
            if path.id != id && path.z_index < current_z.unwrap() {
                path.z_index += 1;
            }
        }
        for image in &mut self.images {
            if image.id != id && image.z_index < current_z.unwrap() {
                image.z_index += 1;
            }
        }
        
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
            return;
        }
        if let Some(path) = self.paths.iter_mut().find(|p| p.id == id) {
            path.z_index = new_z;
            self.save_snapshot();
            return;
        }
        if let Some(image) = self.images.iter_mut().find(|i| i.id == id) {
            image.z_index = new_z;
            self.save_snapshot();
        }
    }

    fn get_shape_z_index(&self, id: u64) -> Option<i32> {
        if let Some(rect) = self.rectangles.iter().find(|r| r.id == id) {
            return Some(rect.z_index);
        }
        if let Some(ellipse) = self.ellipses.iter().find(|e| e.id == id) {
            return Some(ellipse.z_index);
        }
        if let Some(diamond) = self.diamonds.iter().find(|d| d.id == id) {
            return Some(diamond.z_index);
        }
        if let Some(line) = self.lines.iter().find(|l| l.id == id) {
            return Some(line.z_index);
        }
        if let Some(arrow) = self.arrows.iter().find(|a| a.id == id) {
            return Some(arrow.z_index);
        }
        if let Some(text) = self.texts.iter().find(|t| t.id == id) {
            return Some(text.z_index);
        }
        if let Some(path) = self.paths.iter().find(|p| p.id == id) {
            return Some(path.z_index);
        }
        if let Some(image) = self.images.iter().find(|i| i.id == id) {
            return Some(image.z_index);
        }
        None
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
        for path in &self.paths {
            max_z = max_z.max(path.z_index);
        }
        for image in &self.images {
            max_z = max_z.max(image.z_index);
        }
        max_z
    }

    fn normalize_z_indices(&mut self) {
        use std::collections::HashMap;
        
        let mut shapes: Vec<(u64, i32)> = Vec::new();

        for rect in &self.rectangles {
            shapes.push((rect.id, rect.z_index));
        }
        for ellipse in &self.ellipses {
            shapes.push((ellipse.id, ellipse.z_index));
        }
        for diamond in &self.diamonds {
            shapes.push((diamond.id, diamond.z_index));
        }
        for line in &self.lines {
            shapes.push((line.id, line.z_index));
        }
        for arrow in &self.arrows {
            shapes.push((arrow.id, arrow.z_index));
        }
        for text in &self.texts {
            shapes.push((text.id, text.z_index));
        }
        for path in &self.paths {
            shapes.push((path.id, path.z_index));
        }
        for image in &self.images {
            shapes.push((image.id, image.z_index));
        }

        shapes.sort_by(|a, b| a.1.cmp(&b.1));

        let mut id_to_new_z: HashMap<u64, i32> = HashMap::new();
        for (new_z, (id, _)) in shapes.iter().enumerate() {
            id_to_new_z.insert(*id, new_z as i32);
        }

        for rect in &mut self.rectangles {
            if let Some(&new_z) = id_to_new_z.get(&rect.id) {
                rect.z_index = new_z;
            }
        }
        for ellipse in &mut self.ellipses {
            if let Some(&new_z) = id_to_new_z.get(&ellipse.id) {
                ellipse.z_index = new_z;
            }
        }
        for diamond in &mut self.diamonds {
            if let Some(&new_z) = id_to_new_z.get(&diamond.id) {
                diamond.z_index = new_z;
            }
        }
        for line in &mut self.lines {
            if let Some(&new_z) = id_to_new_z.get(&line.id) {
                line.z_index = new_z;
            }
        }
        for arrow in &mut self.arrows {
            if let Some(&new_z) = id_to_new_z.get(&arrow.id) {
                arrow.z_index = new_z;
            }
        }
        for text in &mut self.texts {
            if let Some(&new_z) = id_to_new_z.get(&text.id) {
                text.z_index = new_z;
            }
        }
        for path in &mut self.paths {
            if let Some(&new_z) = id_to_new_z.get(&path.id) {
                path.z_index = new_z;
            }
        }
        for image in &mut self.images {
            if let Some(&new_z) = id_to_new_z.get(&image.id) {
                image.z_index = new_z;
            }
        }
    }

    pub fn serialize(&self) -> String {
        let snapshot = DocumentSnapshot {
            rectangles: self.rectangles.clone(),
            ellipses: self.ellipses.clone(),
            lines: self.lines.clone(),
            arrows: self.arrows.clone(),
            diamonds: self.diamonds.clone(),
            texts: self.texts.clone(),
            paths: self.paths.clone(),
            images: self.images.clone(),
            groups: self.groups.clone(),
            next_id: self.next_id,
        };
        serde_json::to_string(&snapshot).unwrap_or_default()
    }

    pub fn deserialize(&mut self, data: &str) -> bool {
        match serde_json::from_str::<DocumentSnapshot>(data) {
            Ok(snapshot) => {
                self.restore_snapshot(&snapshot);
                self.normalize_z_indices();
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
