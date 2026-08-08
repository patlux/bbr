# Task packet — Portable media-session capability contract

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[native-integration track](../tracks/native-integration.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[sidebar media-control UI](sidebar-media-control.md).

Status: **queued after fresh F2a and G1 as the required contract predecessor
for media UI.** Documentation only today. The contract/native implementation
may be scheduled only when its `bbr-web`/WebKit territory is free; the sidebar
UI remains blocked until this packet is integrated.

The Zen media card is an information-architecture reference only:
`/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-sidebar-media-control.png`.
It does not define BBR capabilities or authorize private APIs.

## Outcome

BBR has a public, portable, capability-first media-session observation and
command contract. It identifies the exact owning tab/space/profile/session,
reports only controls actually supported by public WebKit capabilities or an
isolated content-world bridge, revalidates identity on every command, and
rejects stale commands without controlling another tab. No UI is included.

## Scope

- Define portable engine-facing media identity, observation, capabilities,
  commands, outcomes, and terminal/invalidation events in `bbr-web`.
- Minimum engine snapshot data:
  - opaque media-session identity/generation;
  - owning `WebViewId` without native pointers; `bbr-web` does **not** import
    domain TabId/SpaceId/ProfileId types;
  - title, subtitle/site, favicon identity;
  - playing/paused state;
  - elapsed/duration only when known;
  - volume/muted only when supported;
  - capability set `{play, pause, seek, previous, next, mute}`.
- Define typed commands (`Play`, `Pause`, `Seek`, `Previous`, `Next`, `Mute`/
  `Unmute`) and outcomes (`Applied`, `Unsupported`, `Stale`, `Ended`,
  `NotControllable`, typed adapter failure).
- At the app/controller composition boundary, resolve the reported WebViewId
  through the existing session registry into the current TabId/SpaceId/
  ProfileId for UI and command authorization. Unknown/stale mappings reject;
  this enrichment does not live in `bbr-web` or the native adapter.
- Source the contract only from supported public WebKit APIs or an isolated
  `WKContentWorld::defaultClientWorld` bridge. Record which source is used and
  its lifecycle/teardown behavior.
- Handle two simultaneous sessions, recency/user-control ordering, live or
  unknown duration, navigation, reload, process crash, tab close, profile
  isolation, and session replacement.
- Add deterministic local media fixtures and lower-level/native contract tests.
- Expose only portable types above `bbr-web-macos`; keep all Objective-C/WebKit
  and unsafe code inside the native adapter.

## Non-goals

- No sidebar card or other UI in this packet.
- No WebKit private SPI, private selectors, reverse-engineered framework calls,
  or entitlement tricks.
- No HTML-element heuristics alone as capability authority.
- No assumption that Previous/Next, seek, duration, mute, or volume exists.
- No OS-level global media-key/Now Playing integration unless separately
  contracted.
- No Media Session API feature invention, DRM bypass, media download, page
  script mutation, or cross-profile session control.

## Parallel-lane contract

```text
Base commit:          verified main after F2 closeout; may run after G1 starts
                      only when bbr-web/native adapter/testkit territories are
                      disjoint from active lanes
Depends on:           settled portable identity types and WebEngine/event seams
Provides/API:         MediaSessionSnapshot, MediaCapabilities, MediaCommand,
                      typed outcomes/events, native adapter implementation
Exclusive paths:      focused bbr-web media modules; focused bbr-web-macos
                      public-API adapter; deterministic media fixtures and
                      lower-level/native tests
Shared integration points: WebEngine events/actions, app event pump, testkit
                      wire registration, roadmap/architecture docs
Integration order:    contract/native capability → sidebar media-control UI
```

No concurrent writer may own the same `bbr-web` event/action envelope,
`bbr-web-macos` WebView delegate/content-world bridge, or testkit wire seam.

## Constraints

- Public API only. Any private/SPI dependency is a hard stop, not a fallback.
- Capabilities are explicit facts. Unsupported controls are absent/disabled in
  later UI; they are never simulated from guessed DOM structure.
- Engine commands carry WebViewId + media-session identity/generation and
  revalidate before dispatch. The app additionally revalidates the registry's
  TabId/SpaceId/ProfileId mapping. Stale identity cannot fall through to a
  newer page/session in the same tab.
- Portable crates remain framework-, filesystem-, and unsafe-free.
- Profile isolation is preserved; observations and commands never cross
  `ProfileId` or website data stores.
- Content-world code, if required, is isolated from page globals, removable,
  and non-destructive; page JavaScript cannot forge another tab's identity.
- No new dependency unless public API/source inspection proves it necessary;
  files below roughly 500 lines; no weakened tests or security checks.

## Evidence

### Unit and contract

- [ ] Capability matrix covers every subset relevant to play/pause/seek/
      previous/next/mute and ensures unsupported commands return typed refusal.
- [ ] Stale generation, navigation/reload, ended session, crash, close, and
      WebView replacement reject without affecting another session.
- [ ] Two simultaneous sessions retain independent identity, state, and
      profile isolation; recency ordering is deterministic.
- [ ] Unknown/live duration has no fake total or seek capability.
- [ ] Portable API object-safety/serialization/redaction and no-framework/
      no-unsafe gates pass.

### Real native integration

- [ ] Deterministic local media fixture reports an actual session through the
      production WKWebView path.
- [ ] Supported play/pause/mute/seek commands affect only the bound session;
      unsupported previous/next remain unavailable.
- [ ] Navigation, reload, crash, close, and teardown remove/invalidate the
      session without observer/bridge leaks.
- [ ] Content-world isolation test proves no BBR globals or control channel are
      visible in page world when that bridge is used.

### Performance

- [ ] No UI timing claim here. Observation/command adapter overhead is bounded
      by targeted instrumentation if needed, excluding fixture/network latency;
      user-visible budgets belong to the sidebar UI packet.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-web -p bbr-web-macos
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-web --all-targets -- -D warnings
CARGO_BUILD_JOBS=4 cargo clippy -p bbr-web-macos --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-media-contract
nix develop -c just e2e
nix develop -c just roadmap
```

Any GUI/native suite runs sequentially under the repository lock.

## Delivery sequence

1. Inspect only public WebKit capabilities and choose the smallest defensible
   observation source.
2. Land the portable identity/capability/command contract with fake tests.
3. Implement the isolated macOS adapter and deterministic real-WKWebView
   evidence.
4. Integrate event-pump/testkit seams serially and document implemented facts.
5. Independent architecture/security/native read-only review; one remediation
   writer only for concrete findings; focused recheck.

## Iteration policy

Begin with detection plus Play/Pause if and only if public evidence supports
it. Add each capability independently; never infer one from another. Diagnose
failures as capability source, identity generation, lifecycle invalidation,
content-world isolation, profile isolation, or native teardown.

## Blocked stop condition

If public WebKit APIs and a safe isolated content-world bridge cannot provide a
reliable capability or command, stop with exact API/source/fixture evidence and
leave that capability absent. Never use SPI or DOM heuristics alone to satisfy
the UI plan.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real native integration:
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
