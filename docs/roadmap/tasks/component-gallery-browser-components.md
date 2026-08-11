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

- [ ] Each new `bbr-ui` browser-component module: contrast/ids/models/
      callbacks tests (per the anchor's Evidence contract) — DTO round-trips,
      id constants, recorder-callback firing, and state composition order
      (§10: error > loading > pressed > selected > hover > default).
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
      only its DTO-building composition methods).

### Real-app E2E

- [ ] Each cutover group re-runs its affected suite unchanged:
      `just e2e-shell-tabs`, `just e2e-shell-organize`,
      `just e2e-shell-command` (address/overlay/palette),
      `just e2e-shell-downloads`, `just e2e-shell-find`,
      `just e2e-shell-errors`, `just e2e-auth`, `just e2e-shell-context-menu`
      (and the F2a-integrated footer/spaces suites) — ids, §15 fields, focus
      domains, and typed semantic trees unchanged. Group 3 re-ran its
      affected suite unchanged: `just e2e-shell-command`
      (address/overlay/palette) is green 25/25 on the same byte-identical
      ids/§15 fields/focus domains/semantic trees (the existing suite is the
      migration guard and was not modified).
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
      `StagingModel.profile_hint: Some(...)`).

### Performance

- [x] Existing product performance code, budgets, baselines, sample guards,
      and semantic-commit contracts stay unchanged after each user-visible
      presentation migration. Execution remains deferred/unpassed under the
      active delivery override; no `just e2e-perf` run or pass is claimed.
      Group 3 is Gallery-only (no production migration): no performance
      code/budget/sample/baseline changed, and no performance run or pass is
      claimed — the missing execution stays explicit debt.
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
```

### Group 4 (G2 sidebar components) — packet-only remediation record

```text
Outcome:
  Group 4 of the G2 browser-components packet: the five sidebar-component
  stories (Favorites grid, Folders, Footer, Profile chip S/N/XL, Plus menu)
  render every valid matrix family through the exact production `bbr-ui`
  symbols with plain fixture models and recorder callbacks, and the two
  Group-4 E2E scenarios pass. Gallery-only packet: the production
  extractions are already integrated (Groups 3–4 are Gallery-only), so this
  group adds matrix, fixtures, stories, recorders, and E2E only — no product
  behavior change, no native Portal/AppKit/WebKit z-order claim, no fake
  engine/portal/network. The `bbr-ui` delta is test-support selector-only
  (10 `.debug_selector` additions, no behavior change).

Commits:
  None. Packet-only remediation record; the worktree stays uncommitted
  (0 staged paths) for the orchestrator's serial integration kernel.

Functional evidence:
  - StoryIds 33–37 appended after the pinned 0–32 prefix (Group 3 owns
    31–32): 33 Favorites grid, 34 Folders, 35 Footer, 36 Profile chip
    (S/N/XL), 37 Plus menu — no renumbering, no reservation, no gaps,
    proven by the registry test
    `group4_sidebar_stories_append_after_the_address_prefix`.
  - Matrix contract: `favorites_matrix.rs`, `folders_matrix.rs`,
    `footer_matrix.rs`, `profile_chip_matrix.rs`, `menu_matrix.rs` declare
    VALID/REJECTED families; the E2E walks them and the fixture builders
    refuse rejected combinations.
  - Recorder path: `group4_recorder.rs` (app-owned plain state) reports
    production callback firings (favorite activate, folder toggle/select/
    close, footer downloads/plus, plus-menu item commit) through the same
    seam the mode/state controls use; `snapshot.rs` exposes `group4`.
  - Layout shrink/overflow mechanism: `story_page` gained `.min_w_0()` +
    `.overflow_x_hidden()` (vertical scroll retained), the app root and
    canvas flex-1 content columns gained `.min_w_0()`, and `value_row`
    values are shrinkable (`min_w_0` + `overflow_hidden` + `text_ellipsis`)
    so long values never force the story page wider than the window and the
    real-pointer E2E targets stay within the clickable viewport.
  - Test-support selector-only `bbr-ui` delta: 10 new `.debug_selector`
    additions — `favorites.rs` 3 (FAVORITES_ID, favorite_tile_icon_id,
    favorite_tile_id), `folders.rs` 4 (folder_row_id, folder_toggle_id,
    folder_item_id, folder_item_close_id), `footer.rs` 2 (FOOTER_ID,
    control.id()), `space.rs` 1 (profile_chip_id). No other `bbr-ui`
    change; ids, §15 fields, and focus domains stay byte-identical.
  - Module/dependency constraints: `bbr-ui` keeps
    `#![forbid(unsafe_code)]` and its direct set = pinned GPUI-CE
    (rev 568271c0, default-features=false + font-kit) + `bbr-domain` only;
    `bbr-gallery` adds `bbr-domain` (ProfileHue identity only), gpui,
    gpui_platform, and the vendored gpui-component 0.5.1 production control
    stack; the forbidden set stays browser/engine/persistence/testkit/
    fixtures/app. `just ui-deps` → UI-DEPS PASS (direct set, first-party
    closure, bbr-web paths, and source imports all match the approved
    boundary).

