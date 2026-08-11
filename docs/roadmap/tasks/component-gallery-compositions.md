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

Program position: **G3**, activated from the settled and integrated G2
browser-components state recorded in [Current delivery](../../delivery/current.md).
G2 already delivered the Downloads, Find, Error/Crash, Auth, and Context Menu
surfaces and their component stories. G3 Sidebar is integrated; the Command
composition candidate is code- and gate-complete pending independent review and
integration. Split chrome composition remains serially next.
This packet moves `SidebarModel` plus Callbacks, a plain command-presentation
DTO and renderer, and split chrome into `bbr-ui` and appends their production
Gallery stories after the canonical 44-story G2 catalog. The command cut does
not move the app-owned `CommandSurface`, ranking state machine, activation
payloads, portal host, placement, focus, performance, testkit, or DTO-building
authority. Required variants remain compact/wide, empty/dense, long-title/
missing-icon, cross-profile, reduced-motion, and contrast where the production
composition actually supports them. The gallery owns plain fixture builders
only; it never makes anatomy or visual decisions.

## Outcome

`bbr-app` composes the full sidebar rail and every floating chrome surface
from `bbr-ui` compositions (`SidebarModel` + `Callbacks`, the command
palette/overlay from a plain presentation DTO, and the Downloads/Find/Error/
Auth/Menu/Split surfaces), with no local duplicate anatomy. `bbr-gallery` renders
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
  and field-anchored Cmd-L/click overlay are rendered from one plain UI model:
  a UI-local presentation discriminant (`None`/centered palette/anchored
  overlay), theme, chrome-focus flag, text, UI-local result-state enum,
  selected index, and plain rows containing only the rendered key/group/title/
  subtitle/destructive fields. Stable presentation ids and the render-only
  card anatomy move with this module. `bbr-app` maps its app-owned
  `CommandSurface`, `CommandResultState`, and `CommandResultRow` to the plain
  model for each portal render; `Activation`, collection/ranking internals,
  placement, dismissal, portal identity, and focus never enter `bbr-ui`.
- **Group stories:** Downloads, Find, Error/Crash, Auth, and Context Menu
  component stories are already integrated by G2 and are not reimplemented.
  G3 appends only their composition-in-context coverage where the Sidebar,
  Command, or Split cut needs it, beginning with the Split preview. After their
  independently accepted production packets integrate, append composition
  stories for Tab/Favorite menus with submenus, Folder menu, enriched
  Page-background menu, the safe-titlebar navigation cluster, and the
  conditional Sidebar media card. B2/B3 and the Library stories at StoryIds
  23–30 are already integrated; Saved Page command/menu compositions are now
  eligible but remain append-only and may not reserve an id before their
  production composition exists. Existing StoryIds 0–44 remain stable; the
  Command candidate appends StoryId 45 without renumbering.
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
Base condition:       committed authority after G2 closeout; 44 stories,
                      StoryIds 0–43, and all G2 surface exports integrated.
Depends on:           G1a + G1b + G2 + F2a + F2b integrated in the control
                      ancestry; F2c does not affect this presentation packet.
Provides/API:         bbr-ui::sidebar (SidebarModel + Callbacks + geometry),
                      bbr-ui::surfaces::command plain model/ids/geometry/
                      renderer, bbr-ui::split, and their composition story
                      matrices/fixture builders; no wire or behavior API
                      change.
Exclusive paths:      focused crates/bbr-ui sidebar/command/split modules;
                      focused crates/bbr-gallery composition modules; crates/
                      bbr-app/src/sidebar.rs; app/sidebar_view.rs and its
                      command_bar module; app/render.rs; app/u2_surface.rs;
                      portal/command.rs. For the Command cut, portal/command.rs
                      is deleted after the production renderer moves;
                      app/palette.rs remains the app-owned model adapter and
                      portal/probe.rs remains the app-owned portal builder,
                      placement/dismissal owner and shared integration point.
Shared integration points: root Cargo.toml/Cargo.lock, justfile (no new
                      recipe expected), E2E registries, bbr-app composition,
                      Gallery registry, UI track/ROADMAP roll-ups, and
                      ARCHITECTURE.md implemented facts.
Integration order:    committed G2 closeout → G3 Sidebar → Command → Split.
                      Z-dependent menu stories remain after their production
                      packets and serialize on bbr-ui menu/Gallery registries.
```

## Constraints

- All anchor constraints apply (exact direct dependencies, the approved
  first-party `bbr-domain → bbr-web` identity closure with no direct
  `bbr-web` import, `#![forbid(unsafe_code)]`, dependency-boundary gate, zero
  local duplicates, stable semantics, module size, and no weakened gates).
