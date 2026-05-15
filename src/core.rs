use crate::geometry::Point;

pub const SVG_NS: &str = "http://www.w3.org/2000/svg";

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub options: Option<Options>,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub max_randomness_offset: Option<f64>,
    pub roughness: Option<f64>,
    pub bowing: Option<f64>,
    pub stroke: Option<String>,
    pub stroke_width: Option<f64>,
    pub curve_fitting: Option<f64>,
    pub curve_tightness: Option<f64>,
    pub curve_step_count: Option<f64>,
    pub fill: Option<String>,
    pub fill_style: Option<FillStyle>,
    pub fill_weight: Option<f64>,
    pub hachure_angle: Option<f64>,
    pub hachure_gap: Option<f64>,
    pub simplification: Option<f64>,
    pub dash_offset: Option<f64>,
    pub dash_gap: Option<f64>,
    pub zigzag_offset: Option<f64>,
    pub seed: Option<u64>,
    pub stroke_line_dash: Option<Vec<f64>>,
    pub stroke_line_dash_offset: Option<f64>,
    pub fill_line_dash: Option<Vec<f64>>,
    pub fill_line_dash_offset: Option<f64>,
    pub disable_multi_stroke: Option<bool>,
    pub disable_multi_stroke_fill: Option<bool>,
    pub preserve_vertices: Option<bool>,
    pub fixed_decimal_place_digits: Option<usize>,
    pub fill_shape_roughness_gain: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ResolvedOptions {
    pub max_randomness_offset: f64,
    pub roughness: f64,
    pub bowing: f64,
    pub stroke: String,
    pub stroke_width: f64,
    pub curve_fitting: f64,
    pub curve_tightness: f64,
    pub curve_step_count: f64,
    pub fill: Option<String>,
    pub fill_style: FillStyle,
    pub fill_weight: f64,
    pub hachure_angle: f64,
    pub hachure_gap: f64,
    pub simplification: Option<f64>,
    pub dash_offset: f64,
    pub dash_gap: f64,
    pub zigzag_offset: f64,
    pub seed: u64,
    pub stroke_line_dash: Option<Vec<f64>>,
    pub stroke_line_dash_offset: Option<f64>,
    pub fill_line_dash: Option<Vec<f64>>,
    pub fill_line_dash_offset: Option<f64>,
    pub disable_multi_stroke: bool,
    pub disable_multi_stroke_fill: bool,
    pub preserve_vertices: bool,
    pub fixed_decimal_place_digits: Option<usize>,
    pub fill_shape_roughness_gain: f64,
}

impl Default for ResolvedOptions {
    fn default() -> Self {
        Self {
            max_randomness_offset: 2.0,
            roughness: 1.0,
            bowing: 1.0,
            stroke: "#000".to_string(),
            stroke_width: 1.0,
            curve_fitting: 0.95,
            curve_tightness: 0.0,
            curve_step_count: 9.0,
            fill: None,
            fill_style: FillStyle::Hachure,
            fill_weight: -1.0,
            hachure_angle: -41.0,
            hachure_gap: -1.0,
            simplification: None,
            dash_offset: -1.0,
            dash_gap: -1.0,
            zigzag_offset: -1.0,
            seed: 0,
            stroke_line_dash: None,
            stroke_line_dash_offset: None,
            fill_line_dash: None,
            fill_line_dash_offset: None,
            disable_multi_stroke: false,
            disable_multi_stroke_fill: false,
            preserve_vertices: false,
            fixed_decimal_place_digits: None,
            fill_shape_roughness_gain: 0.8,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FillStyle {
    #[default]
    Hachure,
    Solid,
    Zigzag,
    CrossHatch,
    Dots,
    Dashed,
    ZigzagLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpType {
    Move,
    BCurveTo,
    LineTo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Op {
    pub op: OpType,
    pub data: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpSetType {
    Path,
    FillPath,
    FillSketch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpSet {
    pub set_type: OpSetType,
    pub ops: Vec<Op>,
    pub size: Option<Point>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    Line,
    Rectangle,
    Ellipse,
    Circle,
    LinearPath,
    Arc,
    Curve,
    Polygon,
    Path,
}

#[derive(Debug, Clone)]
pub struct Drawable {
    pub shape: ShapeType,
    pub options: ResolvedOptions,
    pub sets: Vec<OpSet>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathInfo {
    pub d: String,
    pub stroke: String,
    pub stroke_width: f64,
    pub fill: String,
}
