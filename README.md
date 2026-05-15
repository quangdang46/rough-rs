# rough-rs

Pure Rust port of rough.js for sketchy, hand-drawn SVG path generation.

This crate is currently being implemented against the vendored rough.js source
in `legacy/rough`. The intended public surface is a library-first Rust API that
generates SVG path data without requiring a browser, Canvas, Node.js, or WASM at
runtime.

## Status

The repository is in early scaffold form. The Beads task graph in `.beads/`
tracks the remaining implementation work, including strict parity validation
against `legacy/rough`.

## Feature Flags

- `rand`: enables nondeterministic seed fallback behavior for `seed = 0`.
- `svg_path`: enables SVG path parsing support through `svgtypes`.

Both features are enabled by default.
