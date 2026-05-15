# rough.js compatibility matrix

rough-rs targets the pure generator and SVG path-output behavior of the vendored
`legacy/rough` rough.js 4.6.6 source tree. The executable reference is generated
by `verify/generate_reference.mjs` and checked by
`cargo test --test comprehensive_parity`.

## Supported 1:1 surface

| Area | rough.js 4.6.6 behavior | rough-rs status | Evidence |
| --- | --- | --- | --- |
| Generator methods | `line`, `rectangle`, `ellipse`, `circle`, `linearPath`, `polygon`, `arc`, `curve`, `path` | Supported with Rust snake-case where appropriate (`linear_path`) | `tests/comprehensive_parity.rs` renders fixture cases by method from JSON |
| Shape names | `line`, `rectangle`, `ellipse`, `circle`, `linearPath`, `arc`, `curve`, `polygon`, `path` | Matched through `ShapeType::as_rough_str` | `tests/options.rs` |
| Op model | `move`, `lineTo`, `bcurveTo` inside `path`, `fillPath`, `fillSketch` sets | Matched as `OpType` and `OpSetType` | `tests/comprehensive_parity.rs` |
| Default options | rough.js generator defaults from `legacy/rough/src/generator.ts` | Matched in `ResolvedOptions::default` | `tests/options.rs` |
| Option merge | Config defaults plus per-call overrides | Matched through `Generator::resolve_options` | `tests/options.rs` |
| Seeded RNG | rough.js 48271 generator for nonzero seeds, with rough.js dot-fill coordinates intentionally using `Math.random()` | Matched numerically for seeded cases; dot-fill coordinates are nondeterministic like rough.js | `tests/fixtures/reference.json`, `src/math.rs` tests |
| SVG serialization | `opsToPath` and `toPaths` path/fill/fillSketch mapping | Matched structurally and numerically; string formatting is not required byte-for-byte | `tests/comprehensive_parity.rs`, `src/svg.rs` tests |
| Fill styles | `hachure`, `solid`, `zigzag`, `cross-hatch`, `dots`, `dashed`, `zigzag-line` | Supported | `tests/comprehensive_parity.rs` |
| Ellipse/circle fills | Solid and patterned fills before stroke | Supported | `ellipse_solid_fill_seed_42`, `ellipse_hachure_fill_seed_42`, `circle_dots_fill_seed_42` fixtures |
| SVG path parsing | `M/L/H/V/C/S/Q/T/A/Z`, relative commands, simplification | Supported behind the default `svg_path` feature | `tests/comprehensive_parity.rs`, `tests/generator_path.rs` |
| Edge cases | zero roughness, disabled multistroke, preserved vertices, seed 0 structure, tiny/negative dimensions, no stroke/fill strings | Covered by fixtures | `tests/fixtures/reference.json` |

## Rust API differences that do not change generated output

| Difference | Rationale |
| --- | --- |
| Rust uses snake-case method and option field names such as `linear_path`, `stroke_width`, and `fill_style`. | Idiomatic Rust naming; fixture generation records rough.js names and the Rust parity test maps them back to Rust fields. |
| `Generator::path` is feature-gated by `svg_path`. | Keeps the parser dependency optional for consumers that only need primitive rendering. Default features enable it. |
| `SvgPath` stores `stroke_width` instead of rough.js `strokeWidth`. | Rust naming only; attribute semantics are the same. |
| Floating point SVG strings are compared through op data instead of byte-for-byte text. | Rust and JavaScript format equivalent `f64` values differently; numeric op parity is the compatibility contract. |

## Explicit non-goals

| rough.js area | v0.1 status |
| --- | --- |
| Canvas rendering and DOM/SVG element insertion | Not implemented; rough-rs emits path data for downstream renderers. |
| Browser runtime, Node runtime, and WASM bindings | Not required by this crate. |
| Animation or full SVG document generation | Out of scope; examples show how to wrap returned paths in an SVG document. |

## Nondeterministic rough.js behavior

rough.js 4.6.6 uses `Math.random()` for dot-fill center placement. rough-rs
intentionally follows that behavior rather than forcing dot fills through the
seeded per-shape randomizer. Exact dot coordinates are therefore
nondeterministic in both implementations; fixture tests compare dot-fill
structure and op counts instead of random coordinate values.
