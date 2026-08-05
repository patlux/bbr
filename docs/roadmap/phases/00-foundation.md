# Phase 0 — Reproducible foundation

Parent: [Roadmap](../../../ROADMAP.md) · Related track: [Performance and release safety](../tracks/performance-release.md)

## Outcome

Every contributor and automation run gets the same bounded Rust development environment.

## Completed checklist

- [x] Cargo workspace initialized on `main`.
- [x] `flake.nix` and `flake.lock` provide Rust, rustfmt, Clippy, rust-analyzer, build tools, GNU coreutils, and `just`.
- [x] `.envrc` loads the flake through direnv.
- [x] `.gitignore` excludes Cargo, Nix, direnv, and local artifacts.
- [x] Bounded `just fmt`, `check`, `test`, `clippy`, `verify`, and `e2e-smoke` recipes exist.
- [x] Xcode 16.4 workaround is encoded through `BBR_XCODE_DEVELOPER_DIR`.
- [x] Test Unix-socket paths remain below macOS `SUN_LEN` under both direnv and `nix develop`.

## Exit evidence

- `just verify`
- `just e2e-smoke`
- Foundation and timeout commits beginning with `179b686`, `3daf33a`, and `3c272ab`.
