# Task packet — Component gallery: compositions

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 5](../phases/05-command-bar.md) ·
[Phase 6](../phases/06-split-view.md) · [Phase 8](../phases/08-daily-driver.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
Visual contract: [DESIGN.md](../../../DESIGN.md) §11–§13, §15 ·
Anchor: [UI library anchor (bbr-ui)](ui-library-anchor.md) ·
Predecessors: [Gallery MVP](component-gallery-mvp.md) ·
[Browser Components](component-gallery-browser-components.md) ·
Workflow: [development workflow](../../development-workflow.md) ·
Template: [task-packet template](../templates/task-packet.md)

Status: **G1 packet 4.** Bases from the settled browser-components slice.
This packet delivers the composition surfaces — `SidebarModel` plus
Callbacks, `CommandPortalModel` palette/overlay, and the Downloads/Find/
Error/Auth/Menu/Split groups — into `bbr-ui` and their gallery stories,
including the compact/wide, empty/dense, long-title/missing-icon,
cross-profile, reduced-motion, and contrast variants. The gallery owns plain
fixture builders only; it never makes anatomy or visual decisions.

## Outcome

`bbr-app` composes the full sidebar rail and every floating chrome surface
from `bbr-ui` compositions (`SidebarModel` + `Callbacks`, the command
palette/overlay from `CommandPortalModel`, and the Downloads/Find/Error/Auth/
Menu/Split surfaces), with no local duplicate anatomy. `bbr-gallery` renders
each composition as a real production story with fixture-built models and
recorder callbacks, covering the required variant families. No fake
`BrowserController`, WebKit, portal host, network, or persistence appears
anywhere in the gallery.

## Scope

- **`bbr-ui::sidebar` — `SidebarModel` + `Callbacks`:** the complete rail
  composition (brand, command/address slot, favorites grid, pinned rows,
  folders, regular tabs, status, toggle, divider, plus menu, footer +
  post-F2a space strip), per the anchor's composition row. `bbr-app`'s
  `sidebar_view.rs` becomes a thin assembler: it builds `SidebarModel` from
  the controller snapshot and passes `cx.listener` callbacks through the
  `Callbacks` struct. The geometry model (`SidebarGeometry`) moves to
  `bbr-ui::sidebar::geometry` unchanged (pure math; the app keeps the input
  wiring).
- **`bbr-ui::surfaces::command` composition:** the centered Cmd-T palette
  and the field-anchored Cmd-L/click overlay driven by one `CommandPortalModel`
  (surface, theme, chrome_focused, text, state, results, selected) — the
  existing render-only composition, extracted whole.
- **Group stories:** Downloads panel, Find bar, Error/Crash panes, Auth
  sheet, Context menu, Split preview — as compositions in context (not
  isolated rows), with the variant families below. After their independently
  accepted production packets integrate, append composition stories for
  Tab/Favorite menus with submenus, Folder menu, enriched Page-background
  menu, the safe-titlebar navigation cluster, and the conditional Sidebar
  media card. Existing StoryIds remain stable; these are append-only stories
  of the same production components, never implementation authorization.
- **`bbr-ui::split`:** split chrome frames (pane frame hairline, focused
  pane outline, shared-edge divider, split-zone frames per DESIGN §11.7)
  rendered by `bbr-app`'s split branch around the existing
  `NativeWebViewSlot`s; the slot itself stays app-owned.
- **Variant families (all compositions):** compact/wide (window geometry:
  narrow clamp → content floor, wide → full rail), empty/dense (empty space
  vs 500-tab density, empty vs populated downloads/history), long-title/
  missing-icon (truncation, monogram, icon fallback), cross-profile
  (profile chips + command-profile-hint on cross-profile activation; space
  strip identity), reduced-motion (transient indication only — opacity at
  `dur:fine`; state changes 0 ms), contrast (standard/increased: text steps,
  hairlines, focus ring 3 pt double ring, deeper washes).
- **Gallery fixture builders** for every composition; the gallery holds
  fixture construction only — every visual decision stays in `bbr-ui`/DESIGN.

## Non-goals

- **No new component anatomy.** Compositions assemble existing `bbr-ui`
  pieces; the gallery never restyles or invents layout.
- **No authority inside `bbr-ui` or the gallery.** No `BrowserController`,
  fake engine, portal request, session, command ranking, or persistence —
  the compositions receive fully-built models; activation/ranking/lifecycle
  stay in `bbr-app`.
- **No native-boundary evidence.** Split frames and floating surfaces render
  in the gallery without WebKit/Portal; AppKit z-order and hit-testing stay
  the product's portal suites.
- **No product behavior change.** Ids, §15 snapshot fields, focus domains,
  and keyboard semantics are byte-identical after each composition cutover.
- **No new E2E recipe for the product** (existing suites re-run); only the
  gallery's `just e2e-gallery` grows.

## Parallel-lane contract

```text
Base commit:          the settled browser-components slice (bbr-ui
                      browser-component modules + gallery story matrix).
Depends on:           packets 1–3; F2a (footer/space strip) and F2b
                      (downloads) integrated onto main; the F2c app-persistence
                      bootstrap does not affect this packet (no persistence).
Provides/API:         bbr-ui::sidebar (SidebarModel + Callbacks + geometry),
                      bbr-ui::surfaces::command composition, bbr-ui::split,
                      the group story matrices and fixture builders; no wire
                      or behavior API change.
Exclusive paths:      crates/bbr-ui/** (composition modules), crates/
                      bbr-gallery/** (composition stories, fixtures),
                      bbr-app's sidebar_view.rs / render.rs / u2_surface.rs
                      (assembler refactor only, per cutover).
Shared integration points: root Cargo.toml/Cargo.lock, justfile (no new
                      recipe expected), E2E registries, bbr-app composition,
                      UI track and ROADMAP roll-ups, ARCHITECTURE.md facts.
Integration order:    packet 3 → this packet → post-G1 serial integration.
                      Parallel-safe against F3 / U3 / non-UI lanes as long
                      as no lane touches the migrating render modules.
```

## Constraints

- All anchor constraints apply (dependencies, `#![forbid(unsafe_code)]`,
  forbidden-dependency gate, zero local duplicates, stable semantics,
  module size, no weakened gates).
- **`SidebarModel` plus `Callbacks`** is the single input contract of the
  rail: the model is plain data (theme, widths, brand, command/address,
  favorites, folders, tab rows, spaces, status, toggle, divider, footer,
  space strip); the `Callbacks` struct is one set of handler closures
  `bbr-app` fills. The gallery fills recorder callbacks.
- **`CommandPortalModel`** is the existing presentation-oriented model
  (`crates/bbr-app/src/portal/command.rs`) moved to `bbr-ui` unchanged in
  shape; `bbr-app`'s `portal/probe.rs` builds it from the app's ranking
  state.
- **Derive similarly narrow models for other surfaces from source:** the
  Downloads/Find/Error/Auth/Menu/Split models are the existing
  app-side DTO shapes (from `downloads_render.rs`, `app/find.rs`,
  `error_surface.rs`, `auth/records.rs`, `context_menu.rs`, `render.rs`
  split branch) moved to `bbr-ui`; no model is widened to smuggle authority
  out of `bbr-app`.
- **Gallery owns plain fixture builders only, never anatomy/visual
  decisions:** a story's only inputs are its model fixture and its
  State/Variant selector; the visual result is whatever the production
  `bbr-ui` composition renders.
- **Variant families are mandatory:** every composition ships at least one
  story per applicable family (compact/wide, empty/dense, long-title/
  missing-icon, cross-profile, reduced-motion, contrast) — these are the
  §16/§11 "every new surface ships … variants" rule made visible.
- **No fake BrowserController/WebKit/Portal/network/persistence** in any
  story, fixture, or test.
- **Gallery tests supplement, never replace, affected browser E2E/perf.**
- **GUI recipes use the host lock; every gate list contains
  `cargo fmt --all -- --check` and states `CARGO_BUILD_JOBS=4`.**
- **Module size budget** holds for the composition modules and the gallery
  fixture files.

## Story matrix (compositions)

| Composition | Valid variant families (at least one story each) | Rejected (why) |
|---|---|---|
| Sidebar (`SidebarModel` + `Callbacks`) | compact (narrow-window clamp → 64 pt content floor) / wide (full rail 280–400 pt); empty (no tabs) / dense (500 tabs, S density, windowed render); long-title/missing-icon (truncation + monogram); cross-profile (profile chips, command-profile-hint, space strip identity); reduced-motion; contrast standard/increased | a sidebar with zero native-content floor (content stays strictly positive, §5/§11.4); a collapsed rail showing chrome content; two space strips |
| Command palette / overlay (`CommandPortalModel`) | centered Cmd-T palette (New Tab mode, mode chip, results, loading/empty/error state rows); field-anchored Cmd-L/click overlay (Location mode, full-URL-preselected field, §11.5 result ids); reduced-motion (opacity fade `dur:fine` only); contrast | palette + overlay simultaneously (one surface, mutual exclusion); a result row without its stable key id |
| Downloads group | Active (Waiting unknown/known, Running) / History (Succeeded + Reveal eligible/disabled, Failed + Retry, Cancelled + Restart/Delete post-F2b); empty hint; 6-row scroll window; reduced-motion; contrast | a non-terminal group order (Active above History, §11.12); optimistic terminal states (first terminal event wins) |
| Find group | open/focused; query empty/typed; n of m; 0 of 0; stale; reduced-motion; contrast | ordinal without count; step enabled at zero matches |
| Error/Crash group | each TLS/Navigation/Offline category with calm copy + contract-legal action; origin shown/omitted; crash panel; empty pane; reduced-motion; contrast | a non-category-legal recovery action (validated against the category-legal matrix — TLS/Navigation/Offline → Retry only, no bypass, per `crates/bbr-app/src/app/error_surface.rs` `ErrorSurfaceSpec`/`IncidentCategory` and the [error-surface-contract](error-surface-contract.md) packet); raw platform codes |
| Auth group | realm present/absent; origin-server/proxy; username/password focus; masked password; Cancel/Log In; reduced-motion; contrast | unmasked render; outside-press dismissal; serializable credentials |
| Context menu group | existing page menu enabled/disabled items, selection, Back/Forward enablement, reduced-motion, contrast; post-packet append-only contexts: inactive Tab/Favorite source without selection, nested Folder/Space/Share/Close submenus, Folder empty/populated/destructive, Page-background versus WebKit-native ownership metadata, edge flipping and overflow | a zero-item menu; activation of a disabled row; source identity changing while a submenu is open; a Gallery copy of a WebKit element menu |
| Titlebar navigation group (after production packet) | traffic-light-safe compact/wide layouts; sidebar collapsed/expanded; Back/Forward disabled combinations; Reload/loading; empty native drag region; standard/increased contrast | hardcoded traffic-light coordinates; duplicated status-row toggle; a titlebar with no draggable empty region |
| Sidebar + media group (after media contract/UI packets) | no-session rail with zero reserved gap; playing/paused card; seekable known duration; unknown/live duration; capability subsets; multiple-session count/switch; dismissed identity; cross-profile source; compact/wide; reduced-motion; contrast | a card without actual session identity; a vertical card stack; unsupported decorative controls; fake total/scrubber; any story claiming real media command evidence |
| Split preview (`bbr-ui::split`) | single pane (focused frame hairline); two panes (shared-edge divider, focused pane inset outline); split-zone frames (top/right/bottom/left) with accent frame + fill + corner glyph; reduced-motion; contrast | a zone frame without the accent language (zone fill is accent-only); a split with zero live panes (the `pane_layout()` split branch requires both live sessions — no false single-pane degradation — per [split-shared-contract](split-shared-contract.md#unit-and-contract); visuals per DESIGN §11.7) |

## Evidence

### Unit and contract

- [ ] `SidebarModel`/`Callbacks` tests: model round-trip; every callback
      slot fires through recorder callbacks; the assembler in `bbr-app`
      produces a model whose every field matches the controller snapshot it
      derives from.
- [ ] `CommandPortalModel` and group-model tests move with their modules
      (ids, states, callbacks) unchanged.
- [ ] Split geometry/chrome tests: `bbr-ui::split` frame/divider/zone
      rendering matches DESIGN §11.7 tokens; `bbr-split` geometry is
      untouched.
- [ ] Matrix test in the gallery: every composition story builds and
      renders without panic across all six variant families; rejected
      combinations refuse to build.
- [ ] Post-cutover `rg` guard: no duplicate composition symbol and no
      `pub use bbr_ui` alias in `bbr-app`.

### Real-app E2E

- [ ] Each composition cutover re-runs its affected product suite unchanged:
      `just e2e-shell-sidebar`, `just e2e-shell-command`, `just e2e-shell-`
      `downloads`, `just e2e-shell-find`, `just e2e-shell-errors`,
      `just e2e-auth`, `just e2e-shell-context-menu`,
      `just e2e-shell-split-geometry` (and the F2a-integrated footer/spaces
      suites) — ids, §15 fields, focus domains, and typed semantic trees
      unchanged.
- [ ] `just e2e-gallery` extended: composition stories render through the
      production compositions with stable StoryIds appended (packets 2–3
      ids unrenumbered) and the variant families asserted.

### Performance

- [ ] `just e2e-perf` passes unchanged after each composition migration
      (no budget, sample-count, or semantic-commit change).
- [ ] Gallery perf remains sanity-only (packet 2), never a product budget.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo check --workspace --all-targets
CARGO_BUILD_JOBS=4 cargo test --workspace
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p bbr-ui --all-targets -- -D warnings
cargo clippy -p bbr-gallery --all-targets -- -D warnings
just ui-deps
just e2e-shell-<affected-suite>   # per cutover group, unchanged suites
just e2e-perf
just e2e-gallery
just verify                       # after the stable slice
python3 scripts/check_roadmap.py
```

Every cargo invocation states `CARGO_BUILD_JOBS=4`. GUI recipes run
sequentially under `python3 scripts/with_gui_lock.py --`.

## Delivery sequence

1. Confirm packet-3 base is settled and the browser-component modules are
   green.
2. One exclusive writer performs the composition cutovers in Scope order
   (sidebar assembler first, then command composition, then the groups, then
   split chrome); each is a complete extract → migrate → delete → re-run
   cycle.
3. Add the composition stories + fixture builders and the mandatory variant
   families in the same groups.
4. One independent read-only reviewer maps every story back to DESIGN.md
   §11–§13 and the anchor table, checks the rejected-combination logic and
   the "fixture-only" rule, and verifies no authority entered `bbr-ui` or
   the gallery.
5. One remediation writer only for concrete findings, then a focused recheck.

## Iteration policy

Start with the sidebar assembler because every group story composes into it.
For each failure, classify it as assembler/model mismatch, callback wiring,
variant-family gap, matrix validity, or native-boundary confusion; change the
smallest central cause and re-run the focused suite. Never move authority
into `bbr-ui`/the gallery to make a story work, and never drop a mandatory
variant family to shorten the slice.

## Blocked stop condition

If a composition cannot be extracted without changing ids, semantics, focus
domains, or perf evidence, stop with the composition, the conflicting
symbol, the failing suite/budget, the preserved worktree state, and the
smallest product/architecture decision required. If F2a/F2b has not
integrated, the sidebar-footer/downloads composition rows wait; do not
extract from an unintegrated state.

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

Use measured timing only when runtime/command timestamps support it;
otherwise label values as estimates.
