# Task packet — split app composition

Parents: [Phase 6](../phases/06-split-view.md) · [split shared contract](split-shared-contract.md) · [portable split core](split-core.md) · [browser interactions](../../product/browser-interactions.md) · [Design §11.7](../../../DESIGN.md) · [performance contract](../../product/performance-contract.md).

## Outcome

Productionize the existing controller-owned two-pane layout without inventing split creation or close commands: consume `bbr-split` to project two real native WKWebViews around the settled 2 pt divider, derive focused-pane presentation from `BrowserController::pane_layout()`, add pointer divider resizing with no idle invalidation, expose typed semantic/snapshot evidence, and prove the production path through real-app E2E plus optimized presentation measurements.

## Scope

- Add the serial `bbr-app -> bbr-split` dependency.
- Replace the current equal-flex split composition with `bbr_split::geometry::project` over the effective content rectangle.
- Render exactly two horizontal panes in controller order: primary left, secondary right, 2 pt divider.
- Add app-local split UI state containing only:
  - normalized `DividerPosition`;
  - `divider_dragging: bool`.
- Never store pane identities, split-open state, active tab, or focused pane in app-local split UI state.
- `BrowserController::pane_layout()` remains authoritative for split existence, pane order, and focused `WebViewId`.
- Default/new-split divider position is `bbr_split::geometry::DEFAULT_POSITION` (`0.5`).
- Preserve normalized position across pane-focus, sidebar, and window changes; reset after split close/dissolution so a later split starts at `0.5`.
- Render stable production identities:
  - `bbr-pane-0`
  - `bbr-pane-1`
  - `bbr-split-divider`
  - existing keyed `bbr-native-web-slot-<WebViewId>` slots.
- Render focused-pane affordance directly from `PaneLayout::focused`; no duplicate focus authority.
- Divider pointer path:
  - pointer down enters chrome focus and starts drag without changing pane focus;
  - pointer move converts the pointer position to a normalized request and applies `bbr_split::evaluate_resize`;
  - notify/render only for an accepted changed value;
  - pointer up ends drag and preserves the accepted value;
  - no polling, timer, animation, or idle invalidation.
- Keep the existing testkit `SetSplit` / `CloseSplit` seams only as deterministic setup/teardown, never as claimed product invocation.
- Add typed snapshot evidence:
  - `split_divider_position: Option<f32>` only while a split renders;
  - `split_divider_dragging: bool`.
- Add pane/divider semantic nodes; exactly one pane is focused and it matches `pane_layout.focused`.
- Add optimized `split_divider_resize` and `split_pane_focus` production-path scenarios with semantic presentation commits and fast-but-wrong guards.

## Settled values

| Value | Contract |
|---|---|
| Exactly two panes | Product/browser interaction contract |
| Horizontal, primary left / secondary right | Integrated `bbr-split` core |
| Default divider position `0.5` | `bbr_split::geometry::DEFAULT_POSITION` |
| Divider visual width `2 pt` | DESIGN §11.7 |
| Pane order and focused pane | `BrowserController::pane_layout()` |
| Minimum accepted pane width `1 pt` | Integrated `bbr-split::MIN_PANE_WIDTH` |
| No split animation | DESIGN motion contract |
| Warm interaction budget | p95 ≤ 50 ms, max ≤ 100 ms |

Do not invent keyboard resize steps, hit-target expansion, snap points, a larger minimum pane width, close controls/shortcuts, persisted ratio scope, or top/bottom behavior.

## Non-goals

- No production drag-to-glass split creation or `bbr-split-zone-*` preview.
- No keyboard split-creation shortcut.
- No dedicated close-split action, button, menu item, command result, or shortcut.
- No split-divider keyboard controls; values are not settled.
- No native WKWebView click → controller pane-focus synchronization; that requires a later native focus contract.
- No top/bottom or vertical split, tiling tree, third pane, nesting, persistence, or compatibility layer.
- No change to `bbr-browser`, `bbr-domain`, `bbr-web`, `bbr-web-macos`, `bbr-split`, command composition, URL input, or persistence behavior.
- No screenshots/textures of webpages and no WebKit-rendered browser chrome.

## Parallel-lane contract

