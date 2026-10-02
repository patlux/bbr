# Task packet — Tokens, Checkbox and Sidebar visual pilot

Parents: [Phase 4](../phases/04-ui-shell.md) · [Testing track](../tracks/testing.md) · [Visual tooling](visual-snapshot-tooling.md) · [Gallery coverage](gallery-visual-coverage.md) · [Design contract](../../../DESIGN.md).

## Outcome

A fresh, explicitly partial Light/Dark pilot demonstrates static Tokens, an
input-driven production Checkbox round trip, and complete outer pagination of
Sidebar with separately measured nested framing of its default family. It runs
through the existing Gallery/macOS-text/Metal producer in disposable Tart, with
source-bound requirements, original PNG bytes and its own non-accepting review.
This is the first runtime gate of the migration, not full Gallery coverage,
baseline approval or animation-sequence acceptance.

## Scope

- Reuse append-only story identities 0 (Tokens), 16 (Checkbox), 44 (Sidebar),
  existing Gallery selection, production components, upstream comparison/seals
  and the supervised Tart transport. Select before constructing the renderer.
- Preserve the existing selected-story contract: every authored state,
  contrast, density, appearance and page belonging to a selected story is required.
  The focused assertions below do not authorize silently filtering that schedule.
- Add only feature-gated Gallery observation/input/readiness seams necessary for
  the pilot; no control restyling or production model authority. Asset wiring is
  a separate approval prerequisite, not authorized by this packet.
- Establish distinct source-bound identities for Checkbox endpoint stills before
  producing them. They must not overwrite an ordinary unchecked case or masquerade
  as the story's nonexistent State selector. Keep existing static requirements.
- Preserve the historical 936 required pairs and 31 withheld requirements until
  their own replacement evidence passes. Pilot progress clears no full-matrix gap.

## Non-goals

- No browser product change, GPUI/vendor patch, dependency migration, new renderer,
  direct recorder mutation to simulate input, native window, OS pointer/IME claim,
  arbitrary scripted journey, or assumed 60/120 Hz animation sampling.
- No complete dense-500 traversal, new Sidebar family, app-owned command/results/
  downloads surface, Portal/WebKit pixels, or implied coverage of absent branches.
  Those gaps remain with [Gallery coverage](gallery-visual-coverage.md).
- No full-suite Collection/acceptance promotion, guessed page counts, automatic
  baseline/environment adoption, Python removal, host capture or performance run.

## Parallel-lane contract

Not parallelized. One local writer and documented self-review; no subagents.
Dependencies: immutable GPUI Visual consumer and supervised transport, legitimate
Gallery asset-source approval, approved disposable-VM network route and an actually
available pinned guest runtime. Exclusive territory is Gallery capture support and
its consumer contracts. Shared integration points are visual configs/catalogs,
recipes and roadmap. No foreign worktree or separate product authority is touched.

## Constraints

Preserve GPUI-CE `568271c01c5a9d640d7b091e5bfd136b04035f28`, Gallery's dependency
allowlist and normal behavior. Feature-off builds must not gain capture state.
Host compiler calls use bounded mbx; all Gallery GUI/Metal/input execution uses
Tart, including semantic TestPlatform scenarios. No native-window fallback.

Use the existing 1200×1000 logical frame, 2× scale and 2400×2000 RGBA8 output,
48pt page inset, 48pt header and measured consecutive pagination. Never resize,
mask or tolerate differences. Source/guest changes require fresh preparation and
captures; no import or reseal of old raw output. Runtime failure or uncertain
cleanup remains a failure with retained diagnostics.

### Source-confirmed seams and limits

All source paths below are under `/Users/patwoz/dev/Personal/macos/bbr`:

- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture.rs` uses
  `HeadlessAppContext` with the actual macOS text system and Metal renderer,
  initializes `gpui_component::Root`, and filters a numeric story before context
  creation. It currently neither dispatches input nor advances scheduler timers.
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app/capture.rs`
  selects through existing controls and forces BBR Reduced Motion. Selecting a
  case does not reset the Core Controls recorder. Isolate each appearance/case's
  input journey with a fresh Gallery state and assert its initial values.
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/harness.rs` and
  `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/controls.rs`
  demonstrate GPUI event dispatch and snapshot assertions, not macOS input or
  native pixels. The pinned `HeadlessAppContext` has `update_window` and
  `advance_clock`; `Window::dispatch_event` is public. `debug_bounds` belongs to
  `VisualTestContext`, and its rendered-frame map is private. Do not assume the
  existing test helper can be called on the Metal runner.
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/core_controls.rs`
  has an existing hug-child `control_slot`. A capture-only child-prepaint observer
  there can measure the actual Checkbox child without replacing its renderer or
  consulting inaccessible GPUI internals. Bounds must be fresh for the input draw;
  a whole-page/wide wrapper or a hard-coded coordinate is not a control hit target.
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/checkbox.rs`
  routes clicks to the Gallery observer, has an independent keyed toggle state,
  and uses both a 250ms background timer and a 250ms `AnimationElement`.
  At the pinned GPUI revision, the timer uses the test scheduler's clock while
  `AnimationElement` uses `scheduler::Instant::now()` (the native wall clock via
  `web_time::Instant`). Advancing one clock does not advance the other.
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_page.rs`
  measures direct story children, not nested clipping, glyphs or shadows.
  Its static endpoint wait does not establish the Checkbox timer's completion.

