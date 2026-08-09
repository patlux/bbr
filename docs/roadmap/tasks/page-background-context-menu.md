# Task packet — Page-background context-menu follow-up

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[BBR context menus over web](bbr-context-menus-over-web.md) ·
[Portal hardening](native-portal-hardening.md) ·
[native-integration track](../tracks/native-integration.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13.

Activation condition: F2 closeout is complete and Portal/context-menu ownership is
free.** This is the bounded enrichment of the already integrated BBR
page-background menu, not a replacement router.

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
  Copy Link/Open in New Tab with only currently owned commands:
  `Back`, `Forward`, `Reload`, `Select All`, `Copy Page Link`, and `Share…`.
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
Base condition:       verified main after complete F2 closeout
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
Integration order:    F2 closeout → this packet when Portal territory is free;
                      coordinate with G2 extraction rather than duplicating UI
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

- [ ] Typed ownership cases cover background, link, image, selection,
      editable, media, JavaScript, localized/unrecognized, and empty menus.
- [ ] Ambiguous/unrecognized input resolves to WebKit-native without
      cancellation.
- [ ] Availability matrix covers Back/Forward, focused split pane, missing URL,
      missing Save/Source/Developer capabilities, and disabled reasons.
- [ ] Open/dismiss preserves browser/session/Portal identity and source pane.

### Real-app E2E

- [ ] BBR background menu is frontmost, pointer/keyboard operable, and bound to
      the focused pane over a real WKWebView.
- [ ] Link/image/selection/editable/media and page-JavaScript cases retain
      native WebKit behavior.
- [ ] Share/Select All and any integrated Save/Source command target the exact
      page without selecting or recreating another tab.
- [ ] When B2 is integrated, Bookmark/Remove Bookmark uses the same B2 command
      as Cmd-D and Tab/Favorite menus, preserves the focused page/session, and
      never affects WebKit-native element-menu ownership.
- [ ] Resize, move, dismissal, and teardown leave no visible/attached orphan.

### Performance

- [ ] Existing seeded/enforced `context_menu_open` evidence remains green.
- [ ] Add a command-commit kind only for a newly visible path not already
      covered; at least 30 accepted samples and semantic commit enforcement.

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

1. Confirm F2 closeout and free Portal/context-menu ownership.
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