```text
Serial anchor:    93499ae96a4bf41622bf1e4f14b7ede88e25d155
                  (reviewed command composition integrated and documented)
Branch/worktree:  parallel/split-app-composition
                  /Users/patwoz/dev/Personal/macos/bbr-split-app-composition
Depends on:       split shared contract 29c34e6, portable split core 9eeb6b2,
                  command composition 872d222
Provides/API:     production split layout/divider/focused-pane composition;
                  no new controller or native public contract
Exclusive paths:  Cargo.toml workspace dependency alias
                  Cargo.lock deterministic app dependency edge
                  crates/bbr-app/Cargo.toml
                  crates/bbr-app/src/app.rs (field/module wiring only)
                  crates/bbr-app/src/app/render.rs (delegate only)
                  new crates/bbr-app/src/app/split*.rs modules
                  crates/bbr-app/src/app/testkit/snapshot.rs
                  crates/bbr-app/src/shell/snapshot.rs
                  crates/bbr-app/src/shell/semantics/**
                  new crates/bbr-app/src/testkit/dispatch_split.rs
                  new crates/bbr-app/src/testkit/dispatch_benchmark_split.rs
                  crates/bbr-app/src/testkit/{mod,dispatch,dispatch_benchmark}.rs wiring only
                  crates/bbr-app/src/performance.rs minimal wiring
                  new crates/bbr-app/src/performance/split.rs
                  crates/bbr-app/tests/e2e_shell_split_composition.rs or focused module directory
                  new crates/bbr-app/tests/perf_common/split.rs
                  crates/bbr-app/tests/perf_common/drive.rs registration only
                  crates/bbr-app/tests/perf_common/{baseline,mod}.rs
                  crates/bbr-app/tests/e2e_performance*.rs registration only
                  crates/bbr-app/tests/performance-baseline.json split seeds
                  crates/bbr-testkit/src/protocol/{benchmark,request,snapshot,mod}.rs
                  justfile split recipe/order
                  docs/roadmap/tasks/split-app-composition.md ledger
Shared/serial:     ARCHITECTURE.md, E2E_ARCHITECTURE.md, ROADMAP.md and Phase 6
                  status updated by integration owner after review/gates
Forbidden:        crates/bbr-browser/**, crates/bbr-domain/**, crates/bbr-web/**,
                  crates/bbr-web-macos/**, crates/bbr-split/**,
                  crates/bbr-command/**, crates/bbr-persistence/**,
                  command/url-input modules and actions
Integration order: writer starts only after this clean serial anchor; independent
                  review/remediation; affected GUI/perf; canonical just e2e;
                  fast-forward integration; shared-doc record
```

`parallel/url-input` remains preserved and must not be inspected, rebased, reset, cleaned, or mutated. This packet does not reserve its command/input paths.

## Controller/API assessment

No new public controller mutation is required. Consume only the existing seams:

- `BrowserController::pane_layout()`
- `BrowserController::select_tab(index)` for tab-row-driven pane focus
- `BrowserController::navigate(WebAction)` for focused-pane commands
- existing `open_split` / `close_split` through testkit setup/teardown.

A later native-content focus slice may require a `WebViewId` focus event, but not a new controller focus owner.

## Real-app E2E

Add `just e2e-shell-split` while retaining `just e2e-shell-split-geometry`.

1. **Default production composition** — `SetSplit` setup, typed `SplitOpened`, unchanged two `WebViewId`s/sessions, controller order/focus, exact 0.5 position, 2 pt divider, adjacent positive pane rects tiling content, pane/divider/keyed-slot IDs, focused pane semantics, focused rect equals legacy `content_rect`.
2. **Pointer divider drag** — production pointer down/move/up, chrome focus without pane-focus change, changed/clamped normalized value, no session churn, exact 2 pt divider and adjacent projected rects, dragging state true only during drag.
3. **Focused-pane presentation/routing** — select other split member through production tab-row pointer path, preserve pane order/divider/session population, move exact focused semantic node/content rect, route Reload to newly focused `WebViewId`.
4. **Sidebar/window changes** — preserve normalized position through sidebar resize/collapse/reopen and window resize while recomputing absolute geometry and preserving identities/focus.
5. **Close/reopen reset** — after non-default drag, `CloseSplit` returns to one full pane with no orphan split IDs/rects; reopening starts at 0.5 without recreating sessions.
6. **Settled dissolution paths** — selecting an outside tab, closing a pane tab, or moving a pane tab dissolves split state without orphan slots/geometry; next opening resets UI state.

