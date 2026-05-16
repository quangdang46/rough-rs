# rough-rs

Pure Rust rough.js 4.6.6-compatible sketchy SVG path generation.

rough-rs ports the supported rough.js 4.6.6 generator surface to Rust and
checks it against the vendored `legacy/rough` source tree. It generates SVG path
data without requiring a browser, Canvas, Node.js, or WASM at runtime. The crate
is intended as a rendering foundation for Excalidraw-style Rust tools.

## Install

```toml
[dependencies]
rough-rs = "0.1"
```

## Basic Use

```rust
use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, FillStyle, Generator, Options};

let generator = Generator::new(Config::default());
let drawable = generator.rectangle(
    10.0,
    10.0,
    120.0,
    80.0,
    Some(Options {
        seed: Some(42),
        fill: Some("red".to_string()),
        fill_style: Some(FillStyle::Hachure),
        ..Options::default()
    }),
);

let paths = drawable_to_paths(&drawable);
```

Each `PathInfo` contains SVG `d`, `stroke`, `stroke_width`, and `fill` values
that can be embedded in a larger SVG document.

## Supported Surface

The supported surface is intended to map 1:1 to rough.js 4.6.6 behavior for SVG
path generation. Canvas/browser integration, animation, DOM insertion, and full
SVG document generation are explicit non-goals for this crate.

Primitives:

- line
- rectangle
- ellipse and circle
- linear path
- polygon
- arc
- curve
- SVG path data behind the `svg_path` feature

Fill styles:

- hachure
- solid
- zigzag
- cross-hatch
- dots
- dashed
- zigzag-line

Compatibility summary:

| Area | Status |
| --- | --- |
| Generator primitives | Supported and fixture-tested against `legacy/rough`. |
| Fill styles | Supported and fixture-tested against `legacy/rough`. |
| Seeded RNG | Matches rough.js nonzero seed generator. |
| Dot fills | Follow rough.js `Math.random()` dot-center behavior, so exact dot coordinates are nondeterministic. |
| SVG path parsing | Supported behind the default `svg_path` feature. |
| Canvas/browser APIs | Not implemented by design. |

## Determinism

Set `Options { seed: Some(value), .. }` for deterministic output, matching
rough.js seeded behavior. `seed = 0` uses a nondeterministic fallback when the
`rand` feature is enabled.

Dot fills intentionally follow rough.js 4.6.6 and use unseeded randomness for
dot center placement. Their op structure is stable, but exact dot coordinates
are nondeterministic just like rough.js. See `docs/parity-audit.md` for the full
parity contract.

## Feature Flags

- `rand`: enables nondeterministic fallback behavior for `seed = 0`.
- `svg_path`: enables SVG path parsing through `svgtypes`.

Both features are enabled by default.

## Reference Fixtures

Fixtures are generated from the vendored rough.js source in `legacy/rough`.
The `legacy/rough` tree is kept in the repository for development parity work,
but it is intentionally excluded from the crates.io package:

```bash
cd legacy/rough
npm ci
npm run build
cd ../..
node verify/generate_reference.mjs
```

The strict parity gate is:

```bash
cargo test --test comprehensive_parity
```

## Examples And Benchmarks

```bash
cargo run --example basic
cargo run --example excalidraw_element
cargo run --example complex_showcase
cargo bench
```

Benchmarks are informational performance guardrails rather than CI pass/fail
thresholds.

## Release Status

v0.1 has passed the local release quality gates. Publishing still requires an
authenticated crates.io account and explicit human approval.
