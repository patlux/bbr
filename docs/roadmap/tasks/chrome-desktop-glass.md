# Task packet — Desktop glass for browser chrome

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design system](../tracks/ui-design-system.md) · [Native integration](../tracks/native-integration.md) · [DESIGN](../../../DESIGN.md) §2.

## Outcome

The docked sidebar and chassis around the content card reveal a softly blurred
Desktop through native macOS material. The content card, WKWebViews, website
backgrounds and controls remain opaque and unchanged.

## Scope

- One public `NSVisualEffectView` behind each browser window's GPUI root,
  with behind-window blending, native activation and deterministic teardown.
- One translucent chrome tint; keep existing space identity and geometry.
- Live Reduce Transparency and Increase Contrast opaque fallbacks.
- Typed native evidence and real-app regressions for appearance, resize,
  collapse, multiple windows and unchanged web sessions.

## Non-goals

- No WKWebView transparency, private WebKit API, CSS injection or page sampling.
- No blur of page content, no Liquid Glass claim, no floating-Portal redesign.
- No baseline acceptance, host GUI execution, unrelated fixes or publication.

## Parallel-lane contract

Not parallelized. One local writer; no delegation or sibling-worktree changes.

## Constraints

- GPUI owns controls, AppKit owns behind-window material, WebKit owns pages.
- Do not change geometry, profile isolation, focus or page opacity.
- Keep native observer/layer ownership bounded to the browser window.
- Do not weaken contrast, functional or performance checks.

## Evidence

### Unit and contract

- [x] Material policy, opaque fallbacks, worst-case text contrast and tint alpha.
- [x] Testkit serialization and normal/testkit builds.

### Real-app E2E

- [x] Real native material attachment/policy, unchanged opaque WKWebView and session identity.
- [x] Light/dark, reduced transparency, increased contrast, collapse/resize.
- [x] New browser windows retain the same native material contract.
- [x] Official sealed chrome capture; native desktop pixels excluded from GPUI readback.
- [ ] Final desktop-composited appearance, inactive state and native corner
      antialiasing at 1×/2×, through an approved official evidence route.

### Performance

- [x] Existing warm-tab and sidebar toggle/resize/restore scenarios require
      enabled glass, the installed mask and opaque WebViews. Setup/observations
      stay outside the timed production path; optimized executables compile.
- [ ] Execute the optimized scenarios against unchanged budgets after Patrick's
      2026-09-27 performance-execution deferral lifts. That instruction suspends
      performance runs, not merely canonical host runs. VM measurements would
      remain report-only; no performance execution or pass is claimed.

## Completion gates

```bash
just verify
mbx clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-vm e2e-shell-sidebar e2e-windows e2e-shell-spaces
just visual-check
just e2e-vm e2e
```

Use the established hardened Tart wrapper if the legacy runner cannot enforce
the standing isolation and single-process cleanup profile. Do not run on host.
While performance execution is deferred, run the recipe-derived functional
sequence without `e2e-perf`, plus Peek/Gallery. Label it a functional aggregate,
not a canonical `just e2e` pass. Visual exit 1 remains review required; neither
withheld coverage nor changed references may be accepted automatically.

## Delivery sequence

1. Implement the smallest native material and chrome-only paint path.
2. Run focused native E2E and lower-level checks.
3. Review the diff locally; no independent review is claimed without delegation.
4. Run stable-slice checks and report remaining visual/performance acceptance.

## Iteration policy

Start with the actual native layer, then expose only the chrome through it.
Diagnose failures from typed native and layout observations before editing paint.
Keep the website and content card unchanged. Do not broaden into the old queue.

## Blocked stop condition

Stop if public AppKit cannot provide the material behind the existing GPUI root,
or if the disposable VM cannot satisfy its isolation profile. Report exact
checks, preserved changes and the smallest missing environment/input. No private
API, host test or baseline workaround.

## Delivery ledger

### Implemented and locally reviewed — 2026-10-16

The browser owns public AppKit material per window, a single compensated chrome
tint and an owned even-odd card-corner mask. The original opaque chrome returns
for either accessibility fallback or native-install failure. Settings/Gallery
retain their own Root paint; no WebKit opacity setter, private API, CSS injection,
page sampling, profile mutation or geometry change was introduced.

Local review checked installation outside render, click-through hit testing,
window-owned teardown, refusal to replace a foreign mask, coordinate conversion,
cached geometry/tint updates, single-layer animated tint, fallback restoration,
optional-wire compatibility and separate Settings/Portal ownership. No
independent reviewer or delegation is claimed. Actual AppKit full-screen
transition, scale switching and final inactive/composited pixels remain unproven;
the content-card E2E uses the existing typed full-screen presentation override.

