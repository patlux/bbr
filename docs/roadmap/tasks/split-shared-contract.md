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
  split E2E, ARCHITECTURE.md, Phase-6/task docs, justfile
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

- [ ] `WebEngine` stays object-safe with the per-view geometry method.
- [ ] Fake/null engines record geometry against the requested live id and
      ignore unknown ids.
- [ ] macOS geometry caching distinguishes views and suppresses unchanged
      rect updates without changing flip math.
- [ ] `pane_layout()` returns `None` for empty state, one view for single-pane,
      two ordered views for split state, and the domain-active member as focus.
- [ ] Close/move/crash-adjacent controller tests leave registry, engine,
      visibility, and pane-layout evidence consistent.
- [ ] Testkit split requests and snapshot values round-trip as typed data.

### Real-app E2E

- [ ] `SetSplit` presents two simultaneous native child views with distinct,
      adjacent, positive keyed rects and unchanged session identities.
- [ ] Window resize independently reframes both pane ids; focus remains derived
      from the active domain tab.
- [ ] `CloseSplit` returns to one full-width focused pane, keeps both tabs and
      warm sessions, and leaves no orphan visible native view.
- [ ] Existing single-pane sidebar geometry remains exact.

### Performance

- [ ] Existing optimized interaction suites remain green. No new benchmark kind
      is introduced because this packet adds no production user invocation;
      split creation/divider interactions belong to the later split-core slice.
- [ ] Repeated idle frames do not change authoritative keyed geometry or issue
      redundant macOS frame updates.

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
   sequential `just e2e` once the slice is stable.
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
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```
