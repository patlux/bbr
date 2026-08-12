# Task packet — Tab and Favorite context menus

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13 ·
[UI library anchor](ui-library-anchor.md) ·
[Gallery browser components](component-gallery-browser-components.md) ·
[Gallery compositions](component-gallery-compositions.md).

Activation condition: complete F2a and G1a are integrated, and Bookmark rows additionally wait for B2, as recorded in [Current delivery](../../delivery/current.md). The
implementation must not start before the verified integration sequence
F2c → F2b → fresh F2a and the settled G1a `bbr-ui` anchor.

Stable information-architecture references (not feature or visual copies):

- `/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-tab-context-menu-main.png`
- `/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-tab-context-menu-folder-submenu.png`
- `/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-tab-context-menu-move-submenu.png`
- `/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-tab-context-menu-share-submenu.png`
- `/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-tab-context-menu-close-submenu.png`
- `/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-favorite-context-menu.png`

Zen supplies grouping, submenu, and native-macOS interaction reference only.
BBR uses its own Chassis & Glass tokens, copy, domain commands, profile model,
and stable ids.

## Outcome

Secondary-clicking a tab row or Favorite opens one production BBR menu bound
to the exact persistent `TabId` that invoked it. Opening, traversing, or
dismissing the menu does not activate, select, move, reload, close, recreate,
or change the profile/space/session of that tab. A Favorite is another
presentation of the same concrete tab, so both surfaces consume one shared
availability/command authority while presenting the appropriate Favorite
label (`Add to Favorites` / `Remove from Favorites`).

## Scope

- Add an app/controller-owned typed `TabMenuSource`/availability matrix keyed
  by the source `TabId`, its owning `SpaceId`/`ProfileId`, pinned region,
  index, lifecycle, Favorite state, split membership, and advertised product
  capabilities.
- Project that source into a presentation-only `bbr-ui::TabMenuModel` (labels,
  enabled/disabled/selected state, submenu rows, stable presentation source
  key) plus callbacks already bound by `bbr-app` to the real `TabId`.
  `bbr-ui` imports no TabId/controller authority. Sidebar anchors stay
  GPUI-owned and do not use `NativeAboveWeb`.
- Provide native-macOS pointer and keyboard behavior: secondary-click,
  Up/Down traversal, Right/Left submenu traversal, Return commit, Escape and
  outside-click dismissal, edge clamping, disabled reasons, and focus return.
- Initial BBR-owned action groups, only where their authority exists:
  - placement: `Move to Folder ›`, `Move to Space ›` with visible profile
    identity for cross-profile destinations;
  - page/session: `Reload Tab`, Favorite toggle; after integrated [B2 Saved Pages actions](saved-pages-actions-command.md), capability-gated `Bookmark Page` / `Remove Bookmark` rows invoke B2's one app-owned command against the bound tab/Favorite URL without retaining `TabId` in the Saved Page;
  - lifecycle: `Pin Tab` / `Unpin Tab`, `Suspend Tab`, `Duplicate Tab`;
  - system/share: `Copy Link`, the native macOS Share service picker;
  - ordering: `Move to Start`, `Move to End`;
  - close: `Close Tab`, `Reopen Closed Tab` where legal.
- Capability-gated rows are added only with their owning contract:
  `Add to Split…`, `Mute Tab`, `Rename Tab…`, `Change Icon…`,
  `Move to New Window`, and bulk-close scopes. An unavailable whole group is
  omitted rather than filled with decorative dead rows; a useful group may
  retain a disabled row with a deterministic reason.
- Stable ids:
  `bbr-tab-menu`, `bbr-favorite-menu`, `bbr-tab-menu-<action>`,
  `bbr-tab-menu-folder-<n>`, `bbr-tab-menu-space-<n>`,
  `bbr-tab-menu-bookmark` (label Bookmark Page/Remove Bookmark from B2 state),
  and `bbr-tab-menu-close-<scope>`.
- Add typed snapshot/semantic state, real-app E2E, and optimized presentation
  evidence for open and submenu commit.
- Extend Gallery G2/G3 only by registering these real production components
  and compositions after they exist; no menu anatomy or fixture-only copy.

## Non-goals

