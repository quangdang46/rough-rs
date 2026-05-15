//! Pure Rust rough.js-compatible sketchy SVG path generation.
//!
//! The crate is being built as a faithful port of the vendored rough.js source
//! in `legacy/rough`, with SVG path data as the public rendering output.

pub mod core;
pub mod fillers;
pub mod generator;
pub mod geometry;
pub mod math;
pub mod renderer;
pub mod svg;

pub use crate::core::{
    Config, Drawable, FillStyle, Op, OpSet, OpSetType, OpType, Options, PathInfo, ResolvedOptions,
    ShapeType, SvgPath,
};
pub use crate::generator::Generator;
pub use crate::math::RngHelper;
