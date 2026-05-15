#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createRequire } from "node:module";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const legacyRoot = path.join(repoRoot, "legacy", "rough");
const roughBundle = path.join(legacyRoot, "bundled", "rough.cjs.js");
const outputPath = process.argv[2]
  ? path.resolve(process.argv[2])
  : path.join(repoRoot, "tests", "fixtures", "reference.json");

if (!fs.existsSync(roughBundle)) {
  console.error(`Missing ${path.relative(repoRoot, roughBundle)}.`);
  console.error("Build the legacy reference first:");
  console.error("  cd legacy/rough && npm ci && npm run build");
  process.exit(1);
}

const require = createRequire(import.meta.url);
const rough = require(roughBundle);
const { curveToBezier } = await import(
  pathToFileURL(
    path.join(legacyRoot, "node_modules", "points-on-curve", "lib", "curve-to-bezier.js"),
  )
);
const { pointsOnBezierCurves } = await import(
  pathToFileURL(path.join(legacyRoot, "node_modules", "points-on-curve", "lib", "index.js"))
);

let fixtureRandomState = 0x12345678;
Math.random = () => {
  fixtureRandomState = (Math.imul(1664525, fixtureRandomState) + 1013904223) >>> 0;
  return fixtureRandomState / 2 ** 32;
};

const rngSeeds = [1, 42, 12345, 2147483647];
const shapeCases = [
  {
    name: "line_seed_1",
    method: "line",
    args: [10, 10, 200, 100],
    options: { seed: 1 },
  },
  {
    name: "rectangle_seed_42",
    method: "rectangle",
    args: [10, 10, 200, 100],
    options: { seed: 42 },
  },
  {
    name: "ellipse_seed_99",
    method: "ellipse",
    args: [100, 100, 200, 150],
    options: { seed: 99 },
  },
  {
    name: "ellipse_solid_fill_seed_42",
    method: "ellipse",
    args: [100, 100, 200, 150],
    options: { seed: 42, fill: "red", fillStyle: "solid" },
  },
  {
    name: "ellipse_hachure_fill_seed_42",
    method: "ellipse",
    args: [100, 100, 200, 150],
    options: { seed: 42, fill: "red", fillStyle: "hachure" },
  },
  {
    name: "circle_seed_12345",
    method: "circle",
    args: [100, 100, 80],
    options: { seed: 12345 },
  },
  {
    name: "circle_dots_fill_seed_42",
    method: "circle",
    args: [100, 100, 80],
    options: { seed: 42, fill: "red", fillStyle: "dots" },
    strictOps: false,
  },
  {
    name: "polygon_seed_42",
    method: "polygon",
    args: [
      [
        [10, 10],
        [140, 20],
        [120, 90],
        [30, 110],
      ],
    ],
    options: { seed: 42 },
  },
  {
    name: "linear_path_seed_42",
    method: "linearPath",
    args: [
      [
        [10, 10],
        [40, 70],
        [100, 30],
        [160, 90],
      ],
    ],
    options: { seed: 42 },
  },
  {
    name: "arc_open_seed_42",
    method: "arc",
    args: [100, 100, 160, 90, Math.PI / 6, Math.PI * 1.35, false],
    options: { seed: 42 },
  },
  {
    name: "arc_closed_seed_42",
    method: "arc",
    args: [100, 100, 160, 90, Math.PI / 6, Math.PI * 1.35, true],
    options: { seed: 42, fill: "red" },
  },
  {
    name: "curve_seed_42",
    method: "curve",
    args: [
      [
        [10, 80],
        [40, 10],
        [100, 110],
        [160, 40],
      ],
    ],
    options: { seed: 42 },
  },
  {
    name: "svg_path_arc_seed_42",
    method: "path",
    args: ["M80 80 A 45 45, 0, 0, 0, 125 125 L 125 80 Z"],
    options: { seed: 42, fill: "green" },
  },
  {
    name: "line_roughness_zero_seed_42",
    method: "line",
    args: [5, 8, 155, 48],
    options: { seed: 42, roughness: 0 },
  },
  {
    name: "line_disable_multistroke_seed_42",
    method: "line",
    args: [5, 8, 155, 48],
    options: { seed: 42, disableMultiStroke: true },
  },
  {
    name: "line_preserve_vertices_seed_42",
    method: "line",
    args: [5, 8, 155, 48],
    options: { seed: 42, preserveVertices: true },
  },
  {
    name: "rectangle_stroke_none_solid_fill_seed_42",
    method: "rectangle",
    args: [20, 30, 80, 50],
    options: { seed: 42, stroke: "none", fill: "#ffcc00", fillStyle: "solid" },
  },
  {
    name: "rectangle_fill_none_seed_42",
    method: "rectangle",
    args: [20, 30, 80, 50],
    options: { seed: 42, fill: "none" },
  },
  {
    name: "rectangle_hachure_angle_zero_gap_seed_42",
    method: "rectangle",
    args: [10, 10, 120, 80],
    options: { seed: 42, fill: "red", hachureAngle: 0, hachureGap: 8 },
  },
  {
    name: "rectangle_custom_dash_fill_seed_42",
    method: "rectangle",
    args: [10, 10, 120, 80],
    options: { seed: 42, fill: "red", fillStyle: "dashed", dashOffset: 12, dashGap: 4 },
  },
  {
    name: "rectangle_custom_zigzag_fill_seed_42",
    method: "rectangle",
    args: [10, 10, 120, 80],
    options: { seed: 42, fill: "red", fillStyle: "zigzag-line", zigzagOffset: 6 },
  },
  {
    name: "ellipse_negative_dimensions_seed_42",
    method: "ellipse",
    args: [100, 100, -80, 45],
    options: { seed: 42 },
  },
  {
    name: "ellipse_tiny_seed_42",
    method: "ellipse",
    args: [100, 100, 0.5, 0.25],
    options: { seed: 42 },
  },
  {
    name: "polygon_concave_hachure_seed_42",
    method: "polygon",
    args: [
      [
        [10, 10],
        [150, 10],
        [95, 60],
        [150, 115],
        [10, 115],
      ],
    ],
    options: { seed: 42, fill: "red", hachureGap: 7 },
  },
  {
    name: "arc_closed_hachure_seed_42",
    method: "arc",
    args: [100, 100, 160, 90, Math.PI / 6, Math.PI * 1.35, true],
    options: { seed: 42, fill: "red", fillStyle: "hachure" },
  },
  {
    name: "curve_three_points_seed_42",
    method: "curve",
    args: [
      [
        [10, 80],
        [80, 10],
        [160, 70],
      ],
    ],
    options: { seed: 42 },
  },
  {
    name: "curve_repeated_points_seed_42",
    method: "curve",
    args: [
      [
        [10, 80],
        [10, 80],
        [100, 110],
        [160, 40],
      ],
    ],
    options: { seed: 42 },
  },
  {
    name: "svg_path_relative_commands_seed_42",
    method: "path",
    args: ["m10 10 h50 v30 q10 20 30 0 t40 0 s20 20 40 0 a25 15 30 0 1 60 10 z"],
    options: { seed: 42, fill: "green" },
  },
  {
    name: "svg_path_simplification_seed_42",
    method: "path",
    args: ["M10 80 C 40 10, 65 10, 95 80 S 150 150, 180 80"],
    options: { seed: 42, simplification: 0.5 },
  },
  {
    name: "rectangle_seed_zero_structure",
    method: "rectangle",
    args: [10, 10, 120, 80],
    options: { seed: 0, fill: "red", fillStyle: "hachure" },
    strictOps: false,
  },
];

