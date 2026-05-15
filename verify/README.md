# Reference Fixtures

`generate_reference.mjs` records rough.js 4.6.6 behavior from the vendored
`legacy/rough` source tree. The generated JSON is the executable reference for
RNG parity, operation structure, SVG path strings, primitives, and fill styles.

Regenerate fixtures from the repository root:

```bash
cd legacy/rough
npm ci
npm run build
cd ../..
node verify/generate_reference.mjs
```

The script writes `tests/fixtures/reference.json` by default. Commit fixture
changes only when the legacy rough.js source or the intended parity matrix has
changed.