- No `Send to Mobile`, route-for-domain, Firefox Container Tab, QR-code
  creator, bulk Select All, extension-contributed rows, or other Zen/Firefox
  features without a separate BBR contract. Saved Page rows are allowed only
  after [B2](saved-pages-actions-command.md) and must call its shared command;
  this menu packet never owns bookmark persistence or duplicate policy.
- No Rename/Icon before durable tab-label/icon ownership exists.
- No media Mute row before the portable media capability contract advertises
  it for the exact session.
- No split or new-window row before those product authorities exist.
- No invented bulk-close boundary rule. The interaction contract's open
  decision remains authoritative until explicitly resolved.
- No second tab, Favorite, session, or profile authority inside `bbr-ui` or
  the menu renderer.

## Parallel-lane contract

```text
Base condition:       verified main after F2c → F2b → fresh F2a and settled G1a
Depends on:           G1a production bbr-ui menu primitives; integrated F2
                      wave; existing tab/favorite/folder/controller authority;
                      B2 only for Bookmark/Remove Bookmark rows
Provides/API:         app/controller TabMenuSource + availability matrix;
                      presentation-only bbr-ui TabMenuModel; production menu
                      composition and typed command/native adapters
Exclusive paths:      post-G1a bbr-ui tab/favorite menu modules; corresponding
                      bbr-app sidebar menu composition; focused controller
                      command modules; focused E2E/perf modules
Shared integration points: bbr-app input/focus routing, controller facade,
                      native Share adapter, testkit snapshot, perf registry,
                      Gallery G2/G3 registries, roadmap roll-ups
Integration order:    F2c → F2b → fresh F2a → G1aa → this packet; Gallery
                      stories register with the same production cutover
```

No concurrent writer may own `bbr-ui` menu/tab/favorite components,
sidebar rendering, organization commands, or their Gallery stories.

## Constraints

- The app-owned menu state stays bound to the invoking `TabId`; opening never
  implicitly changes selection or focus domain. `bbr-ui` sees only a plain
  presentation DTO and callbacks, never a domain/controller identifier.
- Every mutation routes through a portable command/controller authority;
  render callbacks dispatch the pre-bound command and never mutate domain
  state directly.
- Same-profile moves preserve the live `WebViewId`; cross-profile moves
  recreate exactly once in the destination data store.
- Submenus preserve source identity across hover, keyboard traversal, edge
  flipping, and dismissal.
- Native Share integration is a macOS adapter edge; portable crates carry no
  AppKit types.
- Chassis menu tokens, stable ids, semantic labels, contrast, reduced motion,
  and disabled/danger states follow DESIGN §11.13.
- No new dependency unless the existing GPUI/gpui-component/native APIs are
  first proven insufficient.
- Files remain below roughly 500 lines. Tests, budgets, sample counts, and
  fast-but-wrong guards are not weakened.

## Evidence

### Unit and contract

- [x] Availability matrix covers inactive/selected, pinned/regular,
      first/middle/last, Favorite/non-Favorite, suspended/live, split member,
      same-/cross-profile destination, and missing capabilities.
- [x] Menu open/dismiss leaves the complete controller snapshot and session
      registry unchanged.
- [x] Each enabled action dispatches exactly one typed command against the
      bound `TabId`; disabled and omitted actions cannot dispatch.
- [ ] Share adapter receives only the bound tab's current link and exposes no
      native type to portable crates.
      (the Share row is omitted until a native Share adapter contract exists;
      unsupported rows never enter the model)

### Real-app E2E

- [x] Open on an inactive tab without selecting it; traverse and dismiss by
      Escape/outside click with no mutation.
- [x] Folder/space submenu keyboard traversal preserves source `TabId`;
      same-profile move preserves `WebViewId`, cross-profile move recreates
      once and preserves isolation.
- [x] Pin, suspend, Favorite toggle, Copy Link, ordering, close,
      and reopen target the source tab rather than the selected neighbor.
      (Duplicate is an unsupported row with no authority and is omitted)
- [x] When B2 is integrated, Bookmark/Remove Bookmark captures the bound
      source's current URL/title through B2 without selecting it, creates no
      duplicate canonical URL, and leaves the tab/Favorite/session unchanged.
- [x] Favorite menu resolves and mutates the same concrete `TabId` as its tab
      row presentation.

### Performance