const fillCases = [
  "hachure",
  "solid",
  "zigzag",
  "cross-hatch",
  "dots",
  "dashed",
  "zigzag-line",
].map((fillStyle) => ({
  name: `rectangle_fill_${fillStyle.replace(/[^a-z0-9]+/g, "_")}`,
  method: "rectangle",
  args: [10, 10, 120, 80],
  options: { seed: 777, fill: "red", fillStyle },
  strictOps: fillStyle !== "dots",
}));

function randomSequence(seed, count) {
  let state = seed;
  const values = [];
  for (let i = 0; i < count; i += 1) {
    if (state) {
      state = Math.imul(48271, state);
      values.push(((2 ** 31 - 1) & state) / 2 ** 31);
    } else {
      values.push(null);
    }
  }
  return values;
}

function summarizeDrawable(drawable) {
  return {
    shape: drawable.shape,
    options: drawable.options,
    sets: drawable.sets.map((set) => ({
      type: set.type,
      opCount: set.ops.length,
      ops: set.ops,
      size: set.size,
      path: set.path,
    })),
  };
}

function runCase(testCase) {
  const generator = rough.generator();
  const drawable = generator[testCase.method](...testCase.args, testCase.options);
  return {
    name: testCase.name,
    method: testCase.method,
    args: testCase.args,
    options: testCase.options,
    strictOps: testCase.strictOps !== false,
    drawable: summarizeDrawable(drawable),
    paths: generator.toPaths(drawable),
  };
}

const reference = {
  roughVersion: "4.6.6",
  source: "legacy/rough",
  rng: Object.fromEntries(rngSeeds.map((seed) => [seed, randomSequence(seed, 20)])),
  curveUtilities: curveUtilityReference(),
  cases: [...shapeCases, ...fillCases].map(runCase),
};

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(reference, null, 2)}\n`);
console.log(`Wrote ${path.relative(repoRoot, outputPath)}`);

function curveUtilityReference() {
  const points = [
    [0, 0],
    [10, 15],
    [20, 0],
    [30, 10],
  ];
  const bezier = curveToBezier(points, 0);
  return {
    input: points,
    curveTightness: 0,
    curveToBezier: bezier,
    pointsOnBezierCurves: pointsOnBezierCurves(bezier, 0.15),
    simplifiedPointsOnBezierCurves: pointsOnBezierCurves(bezier, 0.15, 5),
  };
}
