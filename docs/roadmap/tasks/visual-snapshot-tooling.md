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
- Migrate the portable authority to an immutable standalone GPUI Visual export,
  with complete provenance and unchanged licenses. Preserve baseline stores.
- Replace the existing visual Python orchestration with a separate Rust consumer
  after parity evidence, reusing the existing Gallery/browser production renderers
  and Tart infrastructure. Never import the mobile SDK's renderer graph.
- Provide exact producer-side story selection, independent required coverage,
  static multi-suite Review Collections and read-only JSON discovery/inspection.
  Selected runs are explicitly non-accepting development evidence.
- Run all actual captures in disposable Tart guests. Record host build identity
  separately from guest rendering environment; validate sources/artifacts before
  and after capture and return bounded diagnostics and raw PNGs before sealing.
  Preserve cleanup and interrupted-acceptance evidence.

## Non-goals

- No browser appearance or behavior changes, portal capture, native WKWebView
  pixels, AppKit traffic-light pixels, animation-frame or full UI-state matrix.
- No renderer helper dependency, GPUI migration, registry publication, server,
  browser launch, automatic baseline acceptance, tolerance or masking.
- No changes to another writer's worktree or automatic integration of its dirty
  state. Baselines must be re-reviewed after product changes land.
- No baseline acceptance, environment adoption, WebP/LFS conversion, new browser
  composition or sequence claim merely from completing the tooling migration.
  Gallery state/framing coverage remains owned by its linked packet. New browser
  surfaces/sequences require their own bounded follow-up contracts.

## Parallel-lane contract

Base condition: integrated visual-capture feature and production shell.
Depends on: existing authenticated CaptureChrome and typed snapshot seams.
Provides/API: visual.toml plus visual-check/compare/accept recipes.
Exclusive paths: scripts/visual/, tests/visual/, vendor/cargo-gpui-visual/,
visual.toml, crates/bbr-app/tests/e2e_visual_shell_capture.rs.
Shared integration points: justfile, .gitignore, E2E_ARCHITECTURE.md,
ROADMAP.md, testing track, Phase 4 and Current delivery. The existing
`/Users/patwoz/dev/Personal/macos/bbr/scripts/tart_vm.py` lifecycle is shared
infrastructure: narrowly harden cleanup and add an optional offline-clone policy
hook; preserve the E2E runner's default boot/key policy and recipe behavior.
Integration order: Paper-alignment product changes first; snapshot tooling
second, followed by fresh captures and an explicit baseline review.

## Constraints

Use the real optimized app with isolated test profiles and real WKWebViews.
Keep capture behind visual-capture; no new runtime or testkit protocol.
Drive readiness through typed state and deadlines, never pixel stability or
fixed sleeps. Preserve mandatory functional E2E and performance budgets.
All compiler commands use mbx. GUI/Gallery/browser captures execute only in
throwaway headless Tart guests, never on the user's desktop. Preserve host-wide
GPU/compatibility exclusion where required and guest suite serialization;
long-running commands have hard deadlines with process-group/VM cleanup.
The existing private NAT/key-only SSH route needs the applicable network approval
before boot; no bridge, forwarding, unrelated services or host GUI fallback.
Source inventory is narrow and includes canonical renderer/assets/tool inputs,
never secrets, personal browser data, generated reports or baseline images.

## Evidence

### Unit and contract

- [x] Upstream comparator/acceptance contracts pass unchanged.
- [x] BBR adapter tests prove inventory, provenance, bounded execution and
  fail-closed configuration/output handling.
- [x] Source/environment mismatches and changed pixels require review; an
  explicitly selected case pair can be accepted without accepting others.

### Standalone Rust migration

- [ ] Complete immutable standalone import passes provenance, upstream contracts,
  strict Clippy and formatting; old baseline readers remain compatible.
- [ ] Rust consumer reuses the portable capture/seal/compare/collection/accept APIs;
  replaced Python visual entrypoints are removed only after verified parity.
- [ ] Story selection precedes rendering and preserves all appearances/pages;
  missing/extra/duplicate or ignored selection fails without accepting baselines.
- [ ] Independently authored coverage cannot become green by omitting cases from
  producer output. Invalid suites remain visible beside valid siblings.
- [ ] VM environment/build/source identity, output transfer, bounded cancellation,
  process/VM cleanup and stale/no-write acceptance paths have direct tests.
- [ ] Tokens, actual Checkbox pointer transitions and multipage Sidebar pilot
  pass with production assets, not test-only substitutes.
- [ ] Two independent complete VM captures match decoded RGBA8 exactly; actual
  images/reports are inspected and existing eight browser cases are retained.
- [ ] Fresh current-tree functional gates pass; reviewed baseline approval remains
  a separate explicit user decision, never inferred from tooling success.

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
# standalone/consumer mbx tests --locked --offline, strict Clippy, fmt
# canonical just e2e sequence in Tart excluding deferred e2e-perf, plus peek/gallery
just roadmap
```

First missing-baseline check must fail for review. Acceptance names every
reviewed case explicitly; a fresh second run must pass against those baselines.

## Delivery sequence

1. Establish one exclusive writer and recorded clean base. Preserve historical
   migration commits and stopped-owner dirty work; no silent worktree takeover.
2. Import/verify the standalone tool before moving the consumer orchestration.
3. Prove the bounded VM pilot, then full Gallery/browser capture and explicit
   source/environment/coverage/selection/review parity before Python cutover.
4. Self-review source, failure modes and requirement-to-evidence mapping;
   no subagents are authorized for this task.
5. Commit coherent verified slices locally. No publication or baseline acceptance.
   Final completion requires all migration evidence above, not an import alone.

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

**Current Rust host-preparation checkpoint:** `just visual-prepare` now builds
and stages actual Gallery/browser payloads using bounded mbx and the existing
Xcode route; 1,449 sealed sources plus one/two exact Cargo executables were
independently hash-verified. Nothing was executed. Evidence under
`/private/tmp/bbr-visual-prepare.DW3Hlm`; release builds 1m29s / 1m40s.
32 consumer tests, 66 Python units + separate CLI integration, immutable upstream
contracts, strict Clippy/fmt and roadmap pass. These are build/staging artifacts,
not guest or pixel evidence, and become stale after subsequent source edits.
Fresh preparation is required before capture; installation/supervision remain
open. Native Rust execution guard and baseline approval boundary remain closed.

**Earlier Python adapter/integration history (not Rust migration acceptance):**

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
