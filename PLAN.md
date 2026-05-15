# rough-rs — PLAN.md

> Pure Rust port of [rough.js](https://github.com/rough-stuff/rough) — a hand-drawn/sketchy graphics
> primitive library. Produces SVG path data with no browser, no Node.js, no canvas dependency.
> Intended as the foundational crate for `excalidraw-rs`.

---

## Table of Contents

1. [Goals & Non-Goals](#1-goals--non-goals)
2. [Why rough-rs Exists](#2-why-rough-rs-exists)
3. [Architecture Overview](#3-architecture-overview)
4. [Data Model](#4-data-model)
5. [Module Breakdown](#5-module-breakdown)
   - 5.1 [math.rs — Seeded RNG](#51-mathrs--seeded-rng)
   - 5.2 [geometry.rs — Primitives](#52-geometryrs--primitives)
   - 5.3 [renderer.rs — Op Generation](#53-rendererrs--op-generation)
   - 5.4 [fillers/ — Fill Strategies](#54-fillers--fill-strategies)
   - 5.5 [generator.rs — Public API](#55-generatorrs--public-api)
   - 5.6 [svg.rs — SVG Serialization](#56-svgrs--svg-serialization)
6. [Options & Configuration](#6-options--configuration)
7. [Primitive Specifications](#7-primitive-specifications)
   - 7.1 [Line](#71-line)
   - 7.2 [Rectangle](#72-rectangle)
   - 7.3 [Ellipse / Circle](#73-ellipse--circle)
   - 7.4 [Polygon / Linear Path](#74-polygon--linear-path)
   - 7.5 [Arc](#75-arc)
   - 7.6 [Curve (Catmull-Rom)](#76-curve-catmull-rom)
   - 7.7 [SVG Path](#77-svg-path)
8. [Fill Strategies](#8-fill-strategies)
   - 8.1 [Hachure](#81-hachure)
   - 8.2 [Solid](#82-solid)
   - 8.3 [Cross-Hatch](#83-cross-hatch)
   - 8.4 [Dots](#84-dots)
   - 8.5 [Dashed](#85-dashed)
   - 8.6 [Zigzag-Line](#86-zigzag-line)
9. [Seed Parity with rough.js](#9-seed-parity-with-roughjs)
10. [SVG Output Format](#10-svg-output-format)
11. [Crate Structure](#11-crate-structure)
12. [Dependencies](#12-dependencies)
13. [Testing Strategy](#13-testing-strategy)
14. [Implementation Phases](#14-implementation-phases)
15. [Design Decisions & Rationale](#15-design-decisions--rationale)

---

## 1. Goals & Non-Goals

### Goals

- **1:1 algorithm port** of rough.js v4.x renderer logic — same sketchy output given same seed
- **SVG path string output** — `Drawable` → `<path d="..."/>` strings ready for embedding
- **Seed parity** — given the same `seed` as rough.js, produce bitwise-equivalent random sequence
- **No browser dependency** — pure Rust, no WASM, no JS interop
- **Library-first** — designed to be embedded in `excalidraw-rs` and other crates
- **Optional CLI** — `rough` binary for quick shape rendering from CLI (nice-to-have)
- **crates.io publishable** — standalone value independent of excalidraw-rs

### Non-Goals

- Canvas rendering (HTML `<canvas>` equivalent) — SVG only
- Pixel-perfect visual match to rough.js — perceptually equivalent is sufficient
- Animation support
- Full SVG document generation — we emit path `d` strings and stroke/fill attributes only
- Web bindings (WASM) — separate crate concern

---

## 2. Why rough-rs Exists

Excalidraw uses rough.js as its rendering engine. Every element (rectangle, ellipse, arrow, etc.)
is rendered through rough.js with a `seed` value stored in the element JSON. To render
`.excalidraw` files natively in Rust (no browser, no Node), we need a faithful Rust port.

rough.js is small (~9KB gzipped, ~1500 LOC TypeScript across 6 files). The algorithms are
well-documented and the author wrote a [blog post](https://shihn.ca/posts/2020/roughjs-algorithms/)
explaining them. This makes it an unusually good porting target.

**No Rust port exists yet.** There is a Python port (`rough` on PyPI) but nothing for Rust.
Publishing `rough-rs` independently has value beyond excalidraw-rs.

---

## 3. Architecture Overview

```
rough-rs
│
├── src/
│   ├── lib.rs           ← re-exports Generator, Options, Drawable, FillStyle
│   ├── math.rs          ← Mulberry32 seeded RNG, randomOffset, randomize helpers
│   ├── geometry.rs      ← Point, Line, rotate_points, line_intersection, rotate_lines
│   ├── renderer.rs      ← core Op generation: _line, _ellipse, _curveWithOffset, etc.
│   ├── generator.rs     ← public API: line(), rectangle(), ellipse(), polygon(), path()
│   ├── svg.rs           ← Op/OpSet → SVG path `d` string, Drawable → SVG elements
│   └── fillers/
│       ├── mod.rs       ← FillerStrategy trait
│       ├── hachure.rs   ← scan-line hachure (+ cross-hatch, dashed, zigzag variants)
│       ├── solid.rs     ← solid polygon fill
│       └── dots.rs      ← dot pattern fill
│
└── tests/
    ├── seed_parity.rs   ← compare output against reference snapshots from rough.js
    ├── primitives.rs    ← unit tests per primitive
    └── svg_output.rs    ← validate SVG path string format
```

**Data flow:**
```
Options + seed
    │
    ▼
Generator::line(x1,y1,x2,y2)
    │
    ▼
renderer::_line(...)   ← calls RNG internally via Helper(seed)
    │
    ▼
OpSet { ops: Vec<Op> }   ← stroke ops
    │
OpSet { ops: Vec<Op> }   ← fill ops (via filler)
    │
    ▼
Drawable { sets: Vec<OpSet> }
    │
    ▼
svg::drawable_to_paths(drawable) → Vec<SvgPath>
    │
    ▼
"<path d='M10 10 L200 10 ...' stroke='#000' fill='none'/>"
```

---

## 4. Data Model

```rust
/// A single drawing operation — mirrors rough.js Op type
#[derive(Debug, Clone)]
pub struct Op {
    pub op: OpType,
    pub data: Vec<f64>,  // [x, y] for Move/Line; [cp1x, cp1y, cp2x, cp2y, x, y] for BCurveTo
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpType {
    Move,      // moveTo
    LineTo,    // lineTo
    BCurveTo,  // bezierCurveTo (cubic)
}

/// A set of ops that together form one stroke or fill path
#[derive(Debug, Clone)]
pub struct OpSet {
    pub set_type: OpSetType,
    pub ops: Vec<Op>,
    pub size: Option<(f64, f64)>,  // for ellipse path fills
    pub path: Option<String>,      // for path fills
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpSetType {
    Path,           // stroke path
    FillPath,       // solid fill path
    FillSketch,     // hachure/dots/etc fill
}

/// A complete drawable shape — collection of OpSets
#[derive(Debug, Clone)]
pub struct Drawable {
    pub shape: ShapeType,
    pub options: ResolvedOptions,
    pub sets: Vec<OpSet>,
}

#[derive(Debug, Clone)]
pub enum ShapeType {
    Line, Rectangle, Ellipse, Circle, Arc,
    LinearPath, Polygon, Curve, Path,
}

/// Final output: an SVG path element's attributes
#[derive(Debug, Clone)]
pub struct SvgPath {
    pub d: String,           // SVG path data string
    pub stroke: String,      // e.g. "#1e1e1e" or "none"
    pub stroke_width: f64,
    pub fill: String,        // e.g. "red" or "none"
    pub fill_opacity: f64,
}
```

---

## 5. Module Breakdown

### 5.1 `math.rs` — Seeded RNG

The most critical module for seed parity. rough.js uses **Mulberry32** as its PRNG.

**Mulberry32 algorithm (from rough.js source):**
```javascript
function randomSeed() {
  return Math.floor(Math.random() * 2**31);
}

// The actual PRNG used per-shape:
function _getHelper(options) {
  let seed = options.seed || 0;
  return {
    next() {
      if (seed) {
        let t = (seed += 0x6D2B79F5);
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
      } else {
        return Math.random();
      }
    }
  };
}
```

**Rust implementation:**

```rust
pub struct RngHelper {
    seed: u32,
    use_random: bool,  // if seed == 0, use thread_rng
}

impl RngHelper {
    pub fn new(seed: u64) -> Self {
        Self {
            seed: seed as u32,
            use_random: seed == 0,
        }
    }

    /// Returns f64 in [0.0, 1.0) — matches JS output for same seed
    pub fn next(&mut self) -> f64 {
        if self.use_random {
            // non-deterministic fallback (same behavior as rough.js seed=0)
            use rand::Rng;
            return rand::thread_rng().gen::<f64>();
        }
        // Mulberry32 — must use wrapping arithmetic to match JS Math.imul / |0 behavior
        self.seed = self.seed.wrapping_add(0x6D2B79F5);
        let mut t = self.seed;
        t = u32::wrapping_mul(t ^ (t >> 15), t | 1);
        t ^= t.wrapping_add(u32::wrapping_mul(t ^ (t >> 7), t | 61));
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }

    /// Random float in [-half, half]
    pub fn random_offset(&mut self, x: f64, roughness: f64) -> f64 {
        roughness * x * (self.next() - 0.5)
    }
}
```

**Key constraint:** JavaScript `Math.imul` performs 32-bit signed integer multiplication with
wrapping. Rust's `u32::wrapping_mul` is equivalent when both operands are cast to u32 first.
All intermediate values must use wrapping arithmetic — no panics on overflow.

**Verification:** Write a test that seeds with known values and checks the first 10 outputs
match the JS reference table:

```
seed=12345: [0.8148..., 0.1348..., 0.9702..., ...]  (extracted from running rough.js)
seed=42:    [0.0022..., 0.6832..., ...]
seed=1:     [0.4273..., ...]
```

### 5.2 `geometry.rs` — Primitives

Port of rough.js `geometry.ts`. Purely mathematical, no randomness.

```rust
pub type Point = [f64; 2];

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub source: Point,
    pub target: Point,
}

impl Line {
    pub fn length(&self) -> f64 {
        let dx = self.target[0] - self.source[0];
        let dy = self.target[1] - self.source[1];
        (dx * dx + dy * dy).sqrt()
    }
}

/// Rotate a set of points around a center by angle (radians)
pub fn rotate_points(points: &[Point], center: Point, angle: f64) -> Vec<Point> {
    let (sin_a, cos_a) = angle.sin_cos();
    points.iter().map(|p| {
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        [
            center[0] + dx * cos_a - dy * sin_a,
            center[1] + dx * sin_a + dy * cos_a,
        ]
    }).collect()
}

/// Rotate a set of Lines around a center
pub fn rotate_lines(lines: &[Line], center: Point, angle: f64) -> Vec<Line> { ... }

/// Compute intersection of two line segments. Returns None if parallel/no intersection.
pub fn line_intersection(
    a: Point, b: Point,
    c: Point, d: Point,
) -> Option<Point> {
    // Standard parametric line intersection
    let a1 = b[1] - a[1];
    let b1 = a[0] - b[0];
    let c1 = a1 * a[0] + b1 * a[1];
    let a2 = d[1] - c[1];
    let b2 = c[0] - d[0];
    let c2 = a2 * c[0] + b2 * c[1];
    let det = a1 * b2 - a2 * b1;
    if det.abs() < 1e-10 { return None; }  // parallel
    Some([
        (b2 * c1 - b1 * c2) / det,
        (a1 * c2 - a2 * c1) / det,
    ])
}
```

### 5.3 `renderer.rs` — Op Generation

Core of the library. Ports `renderer.ts` from rough.js. Generates `Vec<Op>` for each primitive.

All functions take `&mut RngHelper` and `&ResolvedOptions`.

**Key internal functions:**

```rust
/// Single jittered line from (x1,y1) to (x2,y2). Core building block.
fn _line(
    x1: f64, y1: f64, x2: f64, y2: f64,
    helper: &mut RngHelper,
    opts: &ResolvedOptions,
    move_first: bool,
    overlay: bool,
) -> Vec<Op>

/// Two-pass line (rough.js draws lines twice for hand-drawn look)
pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, opts: &ResolvedOptions) -> OpSet

/// Rectangle as 4 lines
pub fn rectangle(x: f64, y: f64, w: f64, h: f64, opts: &ResolvedOptions) -> OpSet

/// Ellipse — most complex, uses parametric sampling
pub fn ellipse_with_params(
    x: f64, y: f64,
    opts: &ResolvedOptions,
    ellipse_params: &EllipseParams,
) -> (OpSet, Vec<Point>)  // (stroke opset, points for fill)

pub fn generate_ellipse_params(
    width: f64, height: f64,
    opts: &ResolvedOptions,
) -> EllipseParams

/// Linear path through points (polygon without close)
pub fn linear_path(points: &[Point], close: bool, opts: &ResolvedOptions) -> OpSet

/// Arc (partial ellipse)
pub fn arc(
    x: f64, y: f64, w: f64, h: f64,
    start: f64, stop: f64,
    closed: bool,
    rough_closure: bool,
    opts: &ResolvedOptions,
) -> OpSet

/// Catmull-Rom curve through points
pub fn curve(points: &[Point], opts: &ResolvedOptions) -> OpSet

/// Arbitrary SVG path (normalized to M/L/C ops first)
pub fn svg_path(path: &str, opts: &ResolvedOptions) -> OpSet

/// Fill a polygon with hachure/solid/etc
pub fn solid_fill_polygon(
    polygon_list: &[Vec<Point>],
    opts: &ResolvedOptions,
) -> OpSet

pub fn pattern_fill_polygons(
    polygon_list: &[Vec<Point>],
    opts: &ResolvedOptions,
) -> OpSet
```

**The `_line` implementation detail:**

rough.js `_line` uses cubic bezier curves (not straight lines!) with control points offset by
random amounts based on `roughness` and `bowing`. This is what creates the wobbly look:

```
Given line (x1,y1) → (x2,y2):
  mid = midpoint
  cp1 = offset mid by random * roughness perpendicular to line direction
  cp2 = offset mid by random * roughness (different random values)
  result: M x1 y1 C cp1x cp1y cp2x cp2y x2 y2
```

The `bowing` option scales how far control points deviate from the midpoint.

### 5.4 `fillers/` — Fill Strategies

**Trait:**
```rust
pub trait FillerStrategy {
    fn fill_polygon(
        &self,
        polygon_list: &[Vec<Point>],
        opts: &ResolvedOptions,
    ) -> OpSet;
}
```

**`hachure.rs` — Scan-line hachure:**

Algorithm (from rough.js blog post):
1. Rotate all polygon points by `-hachure_angle` around polygon centroid
2. Compute bounding box of rotated polygon
3. Generate horizontal scan lines spaced `hachure_gap` apart across the bounding box
4. For each scan line, find intersections with rotated polygon edges
5. Pairs of intersections = line segments to draw
6. Rotate those line segments back by `+hachure_angle`
7. Apply jitter via `_line()` with `roughness`

```rust
pub fn polygon_hachure_lines(
    polygon_list: &[Vec<Point>],
    opts: &ResolvedOptions,
) -> Vec<Line>
// Returns the hachure lines before converting to Ops.
// Used by cross-hatch (call twice at angle and angle+90°),
// zigzag, dashed, and dots (iterate these lines).
```

**`solid.rs`:**

Close the polygon path and fill with `fill` color. Generates a single `FillPath` OpSet.
No randomness needed.

**`dots.rs`:**

Uses `polygon_hachure_lines` to generate grid positions, then places circles of radius
`fill_weight/2` at each intersection point.

### 5.5 `generator.rs` — Public API

The user-facing API. Mirrors `RoughGenerator` class from rough.js.

```rust
pub struct Generator {
    config: Config,
}

impl Generator {
    pub fn new(config: Config) -> Self

    pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64, options: Option<Options>) -> Drawable
    pub fn rectangle(&self, x: f64, y: f64, w: f64, h: f64, options: Option<Options>) -> Drawable
    pub fn ellipse(&self, x: f64, y: f64, w: f64, h: f64, options: Option<Options>) -> Drawable
    pub fn circle(&self, x: f64, y: f64, diameter: f64, options: Option<Options>) -> Drawable
    pub fn arc(&self, x: f64, y: f64, w: f64, h: f64, start: f64, stop: f64, closed: bool, options: Option<Options>) -> Drawable
    pub fn curve(&self, points: &[Point], options: Option<Options>) -> Drawable
    pub fn polygon(&self, points: &[Point], options: Option<Options>) -> Drawable
    pub fn path(&self, d: &str, options: Option<Options>) -> Drawable
    pub fn linear_path(&self, points: &[Point], options: Option<Options>) -> Drawable
}
```

Internally, `generator.rs` resolves options (merging config defaults with per-call overrides),
creates `RngHelper` from the resolved seed, delegates to `renderer.rs`, and assembles the
stroke + fill OpSets into a `Drawable`.

**Fill assembly logic (mirrors rough.js):**

```
if fill_style == Solid:
    fill_sets = [solid_fill_polygon(shape_points)]
else if fill_style != None:
    fill_sets = [pattern_fill_polygons(shape_points)]

stroke_sets = [draw_shape_stroke()]

Drawable.sets = fill_sets + stroke_sets
```

### 5.6 `svg.rs` — SVG Serialization

Converts `Drawable` → `Vec<SvgPath>` ready for embedding in SVG documents.

```rust
/// Convert a single OpSet to an SVG path `d` string
pub fn ops_to_path(opset: &OpSet, fixed_decimals: Option<usize>) -> String

/// Convert a Drawable to renderable SVG path elements  
pub fn drawable_to_paths(drawable: &Drawable) -> Vec<SvgPath>
```

**`ops_to_path` format:**
```
Move    → "M {x} {y} "
LineTo  → "L {x} {y} "
BCurveTo → "C {cp1x} {cp1y} {cp2x} {cp2y} {x} {y} "
```

Output is a valid SVG path `d` attribute value. Decimals rounded to 3 places by default
(matches rough.js output precision).

**`SvgPath` → final element:**
```xml
<path d="M10 10 C12.3 9.8 ..." stroke="#1e1e1e" stroke-width="1" fill="none"/>
```

---

## 6. Options & Configuration

```rust
/// User-facing options (all optional, use None for defaults)
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub roughness: Option<f64>,
    pub bowing: Option<f64>,
    pub seed: Option<u64>,
    pub stroke: Option<String>,
    pub stroke_width: Option<f64>,
    pub fill: Option<String>,
    pub fill_style: Option<FillStyle>,
    pub fill_weight: Option<f64>,
    pub hachure_angle: Option<f64>,
    pub hachure_gap: Option<f64>,
    pub curve_fitting: Option<f64>,
    pub curve_step_count: Option<f64>,
    pub curve_tightness: Option<f64>,
    pub disable_multi_stroke: Option<bool>,
    pub disable_multi_stroke_fill: Option<bool>,
    pub simplification: Option<f64>,
    pub dash_offset: Option<f64>,
    pub dash_gap: Option<f64>,
    pub zigzag_offset: Option<f64>,
    pub max_randomness_offset: Option<f64>,
    pub preserve_vertices: Option<bool>,
}

/// Resolved options with all defaults applied — used internally
#[derive(Debug, Clone)]
pub struct ResolvedOptions {
    pub roughness: f64,              // default: 1.0
    pub bowing: f64,                 // default: 1.0
    pub seed: u64,                   // default: 0 (non-deterministic)
    pub stroke: String,              // default: "#000000"
    pub stroke_width: f64,           // default: 1.0
    pub fill: Option<String>,        // default: None
    pub fill_style: FillStyle,       // default: FillStyle::Hachure
    pub fill_weight: f64,            // default: -1.0 → computed as strokeWidth/2
    pub hachure_angle: f64,          // default: -41.0 degrees
    pub hachure_gap: f64,            // default: -1.0 → computed as strokeWidth*4
    pub curve_fitting: f64,          // default: 0.95
    pub curve_step_count: f64,       // default: 9.0
    pub curve_tightness: f64,        // default: 0.0
    pub disable_multi_stroke: bool,  // default: false
    pub disable_multi_stroke_fill: bool, // default: false
    pub simplification: f64,         // default: 0.0
    pub dash_offset: f64,            // default: -1.0
    pub dash_gap: f64,               // default: -1.0
    pub zigzag_offset: f64,          // default: -1.0
    pub max_randomness_offset: f64,  // default: 2.0
    pub preserve_vertices: bool,     // default: false
}

#[derive(Debug, Clone, PartialEq)]
pub enum FillStyle {
    Hachure,      // default: parallel lines
    Solid,        // solid color fill
    CrossHatch,   // hachure × 2, 90° apart
    Dots,         // dots along hachure grid
    Dashed,       // dashed hachure lines
    ZigzagLine,   // zigzag hachure lines
}

/// Top-level config (passed at Generator construction, applies to all shapes)
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub options: Options,
    // future: logging, caching, etc.
}
```

**Default resolution rules (matching rough.js v4):**
- `fill_weight`: if -1.0 → `stroke_width / 2`
- `hachure_gap`: if -1.0 → `stroke_width * 4`
- `dash_offset`: if -1.0 → `hachure_gap`
- `dash_gap`: if -1.0 → `hachure_gap`
- `zigzag_offset`: if -1.0 → `hachure_gap`

---

## 7. Primitive Specifications

### 7.1 Line

```rust
generator.line(x1, y1, x2, y2, options)
```

**Internal algorithm:**
1. Create `RngHelper` from seed
2. Call `renderer::_line()` with `move_first=true, overlay=false` → primary stroke
3. If `!disable_multi_stroke`: call `_line()` again with `move_first=false, overlay=true` → second pass
4. No fill for lines
5. Returns `Drawable { sets: [stroke_opset] }`

**`_line` detail:**
```
dx = x2 - x1
dy = y2 - y1
len = sqrt(dx² + dy²)

// roughness offset scaled by length
offset = min(roughness * max_randomness_offset, len / 2)

// Two control points at midpoint, jittered perpendicularly
mid_x = (x1+x2)/2 + random_offset()
mid_y = (y1+y2)/2 + random_offset()

// bowing creates curvature
bow_x = mid_x + bowing * max_randomness_offset * (y2-y1) * rng.next()
bow_y = mid_y + bowing * max_randomness_offset * (x1-x2) * rng.next()

ops = [
    Move(x1 + random_offset, y1 + random_offset),
    BCurveTo(bow_x, bow_y, bow_x, bow_y, x2 + random_offset, y2 + random_offset)
]
```

### 7.2 Rectangle

```rust
generator.rectangle(x, y, width, height, options)
```

**Algorithm:**
1. Compute 4 corner points: `[x,y], [x+w,y], [x+w,y+h], [x,y+h]`
2. Stroke: call `linear_path(points, close=true)` → 4 jittered lines forming the border
3. Fill: if fill set, compute fill from polygon `[[x,y],[x+w,y],[x+w,y+h],[x,y+h]]`
4. Returns `Drawable { sets: [fill_opset?, stroke_opset] }`

### 7.3 Ellipse / Circle

The most complex primitive. Uses parametric approximation.

```rust
generator.ellipse(x, y, width, height, options)
generator.circle(x, y, diameter, options)  // → ellipse(x,y,d,d)
```

**`generate_ellipse_params`:**
Computes how many incremental steps to use based on circumference and `curve_step_count`.
Auto-adjusts `roughness` for size (prevents outer circles looking rougher — bug fix in v4).

```rust
pub struct EllipseParams {
    pub rx: f64,
    pub ry: f64,
    pub increment: f64,  // angular step
}

fn generate_ellipse_params(width: f64, height: f64, opts: &ResolvedOptions) -> EllipseParams {
    let ps_q = (std::f64::consts::PI * 2.0) / opts.curve_step_count;
    let step_count = ((width.abs() + height.abs()) / 2.0 * 2.0 * std::f64::consts::PI / ps_q)
        .ceil()
        .max(opts.curve_step_count);
    let increment = (std::f64::consts::PI * 2.0) / step_count;
    let rx = (width / 2.0).abs();
    let ry = (height / 2.0).abs();
    // curve_fitting adjusts to match actual vs. requested size
    EllipseParams { rx: rx * opts.curve_fitting, ry: ry * opts.curve_fitting, increment }
}
```

**`ellipse_with_params`:**
1. Sample `N` points on ellipse at each `increment` angle
2. Add random offset at each point scaled by `roughness`
3. Fit Catmull-Rom curve through points → cubic bezier ops
4. Draw twice (multi-stroke) unless disabled
5. Return `(OpSet, Vec<Point>)` — points used for fill polygon

### 7.4 Polygon / Linear Path

```rust
generator.polygon(points, options)
generator.linear_path(points, options)
```

`polygon` = `linear_path` with `close=true` + fill if set.

**Algorithm:**
- Iterate `points` as consecutive segments
- Each segment → `_line()` call
- Optionally close back to first point
- Fill via `pattern_fill_polygons([points])`

### 7.5 Arc

```rust
generator.arc(x, y, width, height, start, stop, closed, options)
```

Partial ellipse from angle `start` to `stop`. If `closed=true`, draws line segments back to center.

**Algorithm:**
1. Sample ellipse points between `start` and `stop` angles
2. Fit curve through them
3. If closed: add two lines from center to arc endpoints

### 7.6 Curve (Catmull-Rom)

```rust
generator.curve(points, options)
```

**Algorithm:**
1. Convert Catmull-Rom points to cubic bezier control points via `curve_to_bezier()`
2. Sample dense points on the bezier (via `points_on_bezier_curves()`)
3. Apply jitter to sampled points
4. Fit curve through jittered points

**Port of `points-on-curve` npm package:** This is a dependency of rough.js. Need to either
port it or find equivalent Rust crate. The algorithm:
- `curveToBezier`: converts Catmull-Rom control points to cubic bezier segments
- `pointsOnBezierCurves`: adaptive sampling of cubic bezier at given tolerance

### 7.7 SVG Path

```rust
generator.path(d: &str, options)
```

**Algorithm:**
1. Parse SVG path `d` string → normalized `(Move, Line, CubicBezier)` ops
   using a path parser (port of `path-data-parser` npm package, or use `svgtypes` crate)
2. Sample dense points on the normalized path via `points_on_path()`
3. Apply optional `simplification` to reduce point count
4. Fit rough curve through sampled points

**SVG arc command (`A`):** requires converting arc parameters to cubic bezier approximation.
Algorithm from Mozilla (already in rough.js, will port).

---

## 8. Fill Strategies

### 8.1 Hachure

**Algorithm (scan-line, v4 implementation):**

```rust
fn polygon_hachure_lines(polygon_list: &[Vec<Point>], opts: &ResolvedOptions) -> Vec<Line> {
    let angle = opts.hachure_angle.to_radians();
    let gap = opts.hachure_gap;  // already resolved to positive value
    
    // Compute center of all polygons for rotation
    let center = compute_center(polygon_list);
    
    // Rotate all polygons by -angle
    let rotated = rotate_polygon_list(polygon_list, center, -angle);
    
    // Find bounding box of rotated polygons
    let bbox = bounding_box(&rotated);
    
    // Generate horizontal scan lines
    let mut lines = Vec::new();
    let mut y = bbox.min_y + gap / 2.0;
    while y <= bbox.max_y {
        let scan_line = Line { source: [bbox.min_x - 1.0, y], target: [bbox.max_x + 1.0, y] };
        // Find intersections with all polygon edges
        let mut intersections = find_intersections(&rotated, scan_line);
        intersections.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        // Pair up intersections
        for chunk in intersections.chunks(2) {
            if chunk.len() == 2 {
                lines.push(Line { source: chunk[0], target: chunk[1] });
            }
        }
        y += gap;
    }
    
    // Rotate lines back by +angle
    rotate_lines(&lines, center, angle)
}
```

### 8.2 Solid

Close polygon with `Z`, fill with `opts.fill` color. Single `FillPath` OpSet.

### 8.3 Cross-Hatch

Call `polygon_hachure_lines` twice:
- First pass: at `hachure_angle`
- Second pass: at `hachure_angle + 90°`

Combine into single OpSet.

### 8.4 Dots

1. Get hachure lines
2. For each line, place circles spaced `fill_weight`-diameter apart along the line

### 8.5 Dashed

Like hachure, but each scan line is rendered as a dashed stroke using `dash_offset`/`dash_gap`.

### 8.6 Zigzag-Line

Like hachure, but each consecutive pair of hachure lines is connected by a zigzag.
Each line segment zigzags with amplitude `zigzag_offset`.

---

## 9. Seed Parity with rough.js

This is critical for Excalidraw compatibility. Same element `seed` must produce same visual output.

**Verification process:**

1. Write a Node.js script that uses rough.js directly to generate SVG for known inputs:
```javascript
// verify/generate_reference.js
const rough = require('roughjs');
// Generate reference outputs for test cases
const cases = [
  { type: 'line', args: [10, 10, 200, 100], seed: 12345 },
  { type: 'rectangle', args: [10, 10, 200, 100], seed: 42 },
  { type: 'ellipse', args: [100, 100, 200, 150], seed: 99 },
  // ...
];
// Output: JSON with seed → expected RNG sequence + path data
```

2. Capture reference outputs in `tests/fixtures/reference.json`

3. In Rust tests, run same inputs with same seeds, compare:
   - RNG sequence (first N values must match)
   - Path data structure (same number of ops, same op types)
   - Visual equivalence (don't require exact float match, allow ε tolerance)

**Known parity risk:** JavaScript `Math.imul` simulates C-style 32-bit wrapping multiplication.
Rust's `u32::wrapping_mul` is semantically equivalent but requires careful casting.
Must verify with the exact seed values Excalidraw uses (integer range 1 to 2^31).

---

## 10. SVG Output Format

**`drawable_to_paths` output contract:**

Each `Drawable` produces 1-3 `SvgPath` structs:
- `fill_sketch` OpSet → `SvgPath { stroke: fill_color, fill: "none", stroke_width: fill_weight }`
- `fill_path` OpSet → `SvgPath { fill: fill_color, stroke: "none" }`
- `path` OpSet → `SvgPath { stroke: stroke_color, fill: "none", stroke_width }`

**Excalidraw integration contract:**

`excalidraw-rs` will call:
```rust
let gen = Generator::new(Config::default());
let drawable = gen.rectangle(x, y, w, h, Some(Options {
    roughness: Some(element.roughness),
    seed: Some(element.seed as u64),
    stroke: Some(element.stroke_color.clone()),
    fill: element.background_color.clone(),
    fill_style: Some(element.fill_style.into()),
    stroke_width: Some(element.stroke_width),
    ..Default::default()
}));
let svg_paths = svg::drawable_to_paths(&drawable);
// Embed svg_paths into SVG document
```

---

## 11. Crate Structure

```
rough-rs/
├── Cargo.toml
├── README.md
├── PLAN.md
├── src/
│   ├── lib.rs
│   ├── math.rs
│   ├── geometry.rs
│   ├── renderer.rs
│   ├── generator.rs
│   ├── svg.rs
│   └── fillers/
│       ├── mod.rs
│       ├── hachure.rs
│       ├── solid.rs
│       └── dots.rs
├── tests/
│   ├── seed_parity.rs
│   ├── primitives.rs
│   └── svg_output.rs
├── benches/
│   └── renderer.rs          ← criterion benchmarks
├── examples/
│   ├── basic.rs             ← generate sample SVG
│   └── excalidraw_element.rs ← simulate one excalidraw element
└── verify/
    ├── generate_reference.js ← Node.js script to produce fixture data
    └── reference.json        ← captured reference outputs from rough.js
```

---

## 12. Dependencies

```toml
[dependencies]
# No required runtime dependencies for core lib

# Optional: for SVG path parsing (svg_path feature)
svgtypes = { version = "0.13", optional = true }

# Optional: for non-deterministic fallback (seed=0)
rand = { version = "0.8", optional = true, default-features = false, features = ["std", "std_rng"] }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
serde_json = "1"    # for loading reference fixtures
approx = "0.5"      # float comparison in tests

[features]
default = ["rand", "svg_path"]
svg_path = ["dep:svgtypes"]
```

**Design principle:** The core Mulberry32 implementation and Op generation require **zero
dependencies**. All randomness is inline. `rand` is only needed for the non-deterministic
`seed=0` fallback. `svgtypes` only needed if parsing arbitrary SVG paths.

**No `points-on-curve` equivalent needed:** The Catmull-Rom → bezier conversion is ~50 LOC
and will be ported inline. The bezier point-sampling algorithm is also straightforward to port.

---

## 13. Testing Strategy

### Unit Tests (per module)

**`math.rs`:**
- Mulberry32 output matches reference values for seeds 1, 42, 12345, 2147483647 (2^31-1)
- `random_offset` returns values in expected range
- Seeding with 0 uses thread_rng (non-deterministic, just verify it doesn't panic)

**`geometry.rs`:**
- `rotate_points` — 0°, 45°, 90°, 180° rotation
- `line_intersection` — parallel lines (None), perpendicular, diagonal
- `line_length` — basic cases

**`renderer.rs`:**
- Each primitive generates correct OpSet structure (op count, types)
- Line produces BCurveTo ops (not LineTo — this would indicate wrong algorithm)
- Ellipse generates incrementally more points for larger sizes

**`fillers/hachure.rs`:**
- Rectangle → N lines proportional to height/gap
- Hachure angle 0° → horizontal lines
- Hachure angle 90° → vertical lines
- Concave polygon → correct intersection handling

### Integration Tests

**Seed parity tests** (`tests/seed_parity.rs`):
```rust
#[test]
fn rectangle_seed_12345_matches_reference() {
    let reference = load_fixture("rectangle_12345");
    let gen = Generator::new(Config::default());
    let drawable = gen.rectangle(10.0, 10.0, 200.0, 100.0, Some(Options {
        seed: Some(12345),
        ..Default::default()
    }));
    let paths = svg::drawable_to_paths(&drawable);
    // Compare RNG call count and path structure
    assert_paths_equivalent(&paths, &reference.paths, 1e-2);
}
```

**SVG output validity** (`tests/svg_output.rs`):
- Output is valid SVG path `d` strings (no NaN, no Inf)
- Path starts with `M`
- All ops produce correct string format

### Snapshot Tests

Use `insta` crate (or manual JSON comparison) to snapshot SVG output for:
- All 7 primitives with default options
- All 6 fill styles on rectangle
- roughness=0 (clean), roughness=1 (default), roughness=3 (rough)
- Seeds: 1, 42, 12345

Regenerate snapshots only when intentionally changing algorithms.

### Benchmarks (`benches/renderer.rs`)

Using Criterion:
- `rectangle_cold` — single rectangle, no warmup
- `ellipse_warm` — 100 ellipses with same Generator
- `svg_path_complex` — complex path (US map outline)
- `hachure_large` — 1000px × 1000px rectangle fill

Target: each primitive < 1ms (rough.js equivalent is ~2ms just in browser overhead).

---

## 14. Implementation Phases

### Phase 0 — Scaffold (Day 0, ~2h)
- [ ] `cargo new rough-rs --lib`
- [ ] Set up `Cargo.toml` with features
- [ ] Create all module files (empty stubs)
- [ ] Write `verify/generate_reference.js` and capture `reference.json` for 5 test cases
- [ ] Set up CI (GitHub Actions: `cargo test`, `cargo clippy`)

### Phase 1 — Math & Geometry (Day 1, ~4h)
- [ ] Implement Mulberry32 in `math.rs` with wrapping arithmetic
- [ ] Verify Mulberry32 output against JS reference values (first 20 values for seed=42)
- [ ] Implement `RngHelper` with `next()`, `random_offset()`
- [ ] Implement `geometry.rs`: `Point`, `Line`, `rotate_points`, `rotate_lines`, `line_intersection`
- [ ] Unit tests for math and geometry

### Phase 2 — Core Renderer: Line & Rectangle (Day 1-2, ~6h)
- [ ] Implement `renderer::_line()` with bowing + roughness
- [ ] Implement `renderer::line()` (double-stroke wrapper)
- [ ] Implement `renderer::rectangle()` via 4 `_line()` calls
- [ ] Implement `generator::line()` and `generator::rectangle()`
- [ ] Implement `svg::ops_to_path()` for Move/LineTo/BCurveTo
- [ ] Implement `svg::drawable_to_paths()`
- [ ] Integration test: render rectangle to SVG, visually inspect

### Phase 3 — Ellipse (Day 2-3, ~8h)
- [ ] Implement `generate_ellipse_params()` with auto-adjust roughness
- [ ] Implement `ellipse_with_params()` — parametric sampling + catmull-rom fit
- [ ] Implement Catmull-Rom → Cubic Bezier conversion (port from `points-on-curve`)
- [ ] Implement `generator::ellipse()` and `generator::circle()`
- [ ] Test: ellipse output matches rough.js reference for seed=99

### Phase 4 — Fills (Day 3-4, ~8h)
- [ ] Implement `solid.rs` fill
- [ ] Implement `hachure.rs` scan-line algorithm
- [ ] Implement `pattern_fill_polygons()` dispatcher
- [ ] Wire fills into `generator::rectangle()` and `generator::ellipse()`
- [ ] Implement `cross_hatch` (two hachure passes)
- [ ] Implement `dots.rs`
- [ ] Implement `dashed` and `zigzag_line` variants
- [ ] Test: all 6 fill styles render without panic, hachure matches reference

### Phase 5 — Remaining Primitives (Day 4-5, ~8h)
- [ ] Implement `renderer::linear_path()` and `renderer::arc()`
- [ ] Implement `renderer::curve()` with Catmull-Rom
- [ ] Implement `generator::polygon()`, `generator::linear_path()`, `generator::arc()`, `generator::curve()`
- [ ] Implement `svg_path` parser + `renderer::svg_path()`
  - SVG arc `A` command → cubic bezier (port from Mozilla algorithm in rough.js)
  - Use `svgtypes` for parsing, normalize to M/L/C
- [ ] Test: all primitives render, polygon fill works

### Phase 6 — Seed Parity & Polish (Day 5-6, ~6h)
- [ ] Run full reference fixture comparison for all primitives
- [ ] Fix any RNG drift issues (likely in ellipse step count calculation)
- [ ] Add snapshot tests via manual JSON comparison
- [ ] Benchmarks: all primitives < 1ms
- [ ] README with usage examples
- [ ] `examples/basic.rs` generating a sample SVG output file
- [ ] Clippy clean, no warnings

### Phase 7 — excalidraw-rs Integration (Day 6-7)
- [ ] Tag v0.1.0
- [ ] Publish to crates.io
- [ ] Begin `excalidraw-rs` with `rough-rs` as dependency

---

## 15. Design Decisions & Rationale

### GAP-001: SVG output only (no Canvas)
**Decision:** Only implement SVG path string output, not Canvas 2D context calls.
**Rationale:** Excalidraw-rs needs SVG to pass to `resvg` for PNG rendering. Canvas would
require a 2D drawing context abstraction. SVG path strings are simpler, portable, and
sufficient for all downstream use cases.

### GAP-002: Port Mulberry32 inline, no `rand` crate for core
**Decision:** Implement Mulberry32 directly rather than using `rand` crate's algorithms.
**Rationale:** `rand` crate's `SmallRng` is not guaranteed to match JS Mulberry32 output.
Seed parity requires byte-exact matching of the PRNG sequence. The implementation is 10 LOC.
`rand` is still used as optional dep for the `seed=0` (non-deterministic) fallback.

### GAP-003: Do not require exact float match in parity tests
**Decision:** Use `ε = 0.01` tolerance when comparing path coordinates against JS reference.
**Rationale:** Floating point ordering differences (JS `f64` vs Rust `f64`) can cause tiny
divergences in intermediate calculations even with the same PRNG sequence. The visual output
will be perceptually identical. Excalidraw only uses seed for reproducibility across renders
of the same element, not for cross-language comparisons.

### GAP-004: Port `points-on-curve` inline
**Decision:** Port the ~100 LOC `points-on-curve` npm package inline into `renderer.rs`
rather than finding/wrapping a Rust equivalent.
**Rationale:** No suitable Rust crate with identical algorithm exists. The algorithm is
small and well-understood. Inline port keeps dependencies minimal and ensures correctness.

### GAP-005: `svgtypes` crate for SVG path parsing
**Decision:** Use `svgtypes` crate for parsing SVG path `d` strings, not custom parser.
**Rationale:** SVG path parsing is complex (handles all SVG arc/curve variants, relative/absolute
coordinates, implicit commands). `svgtypes` is mature and handles all edge cases. It's optional
behind the `svg_path` feature flag, so consumers who don't need `generator.path()` don't pay
the dependency cost.

### GAP-006: `FillStyle::Sunburst` deferred
**Decision:** Do not implement `sunburst` fill style in v0.1.
**Rationale:** `sunburst` is not used by Excalidraw. It would add complexity for no gain.
Can be added in v0.2 if requested.

### GAP-007: Thread-safety via `&self` (no internal mutation)
**Decision:** `Generator` methods take `&self`, all randomness state passed on stack.
**Rationale:** `RngHelper` is created fresh per `Drawable` from the resolved seed, so there
is no shared mutable state. This makes `Generator` `Sync` and allows concurrent rendering
across threads — important for `excalidraw-rs` batching multiple elements.