### Authored pilot requirements

The following are semantic requirements, not a producer-generated inventory.
Before a sealed pilot, bind concrete ordered endpoint IDs and Light/Dark page
ranges to the source-controlled plan/catalog and reject unsupported scope. The
current consumer accepts static `gallery-…-state-…` identities only; it needs a
bounded endpoint contract before it can carry Checkbox journey stills. Do not
weaken `Schedule` completeness or attach an unsealed sidecar and call it proof.
A diagnostic measurement may inform authored page ranges; it is not its own
coverage authority. After plan edits, prepare and capture again on unchanged
requirements for the two independent repeats.

| Story | Required observations | Claim boundary |
|---|---|---|
| Tokens (0) | All ten direct children in source order: Materials heading, canvas/sidebar/field/overlay/glass swatches, Elevation heading, three elevation description rows. Assert both appearances and requested BBR theme; retain every scheduled density/contrast case. | Static token/value samples. The elevation text does not paint an elevation shadow; no native glass or input/animation claim. |
| Checkbox (16) | Initial unchecked → first pointer down (still unchecked) → pointer up (checked) → settled checked still → second pointer down (still checked) → pointer up (unchecked) → settled unchecked still, in each appearance. | Production GPUI listener/observer/model round trip and endpoint paint, not OS mouse delivery, keyboard accessibility, disabled/focus coverage or motion interpolation. |
| Sidebar (44) | Every scheduled family's consecutive outer pages, with no missing/duplicated/reordered direct child. In both appearances the default `N open rail — full composition` family must span more than one outer page and its full nested rail must be measured separately. | Outer multipage success is not complete dense-list, placeholder-surface or all-family nested-framing acceptance. Existing withheld requirements remain visible. |

Checkbox input requirements:

1. Select/draw the actual story; require one finite positive, fully framed control
   target and initial `checkbox_checked=false`. Reject absent, duplicate, stale,
   aliased or off-frame target observations before dispatch.
2. Dispatch actual GPUI move/down/up events at the observed control, with left
   button, one click and no modifiers. Do not call the callback, `observe_control`
   or `ControlRecorder::observe` directly. Assert snapshots between down and up;
   drain queued work and draw through the existing runner after each boundary.
3. Prove the existing observer path yields false → true → false, with other
   recorder/theme fields unchanged. A test injecting no event, wrong target or
   bypassed callback must not satisfy the journey. Any event-trace observation
   is Gallery-owned and must not change the control's callback behavior.
4. Start endpoint readiness only after the changed control has laid out and
   registered its production timer/animation. Drain timer registration, explicitly
   advance the existing test scheduler through the source-bound duration, and
   drain completion before the second click. Also require a layout after the
   wall-clock finite animation duration. Keep the existing 5s per-endpoint hard
   readiness bound; no sleeps, pixel-stability loop or arbitrary delay padding.
5. Bind any capture-side duration expectation to the unchanged vendor source and
   reject drift until reviewed; do not export/patch vendor timing just for tests.
   Source/contract tests prove clock ordering, not actual icon paint. Runtime
   evidence must show the legitimate `icons/check.svg`, not an empty source,
   manually drawn check or tolerated SVG error. Initial Radio uses the same asset
   but remains outside this three-story pilot.
6. Capture the three settled stills with distinct identities and fixed pointer/
   focus state. Assert semantic states and frame evidence at readback. A later
   unchanged draw should remain pixel-identical, but never use that equality as
   readiness. Treat these as endpoint stills, not cadence/timeline evidence.

Sidebar and paint-framing requirements:

- The default model has one pinned row, three regular rows, three Favorites, one
  expanded folder/two members, two space chips, address with 125% zoom, titlebar
  controls, footer and divider. Require every expected visible item's own current
  bounds and active clip ancestry; positive wrapper bounds or plain `.id()` text
  printed in the story do not satisfy this check. Verify footer/strip, last row
  and folder members as well as the first row. Labels must not be unexpectedly
  ellipsized; intentional production title truncation is recorded, not disabled.