Real-app E2E:
  - `just e2e-gallery` (real gallery app, GUI lock): 26/26 passed in 9.55s,
    including `e2e::group4::group4_stories_render_the_matrix_families_with_stable_ids`
    and `e2e::group4::real_pointer_presses_report_through_the_group4_recorders`.
  - Affected shell suites re-run unchanged (ids/§15/focus unchanged):
    `just e2e-shell-organize` 2/2 + `e2e_shell_drag_chip` 4/4 (log total
    6/6), `just e2e-shell-sidebar` 7/7, `just e2e-shell-spaces` 9/9 +
    `e2e_shell_spaces_keyboard` 4/4 + `e2e_shell_spaces_hint` 2/2 (log
    total 15/15).

Performance evidence:
  - None executed in this packet. `just e2e-perf` and the canonical
    sequential `just e2e` were not run (packet-only remediation scope);
    gallery perf remains sanity-only (report-only artifacts, never a
    product budget). Performance/canonical e2e unexecuted.

Roadmap/documentation updates:
  - This Group-4 delivery ledger/evidence block only. No ROADMAP.md,
    phase, track, or delivery-document edits (docs/delivery/current.md is
    Orchestrator authority and was not touched).

Open blockers and non-goals:
  - No blockers. Non-goals honored: no compositions (packet 4), no product
    behavior change, no native evidence, no fake engine/authority, no new
    variants beyond DESIGN.md/source.

Worktree state:
  - Worktree: /Users/patwoz/dev/Personal/macos/bbr-g2-group4-components
    (branch parallel/g2-group4-components).
  - HEAD: 5e018b34286daab690c7f88d32c4b97dc3204484.
  - HEAD tree = index tree (git write-tree):
    58d2c8152761c5536944b1b6535d2f75a1a57615 — the base commit's tree;
    nothing is staged, so `git write-tree` reports the base tree, not the
    candidate worktree.
  - All-files candidate tree before this packet edit (the 24 Group-4 code
    files over the base, packet file still at the base blob):
    a9df3d98b38cc874b375dba9a8921916b0b2c476.
  - All-files candidate tree after this correction (the 24 Group-4 code
    files + the corrected packet): recorded by the orchestrator at
    integration — the packet cannot truthfully embed its own tree hash.
  - Status hash (git status --porcelain -z | shasum -a 256):
    1817781e8760a97879ea757e91b7cb9f4e60bb6989b934242e58d3af232186e1.
  - 25 paths, 0 staged (12 modified, 13 untracked).
  - Restored Favorites backup evidence: `crates/bbr-gallery/src/stories/
    favorites.rs` and the pre-remediation backup `/tmp/favorites.rs.bak`
    are byte-identical — both SHA-256
    c26a31dbb43d370ab02265f324064035d92dbc09b448d123101f32433e68a435 —
    proving the story file was restored from the backup before this
    ledger edit.
  - Packet base blob equality (separate from backup evidence): the packet
    file blob 5f27f9e8bfb3aec7bc1af7726d04ae7e06ae5124 is the base
    (HEAD/index) version, byte-identical in the independent review
    worktree /tmp/bbr-g2-group4-review (HEAD
    f4447a308d9751680172fd738b4850a9e47b1d22, clean status); the
    pre-remediation packet was preserved untouched before this ledger
    edit.

