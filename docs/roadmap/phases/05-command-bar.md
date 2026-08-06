# Phase 5 — Unified command bar

Parent: [Roadmap](../../../ROADMAP.md) · UI standards: [UI design-system track](../tracks/ui-design-system.md)

## Outcome

One fast, keyboard-first surface handles navigation, search, tab switching, spaces, history, and browser commands.

## Parallel lane

After the [parallel-delivery foundation](../tasks/parallel-delivery-foundation.md)
lands, activate command core as a bounded lane rooted at that clean commit.
`crates/bbr-command/**` is its exclusive portable surface; app composition,
history-provider wiring, testkit snapshots, and command-bar GPUI remain serial
integration points. Active packet: [portable command core](../tasks/command-core.md),
rooted at the exact foundation anchor and integrated before command UI. The
portable core is implemented and verified; the remaining items below are the
serial app-composition slice.

## Checklist

- [x] Add portable `bbr-command` registry, parsing, ranking, and result types. *(Implemented by the command-core packet: typed modes/identities, URL-vs-search parsing, owned corpus, deterministic registry, group ranking, typed activations, and the 500-tab/10k-history first-result benchmark.)*
- [x] Implement the full location, new-tab, switcher, and action result modes. *(Portable result modes are complete — mode-aware ranking with the navigate/search fallback; the parenthetical gap “ranking/results do not [exist]” is closed. Location editing, staging, focus, and GPUI wiring remain the app-composition slice.)*
- [ ] Select the current URL on `Cmd-L`.
- [x] Make `Cmd-T` enter a distinct new-tab flow without creating a tab before Enter.
- [x] Rank exact/open tabs, current-space/pinned tabs, other-space tabs, standalone spaces, history, actions, and navigate/search fallback. *(Deterministic Phase-5 group order proven by the ranking contract suite; standalone spaces activate by `SpaceId` and exclude the current space.)*
- [x] Label cross-space results with profile identity. *(Other-space tabs carry profile/space names; standalone spaces use `Space` in the current profile and `Profile <name>` across or without known profile context.)*
- [ ] Support arrows, Enter, modified Enter, Escape, and origin-focus restoration.
- [ ] Add cancellable debounce without blocking the render path.
- [ ] Design loading, empty, no-result, error, and destructive-command states.
- [ ] Add table-driven parser/ranking tests and real-app command-bar E2E. *(Table-driven parser/ranking tests ship with the core; real-app command-bar E2E is owned by the app-composition packet.)*
- [x] Keep first local result below 50 ms for 10,000 history entries and 500 tabs. *(Final remediation gate, release mode: 400 samples, p50 0.107792 ms, p95 1.455708 ms, max 1.678958 ms against the strict p95/max <50 ms ceiling; in-process real-app presentation measurement is the app-composition packet's.)*

## Phase exit

- [ ] Every command-bar path is keyboard complete, accessible, and visually consistent with Phase 4.
- [ ] Focus always returns to the originating web pane or browser control.
