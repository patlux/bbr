# Task packet — Sidebar media control

Parents: [Phase 8](../phases/08-daily-driver.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[portable media-session capability contract](media-session-capability-contract.md) ·
[interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.15 ·
[UI library anchor](ui-library-anchor.md) ·
[Gallery browser components](component-gallery-browser-components.md) ·
[Gallery compositions](component-gallery-compositions.md).

Activation condition: complete F2a and G1a plus the integrated media capability
contract.** Documentation only today. It must not interrupt
F2c → F2b → fresh F2a or start before the settled G1a `bbr-ui` anchor.

Stable composition reference, not a feature or visual copy:
`/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-sidebar-media-control.png`.

## Outcome

When at least one actual controllable media session is reported through the
portable capability contract, BBR shows one compact production media card in
the sidebar above the footer hairline and above
`[Downloads] [Space strip] [+]`. The card exposes only advertised controls,
keeps exact tab/space/profile/session identity, consumes no space when absent,
and never controls a stale or different session.

## Scope

- Add a plain presentation-only `MediaCardModel` and callback contract to
  production `bbr-ui`. `bbr-app` resolves the media snapshot's WebViewId
  through the current session registry, enriches it with visible source/profile
  copy, and pre-binds callbacks to the authorized media identity. No WebKit,
  TabId/SpaceId/ProfileId, controller, or session authority enters `bbr-ui`.
- Conditional placement above the footer hairline. No eligible session means
  no card and no reserved gap.
- Card anatomy:
  - title plus source/site metadata and dismiss control;
  - elapsed/duration and determinate scrubber only when duration+seek exist;
  - live/unknown duration without fake total or scrubber;
  - source activation, Previous, Play/Pause, Next, Mute only when advertised;
  - unsupported controls omitted or semantically disabled with reason.
- Dismiss hides the current media identity until that identity changes; it does
  not stop playback. End/invalidation removes the card.
- Multiple sessions: show the most recently user-controlled eligible session
  plus a compact count/switch affordance; never stack cards vertically.
- Source activation switches to the owning profile/space/exact `TabId` without
  copying it. Media commands otherwise do not select or focus the tab.
- Stable ids: `bbr-media-card`, `-title`, `-source`, `-progress`, `-elapsed`,
  `-duration`, `-previous`, `-play-pause`, `-next`, `-mute`, `-dismiss`,
  `-session-count`.
- Add typed semantics/snapshots, deterministic real-WKWebView media E2E,
  optimized `media_control_present` and `media_play_pause_commit` evidence,
  and Gallery G2/G3 stories from the same production component/composition.
- Settled presentation decisions (Patrick, 2026-09-27; Paper board
  “Next · Media card”):
  - the whole title block (favicon, title, source line) is the one source
    activation target;
  - the session-count affordance cycles to the next most recently controlled
    session; it never opens a list or stacks cards;
  - the scrubber is a 3 pt track with no knob at rest; a knob appears on hover
    and keyboard focus only;
  - Previous, Play, Pause, Next, Mute and Unmute are drawn §7 stroke glyphs
    added to the `Icon` catalog with this slice.

## Non-goals

- No UI before the media capability contract is integrated and advertises an
  actual session.
- No WebKit SPI, DOM-heuristic capability invention, decorative Previous/Next,
  fake duration, fake scrubber, or assumed mute/volume support.
- No media-card stack, mini-player overlay over web content, copied Zen blue,
  video download, global OS media-key integration, or page screenshot.
- No session control after stale identity, navigation, reload, crash, close, or
  profile/session replacement.
- No copied Gallery component or fake BrowserController/WebKit story.

## Parallel-lane contract

```text
Base condition:       verified main after F2c → F2b → fresh F2a, settled G1a,
                      and integrated media-session capability contract
Depends on:           media capability API/adapter; G1a bbr-ui foundations and
                      sidebar/footer composition
Provides/API:         production MediaCardModel/component/composition,
                      app selection/dismiss policy, semantic/perf evidence
Exclusive paths:      post-G1a bbr-ui media modules; focused bbr-app sidebar
                      composition/media presentation; focused E2E/perf and
                      Gallery G2/G3 story modules
Shared integration points: app event pump, sidebar focus order/composition,
                      testkit snapshot, perf registry, Gallery registries,
                      Phase 8/UI-track roll-ups
Integration order:    F2c → F2b → fresh F2a → G1aa → media contract → this UI
```

No concurrent writer may own sidebar/footer composition, media components,
media app presentation state, or their Gallery stories.

## Constraints

- Capability-first: every visible/enabled control comes from the exact current
  `MediaSessionSnapshot` capability set.
- The app resolves WebViewId through the current registry before building the
  presentation DTO and again before dispatch. Commands revalidate engine media
  identity/generation; stale commands never fall through to a new session in
  the same tab.
- The card is ordinary GPUI sidebar chrome, not a Portal presentation.
- Card placement never changes the permanent footer order and consumes zero
  height when absent.
- Activating source uses existing exact-TabId cross-space/profile activation
  semantics and visibly presents profile identity when crossing contexts.
- Media commands preserve current selection/focus unless source activation is
  explicitly invoked.
- `material:field`, `elev:1`, `radius:lg`, BBR tokens, `color:accent` progress,
  density/contrast/reduced-motion rules; no copied Zen tint.
- Focus order places media controls before Downloads. Color/motion never carry
  state alone.
- No new dependencies; files below roughly 500 lines; no weakened identity,
  capability, E2E, performance, or sample-count guards.

## Evidence

### Unit and contract

- [ ] Model/visibility matrix covers playing/paused, seekable/not seekable,
      previous/next/mute subsets, unknown/live duration, ended/stale session,
      one/multiple sessions, and dismissed identity.
- [ ] Unsupported controls cannot dispatch; stale identity produces typed
      rejection without affecting another tab/session.
- [ ] Multi-session recency and session-count/switch behavior are deterministic.
- [ ] Source activation resolves exact TabId/SpaceId/ProfileId and commands do
      not otherwise change selection or focus.
- [ ] Card removal restores footer geometry exactly; absent card reserves no
      semantic node or layout height.

### Real-app E2E

- [ ] Deterministic local media fixture causes the card to appear only after an
      actual reported session.
- [ ] Play/Pause, Mute, and Seek work only when supported; unsupported Previous/
      Next are omitted/disabled and no-op.
- [ ] Cross-space/profile source activation selects the exact tab and preserves
      profile isolation; ordinary media commands do not activate it.
- [ ] Dismiss hides until media identity changes; navigation/reload/end/crash/
      close teardown removes or invalidates the card.
- [ ] Multiple sessions show one current card and a count/switch affordance,
      never a vertical stack.
- [ ] Keyboard focus order reaches the card before Downloads; semantic names
      expose source and profile/space identity.

### Performance

- [ ] Seed/enforce `media_control_present` and `media_play_pause_commit`, each
      with at least 30 accepted full-gate samples. Timings start at the app
      command/observation boundary and exclude fixture/network latency.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo test -p bbr-web -p bbr-web-macos -p bbr-browser
CARGO_BUILD_JOBS=4 cargo clippy -p bbr --features testkit --all-targets -- -D warnings
nix develop -c just verify
nix develop -c just e2e-media-contract
nix develop -c just e2e-shell-media
nix develop -c just e2e-shell-sidebar
nix develop -c just e2e-perf
nix develop -c just e2e
nix develop -c just roadmap
```

GUI suites run sequentially under the repository lock.

## Delivery sequence

1. Confirm F2/G1a/media-contract integration and free sidebar/media ownership.
2. Add the plain production model/component and conditional one-card layout.
3. Wire capability-gated commands, dismiss identity, source activation, and
   multi-session selection.
4. Add typed semantics, real-WKWebView E2E, performance evidence, and the same
   production Gallery stories.
5. Independent UI/capability/security read-only review; one remediation writer
   only for concrete findings; focused recheck.

## Iteration policy

Start with one play/pause-capable session and zero-space absence. Add one
capability/state family at a time; never render a control before the contract
advertises it. Diagnose failures as identity, capability, app selection policy,
sidebar layout/focus, lifecycle invalidation, or semantic presentation commit.

## Blocked stop condition

If the integrated public media contract cannot advertise or control a requested
capability, omit that control and stop with the contract evidence. Do not use
SPI, DOM heuristics alone, or decorative controls to match the reference.

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
