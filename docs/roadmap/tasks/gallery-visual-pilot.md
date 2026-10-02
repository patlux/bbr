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
consumer can now plan typed schema-2 Checkbox endpoints, but deliberately refuses
their execution, raw preparation and stored-run review until input/readiness
validation exists. No real plan/catalog is activated. Do not weaken `Schedule`
completeness or attach an unsealed sidecar and call it proof.
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

### Bounded portable endpoint-plan increment

Extend only the consumer's independent plan with schema 2 and a typed optional
endpoint on a case. Schema 1 remains byte-compatible and rejects endpoints.
For each opted-in Checkbox static case, keep that original case and immediately
follow it with exactly `initial-unchecked`, `checked-after-click`, and
`unchecked-after-second-click` endpoint cases in that order. Each endpoint base
is the original base plus `-endpoint-<name>`; metadata and per-appearance child
ranges must equal the original case. Only story 16 / State 00 / `default` is
eligible; these endpoints are not new State selector values. Retain all other
static cases and require the upstream story selection to include every authored
page of both the static case and its entire endpoint triple.

Resolve these requirements before any producer call and bind them to the original
plan/catalog hashes. Exercise portable positive/negative contracts for identity,
ordering, completeness, ranges, unsupported schemas, source drift and selection
nonacceptance. This increment does not implement input execution or invent a
semantic receipt. Until actual input/clock/framing evidence validation is wired,
reject endpoint-bearing plans at request creation, raw preparation, stored-run
review and full-capture preflight (including direct transport and guest command
construction), before any producer/VM call or seal/report publication. Static
selections from a schema-2 plan may still use the unchanged
static path. Do not activate a real schema-2 plan/catalog or modify production
Gallery code, assets, upstream or baselines in this portable increment.

### Bounded Checkbox target-observation increment

Before dispatching input, add a feature-gated per-draw Checkbox target observer
on the existing hug-child container. Observe its sole production child and the
active GPUI content-mask intersection, not the page wrapper or a hard-coded
rectangle. A monotonically increasing draw token invalidates previous/missing
observations; callbacks retain their own token, and late callbacks, repeated
observations, multiple children, nonfinite/nonpositive bounds and clipping fail.
Page resets also invalidate old tokens. Existing static Checkbox capture checks
this target only on the page containing its control child, and requires absence
on other pages and stories. The observer changes no layout, input listeners,
state or assets.

Portable tests prove recorder/token/geometry rejection only. Add a feature-gated,
ignored Gallery scenario comparing the observed child with the existing semantic
control bounds in Light/Dark, repeated draws, a control-only page, absence when
paginated out and after switching stories, and unchanged semantic state. Add a
bounded feature-enabled Gallery E2E recipe so the guest plan cannot silently omit
this scenario; the normal Gallery recipe has the feature disabled. Compile and
inspect the plan on the host; execute it only in the approved Tart guest, with
no runtime download/repair fallback. The ordinary E2E runner's dependency
realization and default clone policy are not this hardened/no-install route;
recipe discoverability alone does not authorize execution. Do not infer painted
extent, inner Checkbox hitbox, pointer delivery, ancestor identities or shadow
framing from this rectangular content-mask intersection. Do not lift the consumer's
endpoint guard. Input dispatch, two-clock readiness and semantic artifact validation
remain subsequent work. Preserve this candidate uncommitted if its affected guest
E2E gate cannot yet run; pure tests alone do not verify the GPUI callback wiring.

### Bounded semantic-gate preparation increment

Reuse the existing host build, source snapshot, payload staging and offline runtime
verification to prepare the feature-enabled Gallery test harness, without running
it. Give this payload the explicit `gallery-semantic` purpose, distinct from the
`gallery` image suite. Require exactly the Cargo-reported release test executable
for the `bbr-gallery` binary with `visual-capture`, plus the standalone consumer;
never substitute the normal Gallery producer, a feature-off harness or a guessed
artifact glob. Preserve compiled paths and all existing byte/mode/source checks.
The fixed host build is `mbx test --no-run --locked --offline --release` with the
existing release build override, package/bin selectors and `visual-capture`.

Add negative-first contracts for wrong/missing/duplicate test artifacts, feature
identity, exact Rust-to-Python payload roles and refusal at capture/bootstrap
boundaries. A host-only prepare command may build/stage/verify prerequisites but
must not execute the harness (even `--list`), boot Tart, install a runtime or emit
capture/acceptance evidence. Keep this bounded increment separate from subsequent
fixed guest action/result validation: until that route is tested, semantic payloads
must explicitly fail bootstrap/capture rather than fall through to image capture.
Use the existing source-frozen bootstrap, supervision, cleanup and return transport
for that subsequent route; no second VM pipeline or generic-runner rewrite.

