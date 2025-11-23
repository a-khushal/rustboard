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
}

impl Line {
    pub fn new(id: u64, start: Point, end: Point) -> Self {
        Self { 
            id, 
            start, 
            end,
            stroke_color: default_stroke_color(),
            line_width: default_line_width(),
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
}

impl Arrow {
    pub fn new(id: u64, start: Point, end: Point) -> Self {
        Self { 
            id, 
            start, 
            end,
            stroke_color: default_stroke_color(),
            line_width: default_line_width(),
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
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub id: u64,
    pub position: Point,
    pub text: String,
	#[serde(rename = "fontSize", default = "default_font_size")]
	pub font_size: f64,
    #[serde(default = "default_stroke_color")]
    pub text_color: String,
}

impl Text {
	pub fn new(id: u64, position: Point, text: String, font_size: f64) -> Self {
		Self { 
            id, 
            position, 
            text, 
            font_size,
            text_color: default_stroke_color(),
        }
    }
}

fn default_font_size() -> f64 {
	16.0
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
