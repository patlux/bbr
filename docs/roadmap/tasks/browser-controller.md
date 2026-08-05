# Task packet — Portable BrowserController

Parents: [Phase 2](../phases/02-browser-core.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · Companion: [Fake WebEngine](fake-web-engine.md)

## Objective

Create `crates/bbr-browser`, a platform-neutral controller that owns `bbr-domain::Browser`, maps tabs to disposable web sessions, routes commands, and applies WebEvents.

## Constraints

- No GPUI, AppKit, WebKit, objc2, filesystem, SQLite, or native pointers.
- Depend only on portable crates, initially `bbr-domain` and `bbr-web`.
- Deterministic typed commands and outcomes; no global notification bus.
- Preserve same-profile sessions; recreate across profile boundaries.
- Keep files under roughly 500 lines.

## API slice

- [ ] Define `BrowserController<E: WebEngine>` or an equivalent testable ownership shape.
- [ ] Define `SessionRegistry` with forward and reverse unique mappings.
- [ ] Define focused target for single-pane and split navigation.
- [ ] Define typed controller errors and mutation outcomes/events.
- [ ] Allocate deterministic WebViewIds through an injected generator.
- [ ] Expose read-only domain/session snapshots needed by future UI and E2E.

## First vertical slice

- [ ] Create a profile and initial space through controller setup.
- [ ] Open a tab → domain tab + native/fake session + visible state.
- [ ] Select another tab → previous session hidden/warm, selected session shown/active.
- [ ] Close active tab → session destroyed, domain neighbor activated and shown.
- [ ] Reopen tab → new session created with preserved URL/title identity.
- [ ] Route Back, Forward, Reload, Stop, and Navigate to the active tab.

## Lifecycle and moves

- [ ] Suspend hidden tab → session destroyed, domain remains suspended.
- [ ] Activate suspended/discarded tab → session recreated and shown.
- [ ] Move within one profile → keep the existing session.
- [ ] Move across profiles → destroy and recreate with destination ProfileId.
- [ ] Page crash → mark owning tab discarded and remove/replace session according to policy.

## Split behavior

- [ ] Opening a split activates and shows both pane sessions.
- [ ] Explicit focused pane receives navigation commands.
- [ ] Closing the split hides/warms the non-selected pane without closing its tab.
- [ ] Moving/closing a pane dissolves the split without orphaning sessions.

## Event application

- [ ] Map UrlChanged and TitleChanged to the correct tab.
- [ ] Track loading and history availability outside persistent domain data where appropriate.
- [ ] Handle NavigationFailed and PageCrashed with typed outcomes.
- [ ] Reject or report events for unknown/stale WebViewIds.

## Tests and evidence

- [ ] Use the [fake engine](fake-web-engine.md) for portable controller contract tests.
- [ ] Cover open/select/close/reopen and navigation routing.
- [ ] Cover warm/suspend/discard lifecycle.
- [ ] Cover same-profile and cross-profile moves.
- [ ] Cover split focus and teardown.
- [ ] Assert registry invariants after every mutation.
- [ ] Add a real-app E2E vertical slice that composes the controller in BBR and proves open/select/close plus navigation routing through the production path.
- [ ] Extend the testkit protocol/snapshot/events as needed; missing harness capability is part of this task.
- [ ] Run `just verify`, `just e2e-smoke`, and the new controller/tab E2E scenario.
- [ ] Update `ARCHITECTURE.md`, `E2E_ARCHITECTURE.md`, and roadmap checkboxes.

## Suggested commit groups

1. Add controller crate and session registry.
2. Add open/select/close navigation slice.
3. Add lifecycle and profile-move handling.
4. Add split focus and event application.
5. Update architecture and roadmap status.

## Non-goals

- GPUI rendering.
- Real WKWebView multi-session storage.
- Persistence.
- Timed LRU suspension.
- Command-bar ranking.
