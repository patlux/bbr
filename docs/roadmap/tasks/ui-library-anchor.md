# Task packet — UI library anchor (bbr-ui) and the BBR Gallery program

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 5](../phases/05-command-bar.md) ·
[Phase 8](../phases/08-daily-driver.md) · [Phase 9](../phases/09-quality-release.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Accessibility track](../tracks/accessibility.md) ·
[Testing track](../tracks/testing.md) ·
[Performance track](../tracks/performance-release.md) ·
Product: [PRODUCT.md](../../../PRODUCT.md) · Visual contract:
[DESIGN.md](../../../DESIGN.md) · Implementation facts:
[ARCHITECTURE.md](../../../ARCHITECTURE.md) ·
[E2E_ARCHITECTURE.md](../../../E2E_ARCHITECTURE.md) ·
Workflow: [development workflow](../../development-workflow.md) ·
[parallel delivery](../../parallel-delivery.md) ·
Template: [task-packet template](../templates/task-packet.md)

Status: **G1 planning anchor (packet 1).** This file is the planning and
implementation contract for packet 1 of the G1 UI-library/Berry-Gallery
program: it defines the exact `bbr-ui` crate/API/dependency boundary and a
source-grounded extraction table for every surface the gallery will later
story. The file itself implements no code; the bounded packet-1
implementation slice executes this packet's Scope — the `bbr-ui` foundations
and primitives cutover (crate, DTOs, ids, forbidden-dependency gate).
Packets 2–4 then implement the Gallery MVP, the browser components, and the
compositions, each basing from the settled previous slice
([Gallery MVP](component-gallery-mvp.md), [Browser Components](component-gallery-browser-components.md),
[Compositions](component-gallery-compositions.md)).

## Outcome

The repository has one reusable, portable presentation crate `crates/bbr-ui/**`
that owns every BBR visual surface — Chassis foundations, primitives, browser
components, and compositions — consumed identically by `bbr-app` and the
dev-only `bbr-gallery` application. No copied or rebuilt components exist
anywhere: `bbr-app` and `bbr-gallery` import the same production symbols and
`Theme` tokens from `bbr-ui`, and `bbr-app` no longer defines any of them.
`bbr-ui` is dependency-clean by an auditable gate: it compiles with
`#![forbid(unsafe_code)]` and its dependency tree provably excludes
`bbr-browser`, `bbr-app`, `bbr-web-macos`, WebKit/AppKit/objc2, persistence,
filesystem, socket/network, testkit gateway, fixture server, and the native
Portal host. The cutover is serial and leaves zero local duplicates.

## Scope

- **Create the `bbr-ui` crate** (`crates/bbr-ui/**`) with the allowed
  dependency set and the forbidden-dependency audit gate (see Constraints).
- **Extract, migrate, delete** — one coherent serial cutover per surface,
  following the extraction table in §Extraction table. Every row's
  "current symbol/file" is moved into the listed `bbr-ui` module, every
  `bbr-app` use site is migrated to the new path in the same commit group,
  the old definition is **deleted** (no `pub use` compatibility alias, no
  re-export shim, no duplicate), and the affected real-app suite re-runs.
- **Move the Chassis foundations**: `Theme`/`ThemeMode`/`ContrastMode`/
  `TypeRole`/`TypeScale`/`Spacing`/`Radius`/`Motion`/`Elevation`/`Density`
  from `crates/bbr-app/src/theme.rs` + `theme/density.rs` and the `Palette`
  color system from `crates/bbr-app/src/palette.rs` into `bbr-ui`, with the
  WCAG contrast gate moving with them.
- **Move the §15 state vocabulary** (`TabVisualState`, `SpaceVisualState`,
  `DownloadVisual`, and the mode fields) as `bbr-ui` DTOs so gallery stories
  and product snapshots share one vocabulary.
- **Define plain presentation models + callbacks** as the bbr-ui input
  contract: components take `(model: &Model, theme: &Theme, callbacks…)`
  where the model is a `bbr-ui` DTO (or the `bbr-ui`-owned derivation) and
  callbacks are the same GPUI handler closures `bbr-app` already passes today
  (see §Model/callback contract). No `BrowserController`, no snapshot
  authority inside `bbr-ui`.
- **Write the honest primitive inventory** (§Primitive inventory): extract
  only repeated same-intent anatomy; do not claim primitives that do not
  exist in the source today.
- **Register the parked decisions** (§Parked decisions) with
  question/options/recommendation/impact.
- **Record the serial-cutover contract** (§Serial cutover) with zero local
  duplicates, per-surface ownership, and the "one writer" re-check rule.

