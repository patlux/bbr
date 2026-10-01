# GPUI Visual

## Scope

This repository is the sole development source for the portable
`cargo-gpui-visual` library/CLI and its original review assets. Read
`docs/migration.md` and `docs/extraction.md` before implementation. SDK/app
checkouts consume immutable exports; never develop a second consumer fork.

- Keep comparison, seals and explicit acceptance GPUI-independent. Renderer
  helpers, production views, fixtures, native readiness and baselines belong to
  their SDK/application and compile against that consumer's own GPUI pin.
- Preserve exact same-size decoded RGBA8, reject precision loss, and never add
  tolerances, masks, resizing, implicit acceptance or acceptance-time recapture.
- Extend the original report, not a replacement frontend. Separate suite source
  seals, environments and baseline authorities remain separate.
- No new general test framework, scenario DSL, speculative plugins or forced
  dependency upgrades. Prefer standard Rust tests and consumer GPUI support.

## Verification and safety

- Compiler-invoking commands use `scripts/with-rust-toolchain.sh mbx ...`, with a
  hard timeout/process-group cleanup. No target-dir or compiler-cache overrides.
- Run locked/offline tests, strict all-target Clippy and rustfmt for each stable
  code slice. Contract fixtures use disposable stores, never real baselines.
- Native/GUI/GPU capture needs the consumer's ownership, authorization and
  host-wide exclusion through descendant cleanup. Portable tests need no device.
- Preserve unrelated work; inspect ownership before consumer writes. No automatic
  baseline adoption, secrets, real app data, listeners, remote, push or publishing.
- No subagents/workflows authorized by the current migration. One local writer;
  do not claim independent review. Commit only owned verified slices locally with
  exact checks and limitations. Keep provenance and licenses immutable.
- Update migration evidence without treating historical checks as fresh results.
  Missing consumer coverage/permissions remain open, never hidden as completion.