This prerequisite is complete only after focused portable tests, strict consumer
Clippy/fmt, visual contracts and a real host-only preparation pass. Actual GPUI
callback evidence and the observer commit still require the approved guest gate.
Missing guest runtime must fail without realization/installation/repair. Network,
asset and report-specific adoption approvals remain unchanged.

### Bounded guest semantic-action increment

Behind the existing actual-admin/VirtualMac and installed-payload boundary, allow
only `guest-execute gallery-semantic semantic`, with no caller-selected filter or
argv. Bind an independently authored sorted inventory of all 49 current ignored
Gallery cases (including the capture-target scenario) to the original payload's
source hashes. First invoke the exact staged harness with ignored-test listing;
refuse missing, extra or duplicate names before any scenario runs. Then execute
the entire ignored suite serially with the recipe's flags plus explicit stable
pretty/no-color output. Preserve each existing test's own assertions/deadlines.

Use immutable upstream process ownership with one 600s budget spanning listing,
execution and source/installed-byte checks, bounded stdout and normal cleanup.
Require actual successful process status and exact per-test `ok` lines plus one
matching zero-failure/zero-ignored/zero-measured libtest summary. Nonzero status,
zero tests, missing/extra/duplicate/failed results, truncated or ambiguous output,
cancellation and source drift must not publish success. Unknown stdout format
fails closed with bounded diagnostics; never assume exit zero proves coverage.
Verify the protocol against a real, inert consumer libtest fixture on the host,
not by executing any Gallery binary there.

Publish only fresh `semantic-results.json` containing original bounded listing/run
stdout, exact requirement and executable identity, and the original payload hash.
Its Rust validator repeats completeness and source binding; it is functional test
evidence, never a screenshot seal, rendering-environment claim, Collection input
or baseline approval. Bracket all invocations/publication with existing installed
and shared-byte checks. No guest download/install/repair is added.

The existing sweep tests write exactly `every-story-renders.json` and
`mode-state-switching.json` under the compiled `target/bbr-gallery-perf` directory.
After complete test success, preflight both as bounded single-link regular files,
require their report-only schema/suite identity and reject any extra entry. Move
the unchanged bytes into distinct semantic diagnostic names in the fresh raw
output, then remove only the now-empty owned diagnostic directory. The unchanged
Python installed-tree verifier must then pass with no extra-file exception.
Missing/invalid outputs are failures, not reasons to disable the existing tests.
These diagnostic measurements are not product-performance acceptance.

Python guest composition may install, invoke this fixed action, recheck original
bytes/environment and package the exact semantic result/diagnostics through its
existing receipt/archive machinery. Keep Python bootstrap and host transport
closed for semantic payloads until host receipt/result validation is proven.

Host validation must reuse the existing bounded returned-byte/marker verifier and
host-versus-guest receipt binding. Keep semantic results a distinct typed result,
never `Captured`/`Suite` or a rendering-environment policy. Require exactly the
result, both named diagnostics and the guest receipt; validate hash-bound bytes
against the host's original payload, source requirements, build hashes and purpose.
Retain the original transport evidence for subsequent rechecks, not a replacement
marker or manifest read back from writable output. Cross-purpose substitution,
missing/extra or mutated files, stale sources/results, non-success transcripts,
changed guest identity and cancellation must fail. A pure return validator proves
neither VM teardown nor process exit: only the existing supervised route may
supply it after both child and independent host cleanup checks succeed.

The subsequent internal integration may generalize only that shared execution
input from image `Suite` to payload `Purpose`; image callers retain their capture
result and semantic callers must reject selections and return only `Verified`.
Use `source_suite` solely for existing source/runtime policy, never native dispatch
or image environment. The source-frozen bootstrap may then carry semantic purpose
unchanged to the fixed guest action, preserving actual guest identity, offline
runtime checks, fresh parents and refusal on hosts. Prove the whole supervised
chain with inert local fixtures: success, guest/SSH failure, cancellation, source
and receipt/result drift, host cleanup rejection, original identities and both
leases held until independent cleanup. No production injection switch or second
runner. Public capture stays closed; this route adds no CLI or approval bypass.
No public VM command or networking is activated by this increment. Required local
gates are negative-first contracts, actual inert libtest protocol verification,
full visual tests, strict consumer Clippy/fmt and roadmap validation. The observer
still cannot be committed or claimed callback-verified before its approved real
guest execution. At failed protocol or unsupported stdout, preserve diagnostics;
at unavailable guest approval/runtime, do not fall back to host execution.

### Authorized offline runtime provisioning

Patrick's 2026-10-02 instruction to fix all reported blockers supersedes the
previous preinstalled-only rule for this bounded guest prerequisite. Export only
the complete source-locked, already present and verified host Nix closure. Transfer
it through the existing isolated SSH route after clone authentication/listener
checks; hash and size-check it before one guest `nix-store --import`. Require actual
admin/VirtualMac identity before guest writes, disable substitution/builders, and
retain every existing post-import NAR/Python hash check before bootstrap execution.
No download, host store write, runtime repair, Golden Image mutation, alternate
Python, signature-policy change, recursive ownership change or new network route.
Failure means cleanup and diagnostics, never retrying an ambiguous import in place.