- **`SidebarModel` plus `Callbacks`** is the single input contract of the
  rail: the model is plain data (theme, widths, brand, command/address,
  favorites, folders, tab rows, spaces, status, toggle, divider, footer,
  space strip); the `Callbacks` struct is one set of handler closures
  `bbr-app` fills. The gallery fills recorder callbacks.
- **Command authority boundary:** the existing app-local
  `CommandPortalModel` cannot move unchanged because it contains app-owned
  `CommandSurface`/`CommandResultState` and rows carrying
  `bbr_command::rank::Activation`. `bbr-ui` instead owns the smallest plain
  presentation model described above plus the stable ids and render-only
  anatomy. `bbr-app` retains the command state machine, ranking, activation,
  focus, native Portal host/placement/dismissal, performance/testkit semantics,
  and the one mapping from app rows to UI rows. The app-local
  `portal/command.rs` renderer is removed rather than retained as an alias or
  second authority.
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
| Command palette / overlay (plain `bbr-ui::surfaces::command` model) | inactive/closed; centered Cmd-T palette (New Tab mode chip, loading/empty/results/activation-error rows); field-anchored Cmd-L/click overlay (Location mode, full-URL-preselected text, §11.5 result ids); selected and destructive rows; long-title truncation; reduced-motion (opacity fade `dur:fine` only); standard/increased contrast | palette + overlay simultaneously (one presentation discriminant, mutual exclusion); a result row without its stable key id. Missing-icon and cross-profile-hint families are inapplicable: production uses a group glyph and the current portal model has no profile-hint field |
| Downloads group | Active (Waiting unknown/known, Running) / History (Succeeded + Reveal eligible/disabled, Failed + Retry, Cancelled + Restart/Delete post-F2b); empty hint; 6-row scroll window; reduced-motion; contrast | a non-terminal group order (Active above History, §11.12); optimistic terminal states (first terminal event wins) |
| Find group | open/focused; query empty/typed; n of m; 0 of 0; stale; reduced-motion; contrast | ordinal without count; step enabled at zero matches |
| Error/Crash group | each TLS/Navigation/Offline category with calm copy + contract-legal action; origin shown/omitted; crash panel; empty pane; reduced-motion; contrast | a non-category-legal recovery action (validated against the category-legal matrix — TLS/Navigation/Offline → Retry only, no bypass, per `crates/bbr-app/src/app/error_surface.rs` `ErrorSurfaceSpec`/`IncidentCategory` and the [error-surface-contract](error-surface-contract.md) packet); raw platform codes |
| Auth group | realm present/absent; origin-server/proxy; username/password focus; masked password; Cancel/Log In; reduced-motion; contrast | unmasked render; outside-press dismissal; serializable credentials |
| Context menu group | existing page menu enabled/disabled items, selection, Back/Forward enablement, reduced-motion, contrast; post-packet append-only contexts: inactive Tab/Favorite source without selection, nested Folder/Space/Share/Close submenus, Folder empty/populated/destructive, Page-background versus WebKit-native ownership metadata, edge flipping and overflow | a zero-item menu; activation of a disabled row; source identity changing while a submenu is open; a Gallery copy of a WebKit element menu |
| Titlebar navigation group (after production packet) | traffic-light-safe compact/wide layouts; sidebar collapsed/expanded; Back/Forward disabled combinations; Reload/loading; empty native drag region; standard/increased contrast | hardcoded traffic-light coordinates; duplicated status-row toggle; a titlebar with no draggable empty region |
| Sidebar + media group (after media contract/UI packets) | no-session rail with zero reserved gap; playing/paused card; seekable known duration; unknown/live duration; capability subsets; multiple-session count/switch; dismissed identity; cross-profile source; compact/wide; reduced-motion; contrast | a card without actual session identity; a vertical card stack; unsupported decorative controls; fake total/scrubber; any story claiming real media command evidence |
| Split preview (`bbr-ui::split`) | single pane (focused frame hairline); two panes (shared-edge divider, focused pane inset outline); split-zone frames (top/right/bottom/left) with accent frame + fill + corner glyph; reduced-motion; contrast | a zone frame without the accent language (zone fill is accent-only); a split with zero live panes (the `pane_layout()` split branch requires both live sessions — no false single-pane degradation — per [split-shared-contract](split-shared-contract.md#unit-and-contract); visuals per DESIGN §11.7) |

B2 and B3 are integrated, including the Library stories at StoryIds 23–30.
Saved Page command/menu compositions are therefore dependency-eligible but
remain strictly append-only: this packet owns their eventual composition
matrix and renderability checks, but no StoryId is reserved before the real
production composition exists.

## Evidence

### Unit and contract

- [x] **Sidebar** `SidebarModel`/`Callbacks` tests: model round-trip and
      every callback slot firing through recorder callbacks are covered by
      the gallery matrix/recorder suites (`sidebar_matrix.rs`
      `every_valid_family_builds_a_model` / `every_rejected_family_refuses_construction`
      and `sidebar_recorder.rs` `observe_records_the_expected_field_only`);
      the assembler's controller-snapshot derivation is proven by the
      real-app composition E2E (the rail renders the exact regions/ids/
      semantics the snapshot derives). Split model tests remain open with
      the Split cutover.
- [x] **Command** presentation model/render tests move the stable ids,
      typed UI states, row rendering, mutual-exclusion validation, and
      rejected missing-key construction into `bbr-ui`
      (`bbr-ui::surfaces::command` model/ids/geometry/render/tests:
      presentation-discriminant round-trip and invalid-raw rejection, the
      missing/empty-key rejection through `CommandModel::validate`, the
      exact §11.4/§11.5/§15 state and row ids, and the render
      construction); the app-owned mapping in `app/palette.rs` strips every
      activation payload and preserves all command authority. No activation
      callbacks are invented because production keyboard/pointer activation
      remains app/Portal-owned.
- [ ] Split geometry/chrome tests: `bbr-ui::split` frame/divider/zone
      rendering matches DESIGN §11.7 tokens; `bbr-split` geometry is
      untouched.
- [x] Matrix test in the gallery: every composition story builds and
      renders without panic across its applicable variant families; rejected
      combinations refuse to build. *(Sidebar evidence is integrated and the
      Command candidate's matrix/recorder evidence is green — `command_matrix.rs`
      `every_valid_family_builds_a_model_through_the_production_validation` /
      `every_rejected_family_refuses_construction` plus the two Command
      scenarios; Split remains open.)*
- [x] Post-cutover `rg` guard: no duplicate composition symbol and no
      `pub use bbr_ui` alias in `bbr-app`. *(Sidebar and Command cutovers
      are clean on the Command candidate — the gallery
      `dependency_contract.rs` guard also asserts `portal/command.rs` is
      deleted and the probe invokes the production
      `bbr_ui::surfaces::command::command_card`; Split re-runs this guard
      with its cutover.)*

### Real-app E2E

- [x] **Sidebar cutover** re-runs its affected product suite unchanged and
      adds composition-level evidence: `just e2e-shell-sidebar` now also
      proves the full §11.4 rail through the one `bbr_ui::sidebar`
      composition (stable §15 ids, typed semantic tree in §11.4 region
      order, one rail authority with no legacy `[Spaces]` surface), the
      app-owned command/downloads children compose without breaking
      focus/Escape/collapse or child visibility/exclusion, the production
      pointer seams (tab row, favorite tile, folder row, space chip, plus
      menu) still route through the composition, the native WKWebView
      geometry stays exactly rail-width/content-floor correct in the
      existing split layout with collapse/expand restore and no
      session/WebView identity churn, and the bounded dense 500-tab/
      50-session window renders through the composition. Ids, §15 fields,
      focus domains, and typed semantic trees stay unchanged.
- [x] **Command cutover** re-runs its affected product suite unchanged:
      `just e2e-shell-command` passes 25/25 on the extracted tree (the
      floating card renderer and its stable ids moved to
      `bbr_ui::surfaces::command`; ids, §15 fields, focus domains, typed
      semantic trees, placement, and Portal identity stay byte-identical).
      Downloads/Find/Error/Auth/Context Menu functional evidence belongs to
      completed G2 and is not rerun merely to restate that extraction.
- [ ] **Split cutover** re-runs its affected product suites unchanged:
      `just e2e-shell-split-geometry` and the affected footer/spaces suites.
      Ids, §15 fields, focus domains, and typed semantic trees stay
      unchanged.
- [x] `just e2e-gallery` extended (Sidebar + Command): composition stories
      render through the production compositions with stable StoryIds
      appended (G1b/G2 ids unrenumbered; Sidebar 44, Command 45, 46-story
      catalog) and the variant families asserted. *(Sidebar is integrated;
      the Command candidate's matrix/recorder evidence is green and its scenarios sweep
      every matrix family with stable ids and observe the production
      presentation model.)*
- [ ] `just e2e-gallery` extended (Split): the Split preview story appends
      after Command with its variant families asserted. *(Split remains
      open with its cutover.)*

### Performance

- [ ] Performance code, budgets, sample counts, baselines, and semantic-commit
      guards remain byte-unchanged because G3 is a presentation extraction.
      Under Patrick's active override, no performance recipe is executed and
      no performance pass is claimed; execution remains explicit debt.
- [ ] Gallery perf remains sanity-only (G1b), never a product budget.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo check --workspace --all-targets
CARGO_BUILD_JOBS=4 cargo test --workspace --lib --bins
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-ui --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-gallery --all-targets -- -D warnings
just ui-deps
just script-tests
just e2e-shell-<affected-suite>   # per cutover group, unchanged suites
just e2e-gallery
python3 scripts/check_roadmap.py
git diff --check
# Performance and canonical `just e2e` are intentionally not executed while
# Patrick's delivery override remains active; preserve all code and debt.
```

Every cargo invocation states `CARGO_BUILD_JOBS=4`. GUI recipes run
sequentially under `python3 scripts/with_gui_lock.py --`.

## Delivery sequence

1. Confirm committed G2 closeout, the 44-story registry, and the
   browser-component modules are green.
2. One exclusive writer performs three bounded cutovers in order: Sidebar
   assembler first, then Command composition, then Split chrome. Sidebar is
   integrated; Command has completed its extract → migrate → delete → re-run
   cycle and awaits independent candidate review/integration. Split chrome
   composition remains serially next. Completed G2 surfaces are consumed, not
   re-extracted.
3. Add each composition story + fixture builder and mandatory variant families
   in the same bounded cut.
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
domains, or performance code/evidence contracts, stop with the composition,
the conflicting symbol, the failing suite/budget, the preserved worktree
state, and the smallest product/architecture decision required. F2a/F2b and
G2 are integrated; never reopen or copy their completed surfaces to clear a
G3 blocker.

## Delivery ledger

```text
Outcome: G3 Command candidate complete after integrated Sidebar. The centered
  Cmd-T palette and anchored Cmd-L/click overlay render through the one plain
  bbr_ui::surfaces::command model/ids/geometry/command_card authority;
  app/palette.rs performs the only app-state → presentation mapping and strips
  Activation, while bbr-app retains command state, ranking, activation, focus,
  Portal identity/placement/dismissal, testkit, semantics, and performance.
  The app-local portal/command.rs renderer is deleted. Gallery appends the
  production renderer at StoryId 45 for a 46-story catalog with valid/rejected
  matrix and stable-id observation; Split remains serially next.
Commits: none in the lane; Orchestrator commits only after independent
  whole-candidate review and any focused remediation/recheck. The whole review
  accepted the product candidate and found one clean-build gate defect: two
  unused test-only state-id imports. The bounded remediation removes only those
  imports. Focused independent recheck reproduced the two-path delta and both
  fresh-target gates and returned direct `G3_COMMAND_CLIPPY_RECHECK_PASS` with
  zero findings; the exact candidate is freeze-ready.
Non-performance evidence: fmt and workspace all-target check green; bbr-ui
  122/122, Gallery 142/142 with 42 ignored, app binary 237/237 with 24
  performance tests filtered; warning-denied UI/Gallery Clippy, ui-deps,
  script-tests, roadmap, and diff-check green. Earlier app/workspace Clippy
  claims are withdrawn because the app log compiled only `--bin bbr` and the
  workspace log was not reproducible from a clean target. Fresh-target
  all-target evidence after the two-import remediation is green and is the only
  accepted replacement: app log
  `/tmp/g3-command-remediation-app-clippy.log`, SHA-256
  `51888531213555b34765a0ec2391ed554294602953b8dd4c4f7f8cd9e07331cb`;
  workspace log `/tmp/g3-command-remediation-workspace-clippy.log`, SHA-256
  `41cdb524867508e52253be1ad1c0e03f0086cace97dcb783ea953118c0f2257b`.
Real-app/Gallery E2E: Gallery 42/42, unchanged Command 25/25, unchanged Portal
  7/7 on the final runtime tree. Sidebar was not rerun because the finalizer
  changed no Sidebar/address composition beyond the already tested Command
  mapping/Portal invocation seam.
Performance evidence: none executed; performance code, budgets, baselines,
  sample guards, manifests, lockfile, and Justfile remain byte-unchanged;
  execution is explicit unpassed debt under Patrick's override.
Roadmap/documentation updates: ARCHITECTURE.md, E2E_ARCHITECTURE.md, this
  packet, ROADMAP current-focus wording, Phase 4/5/6 facts, and UI/testing
  track rollups. Phase 8 required no Command-composition acceptance change.
Open blockers and non-goals: Orchestrator freeze and serial integration remain;
  Split, Z, startup, packaging, accessibility, canonical E2E, and all
  performance execution remain outside this candidate.
Worktree state: parallel/g3-command-composition at activation HEAD d998f058;
  real index unchanged/staged 0; exact final fingerprints are recorded in
  Current delivery before review; no lane process/Git/GUI-lock holder.

Planning/read-only: estimate
Implementation: estimate
Test execution: measured (GUI-locked runs)
Review: whole-candidate findings closed by direct focused zero-finding Pass
Remediation: measured two-import clean-build Clippy correction, independently rechecked
Waiting/retries: none
Total elapsed: estimate
Human intervention: none
```

Use measured timing only when runtime/command timestamps support it;
otherwise label values as estimates.
