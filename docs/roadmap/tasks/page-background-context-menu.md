# Task packet — Page-background context-menu follow-up

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[BBR context menus over web](bbr-context-menus-over-web.md) ·
[Portal hardening](native-portal-hardening.md) ·
[native-integration track](../tracks/native-integration.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13.

Activation condition: the integrated functionally green F2 checkpoint (per
[Current delivery](../../delivery/current.md): `just verify` and all 27
functional E2E suites green) and free Portal/context-menu ownership.** The F2
startup/release closeout remains deliberately deferred Phase-9 debt: the
consumed reproduction stays red at startup p95/max `535.866375 ms > 500 ms`
and is never treated as passed; this activation does not imply startup or
release performance acceptance. This is the bounded enrichment of the already
integrated BBR page-background menu, not a replacement router.

Stable information-architecture reference, not a feature or visual copy:
`/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-page-context-menu.png`.

## Outcome

A secondary-click on blank/background page content opens the enriched BBR page
menu in the existing one-per-window `NativeAboveWeb` Portal. Link, image,
selection, editable, media, JavaScript-owned, and unclassified WebKit menus
remain native and untouched. Opening or dismissing the BBR menu preserves the
exact tab/session/profile/space/split identity.

## Scope

- Preserve and strengthen the existing conservative ownership router:
  background/page-only hit → BBR; link/image/selection/editable/media/
  JavaScript/unclassified hit → WebKit native.
- Extend the existing production page-menu model from Back/Forward/Reload/
  Copy Link/Open in New Tab with the currently owned commands `Back`,
  `Forward`, `Reload`, and `Copy Page Link`, plus the B2-gated bookmark row.
  `Select All` and `Share…` stay omitted until their typed capabilities
  integrate.
- After integrated [B2 Saved Pages actions](saved-pages-actions-command.md),
  add capability-gated `Bookmark Page` / `Remove Bookmark` rows by invoking
  B2's one app-owned command against the captured focused-page URL/title. This
  packet owns Portal placement/source binding/focus evidence, never Saved Page
  persistence, duplicate policy, or tab-open behavior.
- Add `Save Page As…` only after the file-panel/save contract and native
  adapter are integrated.
- Add `View Page Source` only after a typed capability opens source in a new
  BBR tab.
- Add `Inspect` only under an explicit developer-mode capability; production
  default omits it.
- Preserve one pointer-coordinate conversion into window space, one Portal
  presentation, edge clamping, keyboard traversal, disabled reasons, focus
  restoration, resize/move tracking, and teardown.
- Stable ids: `bbr-page-menu`, `bbr-page-menu-back`, `-forward`, `-reload`,
  `-bookmark` (Bookmark Page/Remove Bookmark from B2 state), `-save`,
  `-select-all`, `-copy-link`, `-share`, `-view-source`, `-inspect`.
- Extend typed routing diagnostics, real-WKWebView E2E, semantic snapshots,
  command-commit performance where new visible commands warrant it, and the
  real-production Gallery G2/G3 menu stories after extraction.

## Non-goals

- No global suppression or replacement of WebKit element menus.
- No interception of page JavaScript context-menu behavior beyond the existing
  conservative ownership policy.
- No bookmark implementation local to this menu, extension rows, block-element
  action, video downloader, Send-to-Mobile, accessibility inspector, or
  screenshot row. Saved Page rows exist only after B2 and dispatch its shared
  command; ambiguous/non-background WebKit menus remain native.
- No private WebKit SPI, screenshot/texture page composition, second Portal
  host, or copied Gallery implementation.

## Parallel-lane contract

```text
Base condition:       verified main at the integrated functionally green F2
                      checkpoint (startup/release closeout remains deferred
                      Phase-9 debt; startup p95/max
                      `535.866375 ms > 500 ms` stays red)
Depends on:           integrated existing context-menu/Portal path; free
                      Portal/context-menu ownership; G1a components if already
                      available (otherwise coordinate extraction, never copy);
                      B2 only for Bookmark/Remove Bookmark rows
Provides/API:         enriched PageMenuModel/actions and stronger typed
                      ownership diagnostics; no ownership-policy reversal
Exclusive paths:      focused page-menu app/portal modules; conservative
                      native classifier/diagnostics only where required;
                      focused E2E/perf and Gallery registry modules
Shared integration points: bbr-web events/actions, bbr-app portal stack/input,
                      testkit wire, perf registry, Gallery G2/G3 registry,
                      Phase 8/UI-track roll-ups
Integration order:    integrated functionally green F2 checkpoint → this
                      packet when Portal territory is free; coordinate with
                      G2 extraction rather than duplicating UI. Full F2
                      closeout remains Phase-9-deferred and is not a
                      precondition of this packet
```

No concurrent writer may own Portal/context-menu rendering, the native menu
classifier, or their testkit/performance registries.

## Constraints

- Existing conservative fallback wins: ambiguity leaves WebKit's native menu
  untouched.
- WebKit element-menu classes remain native: link, image, selection, editable,
  media, JavaScript-owned, and unknown.
- Page commands target the focused pane/current tab captured by the menu
  request; open/dismiss does not change that target.
- Exactly one `NativeAboveWeb` host per browser window; GPUI owns menu state
  and content, AppKit owns only observation/attachment/z-order/frame/hit test.
- Native Share/File-panel integration remains in macOS adapters; portable
  crates import no AppKit/WebKit types.
- No new dependency without evidence; modules stay below roughly 500 lines;
  no weakened classifier, Portal, E2E, performance, or sample-count gates.

## Evidence

### Unit and contract

- [x] Typed ownership cases cover background, link, image, selection,
      editable, media, JavaScript, localized/unrecognized, and empty menus.
      (adapter `menu_classify` unit matrix: background/link/image/selection/
      editable/media/page-JavaScript/localized/empty cases).
- [x] Ambiguous/unrecognized input resolves to WebKit-native without
      cancellation. (adapter unit tests: unknown titles, localized menus, and
      empty menus stay `ElementOwned`).
- [x] Availability matrix covers Back/Forward, focused split pane, missing URL,
      missing Save/Source/Developer capabilities, and disabled reasons.
      (controller unit tests: row model, bookmark gate, unowned-capability
      omission, disabled reasons, first-enabled cursor).
- [x] Open/dismiss preserves browser/session/Portal identity and source pane.
      (dismissal/ownership/navigation E2E `assert_identity_preserved`; the
      non-active-view request is a no-op).

### Real-app E2E

- [x] BBR background menu is frontmost, pointer/keyboard operable, and bound to
      the focused pane over a real WKWebView. (real-app navigation/ownership
      suites: real `canGoBack`/`canGoForward` feed, pointer + keyboard
      activation, portal focus/attachment/frame evidence).
- [x] Link/image/selection/editable/media and page-JavaScript cases retain
      native WebKit behavior. (adapter unit matrix; the real-app bridge test
      proves the BBR open path never touches WebKit element-menu ownership).
- [x] Share/Select All and any integrated Save/Source command target the exact
      page without selecting or recreating another tab. (no capability is
      integrated, so the rows never render; the real-app E2E asserts the
      unowned rows are absent and only owned commands reach the page).
- [x] When B2 is integrated, Bookmark/Remove Bookmark uses the same B2 command
      as Cmd-D and Tab/Favorite menus, preserves the focused page/session, and
      never affects WebKit-native element-menu ownership.
      (real-app `bookmark_row_gates_and_toggles_with_cmd_d_parity`: unique
      unsaved fixture, row direction matches B2 state, Cmd-D parity, native
      counters stay zero).
- [x] Resize, move, dismissal, and teardown leave no visible/attached orphan.
      (real-app resize/repin and dismissal suites).

### Performance

- [ ] Existing seeded/enforced `context_menu_open` evidence remains green.
      (execution blocked by Patrick's active performance override; code,
      budgets, and sample guards are preserved and compile.)
- [ ] Add a command-commit kind only for a newly visible path not already
      covered; at least 30 accepted samples and semantic commit enforcement.
      (the `context_menu_bookmark_commit` kind, budget, driver, and render
      gate are implemented and compile; execution blocked by the override.)

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-web -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-web-macos --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-shell-portal
nix develop -c just e2e-shell-context-menu
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI suites run sequentially under the repository lock.

## Delivery sequence

1. Confirm the integrated functionally green F2 checkpoint and free
   Portal/context-menu ownership (startup/release closeout remains deferred
   Phase-9 debt, never treated as passed).
2. Strengthen typed background-versus-element evidence without changing the
   conservative fallback.
3. Add currently owned page commands to the existing production menu.
4. Add capability-gated Save/Source/Inspect only if their owner has integrated.
5. Add real-app/performance evidence and real-production Gallery stories.
6. Native-boundary read-only review; one remediation writer only for concrete
   findings; focused recheck.

## Iteration policy

First keep the existing background menu behavior byte-correct while expanding
routing fixtures. Then add one owned command at a time. Classify failures as
ownership, event source, coordinate conversion, Portal ordering, focused-pane
routing, native adapter, or presentation evidence. Never broaden cancellation
to make a BBR scenario easier to trigger.

## Blocked stop condition

If WebKit cannot classify an element class reliably with public APIs, preserve
native WebKit behavior and stop with event/menu evidence and the smallest typed
policy decision required. Do not globally suppress context menus or use SPI.

## Delivery ledger

```text
Outcome: Enriched conservative BBR page-background menu: Back/Forward/Reload,
the B2 capability-gated Bookmark Page/Remove Bookmark row (Cmd-D parity), and
Copy Page Link, rendered in the one NativeAboveWeb portal. Conservative
ownership untouched: link/image/selection/editable/media/page-JavaScript/
localized/unknown menus stay WebKit-native (adapter classifier unchanged;
only typed unit evidence added). Real-app context-menu suite green 11/11.
Commits: none (writer branch `parallel/dd-page-menu`; work remains unstaged
per the serial-integration contract; HEAD still the preserved activation
base 4d5be3116ec090aebb7eefa4861970710dba7a5b).
Functional evidence: workspace `cargo check --all-targets` clean;
bbr unit tests 263/263; bbr-ui 122/122; bbr-gallery 144/144; bbr-web +
bbr-web-macos contract tests green; clippy `-D warnings` green for
bbr-web-macos and bbr (testkit) in fresh separate target dirs; `just fmt`
green; `just roadmap` valid.
Real-app E2E: `just e2e-shell-context-menu` 11/11 green (the two causal
findings fixed: keyboard traversal Up-step now expects Bookmark Page then
Reload; bookmark fixture uses a per-run unique unsaved URL and dismisses
through the production close seam before the post-Cmd-D reopen). Gallery
context-menu E2E 2/2 green. `just e2e-shell-portal` 6/7: the sole failure
`hardening::host_hides_on_minimize_and_restores` is an environment blocker
(the macOS screen-lock overlay 69840 intercepts the AppKit hit test and the
foreign qemu VM window 59741 overlaps the centered harness window after
minimize/restore); portal host/frame code is unchanged by this lane, the
failure reproduces identically 3x with the same foreign overlay, and the
other frontmost-evidence portal tests pass.
Performance evidence: `context_menu_bookmark_commit` kind, 50/100 ms budget,
driver, and render gate implemented and compiling; execution blocked by
Patrick's active performance override (no recipes/samples/retries).
Roadmap/documentation updates: task-packet evidence checkboxes completed for
the unit/contract, real-app E2E, and Gallery rows; performance rows stay
unchecked (override); delivery ledger filled.
Open blockers and non-goals: portal-suite minimize/restore test blocked by
the screen-lock overlay + overlapping VM window (unlock the screen, ideally
move/close the qemu window); no Save/Source/Share/Inspect/Select All rows
(non-goal until an owner capability integrates); startup/release closeout
remains deferred Phase-9 debt; performance unexecuted by override.
Worktree state: /Users/patwoz/dev/Personal/macos/bbr-dd-page-menu, branch
`parallel/dd-page-menu`, HEAD 4d5be3116ec090aebb7eefa4861970710dba7a5b,
30 paths (29 modified + 1 untracked page-menu story), index unstaged.

Planning/read-only: authority + packet + suite/ledger review, ~30 min.
Implementation: navigation Up-step fix + bookmark unique-URL/close-seam
fix, ~20 min.
Test execution: fmt, workspace check, unit/contract suites, both clippy
gates, context-menu 11/11, gallery context-menu 2/2, portal 6/7 (env
blocker), roadmap — ~2.5 h incl. builds.
Review: n/a (forward-recovery writer; reviewer scheduled by orchestrator).
Remediation: one focused re-fix of the bookmark reopen (close-then-reopen
through the production seam), then full suite re-run green.
Waiting/retries: portal-suite retries 3x for the environmental failure.
Total elapsed: ~3.5 h.
Human intervention: unlock the macOS screen (and ideally move/close the
overlapping qemu VM window) to re-run the one portal test.
```