## Non-goals

- **No gallery implementation here.** The gallery app, tree, canvas, controls,
  `just gallery`, and `just e2e-gallery` are [packet 2](component-gallery-mvp.md).
- **No story matrices for browser components or compositions here.** Those are
  [packet 3](component-gallery-browser-components.md) and
  [packet 4](component-gallery-compositions.md).
- **No behavior, interaction, focus, or E2E-semantics change.** `bbr-ui`
  extraction is presentation-only; ids, §10 state vocabulary, keyboard
  semantics, focus domains, and §15 snapshot fields are byte-identical after
  migration.
- **No new UI feature.** Anything not already rendering in
  `crates/bbr-app/src/{theme,palette,components,app,portal,shell}/*` today is
  out of scope; the extraction table is an honest map of what exists.
- **No native-boundary work.** `bbr-web-macos`, `NativeWebViewSlot`, the
  Portal host geometry/probe (`crates/bbr-app/src/portal/{geometry,native,
  probe,state}.rs`), and `bbr-app`'s snapshot adapters stay app-owned. The
  native slot's *chrome frames* (split divider, pane frame per DESIGN §11.7)
  move to `bbr-ui` as plain presentation; the slot itself never does.
- **No persistence, settings, or history work.** `bbr-app`'s controller,
  `bbr-persistence` store/bootstrap (F2c-owned), and domain authority stay
  where they are.
- **No `justfile`/root-workspace edits by the lane.** Those are serial
  orchestrator points; the lane proposes the recipe, the orchestrator lands
  it.