Keep the archive external to the product payload and bind it to the original
payload/lock and exact export bytes; bounded output, cancellation and process
ownership reuse the existing supervised path. Exercise negative-first contracts
for drift, links, size/hash mismatch, fixed import commands, host refusal and
failure/cancellation cleanup. Then prepare fresh sources and run all49 semantic
scenarios in a fresh clone, diagnosing actual failures without weakening tests.
The offline-import permission changes only this runtime prerequisite; baseline
adoption, product performance and full visual-migration acceptance stay separate.

### Observed guest environment compatibility

Observe the actual guest, never substitute the host environment. macOS 27's
name-selected IORegistry plist may be a direct `Root` dictionary instead of a
singleton array; both must still prove one unlocked admin console. The golden
image's empty `HomeManager` font directory and exact hidden version-marker link
are non-font metadata, not permission for user fonts. The one stock
`/Library/Fonts/Arial Unicode.ttf` alias must name exactly
`/System/Library/Fonts/Supplemental/Arial Unicode.ttf`; reject chained or other
links and require independently inventoried identical target bytes.

Tart's `system_profiler` returns an empty GPU array on this image. Query the
actual active CoreGraphics display modes and default Metal device in a bounded
pinned-Python child instead. Require positive bounded geometry and device identity;
no AppKit application, window, capture, fabricated values or host probe fallback.
Before/after facts must remain identical. These observations establish capability
and environment identity, not Metal rendering or image correctness.

### Runtime-driven SSH observation repair

The approved semantic run exposed a parser mismatch before guest execution:
OpenSSH `sshd -T` emits repeated `listenaddress` and `hostkey` directives for its
normal IPv4/IPv6 and host-key lists. Permit distinct entries only for those two
repeatable observation fields; retain rejection of repeated security settings,
unknown repeated directives, duplicate identical entries and malformed output.
Do not change authentication configuration, listener/access requirements, source
identity, cleanup, guest-runtime policy or test assertions. Reproduce the observed
output shape in a portable regression, run policy/lifecycle and visual contracts,
then retry with fresh source-bound preparation through the same supervised route.
This is a compatibility correction to observation parsing, not relaxed network
acceptance or permission to bypass a failed authentication/runtime gate.

## Evidence

The portable increment is implemented in
`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/src/gallery_plan.rs` and
`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/tests/gallery_endpoints.rs`.
`Plan::validate` owns schema/identity/metadata/appearance-range checks;
`Schedule::resolve` retains source-hash binding and exact upstream selection
completeness. The consumer plan schema is independent of upstream capture schema.
`require_static_capture` refuses all valid endpoint-bearing plans until semantic
validation exists. A complete set of correctly named synthetic RGBA8 PNGs does
not bypass that guard. Static schema-2 selections retain upstream `TEILLAUF`
nonacceptance. Tests also exercise the direct transport/guest and fresh-capture
boundaries. Current real plan remains schema 1, 630 cases / 936 page pairs;
31 withheld catalog rows remain unchanged. These are portable requirements and
refusal evidence, not completed input, framing or native-capture acceptance.

### Actual semantic prerequisite evidence (2026-10-02)

The source-bound supervised run at
`/private/tmp/bbr-visual-runtime-fix/attempt9` passes end to end:49/49 exact ignored
Gallery cases including the capture-target callback across12 appearances, actual
successful process status, both report-only diagnostics, original payload/build
binding, unchanged installed bytes and before/after environment. Host API returned
`status=verified`; test execution5.08s, complete caller96.22s (tooling timings only).
The guest is macOS27.0/26A5353q, VirtualMac2,1, with an active3200×1800 logical /
6400×3600 physical display, Apple Paravirtual Metal device and444 font hashes.
The51-path locked runtime was imported offline and rehashed before execution.
Request-bound resource cleanup and independent host recheck passed before gates
released. A preceding real cancellation also completed observed cleanup.

A final fresh source-bound recheck at
`/private/tmp/bbr-visual-runtime-fix/attempt10` also passes49/49 (5.38s tests,
119.54s caller). All1,513 source inputs match after execution; original identities,
result validation, independent cleanup and gate release pass. This does not count
as a second image capture. Visual contracts and strict Clippy pass; full aggregate
`verify` remains red at the unchanged Command benchmark (67.988/93.215ms p95
against50ms under load), despite a passing isolated19.686ms p95. No budget changed.

The semantic observer and its dependent transport can now form a coherent
intermediate commit with these targeted gates, not a stable-slice handoff. This proves TestPlatform callback
wiring, not native input, Check SVG, rendered Metal pixels, PNG/HTML review,
full functional aggregate or overall pilot completion. The capture endpoint guard
and all acceptance boxes below remain unchanged.

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
