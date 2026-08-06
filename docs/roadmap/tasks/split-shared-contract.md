# Task packet — split-view shared contract

Parents: [Phase 6](../phases/06-split-view.md) · [Browser interactions](../../product/browser-interactions.md) · [Design contract](../../../DESIGN.md) · [Parallel-delivery contract](../../parallel-delivery.md).

## Outcome

The production shell can derive one authoritative active-space pane layout and
frame each visible native web session by `WebViewId`. A test-only `SetSplit` /
`CloseSplit` seam proves two live WKWebViews receive independent geometry and
collapse back to the focused pane without app-owned focus duplication. This
serial anchor makes the later split-core lane safe to branch into its own
worktree.

## Scope

- Replace the id-less `WebEngine::attach_slot` geometry call with a
  per-`WebViewId` geometry contract while preserving `WebRect` and the existing
  AppKit y-flip boundary.
- Make the null, fake, and macOS engines address geometry by live view id;
  unknown ids are no-ops, destroyed ids lose cached geometry, and unchanged
  per-view rects produce no repeated native `setFrame:` call.
- Add a portable `PaneLayout` query derived only from the active space's domain
  split, active tab, and session registry. It identifies primary, optional
  secondary, and focused `WebViewId`; the app owns no second focus value.
- Render one stable native slot per visible pane. Single-pane geometry remains
  the existing sidebar/content contract; split geometry is two adjacent,
  positive horizontal panes without a draggable divider or visual polish.
- Add typed testkit `SetSplit { primary, secondary }` and `CloseSplit` requests,
  pane ids/focused pane, and keyed per-view rect evidence in the semantic
  snapshot.
- Add real-app macOS E2E for opening, resizing, id-addressed geometry, focused
  pane truth, split closure, session preservation, and the single-pane sidebar
  regression.
- Update this packet, Phase 6 status, architecture facts, and the canonical E2E
  recipe truthfully.

## Non-goals

- No divider drag, keyboard resize, split preview/drop zones, focus-ring polish,
  pane-close command, `Cmd-W` decision, or production split-creation UI.
- No independent app focus state; focused pane remains the domain active tab.
- No persistence changes, divider ratio, history/provider work, command-core
  changes, more than two panes, screenshots, or texture composition.
- No Phase-6 checklist promotion beyond the per-view geometry/shared-anchor
  capability actually proven.

## Parallel-lane contract

This anchor is serial and runs on `main` because it changes shared public
contracts and composition surfaces.

```text
Base commit: e1eecca (persistence integration recorded)
Depends on: integrated command core and portable persistence core
Provides/API: WebEngine::set_web_view_rect(WebViewId, WebRect, content_height),
  BrowserController::pane_layout() -> Option<PaneLayout>, typed SetSplit /
  CloseSplit testkit requests, keyed pane geometry snapshot
Exclusive writer paths during this serial packet: bbr-web geometry trait,
  bbr-web-macos geometry adapter, bbr-browser pane facade/fake/tests,
  bbr-app slot/render/testkit/performance integration, bbr-testkit split wire,
  split E2E plus the existing semantic-id E2E assertion,
  ARCHITECTURE.md, E2E_ARCHITECTURE.md, Phase-6/task docs, justfile
Shared integration points: public WebEngine trait, controller facade,
  app composition root, testkit envelope, canonical E2E ordering
Integration order: land and verify this serial anchor; record its exact main
  commit; only then create parallel/split-core and its sibling worktree
```

## Constraints

- GPUI owns chrome; each webpage remains a real native WKWebView child.
- Portable crates import no macOS frameworks and keep `#![forbid(unsafe_code)]`.
- `WebRect` remains host-layout/top-left geometry; `flip_rect` remains the only
  portable coordinate conversion.
- Single-pane slot geometry, profile isolation, live-session counts, focus
  routing, and all existing sidebar/E2E contracts must remain unchanged.
