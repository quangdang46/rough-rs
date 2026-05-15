//! Pure Rust rough.js 4.6.6-compatible sketchy SVG path generation.
//!
//! rough-rs renders the supported rough.js generator surface into SVG path data
//! without a browser, Canvas, Node.js, or WASM runtime. The public entry point
//! is [`Generator`], which returns [`Drawable`] values. Convert drawables to SVG
//! path attributes with [`svg::drawable_to_paths`].
//!
//! Nonzero seeded output follows rough.js seeded behavior. Dot fills also follow
//! rough.js and use unseeded randomness for dot center placement, so exact dot
//! coordinates are nondeterministic in both implementations. The strict
//! compatibility contract is documented in `docs/parity-audit.md` and
//! `docs/roughjs-compatibility.md`.

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
