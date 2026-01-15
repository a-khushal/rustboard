use serde::{Deserialize, Serialize};
use crate::geometry::Point;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    pub id: u64,
    pub position: Point,
    pub width: f64,
    pub height: f64,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default = "default_fill_color")]
    pub fill_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_dash_pattern")]
    pub dash_pattern: String,
    #[serde(default = "default_border_radius")]
    pub border_radius: f64,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Rectangle {
    pub fn new(id: u64, position: Point, width: f64, height: f64) -> Self {
        Self {
            id,
            position,
            width,
            height,
            stroke_color: default_stroke_color(),
            fill_color: default_fill_color(),
            line_width: default_line_width(),
            dash_pattern: default_dash_pattern(),
            border_radius: default_border_radius(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    pub id: u64,
    pub position: Point,
    pub radius_x: f64,
    pub radius_y: f64,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default = "default_fill_color")]
    pub fill_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_dash_pattern")]
    pub dash_pattern: String,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Ellipse {
    pub fn new(id: u64, position: Point, radius_x: f64, radius_y: f64) -> Self {
        Self { 
            id, 
            position, 
            radius_x, 
            radius_y,
            stroke_color: default_stroke_color(),
            fill_color: default_fill_color(),
            line_width: default_line_width(),
            dash_pattern: default_dash_pattern(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub id: u64,
    pub start: Point,
    pub end: Point,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_dash_pattern")]
    pub dash_pattern: String,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Line {
    pub fn new(id: u64, start: Point, end: Point) -> Self {
        Self { 
            id, 
            start, 
            end,
            stroke_color: default_stroke_color(),
            line_width: default_line_width(),
            dash_pattern: default_dash_pattern(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arrow {
    pub id: u64,
    pub start: Point,
    pub end: Point,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_dash_pattern")]
    pub dash_pattern: String,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Arrow {
    pub fn new(id: u64, start: Point, end: Point) -> Self {
        Self { 
            id, 
            start, 
            end,
            stroke_color: default_stroke_color(),
            line_width: default_line_width(),
            dash_pattern: default_dash_pattern(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diamond {
    pub id: u64,
    pub position: Point,
    pub width: f64,
    pub height: f64,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default = "default_fill_color")]
    pub fill_color: Option<String>,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_dash_pattern")]
    pub dash_pattern: String,
    #[serde(default = "default_border_radius")]
    pub border_radius: f64,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Diamond {
    pub fn new(id: u64, position: Point, width: f64, height: f64) -> Self {
        Self {
            id,
            position,
            width,
            height,
            stroke_color: default_stroke_color(),
            fill_color: default_fill_color(),
            line_width: default_line_width(),
            dash_pattern: default_dash_pattern(),
            border_radius: default_border_radius(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

fn default_stroke_color() -> String {
    "#000000".to_string()
}

fn default_fill_color() -> Option<String> {
    None
}

fn default_line_width() -> f64 {
    2.0
}

fn default_border_radius() -> f64 {
    0.0
}

fn default_rotation() -> f64 {
    0.0
}

fn default_z_index() -> i32 {
    0
}

fn default_dash_pattern() -> String {
    "solid".to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub id: u64,
    pub points: Vec<Point>,
    #[serde(default = "default_stroke_color")]
    pub stroke_color: String,
    #[serde(default = "default_line_width")]
    pub line_width: f64,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Path {
    pub fn new(id: u64, points: Vec<Point>) -> Self {
        Self {
            id,
            points,
            stroke_color: default_stroke_color(),
            line_width: default_line_width(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub id: u64,
    pub position: Point,
    pub width: f64,
    pub height: f64,
    pub image_data: String,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Image {
    pub fn new(id: u64, position: Point, width: f64, height: f64, image_data: String) -> Self {
        Self {
            id,
            position,
            width,
            height,
            image_data,
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub id: u64,
    pub position: Point,
    pub width: f64,
    pub height: f64,
    pub content: String,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_font_weight")]
    pub font_weight: String,
    #[serde(default = "default_text_align")]
    pub text_align: String,
    #[serde(default = "default_stroke_color")]
    pub color: String,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
    #[serde(default = "default_z_index")]
    pub z_index: i32,
}

impl Text {
    pub fn new(id: u64, position: Point, width: f64, height: f64, content: String) -> Self {
        Self {
            id,
            position,
            width,
            height,
            content,
            font_family: default_font_family(),
            font_size: default_font_size(),
            font_weight: default_font_weight(),
            text_align: default_text_align(),
            color: default_stroke_color(),
            rotation_angle: default_rotation(),
            z_index: default_z_index(),
        }
    }
}

fn default_font_family() -> String {
    "Arial".to_string()
}

fn default_font_size() -> f64 {
    36.0
}

fn default_font_weight() -> String {
    "normal".to_string()
}

fn default_text_align() -> String {
    "left".to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: u64,
    pub element_ids: Vec<u64>,
    #[serde(default = "default_rotation")]
    pub rotation_angle: f64,
}

impl Group {
    pub fn new(id: u64, element_ids: Vec<u64>) -> Self {
        Self {
            id,
            element_ids,
            rotation_angle: default_rotation(),
        }
    }
}