- Stable slot ids include the `WebViewId`; snapshots key geometry by the same
  id rather than relying on map iteration or creation order.
- No compatibility fallback or parallel mutation of `main`; one serial writer.
- GUI suites run only sequentially through the repository host-wide GUI lock.

## Evidence

### Unit and contract

- [x] `WebEngine` stays object-safe with the per-view geometry method.
- [x] Fake/null engines record geometry against the requested live id and
      ignore unknown ids.
- [x] macOS geometry caching distinguishes views and suppresses unchanged
      rect updates without changing flip math.
- [x] `pane_layout()` returns `None` for empty state, one view for single-pane,
      two ordered views for split state, and the domain-active member as focus.
- [x] Close/move/crash-adjacent controller tests leave registry, engine,
      visibility, and pane-layout evidence consistent.
- [x] Testkit split requests and snapshot values round-trip as typed data.

### Real-app E2E

- [x] `SetSplit` presents two simultaneous native child views with distinct,
      adjacent, positive keyed rects and unchanged session identities.
      *(Owner: `just e2e-shell-split-geometry` green; the suite also waits
      for and asserts the typed `SplitOpened` event.)*
- [x] Window resize independently reframes both pane ids; focus remains derived
      from the active domain tab.
- [x] `CloseSplit` returns to one full-width focused pane, keeps both tabs and
      warm sessions, and leaves no orphan visible native view. *(The suite
      also asserts the typed `SplitClosed` event.)*
- [x] Existing single-pane sidebar geometry remains exact. *(Owner:
      `just e2e-shell-sidebar` green.)*
- [x] Canonical sequential `just e2e` gate is green after remediation; every
      GUI suite, including split geometry and semantic-id coverage, ran
      sequentially through the host-wide lock.

### Performance

- [x] Existing optimized interaction suites remain green. No new benchmark kind
      is introduced because this packet adds no production user invocation;
      split creation/divider interactions belong to the later split-core slice.
      The report-only host run enforced all absolute ceilings with no
      regressions.
- [x] Repeated idle frames do not change authoritative keyed geometry or issue
      redundant frame updates. Suppression is proven on null/fake/macOS engine
      paths, and the canonical GUI/performance run stayed green.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-split-geometry
just e2e-shell-sidebar
just e2e
just roadmap
git diff --check
```

GUI recipes run sequentially through `scripts/with_gui_lock.py` as encoded by
the `justfile`.

## Delivery sequence

1. One serial writer implements the smallest complete production seam on
   `main` and runs targeted portable checks while iterating.
2. Run the affected real-app split/sidebar suites, then the canonical
   sequential `just e2e` once the slice is stable. All are green after the
   final remediation.
3. One independent read-only reviewer maps every contract item to code and
   evidence; one focused remediation pass only for concrete findings.
4. Commit the verified anchor and record its exact hash here and in Phase 6.
5. Create `parallel/split-core` and its sibling worktree from that exact anchor;
   later work consumes the frozen seams rather than evolving them independently.

## Iteration policy

Start with the public geometry and controller contracts, then fake/null/macOS
implementations, then app slot composition, then typed testkit evidence and the
real-app E2E. After a failure, identify whether the defect is contract,
controller/session, GPUI layout, AppKit framing, or harness synchronization
before editing. Re-run focused evidence, then the canonical gate once.

## Blocked stop condition

Stop with the preserved `main` state and exact evidence if two native slot
elements cannot coexist through public GPUI APIs, if a per-view frame requires
changing `WebRect`/`flip_rect`, or if the typed wire cannot represent keyed
geometry without an explicit protocol decision. Never fall back to webpage
screenshots, textures, app-owned focus duplication, or a GPUI fork.

## Delivery ledger

```text
Outcome:
    Serial shared-contract anchor implemented on main as a reviewable
    working-tree diff: per-WebViewId geometry (WebEngine::set_web_view_rect),
    BrowserController::pane_layout() -> Option<PaneLayout>, one slot per
    visible pane (keyed bbr-native-web-slot-<n>), typed SetSplit/CloseSplit
    testkit requests, keyed pane geometry evidence (pane_layout + pane_rects)
    with content_rect preserved as the focused/active view, and the
    real-app split-geometry E2E suite. No commits made by the writer.
