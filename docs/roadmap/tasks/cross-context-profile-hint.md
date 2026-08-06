# Task packet — cross-context profile identity for space switching

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Accessibility track](../tracks/accessibility.md) · [Performance track](../tracks/performance-release.md) · [Performance contract](../../product/performance-contract.md) · [DESIGN.md](../../../DESIGN.md) §11.3/§12/§15.

## Outcome

After a successful cross-profile space activation, the destination profile
appears at the top of the command field for one interaction as a §12 chip plus
a `type:label` hint (`"in Profile · <profile>"`), so a cross-profile
destination is unmistakable without relying on color. Same-profile space
switching never announces a profile transition. The hint renders through the
production command-field path, is semantically exposed with a stable id and
typed snapshot data, preserves focus and native session/profile isolation,
and clears at a deterministic user-interaction boundary (the next chrome
context mutation: another space activation, a tab mutation, or a command-bar
mode entry). It is a distinct transient from the `MoveTab` staged
`command_bar_hint`; the two never coexist.

## Scope

- The transient state model in `BbrApp` (`command_profile_hint`), set only by
  a cross-profile `activate_space` and cleared by the deterministic boundary.
- Rendering through the production command-field component (`sidebar_view`),
  reusing the §12 `profile_chip` component with a stable `bbr-command-profile-hint`
  container id and a `type:label` "in Profile · <title>" text.
- Typed snapshot exposure: `command_profile_hint` on the wire snapshot plus
  `bbr-command-profile-hint`/`bbr-command-profile-chip` ids in `visible_ids`
  and the §15 semantic tree (role `Chip` under the command field).
- Real-app E2E for pointer and keyboard cross-profile switching, hint
  copy/identity/lifetime, same-profile absence, active space/tab/WebView/
  session/focus invariants, and the empty-space branch.
- An optimized `space_switch_cross_profile` performance scenario (view-only
  warm switching across profiles with the hint presented; fast-but-wrong
  guards reject missing/invented hints) in the gate-enforced perf matrix.
- Task packet, roadmap/phase/track, architecture, E2E-architecture, and
  interaction-contract documentation updates.

## Non-goals

- No generic notification/toast framework (no Phase-5 results surface, no
  attention-pulse machinery).
- No change to `MoveTab` staged hints (the existing `command_bar_hint` keeps
  its behavior and its `→ Space` / `in Profile · … → Space` copy).
- No recreation policy change: cross-profile space activation remains a
  view-only warm switch; recreation stays reserved for tabs that actually
  move across profiles.
- No timed expiry (a timer would be nondeterministic and untestable without
  sleeps); lifetime is interaction-driven by design.
- No platform Accessibility exposure (GPUI 0.2.2 has no layer; Phase 9).

## Constraints

- Portable crates (`bbr-web`, `bbr-domain`, `bbr-browser`) stay untouched
  except where a typed value must cross the wire — the hint is app-layer
  presentation state derived from the controller snapshot.
- Same-profile switching must not set the hint; an existing hint must clear.
- The hint never weakens the 50/100 ms warm budgets, sample counts, or
  baseline enforcement; the cross-profile scenario is the same warm
  view-only presentation class as `space_switch`.
- GUI suites run sequentially; no sleeps or pixel assertions; deterministic
  fixtures, typed snapshots, and deadlines throughout.
- The hint is present only while it matches the active profile (any context
  mutation that changes the active profile re-sets it).

## Evidence

### Unit and contract

- [x] App-level unit tests for hint derivation (cross-profile sets, same-profile
      clears, lifetime boundary clears, distinct from `command_bar_hint`).
      *(The app derives the hint exclusively in `activate_space` from the
      source/destination profile comparison; the lifetime boundary (tab
      mutation, command-bar mode entry, staged move) is the single
      `clear_command_profile_hint` seam. The real-app E2E asserts the
      derivation and both clear boundaries; the snapshot `visible_ids` and
      semantic-tree unit tests assert the typed exposure.)*
- [x] Semantic-tree unit tests for the `bbr-command-profile-hint` node.
      *(`cross_context_profile_hint_exposes_chip_and_label_under_the_field`)*
- [x] Snapshot `visible_ids` derivation includes the hint ids only when shown.
      *(`visible_ids_expose_the_cross_context_profile_hint_when_shown`)*

### Real-app E2E

- [x] Pointer suite (`just e2e-shell-spaces`): cross-profile activation shows
      the hint with copy `in Profile · <title>`, the chip + hint ids, typed
      semantic node, chrome focus preserved, sessions/WebView unchanged; the
      hint clears on the next interaction; same-profile activation shows no
      hint; MoveTab staging is a separate hint.
      *(`cross_profile_switch_shows_and_clears_the_command_field_profile_hint`)*
- [x] Keyboard suite (`just e2e-shell-spaces` second binary): `↑`/`↓`
      traversal across profiles sets/clears the hint deterministically.
      *(`keyboard_cross_profile_traversal_sets_and_clears_the_profile_hint`)*

### Performance

- [x] `space_switch_cross_profile` scenario in the gate-enforced matrix:
      30 samples of alternating cross-profile activation with the hint
      asserted in the driver snapshot and guarded in-app; 50/100 ms warm
      budget, same session count across the alternation.
      *(Measured p95 ≈ 0.195 ms / max ≈ 0.48 ms against the 50/100 ms warm
      budget; 30 raw samples recorded.)*

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. One exclusive writer implements and verifies the bounded slice.
2. One independent read-only reviewer maps requirements to evidence.
3. Run one remediation writer only for concrete findings, followed by a focused
   recheck.

## Iteration policy

Start with the smallest end-to-end production path: set the transient in
`activate_space`, render it in the command field, expose it typed, then add
the E2E and the performance scenario. After each failing check, identify the
mechanism before changing the implementation; run targeted feedback while
iterating and the canonical sequential gate once the slice is stable. Do not
broaden scope to solve unrelated findings.

## Blocked stop condition

If no defensible path remains, stop with evidence gathered, approaches
attempted, preserved worktree state and owner, the exact blocker, and the
smallest contract decision or input required next (e.g. a DESIGN §11.3
lifetime ruling that contradicts the interaction-driven boundary).

## Delivery ledger

```text
Outcome:       §11.3 cross-context command-field profile hint, one-interaction lifetime.
Commits:       (recorded on completion)
Functional evidence:  unit + semantic-tree + snapshot tests.
Real-app E2E:  just e2e-shell-spaces (pointer + keyboard suites).
Performance evidence: space_switch_cross_profile in the gate matrix (p95 ≈ 0.195 ms).
Roadmap/documentation updates: ROADMAP, phase 04, ui/accessibility/testing/
               performance tracks, ARCHITECTURE, E2E_ARCHITECTURE,
               browser-interactions, task packet.
Open blockers and non-goals: no timed expiry, no generic notifications,
               no platform Accessibility (Phase 9), canonical dense
               frame-time evidence and Phase-7 persistence remain open.
Worktree state: main @ f570a7c or later, single writer.
```