### Verification

- Full mbx-routed `just verify`, strict testkit all-target Clippy and
  `mbx test -p bbr --features testkit --bin bbr` pass. The last command passes
  **381 tests**; the testkit crate has **39 tests**, UI **242**. Exact log:
  `/tmp/bbr-chrome-glass-verify-final.log`.
- Affected Sidebar/Windows/Spaces suites pass **59 tests / 7 executables**:
  `/tmp/bbr-e2e-vm/run-20261016-214136-13343/`.
- Recipe-derived sequential functional aggregate plus Peek/Gallery passes
  **351 tests / 59 executables**, including Sidebar 23, Windows 9 and Gallery 48:
  `/tmp/bbr-e2e-vm/run-20261016-215841-31674/summary.log`.
  Invocation: `nix develop --command
  /Users/patwoz/.local/share/mise/installs/python/3.12.14/bin/python3
  /tmp/bbr-main-integration-e2e.py functional-aggregate`.
  The wrapper uses the repository planner with hardened clone-only authentication,
  listener/reachability audit, offline declared dependencies and bounded cleanup.
  Rounded executable durations total **370 s**; this is test-run time, not product
  latency. Clone/key/process/disk removal and former SSH unreachability are
  independently recorded in that directory's `behavior-cleanup-observed.json`.
- Optimized short/full performance executables compile without execution:
  `mbx test -p bbr --features testkit --release --test e2e_performance
  --test e2e_performance_full --no-run`. Strict all-target Clippy also passes
  after the new glass guards. Log: `/tmp/bbr-chrome-glass-perf-compile.log`.

### Attempts retained and evidence limits

- The first native startup attempt timed out before the gateway was reachable:
  `/tmp/bbr-e2e-vm/run-20261016-213246-99242/first-failed-suite.log`.
  Moving material installation out of render into window setup fixed the
  observed failure; focused, affected and aggregate runs subsequently passed.
- Early tint candidates failed unchanged D3 contrast bars. The final adaptive
  compensation passes light/dark, all space tints/intensities and black/white
  worst-case backdrops without changing foreground tokens or thresholds.
- Initial `just verify` failed only at roadmap validation because the new packet
  was not yet tracked. Intent-to-add made the packet visible; the complete rerun
  passed. The original log remains `/tmp/bbr-chrome-glass-verify.log`.
- The first sealed batch (`1792180072951890000-55767`) captured opaque fallback:
  the Tart image enables Reduce Transparency. It is not glass evidence. Visual
  fixtures now explicitly select glass and assert native state.
- Batch `1792181239453652000-63247` captured **162 images: 74 same / 88 changed**,
  with no capture errors or rendering-environment mismatch. All 158 accompanying
  app snapshots report attached/enabled/valid-mask state; 142 have a real opaque
  WebView, 16 have no active native WebView. The sealed report is
  `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/batches/1792181239453652000-63247/review/index.html`.
  Local inspection of its representative light/dark rails, dark full shell and
  light History actual/diff shows no obvious geometry displacement; History
  changes lie around the surrounding edge. This is a scoped review, not approval
  of all 88 changes. Native desktop material/mask pixels are excluded and explicitly
  withheld in the catalog. No new baseline was accepted.

### Final-source checkpoint

- After the performance-driver and backward-serialization additions, full
  `just verify` passes again: `/tmp/bbr-chrome-glass-verify-closeout.log`.
  No production behavior changed after the passing functional aggregate.
- Final official batch **`1792182619885735000-78484`** repeats **74 same /
  88 changed**, zero capture errors and no rendering-environment mismatch. All
  **162 PNGs are byte-identical** to the preceding glass batch. All **1,547**
  sealed source hashes match the current source; only documentation is updated
  afterward. Report:
  `/Users/patwoz/dev/Personal/macos/bbr/reports/visual/batches/1792182619885735000-78484/review/index.html`.
  Exit 1 is expected review-required/unproven coverage, not a passing visual gate.
- Final visual clone/key/process/disk cleanup and former SSH unreachability are
  verified in
  `/private/tmp/bbr-visual/1792182619885735000-78484-browser-transport/vm/cleanup-observed.json`.
- Accessibility E2E drives the live per-option testkit seam, not System Settings
  automation. Actual preference observation reuses the existing native display
  notification and now reads Reduce Transparency as well.

Product acceptance remains open for native composited pixels and deferred
performance; this is a verified implementation checkpoint, not a completed phase.
No baseline change, performance run, host GUI, push, delegation or sibling-worktree
change occurred.
