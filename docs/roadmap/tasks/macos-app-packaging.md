# Task packet — deterministic macOS `.app` packaging

Parents: [Phase 9 release safety](../phases/09-quality-release.md) ·
[Performance/release track](../tracks/performance-release.md) ·
[Development workflow](../../development-workflow.md) ·
[Parallel delivery](../../parallel-delivery.md)

Prepared parallel lane **L1**; branch `parallel/mvp-l1-packaging`, worktree
`bbr-mvp-l1-packaging`. This packet exists as a lane-owned implementation
packet; promoting it into `ROADMAP.md`'s active-lane roll-up is the
orchestrator's serial integration step, not part of this lane.

## Outcome

A deterministic macOS `.app` bundle is produced from the built `bbr` release
binary with a standard bundle structure, token-filled `Info.plist`, generated
placeholder icon, ad-hoc codesign, and a bounded real launch smoke proving the
packaged app starts and is cleanly terminated. Identical input binary yields a
byte-identical bundle on every rebuild.

## Scope

- `scripts/package/` packaging tools (stdlib Python only): deterministic
  `.icns` placeholder generator, `.app` assembler with fixed mtimes and ad-hoc
  `codesign` (`--timestamp=none`) plus `codesign --verify --strict`, and a
  bounded launch probe with process-group reaping.
- `packaging/` static assets: `Info.plist.template`, `PkgInfo`, and a README
  documenting the model and placeholders.
- Additive `justfile` recipes: `package-test`, `package`, `package-determinism`,
  `package-smoke`, `package-verify` — all bounded with hard timeout and
  process-tree cleanup.
- `scripts/tests/test_packaging.py`: non-GUI contract tests (icon structure and
  determinism, bundle layout/plist/permissions/mtimes, rebuild byte-identity,
  smoke contract against fake bundle executables including stubborn-process
  reaping).
- A real launch smoke through the bundle executable (`--bbr-hidden`), wrapped
  by the existing host-wide GUI gate, proving the packaged app starts.

## Non-goals

- Notarization, Developer-ID/App Store signing and hard-coded provisioning:
  Phase 9 release hardening, documented as placeholders. (The final product
  icon landed with [v2 S9](v2-brand-icon.md).)
- Bundling/embedding of native frameworks, `.dylib` relocation, or app code
  changes: the lane consumes the already-built binary generically.
- Rejecting `--bbr-*` test flags in non-testkit builds (the release-exclusion
  failsafe, E2E_ARCHITECTURE §8): a separate future gate.
- Any GPUI/anchor integration claim: this lane never touches app code and must
  not be read as shipping a polished runnable browser.

## Parallel-lane contract

```text
Base commit: 574750b (browser-MVP parallel orchestration ledger)
Depends on:  a built `bbr` Mach-O release binary; macOS toolchain (codesign);
             the existing host-wide GUI gate (scripts/with_gui_lock.py)
Provides/API:
  - python3 scripts/package/build_app.py --binary <bbr> --output <dir> [--bundle-id] [--version] [--skip-codesign] [--mtime]
  - python3 scripts/package/smoke_launch.py --app <BBR.app> [--observe] [--launch-timeout] [--no-verify] [--stderr]
  - just package-test | package | package-determinism | package-smoke | package-verify
Exclusive paths:
  - scripts/package/**, scripts/tests/test_packaging.py, packaging/**, docs/roadmap/tasks/macos-app-packaging.md
  - justfile: additive packaging recipes only (no edits to existing recipes)
Shared integration points: none in code — no Cargo.toml/Cargo.lock/ROADMAP.md/
  app-code changes; the GUI gate lock key is derived from the shared git
  common-dir, so the smoke serializes with all other GUI suites.
Integration order: review the lane diff at base 574750b; promote the packet
  into the active-lane roll-up when the reviewer signs off; the recipes and
  scripts land as-is (they consume the binary generically and do not depend
  on the GPUI anchor).
```

## Constraints

- No `Cargo.toml`, `Cargo.lock`, `ROADMAP.md`, or app Rust code changes; no
  new dependencies (pure Python stdlib; `codesign`/`timeout` from the OS).
- Deterministic output: fixed ordered writes, fixed mtimes
  (`SOURCE_DATE_EPOCH`), no timestamps/randomness in the icon or signature
  (`--timestamp=none`).
- Every recipe bounded by a hard timeout with `--kill-after`; the smoke always
  reaps its process group (SIGTERM then SIGKILL), including failure paths.
- The launch smoke is GUI-gated and uses `--bbr-hidden` so the brief window is
  never focused or activated.
- No weakened tests, budgets, or acceptance criteria; the smoke must observe a
  real launch of the real packaged binary.

## Evidence

### Unit and contract