- [ ] Seed and enforce warm `tab_context_menu_open` and submenu-commit kinds,
      at least 30 accepted full-gate samples each, committed from the semantic
      presentation/result rather than command receipt.
      (kinds seeded/enforced in `Baseline::seed` + baseline JSON + guard test;
      sample execution remains under Patrick's active override — compiled only)

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-domain -p bbr-browser
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-browser --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-shell-organize
nix develop -c just e2e-shell-sidebar
nix develop -c just e2e-shell-tab-menu
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI suites run sequentially under the repository lock.

## Delivery sequence

1. Confirm F2 and G1a are integrated and UI ownership is free.
2. Add the portable availability/command seam and prove no-open mutation.
3. Cut over one shared production Tab/Favorite menu in `bbr-ui`.
4. Add pointer/keyboard/submenu semantics and native Share adaptation.
5. Add real-app and performance evidence, then the real-production Gallery
   stories.
6. One independent read-only review; one remediation writer only for concrete
   findings; focused recheck.

## Iteration policy

Deliver the smallest complete menu with the currently owned actions first.
When a requested row lacks an authority, omit it and report the named contract
dependency rather than inventing behavior or leaving a decorative disabled
row. Diagnose failures as source-identity, availability, command routing,
focus/dismissal, submenu geometry, native Share, or presentation evidence.

## Blocked stop condition

Stop with the preserved worktree and exact missing authority if a row would
require inventing bulk-close boundaries, tab-label/icon persistence, media
capabilities, split/new-window behavior, or extension/container/mobile
features. Do not copy Zen/Firefox behavior to clear the blocker.

## Delivery ledger

```text
Outcome:
  Production Tab/Favorite context menu: app/controller-owned TabMenuSource +
  availability matrix keyed by the exact persistent TabId; presentation-only
  bbr-ui TabMenuModel + tab_menu_card; pointer (secondary-click row/favorite,
  hover, press) and keyboard (Up/Down/Right/Left/Enter/Escape) lifecycle with
  clamped placement and origin-focus restoration; typed actions (Reload,
  Pin/Unpin, Suspend, Copy Link, Move to Start/End, Close, Reopen, Move to
  Folder, Move to Space, Favorite toggle, B2 Bookmark Save/Remove); unsupported
  rows omitted; Gallery story at StoryId 46.
Commits:
  none (lane is uncommitted; Orchestrator integrates the preserved tree)
Functional evidence:
  bbr-browser tab_menu tests (availability/dispatch/no-open): 235 passed;
  bbr-app tab_menu unit tests: 273 passed; bbr-gallery: 145 passed
Real-app E2E:
  e2e-shell-tab-menu 12/12; e2e-shell-organize 2+4; e2e-shell-sidebar 11;
  e2e-gallery 42 (includes every-story sweep over StoryId 46)
Performance evidence:
  kinds seeded+enforced (Baseline::seed + baseline JSON + guard test);
  execution under Patrick's override — compiled only, not run
Roadmap/documentation updates:
  packet evidence checkboxes 8 marked [x] (Share + perf items remain open
  with stated reasons); roadmap graph check valid
Open blockers and non-goals:
  Share row omitted (no native adapter authority); Duplicate/split/media/
  rename/icon/new-window/bulk omitted; no Zen/Firefox feature copies
Worktree state:
  /Users/patwoz/dev/Personal/macos/bbr-dd-tab-menu @ 4d5be311…, uncommitted
  reviewed tree a4f1f6c8… (66 paths), real index unstaged, GUI lock free

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
  Review wave `bbr-daily-driver-final-reviews-mspfijvd-flnwef` findings
  fixed in the lane at reviewed tree a4f1f6c8…/66 paths:
  - F1 production submenu_rect is recomputed from the live window
    geometry/theme whenever a submenu opens or the relevant selection
    changes, so the Folder/Space destination card visibly renders and
    edge-flips (app + state recompute, causal app unit tests, real-app E2E
    geometry/ids assertions);
  - F2 Enter/click on a closed submenu parent opens its submenu card and
    never dispatches a placeholder Reload — the parent row carries no
    action (Option<TabMenuAction>);
  - M1 Copy Link is enabled only when the bound tab owns a live
    dispatchable session (suspended sources are disabled; matrix tests and
    real-app E2E cover the suspended case);
  - M2 the Justfile comment no longer lists the unsupported Duplicate row
    as supported;
  - M3 this ledger's fingerprint truth corrected to the reviewed tree.
Waiting/retries:
Total elapsed:
Human intervention:
```