- **No macOS-only gating of `bbr-ui` itself.** The crate is portable GPUI
  presentation and must compile wherever the workspace evaluates (see
  §Parked decisions for the gallery app's own gating).

## Parallel-lane contract

Parallelized only against the F2/F3 wave under the serial-integration rules
of [parallel delivery](../../parallel-delivery.md) and the G1 sequencing in
[ROADMAP.md](../../../ROADMAP.md).

```text
Base commit:          verified `main` AFTER the F2a serial integration is
                      complete and `just verify` + canonical `just e2e` are
                      green on it (today's branch point is 75a22ee; F2a
                      retry is in flight — see §Ownership re-check).
Depends on:           F2a (footer/space-strip/appearance UI, `space_theme`)
                      and F2b (downloads follow-up: history, cancelled
                      restart/delete rows) integrated onto main; F2c may
                      still be in flight — bbr-ui never touches persistence.
Provides/API:         crate `bbr-ui` (workspace member), its public module
                      surface, DTOs, and the forbidden-dependency gate; the
                      extraction table §Extraction table; no wire, testkit,
                      or behavior API change.
Exclusive paths:      crates/bbr-ui/** (new). bbr-app's theme.rs, palette.rs,
                      components/**, shell/{presentation,tab,space}.rs, the
                      listed app/portal render modules (only the pure
                      presentation parts named in the table), and
                      ARCHITECTURE.md/UI-track/ROADMAP entries the slice
                      must document.
Shared integration points: root Cargo.toml/Cargo.lock (workspace member),
                      justfile (new gate recipes), bbr-app imports and
                      composition, E2E registries, DESIGN.md (only if a
                      token name is corrected — unlikely; token names are
                      already contract), UI track and ROADMAP roll-ups.
Integration order:    F2c → F2b → F2a → (serial main verify) → G1 packet 1
                      (this anchor) → packet 2 → packet 3 → packet 4 →
                      post-G1 serial integration of each migrated surface
                      with its affected E2E re-run.
```

G1 may run parallel to F3 Persistence/Warm-Restore, U3/contracts, and
disjoint non-UI work, because `bbr-ui` is presentation-only and none of those
lanes own `crates/bbr-ui/**` or the migrated render modules. It must not run
concurrently with any lane that mutates `theme.rs`, `palette.rs`,
`components/**`, `sidebar_view/**`, `portal/*render*`, or the footer/sidebar
render files.

### Ownership re-check (mandatory before packet 1 begins)

The orchestrator must re-check **every active lane** for write ownership of:
`crates/bbr-app/src/theme*.rs`, `palette.rs`, `components/**`, `space_theme.rs`
(F2a), `app/sidebar_view/**`, `app/u2_surface.rs`, `app/u2.rs`,
`portal/command.rs`, `portal/find.rs`, `portal/context_menu.rs`,
`portal/auth_sheet.rs`, `app/downloads_render/**` (F2b), and
`app/error_surface.rs`. There is exactly one writer for each of these files;
G1 takes ownership of a file only after the owning lane has integrated and
released it. **No second writer.** If an ownership conflict exists, packet 1
waits for the serial settlement before extracting that row.

## Constraints

- **Allowed `bbr-ui` dependencies:** `gpui` (the pinned 0.2.2 GPUI-CE source,
  `default-features = false`, `font-kit`), `bbr-domain` for `ProfileHue` only
  (the single hue authority; a duplicate hue vocabulary is a "two UI
  authorities" violation), and the standard library. Nothing else. No
  `bbr-browser`, `bbr-web`, `bbr-app`, `bbr-web-macos`, `bbr-testkit`,
  `bbr-fixtures`, `bbr-persistence`, `bbr-split`, `bbr-command`,
  `objc2*`/`web-kit`/`app-kit`/`foundation`, `raw-window-handle`, `serde`,
  `rusqlite`, `smol`, or any filesystem/network/socket API.
- **`#![forbid(unsafe_code)]`** at the crate root (the same rule as
  `bbr-web`/`bbr-domain`).
- **Forbidden-dependency audit gate** (compile + audit, part of every G1
  gate list): a `just`-recipe-plus-script check runs
  `cargo tree -p bbr-ui --edges normal --depth 2` (and a `cargo metadata`
  closure check) and fails on any forbidden package name; a `bbr-ui` unit
  test re-derives the same allowlist from `CARGO_MANIFEST_DIR`'s manifest
  `[dependencies]` and asserts the set exactly. The recipe is proposed by the
  lane, landed by the orchestrator (justfile is serial).
- **Plain presentation models and callbacks only.** `bbr-ui` holds no
  browser/session/profile/space authority: no `BrowserController`, no
  `ControllerSnapshot` type, no engine, no portal host, no command ranking,
  no persistence. Where a surface today derives from
  `bbr_browser::ControllerSnapshot` (e.g. `shell::space_groups`), `bbr-ui`
  owns a DTO-based derivation and `bbr-app` builds the DTOs from the
  controller snapshot.
- **Prefer `bbr-ui` DTOs over `bbr-browser` dependency.** The extraction
  table's "model/callback input" column names the DTO; no row may drag
  `bbr-browser` into `bbr-ui` to avoid defining one.
- **Zero local duplicates.** After each surface's cutover commit group,
  `rg` over `crates/bbr-app/src` finds no copy of the moved symbol and no
  `pub use bbr_ui` re-export alias in `bbr-app`. The old file is deleted in
  the same group.
- **Stable semantics.** `bbr-*` ids, the §10 state names, focus domains,
  keyboard behavior, and every §15 snapshot field are unchanged; the
  migrated modules must produce byte-identical `visible_ids` and semantic
  trees (existing snapshot/geometry tests are the guard).
- **Token names are contract.** `bbr-ui` keeps the exact
  `color:*`/`type:*`/`sp:*`/`radius:*`/`elev:*`/`dur:*`/`density:*` names
  from DESIGN.md; tests derive contrast from tokens, never copy ratios.
- **Module size budget:** every `bbr-ui` module stays below roughly 500
  lines; split by responsibility as today's `components/**` and
  `downloads_render/**` already are.
- **No weakened tests, budgets, validators, or acceptance criteria.** The
  contrast gate, the §15 mode fields, the perf budgets, and the E2E
  semantics all survive the move unchanged.
- **Each app migration re-runs its existing browser E2E and relevant perf:**
  the affected `just e2e-shell-*` suite(s) and `just e2e-perf` for
  user-visible presentation paths (per surface in §Extraction table).
- **GUI recipes use the host lock:** every G1 `just e2e-gallery*`/`just
  gallery-*` recipe that opens a GPUI window runs under
  `python3 scripts/with_gui_lock.py --` (the repository's `gui_gate`
  wrapper); gallery recipes never join the canonical `just e2e`.
- **Every future implementation gate list** in packets 2–4 contains
  `cargo fmt --all -- --check` (the `just fmt` recipe is exactly this
  command) and states `CARGO_BUILD_JOBS=4` for every cargo invocation
  (bounded parallel build, per the F2 wave's OOM lesson).
- **No compatibility layer, no temporary duplicate exports, no two UI
  authorities.** Extract, migrate all production uses, delete the old
  definition — one coherent cutover per surface. This is the serial
  kernel's rule, not an option.

## Model/callback contract

`bbr-ui` components keep the exact call shape today's app components use, so
extraction is mechanical and gallery stories can drive the same components:

- **Model:** a plain `bbr-ui::model::*` DTO (plain data, `Clone`, `Debug`;
  no GPUI types except `Theme`/`SharedString` where a component already
  renders text). The DTO is built by `bbr-app` from the controller snapshot
  (app-owned derivation) or by the gallery fixture builders.
- **Theme:** the component receives `&Theme` (mode/contrast/density resolved
  by the caller). The gallery's mode controls construct `Theme::new(mode,
  contrast, density)` — the existing public constructor.
- **Callbacks:** the same handler closures `bbr-app` passes today — GPUI
  `cx.listener(...)` results / `impl Fn(&MouseDownEvent, &mut Window, &mut
  App) + 'static`-style callbacks (see today's `components::tab_row` and
  `error_surface` signatures). Gallery stories pass recorder callbacks that
  append to a `Vec<CallEvent>` so stories prove which callback fired.
- **State/ids:** `bbr-ui` owns the state vocabulary (`TabVisualState`,
  `SpaceVisualState`, `DownloadVisual`, mode fields) and the stable
  `bbr-*` id constants; `bbr-app`'s snapshot/`visible_ids` derivation imports
  them from `bbr-ui` so ids cannot drift.

## Primitive inventory (honest)

Only extract anatomy that already exists with repeated same intent. The
following are **not** present as standalone primitives in the source and are
**not** claimed by this program: Button, Input/TextField, Icon *button*
(a bare icon glyph exists; the button row anatomy lives inline in each
surface), Badge, Chip (a `SpaceIdentity` chip is a composition of
`profile_chip` + label + rail — see [packet 4](component-gallery-compositions.md)),
Divider (the `hairline` is a 1 px `div` + token today, not a component),
Progress (only the downloads secondary-line bar and the §10.5 loading arc
exist). The gallery Foundations pages therefore story **existing** pieces
(hairline/rule tokens, section labels, favicon/monogram tiles, profile
chips, status glyphs, loading arc) and the MVP packet may add *component*
stories only where the same anatomy already renders in two or more surfaces
(that is the "repeated same-intent anatomy" bar). The four packets' story
matrices stay inside this inventory; inventing a Button/Input/Badge
primitive is out of scope until a product surface genuinely needs it and
extracts it.

## Extraction table

Columns: **current symbol/file** → **future `bbr-ui` module** →
**model/callback input** → **`bbr-app` migration owner** → **gallery story
owner (packet)**. Paths are relative to `crates/bbr-app/src/` unless
prefixed.

### Foundations

| Current symbol / file | Future `bbr-ui` module | Model/callback input | `bbr-app` migration owner | Gallery story owner |
|---|---|---|---|---|
| `Theme`, `ThemeMode`, `ContrastMode`, `TypeRole`, `TypeStep`, `TypeScale`, `Spacing`, `Radius`, `Motion`, `Elevation`, `Theme::{dark,new,for_collection}`, `type_scale` — `theme.rs` | `bbr-ui::theme` | mode/contrast/density fields only (constructors stay callable) | `app.rs::BbrApp::theme()`, `app/render.rs`, every `crate::theme::*` use site | Packet 2 Foundations: Tokens, Typography, Spacing/Radius, Motion, Elevation pages |
| `Density` + S/N/XL geometry tokens (`row`, `row_tile`, `sidebar_width`, `palette_width`, `hit_min(_close)`, `row_inset`, `section_gap`, `favorites_gap`, `state_slot`, `row_padding_x`, `profile_chip`, `density_radius`, `space_row`, `icon`) — `theme/density.rs` | `bbr-ui::theme::density` | density enum only | same as above; `density_geometry_is_uniform_per_density` test moves | Packet 2 Foundations: Density page (S/N/XL table) |
| `Palette`, `Palette::{dark_standard,dark_increased,light_standard,light_increased}`, `FaviconSlate`, `ProfileHueRamp`, `Hex` WCAG gate — `palette.rs` | `bbr-ui::palette` | `ThemeMode` + `ContrastMode`; `bbr_domain::ProfileHue` for `ProfileHueRamp::tile` | `app.rs` theme resolution; contrast tests move into `bbr-ui::palette::tests` | Packet 2 Foundations: Color pages (neutral/accent/status/tint per mode+contrast) |
| `TabVisualState`, `SpaceVisualState`, `DownloadVisual` (+ mode fields) — `components/mod.rs`, `app/downloads.rs` | `bbr-ui::state` | plain enums, no input | all `crate::components::*VisualState` use sites; testkit snapshot mapping imports from `bbr-ui` | Packet 2 Foundations: States page (the §10 vocabulary) |

### Primitives (repeated same-intent anatomy only)

| Current symbol / file | Future `bbr-ui` module | Model/callback input | `bbr-app` migration owner | Gallery story owner |
|---|---|---|---|---|
| `Icon` catalog + `u2_glyphs` geometry — `components/icons.rs`, `components/u2_glyphs.rs` | `bbr-ui::icons` (+ `bbr-ui::icons::glyphs` for the shared path geometry) | `Icon::canvas(theme, size)` (theme + density icon size) | `components/icons.rs` deleted; every `components::Icon` use site | Packet 2 Foundations: Iconography (every glyph × S/N/XL sizes, stroke, fallback) |
| `section_label` — `components/section.rs` | `bbr-ui::components::section` | theme + text + focused flag | `app/sidebar_view.rs::tab_surface`, `app/u2_surface.rs` | Packet 2 Foundations: Labels / Section |
| `FaviconSpec`, `monogram_letter`/`monogram_hue` (FNV-1a) — `components/mod.rs`, `shell/presentation.rs` | `bbr-ui::components::favicon` (tile + monogram ramp) | `FaviconSpec` DTO (letter, hue index); theme | `app/sidebar_view.rs::row_visuals`, `shell/presentation.rs` deletion of `monogram_*` | Packet 2 Foundations: Favicon tile + missing-favicon rule |
| `RowVisuals`, `tab_row`, `drag_tick` — `components/tab_row.rs`, `components/mod.rs` | `bbr-ui::components::tab_row` | `RowVisuals` DTO + select/close/pin/drag handler closures | `app/sidebar_view.rs::render_row`, `tab_surface` | Packet 3 Browser: Tab Row |
| `ProfileChipSpec`, `profile_chip`, `space_row`, `space_new_row`, `SpaceRowVisuals`, `SpaceVisualState` — `components/space.rs`, `shell/space.rs` | `bbr-ui::components::space` | `SpaceRowVisuals` DTO (index, profile_index, title, count, state, chip) + select/new-space callbacks; `ProfileHue` | `app/sidebar_view.rs::space_surface`; `shell::space::{profile_monogram, active_profile_chip, space_groups, space_title_budget_chars}` become `bbr-ui` DTO derivation or stay as `bbr-app` DTO builders (space_groups takes `bbr-ui` DTOs, never `ControllerSnapshot`) | Packet 3 Browser: Space row; Packet 2 Foundations: Profile chip |
| `empty_space_pane`, `crash_panel` — `components/pane.rs` | `bbr-ui::components::pane` | theme + profile line + new-tab/reload/close callbacks | `app/render.rs` content branch (error/crash/empty) | Packet 3 Browser: Empty / Crash |
| `favorite_tile(s)`, `favorites_grid`, `FavoriteTileSpec` (+ ids) — `components/favorites.rs` | `bbr-ui::components::favorites` | `FavoriteTileSpec` DTOs + tile activation callback + drag-target flag | `app/u2_surface.rs::favorites_surface`, `sidebar_view.rs` (u2 rail data) | Packet 3 Browser: Favorites |
| `folder_row`, `folder_item_row`, `folder_row_specs`, `FolderRowSpec`, `FolderMemberVisual`, `FolderPresentation` (+ ids) — `components/folders.rs` | `bbr-ui::components::folders` | folder specs DTOs + toggle/select callbacks + focus | `app/u2_surface.rs::folders_surface`, `app/sidebar_view.rs::tab_surface` | Packet 3 Browser: Folders |
| `footer_surface`, `FooterControl` (+ ids) — `components/footer.rs` | `bbr-ui::components::footer` | footer DTO (downloads count, space strip chips, plus) + per-control callbacks | `app/u2_surface.rs` (post-F2a footer: permanent space strip + appearance tint) | Packet 3 Browser: Footer / Space strip |
| `plus_menu_items`, `PlusMenuState` (+ ids) — `components/menu.rs` | `bbr-ui::components::menu` | open/selected state + commit callback | `app/u2_surface.rs::plus_menu_surface`, `app/u2.rs` | Packet 3 Browser: Menu |
| `address_control`/`staging_surface` (the §11.5 pill + staging field), `address_domain`, `address_domain_budget_chars`, `mid_ellipsis`, `accessory_slot_width` — `app/sidebar_view/command_bar.rs`, `command_bar.rs` | `bbr-ui::components::address` (+ `bbr-ui::text` for mid-ellipsis/budgets) | `AddressModel` DTO (domain label, hovered) + open-overlay callback + optional profile-hint chip; staging `CommandBarMode` DTO | `app/sidebar_view.rs::command_bar` | Packet 3 Browser: Address / input pieces |
| `Icon` canvas + `elev2_shadows`/`elev2_shadow` (duplicated today in `portal/command.rs`, `portal/find.rs`, `portal/context_menu.rs`, `app/downloads_render.rs`) | `bbr-ui::theme::elevation` (one `elev2` helper) | theme only | all four duplicate helpers deleted | Packet 2 Foundations: Elevation |

### Browser components (presentation models + callbacks)

| Current symbol / file | Future `bbr-ui` module | Model/callback input | `bbr-app` migration owner | Gallery story owner |
|---|---|---|---|---|
| `CommandPortalModel`, `command_card`, `palette_card`, `overlay_card`, `field_row`, `result_row`, `state_row`, `result_rows`, `group_glyph` — `portal/command.rs` | `bbr-ui::surfaces::command` | `CommandPortalModel` DTO (surface, theme, chrome_focused, text, state, results, selected) — render-only, no callbacks (activations stay app-owned) | `portal/probe.rs::PortalRoot` render path; `portal/command.rs` deleted; `bbr-app/src/app/palette.rs::surface_ids` imports ids from `bbr-ui` | Packet 4 Compositions: Command palette / Overlay |
| `find_bar`, `field`, `control_button` — `portal/find.rs` | `bbr-ui::surfaces::find` | `FindBarModel` DTO (theme, focused, query, match_count, current_match) + previous/next/close callbacks | `portal/probe.rs`; `portal/find.rs` deleted; `bbr-app/src/app/find.rs` builds the DTO | Packet 3 Browser: Find |
| `auth_sheet`, `header`, `field`, `button`, `password_mask` — `portal/auth_sheet.rs` | `bbr-ui::surfaces::auth` | `AuthSheetModel` DTO (authority, realm, host, username, focused field, password_entered) + submit/cancel callbacks. **Source truth:** today's sheet hardcodes `Theme::dark()` (`portal/auth_sheet.rs`); the story must not enshrine that — the `bbr-ui` surface takes the theme from the model like every other surface | `portal/probe.rs`; `portal/auth_sheet.rs` deleted; `bbr-app/src/app/auth/records.rs` builds the DTO | Packet 3 Browser: Auth |
| `context_menu_card` — `portal/context_menu.rs` | `bbr-ui::surfaces::menu` | `ContextMenuModel` DTO (theme, focused, selected, items) + row hover/press callbacks | `portal/probe.rs`; `portal/context_menu.rs` deleted; `bbr-app/src/app/context_menu.rs` builds the DTO | Packet 3 Browser: Context menu |
| `downloads_panel_surface`, `downloads_row`, `downloads_group_label`, `downloads_panel_width`, `download_row_id`/`download_slot_id`, `elev2_shadow`, `row_detail` — `app/downloads_render.rs`, `app/downloads_render/row_detail.rs` | `bbr-ui::surfaces::downloads` | `DownloadsPanelModel` DTO (open, focused, rows: `Vec<DownloadRowModel>`, bounded window, layout) + per-row action callbacks (cancel/retry/reveal/restart/delete — post-F2b row actions) | `app/downloads_render/**` deleted; `app/downloads.rs` keeps the controller and builds DTOs; `downloads_visible_ids` imports from `bbr-ui` | Packet 3 Browser: Downloads |
| `error_surface`, `ErrorSurfaceSpec` — `app/error_surface.rs` (presentation half only; the incident lifecycle stays app-owned) | `bbr-ui::surfaces::error` | `ErrorSurfaceModel` DTO (title, copy, origin, action label, dismiss label) + action/dismiss callbacks | `app/render.rs` content branch; `app/error_surface.rs` keeps `ErrorSurfaceState` (lifecycle authority) and builds the DTO | Packet 3 Browser: Error / Crash |

### Compositions

| Current symbol / file | Future `bbr-ui` module | Model/callback input | `bbr-app` migration owner | Gallery story owner |
|---|---|---|---|---|
| `sidebar_surface` + `u2_pre_tab_children`/`u2_post_tab_children` rail composition — `app/sidebar_view.rs`, `app/u2_surface.rs`; `SidebarGeometry` — `sidebar.rs` + `app/sidebar.rs` (geometry model only) | `bbr-ui::sidebar` (`SidebarModel` + `Callbacks`) + `bbr-ui::sidebar::geometry` | `SidebarModel` DTO (theme, width, visible, brand, command/address, favorites tiles, folder rows, tab rows, spaces, status, toggle, divider, footer DTOs) + callback struct (command-open, row select/close/pin/drag, space activate, toggle, divider drag/keyboard, downloads toggle, space-strip, plus) | `app/sidebar_view.rs` becomes a thin `bbr-app` assembler that builds `SidebarModel` from the controller snapshot and passes `cx.listener` callbacks; `app/render.rs` root | Packet 4 Compositions: Sidebar |
| Split chrome frames: `app/render.rs` split `div` composition (the two-slot row + shared-edge divider) per DESIGN §11.7 — the slot itself (`NativeWebViewSlot`) stays app-owned; `crates/bbr-split` geometry stays as-is | `bbr-ui::split` (`SplitPaneFrame`, `SplitDivider`, split-zone frames) | `SplitModel` DTO (pane count, focused pane, divider geometry) + no callbacks (render-only; splits are app-created) | `app/render.rs` split branch renders `bbr-ui::split` chrome around the existing slots | Packet 4 Compositions: Split preview |
| Space strip chips + `space_theme` tint (post-F2a: `crates/bbr-app/src/space_theme.rs` nine-tone contract) | `bbr-ui::components::space_strip` + `bbr-ui::theme::space_tint` | `SpaceChipModel` DTO (identity, active, tint token) + activate callback; `SpaceTheme` DTO mapped from the F2a contract | F2a-integrated `u2_surface.rs`/`space_theme.rs` consumers | Packet 4 Compositions: Cross-profile / Space strip |
| `status_line`, `bbr-status` render — `shell/presentation.rs`, `app/sidebar_view.rs` | `bbr-ui::components::status` | status text DTO | `app/sidebar_view.rs` | Packet 2 Foundations: Status line |
| Brand block (inline in `sidebar_surface`) | `bbr-ui::components::brand` | brand text DTO | `app/sidebar_view.rs` | Packet 2 Foundations: Brand |

## Serial cutover

- **Order:** Foundations → primitives → browser components → compositions,
  matching packets 2–4. Within a surface: extract into `bbr-ui` (with its
  DTO + tests), migrate every `bbr-app` use site, delete the old definition,
  run the affected `just e2e-shell-*` suite + `just e2e-perf`, commit.
- **Zero local duplicates:** the cutover commit group is atomic per surface;
  intermediate commits inside one group are coherent but never claim
  completion (per [development workflow](../../development-workflow.md)).
  `rg` guards in the gate list prove no duplicate and no `pub use` shim.
- **No compatibility layer:** no temporary re-export from `bbr-app`, no
  `pub use bbr_ui` alias, no feature-gated duplicate behind the old path.
  The old module file is deleted in the same group.
- **`bbr-app` keeps composition only:** controller, snapshot adapters,
  portal geometry/probe/native, `NativeWebViewSlot`, testkit gateway, and
  DTO building stay in `bbr-app`; all *visual* code moves.
- **Base verification:** packet 1's implementation starts only from the
  verified `main` after F2a (per the parallel-lane contract above), and every
  subsequent surface cutover re-verifies with its affected gates before the
  next cutover.

## Evidence

### Unit and contract

- [ ] `bbr-ui` unit tests cover the moved contrast gate (derived from
      tokens, never copied ratios), the exact token-name mapping
      (DESIGN.md §2–§8), density geometry, and the id constants.
- [ ] `bbr-ui` model/callback tests: each DTO round-trips its fields; each
      component's story-level contract test renders with recorder callbacks
      and asserts the exact callback fired (ids/models/callbacks coverage).
- [ ] Forbidden-dependency test: `bbr-ui`'s manifest `[dependencies]` equals
      the allowlist exactly (gpui, bbr-domain, std) and the crate carries
      `#![forbid(unsafe_code)]`.
- [ ] Per-surface migration guard: `rg` finds no duplicate symbol and no
      `pub use bbr_ui` alias in `crates/bbr-app/src` after each cutover
      group.

### Real-app E2E

- [ ] Each migrated surface re-runs its existing affected suite
      (`just e2e-shell-<suite>`) with unchanged `bbr-*` ids, §15 snapshot
      fields, focus domains, and typed semantic trees (the existing suites
      are the migration guard).
- [ ] The canonical sequential `just e2e` runs green once after the complete
      stable cutover wave (per the delivery workflow).

### Performance

- [ ] `just e2e-perf` passes unchanged after each user-visible presentation
      migration (no budget, sample-count, or semantic-commit change).
- [ ] Gallery perf is **sanity only**, never a product budget (see
      [packet 2](component-gallery-mvp.md)); it is excluded from
      `just e2e-perf` by construction.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo check --workspace --all-targets
CARGO_BUILD_JOBS=4 cargo test --workspace
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p bbr-ui --all-targets -- -D warnings
just ui-deps                    # forbidden-dependency audit gate (proposed here, landed by orchestrator)
just e2e-shell-<affected-suite> # per migrated surface
just e2e-perf                   # after user-visible presentation migrations
just e2e                        # once after the stable cutover wave
python3 scripts/check_roadmap.py
```

Every cargo invocation states `CARGO_BUILD_JOBS=4` (bounded parallel build).
GUI recipes run sequentially under the host-wide lock
(`python3 scripts/with_gui_lock.py --`).

## Delivery sequence

1. Orchestrator re-checks lane ownership (theme/components/footer/sidebar/
   portal-render files) and confirms `main` is verified after F2a.
2. One exclusive writer creates `crates/bbr-ui/**` from this anchor's
   boundary and extraction table; foundations first.
3. Per surface: extract → migrate all `bbr-app` use sites → delete the old
   definition → run the affected suite + perf → commit (small, coherent
   groups; Angular subjects with scope `ui`).
4. One independent read-only reviewer maps every extracted symbol back to
   this table and checks for duplicates/shadows/aliases and unchanged
   semantics.
5. One remediation writer only for concrete findings, then a focused recheck.
6. Orchestrator lands the workspace-member + justfile recipe integration
   (serial points), then the canonical `just e2e`.

## Iteration policy

Start with Foundations (theme/palette/density/icons) because every later row
consumes them. For each failure, classify it as dependency-boundary, DTO
shape, call-shape mismatch, id/semantics drift, module-size budget, or gate
environment; change the smallest central cause and re-run the affected
suite. Never relax the forbidden-dependency gate or the "zero local
duplicates" rule to shorten the cutover.

## Blocked stop condition

If a surface cannot be extracted without changing ids, semantics, focus
domains, or perf evidence, stop with the surface, the conflicting symbol,
the failing suite/budget, the preserved worktree state, and the smallest
product/architecture decision required. If F2a (or F2b for downloads) has
not integrated, the affected rows wait; do not extract from an unintegrated
worktree state.

## Parked decisions

Decisions that cannot be inferred from source are parked with
question/options/recommendation/impact. **Do not ask Patrick now and do not
invent.** The implementation proceeds on the recommendation; the parked
status is recorded so the reviewer and orchestrator can settle it before
integration if evidence contradicts the recommendation.

1. **Gallery packaging/name** — *Question:* how is the dev-only gallery
   packaged and named? *Options:* (a) one new workspace bin crate
   `crates/bbr-gallery` (package `bbr-gallery`, binary `bbr-gallery`) —
   mirrors the `bbr-app` bin-crate pattern and keeps gallery-only code out of
   `bbr-ui`; (b) a `bbr-ui` example binary (`examples/gallery.rs`) — least
   workspace surface but mixes a full app into the library crate and makes
   the E2E harness awkward; (c) `crates/bbr-gallery` lib+bin split for
   testable controller + thin bin — more surface than needed for v1.
   *Recommendation:* (a). *Impact:* one workspace-member addition (serial
   point), a new `just gallery` / `just e2e-gallery` recipe pair, and the
   gallery is never packaged into the `.app` and never joins `just e2e`.
2. **Semantic-vs-pixel snapshot strategy** — *Question:* what does
   `just e2e-gallery` assert? *Options:* (a) semantic ids + typed mode/state
   fields + stable `StoryId` only (DESIGN §15's "never by pixel comparison"
   rule extended to the gallery); (b) semantic assertions plus a dev-only
   screenshot/export command for human review (never an assertion);
   (c) golden-image diff assertions. *Recommendation:* (a) plus (b) as a
   manual convenience. *Impact:* keeps the gallery test stack consistent
   with the product's anti-pixel stance and the AGENTS.md "no pixel
   comparisons" rule; (c) would introduce a second, flaky assertion
   vocabulary for zero product value.
3. **Non-macOS compilation** — *Question:* on which targets do `bbr-ui` and
   `bbr-gallery` compile? *Options:* (a) `bbr-ui` portable on every target
   the workspace evaluates (it is pure GPUI presentation, no native code;
   GPUI-CE already builds on Linux via the `wayland` feature), and
   `bbr-gallery` gated `#[cfg(target_os = "macos")]` for v1 like
   `bbr-web-macos`; (b) both compile everywhere; (c) both macOS-only.
   *Recommendation:* (a). *Impact:* keeps `just verify` green on Linux
   (bbr-ui participates in the workspace gates), avoids promising a
   cross-platform gallery before Phase 10, and matches the current host
   reality. The gallery's lack of WebKit makes full cross-platform support
   cheap later, which is why the crate itself stays portable.

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
