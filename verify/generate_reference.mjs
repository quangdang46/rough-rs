#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
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
    name: "circle_seed_12345",
    method: "circle",
    args: [100, 100, 80],
    options: { seed: 12345 },
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
    drawable: summarizeDrawable(drawable),
    paths: generator.toPaths(drawable),
  };
}

const reference = {
  roughVersion: "4.6.6",
  source: "legacy/rough",
  rng: Object.fromEntries(rngSeeds.map((seed) => [seed, randomSequence(seed, 20)])),
  cases: [...shapeCases, ...fillCases].map(runCase),
};

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(reference, null, 2)}\n`);
console.log(`Wrote ${path.relative(repoRoot, outputPath)}`);
