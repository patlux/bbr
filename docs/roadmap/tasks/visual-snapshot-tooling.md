# Task packet — Reusable visual snapshot tooling

Parents: [Phase 4](../phases/04-ui-shell.md) · [Testing track](../tracks/testing.md) · [Design contract](../../../DESIGN.md).

## Outcome

BBR's real production chrome has a repeatable Light/Dark screenshot matrix,
reviewed exact-pixel baselines and the shared cargo-gpui-visual HTML review and
selective acceptance workflow, without adopting the mobile SDK's GPUI graph.

## Scope

- Pin the unmodified, Apache-2.0 portable CLI from GPUI Mobile Kit, including
  license, notices, source hashes and its upstream contract tests.
- Adapt the existing optimized real-app capture to ordinary/representative
  fixtures in Light/Dark, full chrome and a precisely defined Sidebar crop.
- Add a source/environment seal, explicit inventory, bounded recipes and
  integration tests; document native-pixel exclusions and baseline review.

## Non-goals

- No browser appearance or behavior changes, portal capture, native WKWebView
  pixels, AppKit traffic-light pixels, animation-frame or full UI-state matrix.
- No renderer helper dependency, GPUI migration, registry publication, server,
  browser launch, automatic baseline acceptance, tolerance or masking.
- No changes to the concurrent Paper-alignment lane or automatic integration
  into its worktree. Baselines must be re-reviewed after that lane lands.

## Parallel-lane contract

Base condition: integrated visual-capture feature and production shell.
Depends on: existing authenticated CaptureChrome and typed snapshot seams.
Provides/API: visual.toml plus visual-check/compare/accept recipes.
Exclusive paths: scripts/visual/, tests/visual/, vendor/cargo-gpui-visual/,
visual.toml, crates/bbr-app/tests/e2e_visual_shell_capture.rs.
Shared integration points: justfile, .gitignore, E2E_ARCHITECTURE.md,
ROADMAP.md, testing track, Phase 4 and Current delivery.
Integration order: Paper-alignment product changes first; snapshot tooling
second, followed by fresh captures and an explicit baseline review.

## Constraints

Use the real optimized app with isolated test profiles and real WKWebViews.
Keep capture behind visual-capture; no new runtime or testkit protocol.
Drive readiness through typed state and deadlines, never pixel stability or
fixed sleeps. Preserve mandatory functional E2E and performance budgets.
All compiler commands use mbx; GUI execution takes the shared repository lock;
long-running commands have hard deadlines with process-group cleanup.
Source inventory is narrow and includes canonical renderer/assets/tool inputs,
never secrets, personal browser data, generated reports or baseline images.

## Evidence

### Unit and contract

- [x] Upstream comparator/acceptance contracts pass unchanged.
- [x] BBR adapter tests prove inventory, provenance, bounded execution and
  fail-closed configuration/output handling.
- [x] Source/environment mismatches and changed pixels require review; an
  explicitly selected case pair can be accepted without accepting others.

### Real-app E2E

- [x] Both fixtures pass semantic readiness/geometry/session checks in both
  appearances, before and after production Metal readback.
- [x] Eight images are inspected; two independent captures are exactly equal.
- [x] Affected capture suite, normal testkit checks and stable functional
  aggregate pass. Combined-tree evidence is recorded in
  [Current delivery](../../delivery/current.md); visual baseline re-review after
  Paper alignment remains separate from these functional gates.

### Performance

No user-visible interaction or timing is changed. The captures are optimized,
not performance measurements. Existing performance execution remains deferred
by Current delivery; no performance acceptance is claimed.

## Completion gates

```bash
just visual-test
just visual-check
just visual-compare <explicit-run>
just verify
# bounded mbx clippy -p bbr --features testkit --all-targets -- -D warnings
# bounded mbx clippy -p bbr --features visual-capture --test e2e_visual_shell_capture -- -D warnings
# canonical just e2e sequence excluding deferred e2e-perf, plus peek/gallery
just roadmap
```

First missing-baseline check must fail for review. Acceptance names every
reviewed case explicitly; a fresh second run must pass against those baselines.

## Delivery sequence

1. Isolated sole writer, as explicitly authorized by Patrick.
2. Add bounded adapter and its tests, then capture and inspect actual images.
3. Self-review source, failure modes and requirement-to-evidence mapping;
   no subagents are authorized for this task.
4. Commit coherent verified slices locally; integration waits for handoff.

## Iteration policy

Run targeted contracts first, diagnose failures before changing code, and
never accept unexplained differences. Re-check source and environment seals
after capture-affecting edits. Keep integration-baseline review separate from
acceptance of the implementation on its recorded base.

## Blocked stop condition

Stop if GUI ownership/environment, capture determinism, native boundaries or
an unresolvable required gate prevents truthful evidence. Preserve the worktree,
exact run paths, passed/failed checks and smallest needed user input. Do not
change product rendering or weaken comparisons to obtain a pass.

## Delivery ledger

The adapter, unchanged portable CLI, documentation and eight reviewed baseline
images are implemented. Fifteen adapter contracts and 29 upstream contracts
pass, as do real optimized captures, exact independent repeat comparison,
source-staleness rejection, workspace verification and both Clippy variants.
Self-review added timeout/GUI-lock cleanup, extra-vendor-source and non-regular
PNG checks; no product rendering was changed.

The user-authorized local integration preserves the Paper product commits.
Fresh combined-tree functional gates, including titlebar reload parity, portal,
Peek and Gallery, now pass. The old branch failures remain historical evidence,
not current functional blockers. All eight browser snapshots differ from their
pre-Paper references and still require explicit review and acceptance; integration
alone does not approve pixels. Exact evidence and integration state live in
[Current delivery](../../delivery/current.md). No performance pass is claimed.