No sleeps or pixel comparison. Use typed events, snapshots, semantic IDs, exact deadlines, and real native WKWebViews.

## Performance

Add `split_divider_resize` and `split_pane_focus`; p95 ≤ 50 ms, max ≤ 100 ms.

- `split_divider_resize` measures the production divider mutation, alternating accepted non-default positions. Commit only after exact normalized position, projected geometry, 2 pt divider, unchanged focus/active view/two-session population, dragging=false, and split IDs/keyed slots render.
- `split_pane_focus` measures production tab-row activation through controller focus mutation to focused-pane semantic/frame presentation. Commit only after requested focused member, unchanged pane order/divider, exactly one focused pane node, `content_rect` matching that pane, unchanged sessions, and chrome focus.
- `SetSplit` / `CloseSplit` may prepare state but are not measured product invocation.
- Report-only host evidence is valid for absolute ceilings only; canonical baseline requires the qualified runner.

## File-size plan

Do not grow existing monoliths with split logic:

- split app state/mutations → `app/split.rs`
- split composition → `app/split_view.rs`
- split benchmark entry → `app/split_benchmarks.rs`
- split performance expectations → `performance/split.rs`
- split testkit request arms → `dispatch_split.rs`
- split benchmark request arms → `dispatch_benchmark_split.rs`
- split perf helpers → `tests/perf_common/split.rs`
- `render.rs`, dispatch parents, performance parent, and drive matrix remain wiring/delegation only.

If a touched protocol file crosses ~500 lines, split it by concern in this slice.

## Evidence

### Lower-level

- [ ] `bbr-split` projection is the only pane-geometry policy used by production composition.
- [ ] App state contains divider position/dragging only; pane/focus/open authority is not duplicated.
- [ ] New-split/reset, focus-preservation, resize-preservation, close/dissolution reset, clamping/no-op and invalid-geometry preservation are covered.
- [ ] Snapshot and semantic mappings are exact typed values.
- [ ] Fast-but-wrong performance guards reject wrong ratio/geometry/focus/session/drag state/IDs.

### Real-app

- [ ] Default split composition.
- [ ] Pointer divider drag and bounds.
- [ ] Focused pane + command routing.
- [ ] Sidebar/window resize preservation.
- [ ] Close/reopen reset.
- [ ] Outside-select/close/move dissolution.

### Performance

- [ ] `split_divider_resize` production path, 50/100 ms budget.
- [ ] `split_pane_focus` production path, 50/100 ms budget.

## Completion gates

```bash
cargo test -p bbr-split
cargo test -p bbr --features testkit split
cargo test -p bbr-testkit split
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-split-geometry
just e2e-shell-split
just e2e-shell-sidebar
just e2e-shell-tabs
just e2e-perf
just verify
just e2e
just roadmap
git diff --check
```

Do not run or claim `just e2e-perf-full` as Phase-6 closeout: production split creation, native-content focus synchronization, and keyboard divider behavior remain open.

## Iteration policy

Start with divider-only app state and pure projected rendering, then pointer resize, semantics/snapshot evidence, E2E, and performance. After each failure, classify it as portable projection, controller authority, GPUI layout/listener, native geometry, snapshot semantics, or presentation-commit ordering before editing. Never weaken the 2 pt divider, 0.5 default, focus authority, test deadlines, or performance budgets.

## Blocked stop condition

Preserve the worktree and stop with exact evidence if implementation requires storing pane/focus authority in `BbrApp`, changing domain/controller/WebEngine contracts, painting webpages into GPUI, inventing production split/close/keyboard behavior, accepting geometry rejected by `bbr-split`, using sleeps/pixels, or growing existing monoliths instead of splitting them. Report the minimum API/product decision needed.

## Delivery ledger

```text
Outcome:       pending
Serial anchor: pending
Writer:        pending
Review:        pending
Functional evidence: pending
Real-app E2E:  pending
Performance:   pending
Commits:       pending
Open blockers: production drag-to-glass creation, top/bottom-zone meaning,
               close invocation, keyboard divider behavior, native-content
               click -> controller pane-focus synchronization
```
