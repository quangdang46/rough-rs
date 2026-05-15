# v0.1.0 release checklist

Publishing requires explicit human approval.

- [ ] Confirm every Bead for v0.1 is closed.
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo check --all-targets`.
- [ ] Run `cargo clippy --all-targets -- -D warnings`.
- [ ] Run `cargo test`.
- [ ] Run `cargo check --no-default-features`.
- [ ] Run `cargo bench --no-run`.
- [ ] Run `br dep cycles`.
- [ ] Run `bv --robot-insights`.
- [ ] Regenerate rough.js fixtures and confirm no unexpected diff.
- [ ] Review `docs/parity-audit.md` for nondeterministic rough.js cases and confirm no accepted divergences are recorded.
- [ ] Review crate metadata in `Cargo.toml`.
- [ ] Update `CHANGELOG.md` date.
- [ ] Tag `v0.1.0`.
- [ ] Publish only after explicit approval.
