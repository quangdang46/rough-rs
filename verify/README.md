# Reference Fixtures

`generate_reference.mjs` records rough.js 4.6.6 behavior from a local upstream
rough.js checkout at `legacy/rough`. The generated JSON is the executable
reference for RNG parity, operation structure, SVG path strings, primitives,
and fill styles.
The matrix intentionally includes every supported generator method, all fill
styles, representative option interactions, SVG path command families, seed=0
structural coverage, and edge cases such as tiny or negative dimensions.
The generator installs a deterministic `Math.random` implementation so
rough.js's nondeterministic dot-fill and seed=0 paths can be regenerated without
fixture churn; tests still treat those cases as structural comparisons when the
runtime behavior is intentionally nondeterministic.

Regenerate fixtures from the repository root:

```bash
git clone https://github.com/rough-stuff/rough.git legacy/rough
cd legacy/rough
npm ci
npm run build
cd ../..
node verify/generate_reference.mjs
```

The script writes `tests/fixtures/reference.json` by default. Commit fixture
changes only when the legacy rough.js source or the intended parity matrix has
changed.