Commits:
    None (writer leaves a reviewable working-tree diff; owner commits).
Functional evidence:
    just verify green (fmt/check/test/clippy/roadmap/script-tests, 2026-08-06
    run by the writer) incl. bbr-web 10, bbr-browser 75 (11 new pane-layout /
    keyed-geometry / missing-member regressions), bbr-testkit 18 (5 new
    exact-wire-shape + envelope tests), bbr-app 64 unit tests (incl. the
    NullEngine contract and the PaneGeometry module); cargo clippy -p bbr
    --features testkit --all-targets -- -D warnings green.
Real-app E2E:
    Owner ran `just e2e-shell-split-geometry` and `just e2e-shell-sidebar`
    green (post-review: split suite also waits for/asserts the typed
    `SplitOpened`/`SplitClosed` events; the native content-height correction
    after resize is preserved). Canonical sequential `just e2e` is green;
    one stale unkeyed semantic-id assertion failed on the first canonical run,
    was corrected to the rendered keyed id, and the full gate then passed.
Performance evidence:
    No production user invocation added → no new benchmark kind. Existing
    focused-slot content_rect/presentation wiring is preserved; a secondary
    pane cannot satisfy a pending benchmark. The optimized report-only host
    run is green with no regressions; representative p95 values: startup
    403.834 ms, warm tab switch 0.077 ms, sidebar toggle 0.930 ms,
    sidebar resize 0.002 ms, dense scroll 3.464 ms. This host is explicitly
    non-canonical, so no canonical-runner claim is made.
Roadmap/documentation updates:
    ARCHITECTURE.md keyed geometry + pane layout facts, justfile recipe
    e2e-shell-split-geometry (+ canonical just e2e position), task packet
    evidence/ledger. Phase-6 checklist: only the per-WebViewId geometry item
    is marked complete (fully proven by the green split/sidebar suites); all
    broader split-interaction items stay open. ROADMAP.md unchanged.
Open blockers and non-goals:
    Non-goals honored: no divider drag/keyboard resize/preview/focus-ring
    polish, no app-owned focus state, no persistence/command-core changes,
    no protocol version bump (additive wire growth on version 6).
Worktree state:
    bbr/main at ec52bf0 with an uncommitted working-tree diff; no sibling
    worktrees touched.

Planning/read-only:
    ~1.1 h (estimate; includes doc and codebase survey)
Implementation:
    ~1.9 h (estimate)
Test execution:
    ~0.4 h (estimate)
Review:
    independent reviewer completed; findings remediated by a focused
    remediation pass (see below)
Remediation:
    ~1 h (estimate): NullEngine live-id/keyed-geometry contract + tests;
    pane_layout split-branch requires both sessions (no false single-pane
    degradation) + missing-primary/secondary and genuine non-focused-crash
    regressions; exact wire-shape JSON assertions (incl. fixed-literal
    deserialization) for SetSplit/CloseSplit/SplitOpened/SplitClosed/
    PaneLayoutSnapshot/PaneRectSnapshot; typed SplitOpened/SplitClosed event
    waits in the real-app split E2E; pane-geometry state extracted into the
    testkit-only performance::pane_geometry module; synthetic unkeyed
    visible-id compatibility evidence removed and the existing semantic E2E
    updated to the rendered keyed slot id; E2E_ARCHITECTURE.md and Phase-6/task
    documentation updated truthfully
Waiting/retries:
    none
Total elapsed:
    ~4.4 h (estimate; no measured runtime timestamps available)
Human intervention:
    none
```
