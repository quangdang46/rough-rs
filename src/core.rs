use crate::geometry::Point;
use std::fmt;
use std::str::FromStr;

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

impl ResolvedOptions {
    pub fn from_options(options: &Options) -> Self {
        Self::default().merge(options)
    }

    pub fn merge(mut self, options: &Options) -> Self {
        if let Some(value) = options.max_randomness_offset {
            self.max_randomness_offset = value;
        }
        if let Some(value) = options.roughness {
            self.roughness = value;
        }
        if let Some(value) = options.bowing {
            self.bowing = value;
        }
        if let Some(value) = &options.stroke {
            self.stroke = value.clone();
        }
        if let Some(value) = options.stroke_width {
            self.stroke_width = value;
        }
        if let Some(value) = options.curve_fitting {
            self.curve_fitting = value;
        }
        if let Some(value) = options.curve_tightness {
            self.curve_tightness = value;
        }
        if let Some(value) = options.curve_step_count {
            self.curve_step_count = value;
        }
        if let Some(value) = &options.fill {
            self.fill = Some(value.clone());
        }
        if let Some(value) = options.fill_style {
            self.fill_style = value;
        }
        if let Some(value) = options.fill_weight {
            self.fill_weight = value;
        }
        if let Some(value) = options.hachure_angle {
            self.hachure_angle = value;
        }
        if let Some(value) = options.hachure_gap {
            self.hachure_gap = value;
        }
        if let Some(value) = options.simplification {
            self.simplification = Some(value);
        }
        if let Some(value) = options.dash_offset {
            self.dash_offset = value;
        }
        if let Some(value) = options.dash_gap {
            self.dash_gap = value;
        }
        if let Some(value) = options.zigzag_offset {
            self.zigzag_offset = value;
        }
        if let Some(value) = options.seed {
            self.seed = value;
        }
        if let Some(value) = &options.stroke_line_dash {
            self.stroke_line_dash = Some(value.clone());
        }
        if let Some(value) = options.stroke_line_dash_offset {
            self.stroke_line_dash_offset = Some(value);
        }
        if let Some(value) = &options.fill_line_dash {
            self.fill_line_dash = Some(value.clone());
        }
        if let Some(value) = options.fill_line_dash_offset {
            self.fill_line_dash_offset = Some(value);
        }
        if let Some(value) = options.disable_multi_stroke {
            self.disable_multi_stroke = value;
        }
        if let Some(value) = options.disable_multi_stroke_fill {
            self.disable_multi_stroke_fill = value;
        }
        if let Some(value) = options.preserve_vertices {
            self.preserve_vertices = value;
        }
        if let Some(value) = options.fixed_decimal_place_digits {
            self.fixed_decimal_place_digits = Some(value);
        }
        if let Some(value) = options.fill_shape_roughness_gain {
            self.fill_shape_roughness_gain = value;
        }
        self
    }

    pub fn effective_fill_weight(&self) -> f64 {
        if self.fill_weight < 0.0 {
            self.stroke_width / 2.0
        } else {
            self.fill_weight
        }
    }

    pub fn effective_hachure_gap(&self) -> f64 {
        if self.hachure_gap < 0.0 {
            self.stroke_width * 4.0
        } else {
            self.hachure_gap
        }
    }

    pub fn effective_dash_offset(&self) -> f64 {
        if self.dash_offset < 0.0 {
            self.effective_hachure_gap()
        } else {
            self.dash_offset
        }
    }

    pub fn effective_dash_gap(&self) -> f64 {
        if self.dash_gap < 0.0 {
            self.effective_hachure_gap()
        } else {
            self.dash_gap
        }
    }

    pub fn effective_zigzag_offset(&self) -> f64 {
        if self.zigzag_offset < 0.0 {
            self.effective_hachure_gap()
        } else {
            self.zigzag_offset
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

impl FillStyle {
    pub fn as_rough_str(self) -> &'static str {
        match self {
            Self::Hachure => "hachure",
            Self::Solid => "solid",
            Self::Zigzag => "zigzag",
            Self::CrossHatch => "cross-hatch",
            Self::Dots => "dots",
            Self::Dashed => "dashed",
            Self::ZigzagLine => "zigzag-line",
        }
    }
}

impl fmt::Display for FillStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_rough_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFillStyleError {
    value: String,
}

impl fmt::Display for ParseFillStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown rough.js fill style: {}", self.value)
    }
}

impl std::error::Error for ParseFillStyleError {}

impl FromStr for FillStyle {
    type Err = ParseFillStyleError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "hachure" => Ok(Self::Hachure),
            "solid" => Ok(Self::Solid),
            "zigzag" => Ok(Self::Zigzag),
            "cross-hatch" => Ok(Self::CrossHatch),
            "dots" => Ok(Self::Dots),
            "dashed" => Ok(Self::Dashed),
            "zigzag-line" => Ok(Self::ZigzagLine),
            _ => Err(ParseFillStyleError {
                value: value.to_string(),
            }),
        }
    }
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

impl Op {
    pub fn new(op: OpType, data: impl Into<Vec<f64>>) -> Self {
        Self {
            op,
            data: data.into(),
        }
    }
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

impl OpSet {
    pub fn new(set_type: OpSetType, ops: Vec<Op>) -> Self {
        Self {
            set_type,
            ops,
            size: None,
            path: None,
        }
    }
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

impl ShapeType {
    pub fn as_rough_str(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Rectangle => "rectangle",
            Self::Ellipse => "ellipse",
            Self::Circle => "circle",
            Self::LinearPath => "linearPath",
            Self::Arc => "arc",
            Self::Curve => "curve",
            Self::Polygon => "polygon",
            Self::Path => "path",
        }
    }
}

impl fmt::Display for ShapeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_rough_str())
    }
}

#[derive(Debug, Clone)]
pub struct Drawable {
    pub shape: ShapeType,
    pub options: ResolvedOptions,
    pub sets: Vec<OpSet>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SvgPath {
    pub d: String,
    pub stroke: String,
    pub stroke_width: f64,
    pub fill: String,
}

pub type PathInfo = SvgPath;
