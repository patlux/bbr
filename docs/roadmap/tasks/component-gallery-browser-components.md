# Task packet — Component gallery: browser components

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 5](../phases/05-command-bar.md) ·
[Phase 8](../phases/08-daily-driver.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
Visual contract: [DESIGN.md](../../../DESIGN.md) §10–§12, §15 ·
Anchor: [UI library anchor (bbr-ui)](ui-library-anchor.md) ·
Predecessor: [Gallery MVP](component-gallery-mvp.md) ·
Workflow: [development workflow](../../development-workflow.md) ·
Template: [task-packet template](../templates/task-packet.md)

Program position: **G2**, continuing from the integrated Gallery MVP, Status
Line (StoryId 21), Tab Row (22), Saved Pages Library component rows/controls
(23–29), Library composition (30), and Renderer-Isolation checkpoint recorded
in [Current delivery](../../delivery/current.md). The next bounded cut is Group
3 Address/Input; Group 4 follows it, then Groups 5–9 may proceed in isolated
lanes with canonical semantic integration order 4 → 5 → 6 → 7 → 8 → 9
(portal cluster 6 → 8 → 9). Groups 3–4 already consume production `bbr-ui`
components extracted by G1a, so their remaining work is Gallery matrix,
fixtures, stories, recorder E2E, and documentation. Groups 5–9 still perform
the real `bbr-ui` extraction/app migration before adding Gallery stories.
Stories construct **production presentation models and callback recorders** —
gallery renderability is explicitly not native Portal/AppKit/WebKit z-order
evidence.

## Outcome

Every browser-component surface named below is a real `bbr-ui` module
consumed by `bbr-app` (serial cutover per the anchor) and storyable in
`bbr-gallery`. The gallery story matrix enumerates the meaningful valid
variants per component, rejects impossible combinations, and every story
renders through the production component with a fixture-built presentation
model and a recorder callback proving which interaction fired. No fake
`BrowserController`, WebKit, portal host, or network participates.

## Scope

- **Staged real adoption** (in this order, each a serial cutover group):
  1. Tab Row (row anatomy, states, close/pin/drag-tick) — the row's
     favicon slot embeds the already-extracted G1b
     `bbr-ui::components::favicon` component; Tab Row stories cover the
     missing-favicon monogram and loading-arc variants inline, with no
     separate favicon story owner in this packet;
  2. Status line (`bbr-app/src/shell/presentation.rs` → a real
     `bbr-ui::components::status` production component, followed by app
     migration/deletion of the old definition and the first append-only
     Gallery story). G1b intentionally could not story this surface because
     its only implementation remained app-owned and G1b forbids both
     `bbr-app` imports and Gallery-local copies;
  3. Address / input pieces (§11.5 pill + staging field + accessory slot,
     mid-ellipsis budgets);
  4. Favorites grid, Folders rows/members, Footer surface (+ post-F2a space
     strip chips), Plus menu;
  5. Downloads panel (post-F2b rows: cancel/retry/reveal/restart/delete);
  6. Find bar; 7. Error/Crash surfaces; 8. Auth sheet; 9. Context menu.
  After their separately scheduled production packets integrate, append —
  without renumbering any existing StoryId — Tab/Favorite menu, Folder menu,
  enriched Page-background menu, Titlebar navigation controls, and Sidebar
  media-card components. B2/B3 are integrated: Library component stories own
  ids 23–29 and the Library composition owns id 30; no reservation machinery
  remains. These rows register the same production symbols; this Gallery
  packet does not authorize their behavior implementation.
  Groups 3–4: production extraction/app migration is already complete; add
  only Gallery matrix/fixtures/stories/recorder E2E and re-run the unchanged
  affected real-app suite. Groups 5–9: extract to `bbr-ui` → migrate
  `bbr-app` → delete old definition → re-run the affected
  `just e2e-shell-<suite>`. Performance code/budgets remain mandatory, but
  performance execution is deferred/unpassed under the active delivery override.
- **Story matrix** for each component in `bbr-gallery`, per the matrix
  below. Stories use the `bbr-ui` presentation-model DTOs from the anchor
  table and recorder callbacks; the gallery owns plain fixture builders only.
- **Variant correctness:** enumerate meaningful valid variants; reject
  impossible combinations (the matrix's "rejected" column states why — e.g.
  a `Cancelled` download row renders no primary action today, §17.8 F2b
  changes that to restart+delete; a crash panel never shows on a healthy
  tab; the address pill never shows a field cursor).

## Non-goals

- **No framework-core-control re-stories here.** Button, real Text Input,
  Checkbox, Switch, Radio, Slider, and Progress are G1b stories over the
  already established vendored `gpui-component` authority. G2 owns BBR browser
  components such as Address/Input anatomy only after their production
  extraction into `bbr-ui`; it neither copies nor wraps the framework stories.
- **No compositions here.** The full Sidebar, CommandPortalModel palette/
  overlay *compositions*, Downloads/Find/Error/Auth/Menu/Split *groups*,
  cross-profile and compact/wide stories are
  [G3](component-gallery-compositions.md). This packet stories the
  components; G3 stories their composition in context.
- **No product behavior change.** Ids, states, focus, and §15 snapshot
  fields stay byte-identical through each cutover (the existing suites are
  the guard).
- **No native evidence.** Rendering a find bar or auth sheet in the gallery
  does not prove AppKit z-order, portal attachment, or WebKit hit-testing;
  those remain the product's `just e2e-shell-*` portal suites. The gallery
  is a presentation catalog, not a native-boundary harness.
- **No fake engine / no authority.** Stories never construct a
  `BrowserController`, `WebEngine`, portal request, session, or persistence
  record.
- **No new variants beyond what DESIGN.md and the source define.** The
  matrix derives from DESIGN.md §10–§12/§15 and the current render modules;
  it does not invent product behavior.

## Parallel-lane contract

```text
Base condition:       the settled Gallery MVP slice (bbr-gallery registry,
                      tree, canvas, controls, Foundations stories).
Depends on:           G1b; G1a's bbr-ui boundary and DTOs; F2a
                      (footer/space strip) and F2b (downloads follow-up)
                      integrated onto main before their rows are extracted.
Provides/API:         bbr-ui browser-component modules (per the anchor's
                      extraction table), their presentation-model DTOs, and
                      the gallery story matrix rows for this packet; no wire
                      or behavior API change.
Exclusive paths:      crates/bbr-ui/** (the listed browser-component
                      modules) and crates/bbr-gallery/** (story matrix,
                      fixture builders). bbr-app's listed render modules are
                      touched only by the serial cutover of their surface
                      (migrate + delete old definition).
Shared integration points: root Cargo.toml/Cargo.lock, justfile (no new
                      recipe expected — existing suites re-run), E2E
                      registries, bbr-app imports/composition, UI track and
                      ROADMAP roll-ups, ARCHITECTURE.md facts.
Integration order:    G1b → G2 → G3 → post-G1a serial
                      integration. Parallel-safe against F3 / U3 / non-UI
                      lanes (disjoint paths) as long as no lane touches the
                      migrating render modules.
```

## Constraints

- All anchor constraints apply: exact direct `bbr-ui` dependencies (`gpui`,
  `bbr-domain` for `ProfileHue`, implicit std), the approved first-party
  `bbr-domain → bbr-web` identity closure with no direct `bbr-web` import,
  `#![forbid(unsafe_code)]`, dependency-boundary audit gate, zero local
  duplicates, stable `bbr-*` ids and §15 fields, module size budget, and no
  weakened tests/budgets/validators.
- **Presentation models only:** each `bbr-ui` surface takes its DTO +
  callbacks; `bbr-app` keeps the controller, lifecycle, and DTO building
  (e.g. `ErrorSurfaceState` keeps the incident lifecycle; `downloads.rs`
  keeps the controller; `find.rs` keeps the session; `auth/records.rs` keeps
  the redacted store).
- **Post-F2a/F2b grounding:** the Footer/Space-strip rows base from the
  F2a-integrated footer (permanent space strip + appearance tint), and the
  Downloads rows base from the F2b-integrated row actions (history
  persistence, cancelled restart/delete). The matrix below notes both.
- **Story matrix is a contract:** the gallery's fixture builders must cover
  every "valid" row and must never construct a "rejected" combination; a
  gallery test walks the matrix and asserts validity/exclusion.
- **Gallery tests supplement, never replace, affected browser E2E/perf.**
- **GUI recipes use the host lock; every gate list contains
  `cargo fmt --all -- --check` and states `CARGO_BUILD_JOBS=4`.**
- **Gallery renderability is not native Portal/AppKit/WebKit z-order
  evidence** — stated in the packet's outcome and enforced by keeping the
  portal geometry/probe suites as the only native-boundary evidence.

## Story matrix (browser components)

Conventions: a story = (component, model fixture, state/variant selector).
"Rejected" rows are combinations the component must never receive; the
gallery fixture builder refuses to build them.

| Component | Valid variants | Rejected (why) |
|---|---|---|
| Tab Row (`bbr-ui::components::tab_row`) | states: default/hover/active/selected/loading/attention/error/disabled/dragging/drag-target/focused (§10); pinned + regular; S (icon-only pinned) / N / XL; long-title truncation; favicon slot: monogram tile / missing-favicon host-scheme fallback / loading-arc replacement (the embedded G1b `bbr-ui::components::favicon` component — its variants are consumed inline, not re-storied); reveal-close on hover/selected; drag-tick legal vs clamped-invalid | attention on a crashed row (error wins, §10 composition order); selected+dragging without the drag state machine; S-density pinned with a title (icon-only, §8/§11.2); a real favicon bitmap (the tile is monogram-or-arc today, §7.2) |
| Status line (`bbr-ui::components::status`, extracted from the current app-owned `shell::presentation::status_line`) | healthy/empty, active loading, navigation error, crashed tab, and relevant active-space count/selection states derived through the migrated production presentation model; stable semantic root and append-only Gallery StoryIds | a Gallery-local formatter or copied status anatomy; direct `bbr-app` imports; raw controller ownership in `bbr-ui`; a story registered before production extraction and app migration are complete |
| Profile chip / Space-strip pieces (`bbr-ui::components::space::profile_chip` + the app-owned permanent space strip pending G3 extraction; DESIGN §11.3–§11.4) | profile chip S/N/XL density, 1–2 character monogram, hue tile, profile-keyed id `bbr-profile-chip-<profile_index>`; Group 4 Footer story may show the already integrated strip in context without claiming a standalone `space_row`; G3 owns extraction/storying of the full permanent strip composition | removed legacy `space_row`/`space_new_row` list anatomy; any chip keyed to a space index; any wrapped/second-line strip; any Group-4 fixture that creates a second strip or navigation authority |
| Address / input pieces (`bbr-ui::components::address`) | domain pill (with/without www, IDN, port); accessory slot reserved-empty vs Copy-Link (future); hover fill step; profile-hint chip presence; staging field (New Space / Move Tab / empty New Tab) with block cursor | pill-as-text-field (no editing state, no cursor, no selection — §11.5); a focus ring on the pill (never, any state) |
| Favorites grid (`bbr-ui::components::favorites`) | S 5-col / N 4-col / XL 3-col; tile default/hover/active/focused; drag-target wash; empty region (label hidden) | reorder during selection (tiles never reorder, §11.10); a tile without a resolved TabId+SpaceId+ProfileId semantic name |
| Folders (`bbr-ui::components::folders`) | collapsed/expanded; nested rows with full §10 vocabulary; empty collapsible folder; count `tnum`; hairlines between groups | nested rows in a collapsed folder (they render only when expanded); persisted folders (session-local only, §11.11) |
| Footer + Space strip (`bbr-ui::components::footer` + post-F2a strip) | downloads control with/without running count; space strip: one chip, many chips, overflow clip (single line, horizontal scroll); active chip selected surface + outline; inactive chips quiet; plus control | a wrapped/second-line strip (single-line clip, §11.4); a chip without its accessible name containing space+profile names (§11.3) |
| Plus menu (`bbr-ui::components::menu`) | closed/open; selection; keyboard ↑/↓/Enter/Escape; the two v1 items (New Space, New Folder) | a third item (exactly two in v1, §11.4); an item while the menu is closed |
| Downloads panel (`bbr-ui::surfaces::downloads`, post-F2b rows) | Waiting (unknown total, indeterminate); Waiting (known total); Running (progress fraction); Succeeded (full bar + Reveal eligibility on/off → enabled/disabled); Failed (Retry); Cancelled (post-F2b: Restart + Delete); Active/History groups; empty hint; >6-row scroll window; keyboard cursor focus | a `Cancelled` row with no action post-F2b (the decision replaces the §17.8 provisional no-action form); a `Retry` on Succeeded/Failed-unknown (retry exists only on `Failed`, §11.12/F2b); progress going backwards (monotonic, §11.12); `NaN` fraction (zero-byte full bar) |
| Find bar (`bbr-ui::surfaces::find`) | open/focused; query empty (placeholder) vs typed; count/ordinal ("n of m"); 0 of 0; stale flag; prev/next/close controls at `density:hit-min(_close)` | an ordinal without a count (WebKit reports both or neither); a "next" enabled at 0 matches (no matches → no step) |
| Error / Crash (`bbr-ui::surfaces::error`, `bbr-ui::components::pane`) | error: TLS (expired/untrusted/revoked/mismatch/handshake), Navigation (server-not-found/refused/timeout/redirect-loop/unsupported-scheme), Offline (no-network/proxy-unreachable/network-unreachable) — each with its calm title/copy + contract-legal action; origin shown vs omitted; crash panel (Reload/Close tab); empty pane (profile line + New Tab ⌘T) | a TLS action other than Retry (no bypass — `crates/bbr-app/src/app/error_surface.rs` `ErrorSurfaceSpec`/`IncidentCategory` maps TLS/Navigation/Offline to Retry only and the closed action set has no bypass variant, per the [error-surface-contract](error-surface-contract.md) matrix); a crash panel on a healthy tab; raw platform error codes in any copy |
| Auth sheet (`bbr-ui::surfaces::auth`) | realm present vs absent; origin-server vs proxy authority; username typed/empty; password masked bullets; focus username vs password; Cancel vs Log In | an unmasked password render (bullets always); outside-press dismissal (no backdrop dismissal for credentials); any serializable credential shape |
| Context menu (`bbr-ui::surfaces::menu`) | existing page menu: item enabled/disabled, selection, Back/Forward enablement, Open New Tab / Reload / Copy Link / Back / Forward, keyboard cursor + pointer hover; post-packet append-only families: Tab/Favorite closed/open/submenu/disabled/danger/overflow bound to exact TabId; Folder empty/populated/move-submenu/destructive bound to FolderId; Page-background owned-command matrix while WebKit element-menu classes are represented as ownership metadata, not copied menus | a menu with zero items; activation of a disabled row; any source-less Tab/Favorite/Folder menu; any Gallery-owned rendering of WebKit link/image/selection/editable/media menus |
| Titlebar navigation (`bbr-ui::components::titlebar_navigation`, after its production packet) | sidebar toggle collapsed/expanded; Back/Forward all enablement combinations; Reload/loading; S/N/XL; standard/increased contrast; focused/hover/disabled | hardcoded traffic-light geometry; a second sidebar-collapse or navigation authority; titlebar controls inside the address component |
| Sidebar media card (`bbr-ui::components::media_card`, only after media contract + production UI) | absent; playing/paused; seekable known-duration; live/unknown duration; capability subsets for previous/next/mute; one/multiple sessions; dismissed identity; stale/ended disabled/removal state; cross-profile source identity; reduced motion/contrast | decorative unsupported controls; fake duration/scrubber; a model without opaque session + TabId/SpaceId/ProfileId identity; any fixture that claims WebKit control |

B2/B3 are integrated. Saved Pages Library rows/controls are registered at
StoryIds 23–29 and the Library composition at 30. Later Saved Page menu/result
families may append only after their production component exists; this packet
reserves no ids and reintroduces no marker machinery.

## Evidence

### Unit and contract

- [ ] Each new `bbr-ui` browser-component module: contrast/ids/models/
      callbacks tests (per the anchor's Evidence contract) — DTO round-trips,
      id constants, recorder-callback firing, and state composition order
      (§10: error > loading > pressed > selected > hover > default).
- [ ] Matrix test in the gallery: every "valid" row builds and renders
      without panic; every "rejected" row fails fixture construction; the
      matrix table and the fixture builders cannot drift (one declaration
      source).
- [ ] Post-cutover `rg` guard: no duplicate symbol and no `pub use bbr_ui`
      alias in `bbr-app` for the migrated modules.

### Real-app E2E

- [ ] Each cutover group re-runs its affected suite unchanged:
      `just e2e-shell-tabs`, `just e2e-shell-organize`,
      `just e2e-shell-command` (address/overlay/palette),
      `just e2e-shell-downloads`, `just e2e-shell-find`,
      `just e2e-shell-errors`, `just e2e-auth`, `just e2e-shell-context-menu`
      (and the F2a-integrated footer/spaces suites) — ids, §15 fields, focus
      domains, and typed semantic trees unchanged.
- [ ] `just e2e-gallery` extended: the new story matrix renders through the
      production components with stable StoryIds appended (G1b ids
      unrenumbered).

### Performance

- [ ] Existing product performance code, budgets, baselines, sample guards,
      and semantic-commit contracts stay unchanged after each user-visible
      presentation migration. Execution remains deferred/unpassed under the
      active delivery override; no `just e2e-perf` run or pass is claimed.
- [ ] Gallery perf remains sanity-only (G1b), never a product budget.

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
# Do not run just e2e-perf while the active delivery override applies.
just e2e-gallery
just verify                       # after the stable slice
python3 scripts/check_roadmap.py
```

Every cargo invocation states `CARGO_BUILD_JOBS=4`. GUI recipes run
sequentially under `python3 scripts/with_gui_lock.py --`.

## Delivery sequence

1. Confirm packet-2 base is settled and the anchor's browser-component DTOs
   are defined.
2. Group 3 runs first as one isolated Gallery-only lane. After its reviewed
   integration, Groups 4–9 may run in isolated sibling worktrees with one
   writer each and predeclared shared registry/composition points. The
   Orchestrator integrates semantically in order 4 → 5 → 6 → 7 → 8 → 9;
   portal groups integrate 6 → 8 → 9. Groups 3–4 skip extraction because the
   production components are already consumed; Groups 5–9 retain the complete
   extract → migrate → delete → re-run cycle.
3. Extend the gallery story matrix and fixture builders in the same groups;
   StoryIds append sequentially from 31 with no gaps, renumbering, or reservations.
4. One independent read-only reviewer maps each story back to DESIGN.md and
   the anchor table, checks the rejected-combination logic, and verifies no
   fake engine/portal/network entered the gallery.
5. One remediation writer only for concrete findings, then a focused recheck.

## Iteration policy

Preserve the canonical semantic integration order because Gallery StoryIds
and shared registries are append-only; this is an integration-order contract,
not a false compile-time dependency between Groups 4–9. For each failure,
classify it as cutover (migration/import), DTO shape, matrix validity, or
native-boundary confusion; change the smallest central cause and re-run the
focused suite.
Never render a native-boundary claim in the gallery, and never relax the
"rejected" matrix to make a fixture build.

## Blocked stop condition

If a surface cannot be storied or extracted without changing ids, semantics,
focus domains, or performance evidence, stop with the surface, conflicting
symbol, failing suite/budget, preserved worktree state, and smallest decision
required. F2a/F2b and Renderer are integrated; do not reopen their settled
contracts or use them as a reason to widen a G2 lane.

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