- Measure layout and painted extent separately. Text/glyphs, control shadows,
  selection lift and focus rings where present must fit the intended clip/frame.
  Preserve legitimate rounded-surface clipping. Actual PNG inspection complements
  these measurements; geometry formulas alone cannot prove shadow/glyph paint.
- Reuse Gallery observation seams and existing Sidebar row/folder/anchor slots
  where sufficient. Anchor bounds must be shown to match the intended region;
  an invisible anchor is not automatically a measured descendant or paint extent.
  Missing access to a required region remains a concrete blocker, not a guessed
  rectangle, newly fabricated component or silent production-instrumentation edit.
- The dense family carries ten pinned plus 64 regular window rows out of 500,
  with a 436-row spacer at offset zero. Label it bounded-window diagnostic evidence;
  do not reinterpret its outer pages or scroll extent as 500 visible rows. Full
  traversal needs the legitimate model/windowing authority in a later slice.
- Historical default page ranges `[0,8)` / `[8,14)` are a diagnostic reference,
  not current native proof. Do not copy them into a newly active contract without
  fresh measured checks of child identities, ranges and nested layout.

## Evidence

### Unit and contract

- [ ] Endpoint request/identity/order and existing selected-story completeness
  fail closed on unknown/missing/extra/duplicate steps, pages or appearances.
- [ ] Fresh target/frame provenance, input ordering, semantic transition and both
  clock boundaries have focused regressions; synthetic results are labeled only
  as protocol evidence. No fake PNG or geometry fixture proves native coverage.
- [ ] Missing nested content/paint evidence fails rather than promoting wrapper
  geometry; stale source/config/plan/report, partial acceptance and cancellation
  checks remain intact. Feature/dependency boundaries remain green.

### Real-app E2E

- [ ] Approved guest route verifies actual runtime, source/executable identity,
  legitimate assets, text/Metal environment and absence of host/native windows.
- [ ] All authored pilot observations pass, including the checked icon and second
  click, complete outer Sidebar ranges and default-family nested/paint framing.
- [ ] Two independently prepared same-source/environment pilot processes match
  decoded RGBA8; inspect actual PNGs and the explicit partial HTML report.
- [ ] Gallery semantic regressions and affected capture gates pass in Tart;
  normal/timeout/cancel cleanup evidence comes from real owned resources.

### Performance

Not applicable to this tooling-only packet: no user-visible product behavior or
budget changes. Capture/test durations are reported as tooling measurements.
Product-performance execution remains deferred, not passed.

## Completion gates

Use repository-bounded recipes or explicit hard timeouts with process-tree
cleanup. Run targeted portable tests during implementation, strict all-target
consumer/Gallery capture-feature Clippy and formatting, then `just visual-test`,
`just verify` with compiler calls through mbx, and `just roadmap`.
After approvals run `just e2e-vm e2e-gallery`, affected capture/pilot checks through
the supervised Tart route and the parent packet's stable functional order (only
deferred performance excluded, plus Peek/Gallery). No existing host capture recipe
is safe to invoke. No completion checkbox is earned by this packet's publication.

## Delivery sequence

1. Define source-bound endpoint identities and portable rejection contracts;
   preserve full selected-story scheduling and all historical requirements.
2. Add the smallest capture-only target/input/readiness observation, keeping the
   existing component and event authority. Do not stage asset changes without
   Patrick's specific permission. Review exact diff and affected test seams.
3. After network/asset approval, verify guest prerequisites and run fresh pilot
   diagnostics. Resolve measured framing/readiness failures within the allowed
   Gallery scope; ask before crossing product/vendor/runtime boundaries.
4. Author/recheck requirements from the reviewed evidence, then run two new exact
   independent captures and inspect images/reports. Document self-review and any
   concrete remediation. Commit only verified coherent slices; never auto-accept.

## Iteration policy

Prefer the smallest unproven boundary, not another broad source audit. Keep input,
clock, nested-layout, painted extent and comparison evidence distinct. A failed
native gate requires a causal diagnosis before retry. Freeze the source/requirement
tree for repeats; any change invalidates the prior seal. This pilot supplements,
not replaces, the full-migration and full-Gallery acceptance requirements.

## Blocked stop condition

Stop the affected path when network/asset approval, actual guest runtime, a
legitimate measurement seam or a required native gate prevents the next step.
Continue useful authorized portable work only where it proves a concrete missing
contract. If none remains, preserve exact tree/run/logs and report attempted paths,
unmet criterion and minimal user decision; no host fallback, runtime repair,
weakened comparison or speculative substitute.

## Delivery ledger

Execution status, exact commits, measured timings and approval boundaries live in
[Current delivery](../../delivery/current.md). This packet defines unproven
requirements; source inspection is not evidence that its gates have passed.
