# Task packet — Blueprint D13: Open-in-app, pop-up and link-routing UI

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.13 §18.4, §19.

Status: implemented 2026-09-30 (DESIGN §19); position 20 of 24. Paper boards: M-12 · Open in another app, M-13 · Pop-ups & new windows, PR-04 · Open links — foreground, background, Peek, external, pop-up, PR-20 · Links from other apps (file
“BBR — Browser Shell”).

## Outcome

An open-in-app sheet with 'always allow', a no-app banner, a pop-up notice with Open / Always Allow, a small pop-up window for sized sign-in pop-ups, and an optional space chooser for external links.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.13 §18.4 in the same group and flip the
  D13 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: External schemes open known apps directly; blocked pop-ups and unknown schemes give no feedback; links from other apps always open in the last space.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] Unit/contract: `site_permissions` pop-up and app allowances round trip, `preferences` external-links key, `open_app::scheme_of`, `bbr-web-macos` `external_schemes::renders_in_browser`.

### Real-app E2E

- [x] `just e2e-link-destinations` (`e2e_link_destinations.rs`, real WKWebView): load-time pop-up blocked with the banner, Open and Always Allow, allowed pop-up opening through the WebKit delegate, site-settings Pop-ups row; a page going to an unhandled scheme shows the no-app banner, `mailto:` asks naming the app and site, Cancel, Always allow + Open (recorded, never launched), direct hand-over; links from other apps open in the last active space, Ask shows the chooser, ↓ Return opens in Work, Escape drops; new-window, external-url, background-links, errors, gestures, persistence, smoke, settings and permissions suites green.

### Performance

- [ ] Extend `just e2e-perf` with the banner, sheet and chooser presentation scenarios (deferred).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-link-destinations
just e2e-external-url
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D13 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                Blocked pop-up banner with Open / Always Allow, open-in-app sheet and no-app banner through WebKit navigation policy, space chooser for links from other apps (Settings › General).
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    Unit/contract: `site_permissions` pop-up and app allowances round trip, `preferences` external-links key, `open_app::scheme_of`, `bbr-web-macos` `external_schemes::renders_in_browser`.
Real-app E2E:           `just e2e-link-destinations` (`e2e_link_destinations.rs`, real WKWebView): load-time pop-up blocked with the banner, Open and Always Allow, allowed pop-up opening through the WebKit delegate, site-settings Pop-ups row; a page going to an unhandled scheme shows the no-app banner, `mailto:` asks naming the app and site, Cancel, Always allow + Open (recorded, never launched), direct hand-over; links from other apps open in the last active space, Ask shows the chooser, ↓ Return opens in Work, Escape drops; new-window, external-url, background-links, errors, gestures, persistence, smoke, settings and permissions suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.13 blocked pop-ups, links to/from other apps, §11.18 General row, §19 D13 row.
Open blockers and non-goals: the small pop-up window for sized sign-in pop-ups is not built — WebKit keeps a pop-up's opener link only when the browser returns a web view made from WebKit's configuration, and BBR's sessions are controller-owned tabs in one window; it needs an opener-linked session contract and a second window (with D19 window lifecycle). The testkit build records links handed to other apps instead of launching them. Performance scenarios deferred
Worktree state:         main, clean after commit
```
