# Phase 5 — Unified command bar

Parent: [Roadmap](../../../ROADMAP.md) · UI standards: [UI design-system track](../tracks/ui-design-system.md)

## Outcome

One fast, keyboard-first surface handles navigation, search, tab switching, spaces, history, and browser commands.

## Parallel lane

The [parallel-delivery foundation](../tasks/parallel-delivery-foundation.md)
anchored the bounded portable command-core lane. `crates/bbr-command/**` remains
its exclusive portable authority; app composition, loaded-provider wiring,
testkit snapshots, and command-bar GPUI remain app-owned integration points.
The [portable command core](../tasks/command-core.md) is implemented and
verified, and the bounded [command app-composition packet](../tasks/command-composition.md)
was integrated at `872d222ca91395fc6e7e60c98ff942630b74cf6d` after independent
review and remediation. The native Portal host and browser-MVP palette UI are
also integrated. The G3 Command candidate now moves only the floating Cmd-T
palette / Cmd-L overlay presentation into the plain
`bbr_ui::surfaces::command` model, stable ids, geometry, and renderer;
`bbr-app` retains the DTO mapping, command state/ranking/activation, focus,
and Portal host/placement/dismissal. The unchanged real-app Command suite is
25/25 green, and Gallery appends the candidate composition at StoryId 45 in a
46-story catalog. Persistence-backed history remains owned by Phase 7 rather
than this presentation extraction.

## Checklist

- [x] Add portable `bbr-command` registry, parsing, ranking, and result types. *(Implemented by the command-core packet: typed modes/identities, URL-vs-search parsing, owned corpus, deterministic registry, group ranking, typed activations, and the 500-tab/10k-history first-result benchmark.)*
- [x] Implement the full location, new-tab, switcher, and action result modes. *(Portable result modes are complete — mode-aware ranking with the navigate/search fallback; the parenthetical gap “ranking/results do not [exist]” is closed. Location editing, staging, focus, and GPUI wiring remain the app-composition slice.)*
- [x] Select the current URL on `Cmd-L`. *(The app-owned UTF-8 text model selects the full live URL; typing replaces it, repeated Cmd-L preserves the original Escape return transient, and real-app E2E covers web/chrome origins.)*
- [x] Make `Cmd-T` enter a distinct new-tab flow without creating a tab before Enter.
- [x] Rank exact/open tabs, current-space/pinned tabs, other-space tabs, standalone spaces, history, actions, and navigate/search fallback. *(Deterministic Phase-5 group order proven by the ranking contract suite; standalone spaces activate by `SpaceId` and exclude the current space. B2 adds the global Saved Page group exactly between standalone Spaces and History — `ExactTab < CurrentPinned < CurrentTab < OtherSpace < Space < SavedPage < History < Action < Fallback` — consuming only the loaded in-memory provider; no existing group's relative order changes.)*
- [x] Label cross-space results with profile identity. *(Other-space tabs carry profile/space names; standalone spaces use `Space` in the current profile and `Profile <name>` across or without known profile context.)*
- [x] Support arrows, Enter, modified Enter, Escape, and origin-focus restoration. *(Enter routes typed activations through app/controller seams; Cmd-Enter opens exactly one focused destination tab; stale moved-tab results resolve by current persistent identity; Escape restores the captured origin.)*
- [x] Add cancellable debounce without blocking the render path. *(A platform-driven 30 ms GPUI timer cancels stale generations; ranking never runs from `Render::render`; functional E2E has a deterministic typed flush seam.)*
- [x] Design loading, empty, error, and destructive-command states. *(The production Location/NewTab fallback makes no-result unreachable, so no false NoMatch state is retained; unavailable actions expose activation error and Close Tab uses the destructive tint without invented confirmation.)*
- [x] Add table-driven parser/ranking tests and real-app command-bar E2E. *(Portable parser/ranking suites plus the current `just e2e-shell-command` 25/25 cover text/focus, typed states, ranking/result navigation, activation/actions, cross-space/profile identity, History/Saved Pages providers, stale moved-tab activation, address-control coexistence, Portal identity, and truthful failure states.)*
- [x] Keep first local result below 50 ms for 10,000 history entries and 500 tabs. *(Final parser-remediation gate, release mode: 400 samples, p50 0.117000 ms, p95 1.452917 ms, max 1.539583 ms against the strict p95/max <50 ms ceiling; in-process real-app presentation measurement is the app-composition packet's.)*

Phase 8 extends this one provider/composition seam through
[B2 Saved Page actions and command provider](../tasks/saved-pages-actions-command.md)
after [B1](../tasks/saved-pages-portable-persistence.md) integrates. B2 is the
canonical checklist owner: ranking consumes one loaded in-memory snapshot,
while `bbr-command` remains provider-neutral and never queries SQLite.

## Phase exit

- [x] Every implemented command-bar path is keyboard complete, exposed through the typed semantic tree, and visually consistent with Phase 4. *(History remains an intentionally empty real collection until Phase 7.)*
- [x] Focus always returns to the originating web pane or browser control on Escape, while terminal web activations focus the destination page.