Planning/read-only:
  - Read ROADMAP.md (current focus: Phase 4 vertical-tab shell; the
    component-gallery packet is the active G2 lane) and this packet
    completely before writing. No code/test was edited.

Implementation:
  - Gallery-only: 11 modified files (bbr-gallery app/canvas/e2e-mod/main/
    registry/snapshot/stories + bbr-ui favorites/folders/footer/space) and
    13 new files (e2e/group4.rs, five *_matrix.rs, group4_app.rs,
    group4_recorder.rs, five stories/*.rs).

Test execution:
  - /tmp/g4-verify.log
    a8579d809df7cd234eb97bdddeb8aa9fdf56534f5bd94cef16762425da33eece:
    full `just verify` sequence — cargo fmt --all -- --check; cargo check
    --workspace --all-targets (7.29s); cargo test --workspace (215 unit
    tests passed, e2e harness tests 1-passed-each); cargo clippy
    --workspace --all-targets -- -D warnings (8.25s, zero diagnostics);
    scripts/check_roadmap.py → "Roadmap control graph valid (95 tracked
    first-party Markdown files checked from worktree)";
    scripts/check_ui_dependencies.py → UI-DEPS PASS; 46 script tests OK
    (15.031s); py_compile OK. Zero `error[` occurrences; only
    informational warnings (block v0.1.6 future-incompat note, expected
    "Git tree is dirty", grandfathered roadmap warnings).
  - /tmp/gallery-e2e.log
    851bb9808ee6da4416392ccc7552f32578d2819ab5b29b58c2f2a1de3a3d40dc:
    `cargo test -p bbr-gallery --bin bbr-gallery -- --ignored --nocapture`
    under the GUI lock — 26/26 passed (9.55s).
  - /tmp/organize.log
    af337f3f03435f037446d1d6af3dfee3e4286c77852d3b89eee18bcd3b974917:
    e2e_shell_organize 2/2 (2.93s) + e2e_shell_drag_chip 4/4 (3.57s) —
    log total 6/6 (the orchestrator's "4/4" names the drag_chip suite).
  - /tmp/sidebar.log
    2a4f71c1d2114e278f6494d82efc16a981e3e7041d29d8491b2fd1cf529d4479:
    e2e_shell_sidebar 7/7 (4.82s).
  - /tmp/spaces.log
    c74ffa83ffdda7e3b68a2927c386343bc2fa1172d88b767cc38fda9afaf8c361:
    e2e_shell_spaces 9/9 (6.11s) + e2e_shell_spaces_keyboard 4/4 (3.73s)
    + e2e_shell_spaces_hint 2/2 (2.29s) — log total 15/15 (the
    orchestrator's "2/2" names the spaces_hint suite).

Review:
  - Independent read-only review worktree /tmp/bbr-g2-group4-review at
    HEAD f4447a308d9751680172fd738b4850a9e47b1d22 (clean status); packet
    blob identical to the base (HEAD/index) packet blob (packet base blob
    equality above).

Remediation:
  - Earlier gallery runs failed before the final pass:
    /tmp/g4-e2e-gallery.log 24 passed / 2 failed and
    /tmp/g4-e2e-gallery2.log 25 passed / 1 failed; the remediation
    resolved the failures and the final run is 26/26
    (/tmp/gallery-e2e.log).

Waiting/retries:
  - GUI-lock serialization only (each suite acquired the host GUI gate
    sequentially); no retries in the final evidence set.

Total elapsed:
  - Estimate (no wall-clock start/end recorded for the whole packet):
    measured command durations sum to ≈ 1.5 min of test execution
    (verify 15.0s script tests + 7.3s/8.3s builds, gallery 9.55s,
    organize 6.5s, sidebar 4.82s, spaces 12.1s); full packet wall time
    including planning/review is not measured.

Human intervention:
  - None recorded.
```

Use measured timing only when runtime/command timestamps support it;
otherwise label values as estimates.
