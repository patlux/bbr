# Task packet — Blueprint D02: Hover reveal of a hidden rail

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §2 §11.4 §11.14, §19.

Status: adopted 2026-09-28 (DESIGN §19); implementation position 9 of 24. Paper boards: W-3 · Shell — sidebar hidden ⌘S & hover reveal, V-6 · Motion — transitions (file
“BBR — Browser Shell”).

**Decision and review 2026-09-29.** Patrick rejected a click-only floating
rail: everything in the floating rail must work exactly like the docked
rail. An independent read-only architecture review (gpt-6-astra, 2026-09-29)
compared a second key-capable rail window, a main-window rail under a
masked WKWebView, and an interactive Portal. It recommends the third: the
one Portal becomes an interactive chrome host that may own key focus while
the floating rail is operated. The browser window stays the main/document
owner, and the Portal renders the exact rail composition. `BbrApp` stays
the only authority, and no WKWebView frame changes. The review corrected
three premises of the earlier block note. The Portal has a hit-region mode.
GPUI's panel class answers YES to `canBecomeKeyWindow`. The rail callbacks
capture the app entity and use the event's window. The masked-web-view
variant keeps input simple, but a soft shadow and fade under a masked page
do not match compositing above it.

Parity gaps the review found in the docked rail as well (verified): the
command field has no IME/marked-text input handler (it edits raw key
events), and the tab drag has no folder drop target. Some drop targets also
match on y alone. "Works exactly the same" therefore first needs a shared
command editor with IME/selection and a 2-D drag-target model with a folder
target, implemented once for both rails.

Decided 2026-09-29: Patrick adopted every recommendation below, and the
sidebar toggle moved from ⌘B to ⌘S. The former open decisions (recommendations in brackets): what counts as
leaving the rail (the rail and its menus, panels and drags form one
domain); Escape order (IME, then nested menu, then the rail); whether ⌥⌘S
and ⌘L on a hidden rail dock it or float it (float); whether ⌘S while
floating docks it at the remembered width (yes); a fullscreen hotspot
(narrow native-observed edge); dismissal on deactivate/minimize (yes).
Full review: session artifact `/private/tmp/bbr-d02-astra-review.md`.

**Trigger decision (Patrick, 2026-09-29):** the rail appears at once when
the pointer reaches the left edge moving fast and mostly leftward. A slow
arrival keeps the 300 ms dwell, so a careful click near the page's left edge
does not open it. Proposed values: at least 600 pt/s of horizontal speed
over the last ~80 ms of pointer samples, with the horizontal component
dominant. There is no trigger while a mouse button is held or while a menu
or the palette is open. When a fast pointer overshoots a window that is
not at the screen edge, the rail appears and waits about 1.2 s for the
pointer to come back. Detection needs the pointer samples over the web
view, too; a local mouse-moved monitor (like `portal_hit`) sees them without
taking events from the page. Patrick can try the threshold in the
explainer `~/Downloads/bbr-d02-schwebende-leiste.html`.

## Outcome

A 300 ms dwell at the left edge reveals the rail as a floating panel over the card (elev:2), without reflowing the card; leaving or Escape hides it.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §2 §11.4 §11.14 in the same group and flip the
  D02 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: ⌘S hides the rail; only ⌘S or the toggle bring it back.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [ ] Portable model / controller / component tests for the new behaviour
      and its states.

### Real-app E2E

- [ ] `just e2e-shell-sidebar` covers the flow drawn on the boards through the
      production path with semantic ids and typed snapshots.

### Performance

- [ ] Extend `just e2e-perf` with the presentation scenario for this interaction (performance contract).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D02 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

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
```