- [x] `just package-test`: 12 tests — icns structure/sizes/determinism,
      render-shape pixel checks, bundle layout + plist tokens + permissions,
      rebuild byte-identity + fixed mtimes, version/bundle-id override, missing
      binary rejection, signed-bundle verify + fixed mtimes (macOS/codesign
      only), smoke success/crash/stderr/missing-executable, and stubborn-process
      reaping. *(Verified in this lane; see Delivery ledger.)*

### Real-app E2E

- [x] Bounded real launch smoke of the packaged `BBR.app`: bundle executable
      launched directly with `--bbr-hidden`, alive after the 5 s observation
      window, process group SIGTERM-terminated (exit −15), no stray processes,
      stderr captured. GUI-gated through the existing lock.

### Performance

- [x] Not applicable as a user-visible interaction slice; packaging cost is
      bounded by recipe timeouts (`BBR_PACKAGE_TIMEOUT`, `BBR_PACKAGE_SMOKE_TIMEOUT`)
      and the smoke observes a short fixed window. Determinism is the acceptance
      criterion here and is proven byte-identically.

## Completion gates

```bash
just package-verify     # package-test + package + package-determinism + package-smoke
just roadmap            # roadmap link validation (part of just verify)
```

The full `just verify` suite and canonical `just e2e` remain the serial
kernel's gates and are unaffected by this lane (no app code touched).

## Delivery sequence

1. One exclusive writer implemented and verified the bounded slice.
2. One independent read-only reviewer maps requirements to evidence.
3. One remediation writer only for concrete findings, then a focused recheck.
4. No separate final audit for this ordinary slice (per development-workflow).

## Iteration policy

Started from the smallest end-to-end path: stdlib icon generator → assembler →
smoke probe → unit contracts → real build/codesign → determinism diff → real
smoke. Fix the mechanism on each failing check (e.g., inverted render scale,
non-context-manager `plistlib.load`) rather than weakening assertions.

## Blocked stop condition

No blocker observed. If a future run cannot produce a signed bundle or the
smoke cannot observe a live launch (e.g., headless host), stop with the
preserved worktree, the exact gate failure, and the smallest environment
input required (a macOS GUI session with `codesign`).

## Delivery ledger

```text
Outcome: deterministic macOS .app packaging — bundle structure, Info.plist,
  placeholder icon, ad-hoc codesign, bounded launch smoke. Implemented.
Commits: none — left uncommitted at base 574750b for the orchestrator
  (worktree bbr-mvp-l1-packaging, branch parallel/mvp-l1-packaging).
Functional evidence:
  - python3 -m unittest scripts/tests/test_packaging.py → 11/11 OK (28 s)
  - just package-verify → package-test OK, package OK (release build 1m11s +
    bundle), determinism diff -r byte-identical, real smoke OK
  - codesign -dv: Identifier=dev.patwoz.bbr, Signature=adhoc;
    codesign --verify --strict: "valid on disk", satisfies DR
  - Bundle: Contents/{Info.plist,PkgInfo,MacOS/bbr,Resources/AppIcon.icns,_CodeSignature}
  - plutil: CFBundleIdentifier dev.patwoz.bbr, versions 0.1.0,
    LSMinimumSystemVersion 13.0, NSHighResolutionCapable true
Real-app E2E: launch smoke — app alive 5.1 s, terminated via SIGTERM (exit
  −15), no stray processes, GUI gate acquired/released.
Performance evidence: determinism proven byte-identically (two independent
  codesigned builds of the same binary diff clean); timing bounded by recipe
  timeouts.
Roadmap/documentation updates:
  - docs/roadmap/tasks/macos-app-packaging.md (this packet)
  - packaging/README.md (model, placeholders, boundaries)
  - ROADMAP.md roll-up intentionally NOT touched (orchestrator-owned).
Open blockers and non-goals: notarization/Developer-ID signing, final icon,
  release-exclusion failsafe, and GPUI-anchor integration are later work.
Worktree state: uncommitted changes on parallel/mvp-l1-packaging at base
  574750b; target/ artifacts are gitignored.

Planning/read-only: ~10 min (estimate)
Implementation: ~30 min (estimate)
Test execution: measured — unit 28 s / gate ~2.5 min (incremental build)
Review: reviewer confirmed functionality; one remediable finding — codesign
  leaves wall-clock mtimes on the executable and bundle directories, so the
  signed output violated the fixed-mtime invariant.
Remediation: after codesign + verify, every file and directory in the bundle
  is pinned back to the fixed epoch (deepest first, including the app root);
  added `test_signed_bundle_verifies_with_fixed_mtimes` (skipped where
  codesign is unavailable). Re-verified: 12/12 unit tests, signed bundle fully
  pinned, `codesign --verify --strict` OK, two signed bundles byte-identical
  via `diff -r`, GUI-gated launch smoke alive 5.1 s / SIGTERM / exit −15.
Waiting/retries: none
Total elapsed: ~50 min (estimate)
Human intervention: none
```
