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
(23–29), Library composition (30), Address control (31), Command staging (32),
and Renderer-Isolation checkpoint recorded in
[Current delivery](../../delivery/current.md). Group 3 Address/Input is
integrated. Groups 4–9 may now proceed in isolated lanes from the same recorded
base with canonical semantic integration order 4 → 5 → 6 → 7 → 8 → 9
(portal cluster 6 → 8 → 9). Group 4 already consumes production `bbr-ui`
components extracted by G1a, so its remaining work is Gallery matrix, fixtures,
stories, recorder E2E, and documentation. Groups 5–9 still perform the real
`bbr-ui` extraction/app migration before adding Gallery stories.
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
| Address / input pieces (`bbr-ui::components::address`) | domain pill (with/without www, IDN, port); accessory slot reserved-empty vs Copy-Link (future); hover fill step; profile-hint chip presence; staging field (New Space / Move Tab / empty New Tab) with block cursor true/false, hint present/absent (incl. the unfocused Move Tab), and profile-hint chip presence proven independent of cursor and hint | pill-as-text-field (no editing state, no cursor, no selection — §11.5); a focus ring on the pill (never, any state) |
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

- [x] Each new `bbr-ui` browser-component module: contrast/ids/models/
      callbacks tests (per the anchor's Evidence contract) — DTO round-trips,
      id constants, recorder-callback firing, and state composition order
      (§10: error > loading > pressed > selected > hover > default).
      Group 6: `bbr-ui::surfaces::find` pins the seven §15 ids, round-trips
      the `FindSurface` DTO and `FindBarRect` geometry, and derives the
      §11.5 count copy through the production `count_label` (n of m, 0 of 0,
      empty-while-pending).
- [x] Matrix test in the gallery: every "valid" row builds and renders
      without panic; every "rejected" row fails fixture construction; the
      matrix table and the fixture builders cannot drift (one declaration
      source). Group 3: `address_matrix.rs` is the one Address/Staging
      declaration source — every `ADDRESS_VALID`/`STAGING_VALID` family
      builds, every `ADDRESS_REJECTED` combination (pill-as-text-field, a
      focus ring, Copy-Link) refuses fixture construction, and the pinned
      label tests keep the table and the builders from drifting. The
      staging matrix explicitly asserts its dimension coverage so it
      cannot regress: cursor true/false, hint present/absent (incl. the
      unfocused Move Tab — cursor=false + hint present), and profile-hint
      present/absent through the production
      `StagingModel.profile_hint`/`AddressProfileHint`, with the
      profile-hint-present families covering both cursor values and both
      hint values (presence independent of cursor and hint).
- [x] Post-cutover `rg` guard: no duplicate symbol and no `pub use bbr_ui`
      alias in `bbr-app` for the migrated modules. Group 3: the guard now
      also pins the `address_control`/`staging_surface`/`AddressModel`/
      `StagingModel`/`AddressProfileHint` production symbols (the app keeps
      only its DTO-building composition methods). Group 6: the guard now
      also pins `pub fn find_bar(`/`struct FindSurface`/`const FIND_BAR_ID`
      and asserts the portal root renders through
      `bbr_ui::surfaces::find::find_bar`.

### Real-app E2E

- [x] Each cutover group re-runs its affected suite unchanged:
      `just e2e-shell-tabs`, `just e2e-shell-organize`,
      `just e2e-shell-command` (address/overlay/palette),
      `just e2e-shell-downloads`, `just e2e-shell-find`,
      `just e2e-shell-errors`, `just e2e-auth`, `just e2e-shell-context-menu`
      (and the F2a-integrated footer/spaces suites) — ids, §15 fields, focus
      domains, and typed semantic trees unchanged. Group 3 re-ran its
      affected suite unchanged: `just e2e-shell-command`
      (address/overlay/palette) is green 25/25 on the same byte-identical
      ids/§15 fields/focus domains/semantic trees (the existing suite is the
      migration guard and was not modified). Group 6 re-ran its affected
      suite unchanged: `just e2e-shell-find` is green 6/6 on the same
      byte-identical ids/§15 fields/focus domains/semantic trees (the
      existing suite is the migration guard and was not modified).
- [x] `just e2e-gallery` extended: the new story matrix renders through the
      production components with stable StoryIds appended (G1b ids
      unrenumbered). Group 3 appended the Address control (StoryId 31) and
      Command staging (StoryId 32) stories after the pinned 0–30 prefix; the
      GUI-locked Xcode-26.2 suite is green 24/24, the every-story-renders
      sweep covers the two new stories automatically, and the focused
      `e2e/address.rs` scenarios prove real pointer hover/press fires the
      production `on_hover`/`on_open` recorder callbacks. The focused
      staging assertions also pin the stable 7-family staging count and
      assert the staging story includes the production
      `bbr-command-profile-hint` semantic id (the profile-hint families
      render the production `staging_surface` with
      `StagingModel.profile_hint: Some(...)`). Group 6 appended the Find
      bar story (StoryId 33) after the pinned 0–32 prefix; the GUI-locked
      Xcode-26.2 suite is green 26/26, the every-story-renders sweep covers
      the new story automatically, and the focused `e2e/find.rs` scenarios
      prove real pointer presses fire the production `on_previous`/
      `on_next`/`on_close` recorder callbacks and that the story renders
      the production §15 ids (registered as test-target keys on the first
      family, the Address precedent).

### Performance

- [x] Existing product performance code, budgets, baselines, sample guards,
      and semantic-commit contracts stay unchanged after each user-visible
      presentation migration. Execution remains deferred/unpassed under the
      active delivery override; no `just e2e-perf` run or pass is claimed.
      Group 3 is Gallery-only (no production migration): no performance
      code/budget/sample/baseline changed, and no performance run or pass is
      claimed — the missing execution stays explicit debt. Group 6 is a
      production presentation migration (extract → migrate → delete): the
      find performance code (`find_query_result`/`find_step` benchmarks,
      `schedule_find_render_commit`, the armed-report render milestone) is
      preserved unchanged in `bbr-app`; no budget/sample/baseline/
      semantic-commit change, and no performance run or pass is claimed —
      the missing execution stays explicit debt.
- [x] Gallery perf remains sanity-only (G1b), never a product budget.

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
  Group 3 Address/Input (Gallery-only) implemented and settled as an exact
  writer candidate. One Address/Staging matrix+fixture authority
  (address_matrix.rs) declares every valid pill/staging family and refuses
  the rejected no-edit/no-focus-ring/no-Copy-Link combinations; the gallery
  appends the exact production address_control/staging_surface stories at
  StoryIds 31–32 (0–30 unrenumbered) with recorder/observer wiring; a
  focused real-pointer E2E proves on_hover and on_open fire; the
  every-story-renders sweep covers the new stories; app.rs stays at 459
  lines (≤498). No production bbr-ui/bbr-app change, no manifest/lock/
  justfile/perf change, no new ids or product policy. Remediation closed
  the review finding: the staging matrix now covers cursor=false +
  hint-present (unfocused Move Tab) and profile-hint presence through the
  production StagingModel.profile_hint/AddressProfileHint, with an
  explicit dimension-coverage test (cursor true/false, hint
  present/absent, profile-hint present/absent, profile-hint presence
  independent of cursor and hint); the staging story renders all 7
  families and the focused E2E asserts the production
  bbr-command-profile-hint semantic id and the stable 7-family count.
Commits:
  Exact reviewed lane ed5931175cfc121fbbc146dce28df4f41cf8df77 / tree
  e448da5b35c87444f76c1b016d84eca5424f1b74; semantic Orchestrator merge
  d6cd4353434ed8b31319e2fd12c9e9d991071cbc / tree
  7b9e9ee4c1a408d961cbb892bb1594a1fc5d4655, parents Control
  25a4ef0a7c2746faa6e148ebf29e187bc3fe7fc7 + exact lane commit.
Functional evidence:
  Gallery unit tests 48/48 (matrix valid/rejected incl. the staging
  dimension-coverage test, recorder, registry pinned-prefix incl. 23–30,
  rg guard incl. address symbols). Gallery Clippy --all-targets -D
  warnings clean. GUI-locked Xcode-26.2 `just e2e-gallery` 24/24 incl.
  e2e::address::real_hover_and_press_report_through_the_recorder, the
  staging profile-hint semantic-id assertion, and the every-story sweep.
Real-app E2E:
  `just e2e-shell-command` (address/overlay/palette) re-run unchanged and
  green 25/25 — ids, §15 fields, focus domains, typed semantic trees
  byte-identical (the existing suite is the migration guard, unmodified).
Performance evidence:
  None executed; performance remains Patrick-deferred/unpassed. No
  budget/sample/baseline/semantic-commit change (Gallery-only cut); missing
  execution recorded as explicit debt.
Roadmap/documentation updates:
  G2 packet evidence/ledger only (this file). ROADMAP.md / ARCHITECTURE.md /
  UI track roll-ups are Orchestrator-owned serial points.
Open blockers and non-goals:
  Copy-Link accessory (§17.14), internal-page naming (§17.13), pill editing/
  cursor/selection/focus ring, foreground/background modifier and opener
  policy — all remain open decisions, none invented. Performance execution
  and canonical `just e2e` remain unexecuted/unpassed.
Worktree state:
  Branch parallel/g2-address-input at base 81822d1d13571e63e443ae7e8c3dc32
  daf1257e2 (tree 5ed8442e0d175d66199f1a2c262231d7f05cee8e); real index
  unchanged at the base tree; working-tree candidate only, diff-clean,
  no staged paths, no commit, no process/lock.

Planning/read-only:
  Read ROADMAP/current delivery/DESIGN §11.5/packet/production address +
  gallery Tab Row & Library patterns; advisory refreshed post-activation.
Implementation:
  address_matrix.rs, address_recorder.rs, address_app.rs, control_observer
  .rs (behavior-preserving app.rs extraction), stories/address.rs, e2e/
  address.rs, registry/canvas/snapshot/stories/main wiring, rg-guard
  extension, packet evidence/ledger.
Test execution:
  cargo check + focused gallery tests (48/48), Gallery Clippy, GUI-locked
  just e2e-gallery 24/24, unchanged just e2e-shell-command 25/25, just
  ui-deps, just roadmap, fmt, just verify, diff-check.
Review:
  Two broad read-only attempts timed out without authority after identifying
  the same concrete staging-matrix gap. The ultra-focused exact-candidate
  recheck returned G2_ADDRESS_RECHECK_PASS. The staged semantic union then
  returned G2_ADDRESS_STAGED_UNION_PASS: all 15 lane paths were staged, every
  product/packet blob matched the reviewed lane, and Control's sole current
  delivery authority remained the only lane-to-union tree difference.
Remediation:
  One remediation writer closed the review finding in the authorized
  files only: address_matrix.rs (StagingCase.profile_hint + 3 new staging
  families: unfocused Move Tab — hint, no cursor; Move Tab — cursor + hint
  + profile hint; unfocused New Space — profile hint, no cursor; the
  dimension-coverage test; build_staging_fixture maps the production
  StagingModel.profile_hint/AddressProfileHint), stories/address.rs (all 7
  families render; the profile-hint families register the production
  bbr-command-profile-hint semantic id as a test-target key; meta line
  and ids/profile-hint value rows updated), e2e/address.rs (stable
  7-family staging count, profile-hint families render, production
  bbr-command-profile-hint semantic id asserted), and this packet's
  evidence/ledger. No production change, no new product modes/ids/
  Copy-Link, StoryIds stay 31/32.
Waiting/retries:
  The first post-merge repository-target `just verify` reached Clippy after all
  tests passed but failed on read-only derived metadata. A fresh `/tmp` target
  with repository Xcode 26.2 passed unchanged; this was cache recovery, not a
  source or acceptance relaxation.
Post-merge evidence:
  Fresh-target `just verify` passed on exact merge tree 7b9e9ee4...; log
  `/tmp/bbr-g2-address-postmerge-verify-fresh.log`, SHA-256
  fbb20f2a48a111f3f45f97285410c8d82708010014dcfd6f36eb6445999ea352.
  GUI-locked Xcode-26.2 Gallery passed 24/24; log
  `/tmp/bbr-g2-address-postmerge-gallery.log`, SHA-256
  ae7479915cc702e556174b48e76248b8bbaa7f9ffad58be20d8e01d780f937e5.
  Unchanged Command passed 25/25; log
  `/tmp/bbr-g2-address-postmerge-command.log`, SHA-256
  26cd6ec4f65faba70ab80eee072264001328ed36ba7f6ea03f031702b5c3b49f.
  HEAD/tree/status remained exact and the GUI lock released. Performance and
  canonical `just e2e` remain unexecuted/unpassed.
Total elapsed:
  Estimate (single writer session).
Human intervention:
  None.

Group 6 Find bar:
Outcome:
  The §11.4 find-in-page bar is extracted to the production
  `bbr-ui::surfaces::find` module (find_bar/FindSurface/FindBarRect/
  count_label + the seven §15 ids), `bbr-app` migrates to it (the app
  keeps the FindController/session/adapter/lifecycle/portal placement and
  builds the DTO; the old app-owned FindSurface and portal/find.rs render
  are deleted), and the gallery appends the Find bar story at StoryId 33
  (0–32 unrenumbered) with one matrix authority (find_matrix.rs: 6 valid
  families, 2 rejected with reasons, one validity authority, fixture
  builder refusing rejected cases), recorder/observer wiring, and a
  focused real-pointer E2E proving on_previous/on_next/on_close fire.
  The story registers the production §15 ids as test-target keys on the
  first family (the Address precedent) so the E2E can assert the
  production ids render; the pointer E2E derives the control centers from
  the bar bounds (wrapper origin + settled 360 pt × density:row), not the
  wrapper center/right edge. app.rs stays at 467 lines (≤498); the new
  modules are 303/337/97/130/201/176 lines. No manifest/lock/justfile/
  perf change, no new product ids or policy.
Commits:
  None — working-tree candidate only; Orchestrator commits/integrates.
Functional evidence:
  Gallery unit tests 55 passed / 26 ignored (find matrix valid/rejected,
  count-label derivation, stale-is-lifecycle, recorder, registry pinned
  prefix incl. 33, rg guard incl. find symbols). bbr-ui 79/79. Gallery
  Clippy --all-targets -D warnings clean; workspace Clippy clean; full
  workspace test 72 binaries green. GUI-locked Xcode-26.2 `just
  e2e-gallery` 26/26 incl. e2e::find::find_story_renders_the_matrix_
  families_with_stable_ids and real_pointer_presses_report_through_the_
  recorder plus the every-story sweep.
Real-app E2E:
  `just e2e-shell-find` re-run unchanged and green 6/6 — ids, §15 fields,
  focus domains, typed semantic trees byte-identical (the existing suite
  is the migration guard, unmodified).
Performance evidence:
  None executed; performance remains Patrick-deferred/unpassed. The find
  performance code (find_query_result/find_step benchmarks,
  schedule_find_render_commit, the armed-report render milestone) is
  preserved unchanged in bbr-app; no budget/sample/baseline/
  semantic-commit change; missing execution recorded as explicit debt.
Roadmap/documentation updates:
  G2 packet evidence/ledger only (this file). ROADMAP.md / ARCHITECTURE.md /
  UI track roll-ups are Orchestrator-owned serial points.
Open blockers and non-goals:
  The packet's stale flag is lifecycle state (FindState.stale), not a
  presentation variant — the DTO carries no stale field and the gallery
  never fakes a stale visual. Foreground/background modifier and opener
  policy remain open decisions. Performance execution and canonical
  `just e2e` remain unexecuted/unpassed.
Worktree state:
  Branch parallel/g2-group6-find at base 5e018b34286daab690c7f88d32c4b97
  dc3204484 (index tree 58d2c8152761c5536944b1b6535d2f75a1a57615); real
  index unchanged at the base tree; working-tree candidate only,
  diff-clean, no staged paths, no commit, no process/lock.
Planning/read-only:
  Read ROADMAP/current delivery/DESIGN §11.4–§11.5/packet/production find
  + gallery Address & Tab Row patterns; advisory refreshed post-activation.
Implementation:
  bbr-ui/src/surfaces/find.rs + mod.rs, bbr-app/src/app/find.rs migration
  (DTO building only), portal/probe.rs render migration, portal/find.rs
  deletion, find_matrix.rs, find_recorder.rs, find_app.rs, stories/find.rs,
  e2e/find.rs, registry/canvas/snapshot/stories/main wiring, rg-guard
  extension, packet evidence/ledger.
Test execution:
  cargo check + focused gallery tests, Gallery/workspace Clippy, GUI-locked
  just e2e-gallery 26/26, unchanged just e2e-shell-find 6/6, just ui-deps,
  just roadmap, fmt, just verify (log /tmp/g6-verify.log, SHA-256
  2c85083008d823e795376e206b3fd9ddc90d24d01c0ba41303a106b9ff50d71a),
  diff-check.
Review:
  Pending — independent read-only review of the exact candidate tree.
Remediation:
  None yet — the two focused find E2E failures found during the writer's
  own gate run were fixed centrally: the story now registers the
  production §15 ids as test-target debug_selector keys (the Address
  precedent; debug_bounds is keyed by debug_selector, not .id()), and
  the pointer E2E derives the control centers from the bar bounds
  (wrapper origin + settled bar size) instead of the wrapper center/right
  edge (the meta line below can exceed 360 pt and adds height).
Waiting/retries:
  None.
Post-merge evidence:
  Pending Orchestrator integration.
Total elapsed:
  Estimate (single writer session).
Human intervention:
  None.
```

Use measured timing only when runtime/command timestamps support it;
otherwise label values as estimates.
