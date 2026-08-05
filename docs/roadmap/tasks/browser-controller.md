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

- [x] Define `BrowserController<E: WebEngine>` or an equivalent testable ownership shape.
- [x] Define `SessionRegistry` with forward and reverse unique mappings.
- [x] Define focused target for single-pane and split navigation.
- [x] Define typed controller errors and mutation outcomes/events.
- [x] Allocate deterministic WebViewIds through an injected generator.
- [x] Expose read-only domain/session snapshots needed by future UI and E2E.

## First vertical slice

- [x] Create a profile and initial space through controller setup.
- [x] Open a tab → domain tab + native/fake session + visible state.
- [x] Select another tab → previous session hidden/warm, selected session shown/active.
- [x] Close active tab → session destroyed, domain neighbor activated and shown.
- [x] Reopen tab → new session created with preserved URL/title identity.
- [x] Route Back, Forward, Reload, Stop, and Navigate to the active tab.

## Lifecycle and moves

- [x] Suspend hidden tab → session destroyed, domain remains suspended.
- [x] Activate suspended/discarded tab → session recreated and shown.
- [x] Move within one profile → keep the existing session.
- [x] Move across profiles → destroy and recreate with destination ProfileId.
- [x] Page crash → mark owning tab discarded and remove/replace session according to policy.

## Split behavior

- [x] Opening a split activates and shows both pane sessions.
- [x] Explicit focused pane receives navigation commands.
- [x] Closing the split hides/warms the non-selected pane without closing its tab.
- [x] Moving/closing a pane dissolves the split without orphaning sessions.

## Event application

- [x] Map UrlChanged and TitleChanged to the correct tab.
- [x] Track loading and history availability outside persistent domain data where appropriate.
- [x] Handle NavigationFailed and PageCrashed with typed outcomes.
- [x] Reject or report events for unknown/stale WebViewIds.

## Tests and evidence

- [x] Use the [fake engine](fake-web-engine.md) for portable controller contract tests.
- [x] Cover open/select/close/reopen and navigation routing.
- [x] Cover warm/suspend/discard lifecycle.
- [x] Cover same-profile and cross-profile moves.
- [x] Cover split focus and teardown.
- [x] Assert registry invariants after every mutation.
- [ ] Add a real-app E2E vertical slice that composes the controller in BBR and proves open/select/close plus navigation routing through the production path.
- [ ] Extend the testkit protocol/snapshot/events as needed; missing harness capability is part of this task.
- [ ] Run `just verify`, `just e2e-smoke`, and the new controller/tab E2E scenario.
- [x] Update `ARCHITECTURE.md`, `E2E_ARCHITECTURE.md`, and roadmap checkboxes.

Evidence so far (framework-free): `just verify` green, `just e2e-smoke` green,
46 bbr-browser tests; 28 controller behavior tests plus one snapshot test drive the fake engine through
the same `Rc<RefCell<dyn WebEngine>>` seam the app will use. The three
unchecked evidence items require wiring the controller into the real app via
the Phase 3 multi-session native adapter; they stay open until then.

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
