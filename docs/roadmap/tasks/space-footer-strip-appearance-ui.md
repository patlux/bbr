# Task packet — footer space strip, space appearance, and sidebar swipe

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Performance track](../tracks/performance-release.md) ·
[Interaction contract](../../product/browser-interactions.md) ·
[Performance contract](../../product/performance-contract.md) ·
[DESIGN.md](../../../DESIGN.md) §5/§7/§8/§10/§11.3/§11.4/§15.

The portable space-appearance contract (curated theme tokens plus the validated
emoji/glyph `SpaceIdentity` catalog) is delivered but not yet integrated; its
task packet lands with that serial integration and is linked from here then.

Status: **queued post-MVP UI slice.** It follows the browser-MVP closeout and
the [global UI scale pass](global-ui-scale-density.md), and it follows serial
integration of the portable space-appearance contract.

## Product revision context

Patrick reviewed the delivered U2 sidebar and revised the product target: the
footer `[Spaces]` toggle that reveals space rows above Favorites **is not the
intended interaction and is removed**. The footer becomes
`[Downloads icon] [Spaces strip, all remaining width, horizontally scrollable]
[Plus icon]`, where every space is an identity chip from the curated appearance
contract, the active space is highlighted, and clicking a chip performs the
existing warm space switch.

U2 itself is *correct as delivered* against the specification in force at the
time. This packet is a product revision, not a U2 defect, and the U2 assertions
that encode the old `[Spaces]` toggle are revised here rather than treated as
regressions.

Because the strip is also the natural home for the horizontal swipe and for the
space theme color, this packet deliberately merges the three surface-sharing
items from the night queue — footer revision, space-appearance UI, and sidebar
swipe — into one slice instead of three sequential edits of the same footer.

## Outcome

The sidebar footer permanently presents every space as a curated identity chip
in one horizontally scrollable strip. The active space is unmistakable, the
sidebar chrome carries that space's curated theme tint, pointer/keyboard/swipe
all reach the same space-switch authority with unchanged warm-session behavior,
and a direct horizontal swipe interpolates only BBR-owned chrome color while it
is in progress.

## Scope

- **Footer composition:** `bbr-downloads` (fixed), `bbr-space-strip` (flex,
  single line, horizontally scrollable, never wraps or shrinks the fixed
  controls), `bbr-plus` (fixed). Remove the `[Spaces]` toggle control, the
  revealable space-row region, and its reveal state.
- **Identity chips:** one `bbr-space-chip-<n>` per space in canonical order
  with `bbr-space-identity-<n>` rendering the validated emoji/glyph identity
  from the portable appearance contract. Stable accessible names carry space
  and profile names; identity is never color-only. Selected chip is scrolled
  fully into view without reordering.
- **Activation:** pointer click and keyboard activation route through the
  existing space-switch command path — no second switching authority, no
  session recreation, no change to per-space active-tab restoration.
- **Appearance tint:** the active space's curated theme token tints the
  BBR-owned sidebar material through a contrast-safe desaturated chrome
  variant; text/hairline/accent contrast contracts in DESIGN §3 continue to
  hold and are re-verified by the token gate.
- **Swipe:** horizontal trackpad swipe over the sidebar moves to the
  previous/next space in list order. In-progress gesture commits nothing;
  release below threshold restores the source space and emits no switch;
  release beyond threshold commits exactly one switch. Chrome tint may
  interpolate with gesture progress; the native web view is never animated,
  sampled, reparented, or interpolated.
- **Drag target:** dragging a tab onto a chip retains the §10.9 drag-target
  treatment plus the destination profile companion and routes through the
  existing move policy.
- **Assertions/snapshot:** new ids and typed strip/chip/appearance state on the
  snapshot and in the §15 semantic tree; migrate the U2-era spaces/sidebar E2E
  from the toggle model to the permanent strip.
- **Docs:** this packet, DESIGN §11.3/§11.4 (already revised), the interaction
  contract space-switch/drag sections (already revised), and the roadmap
  roll-ups at integration.

## Non-goals

- No new persistence: appearance values are consumed from the integrated
  portable contract. Schema v1→v2 migration remains its own serial decision.
- No space settings page, no color picker, no emoji picker UI (separate queue
  items); this slice consumes already-chosen values and their defaults.
- No change to profile isolation, tab↔space ownership, favorites semantics,
  command surfaces, Portal hosting, or dense-list rendering.
- No page zoom, no web-content tinting, no screenshot/texture composition.
- No new dependency and no gesture framework; use the existing production input
  path.

## Parallel-lane contract

```text
Base commit:          the verified main commit after MVP closeout, the global
                      UI scale slice, and serial integration of the portable
                      space-appearance contract
Depends on:           portable space-appearance contract integrated; U2 sidebar
                      integrated; S1 token table settled
Provides/API:         permanent footer space strip, chip identity/tint
                      presentation, swipe-driven space switching
Exclusive paths:      bbr-app sidebar footer/space presentation modules, their
                      snapshot/semantic exposure, the affected sidebar/spaces
                      E2E suites, and this packet
Shared integration points: testkit snapshot wire fields, perf-matrix scenario
                      registration, DESIGN §11.3/§11.4, interaction contract,
                      roadmap roll-ups
Integration order:    MVP closeout → global UI scale → space-appearance
                      contract → this slice
```

### F2a completion lanes after the verified Tint checkpoint

The remaining product layers start from the same reviewed and frozen checkpoint
`e08cc6f9455b95b03e25ade86aa7b5d518554f40`. They are separate isolated
writer lanes because their product authorities are independent; shared wire,
snapshot, semantics, and E2E registries are planned serial-integration costs.

| Lane | Exclusive authority and primary territory | Performance authority | Integration |
|---|---|---|---|
| Drag-to-chip | `DragTarget::Chip { space: SpaceId }`, production chip-entry/hit path, target visuals and destination-profile companion, dragged-tab identity, and commit through the existing tab-move policy; focused Drag/Organize/Spaces/Sidebar testkit and E2E seams | Extend the existing `tab_move_same_profile` and `tab_move_cross_profile` performance scenarios so their measured production entry is drag-to-chip rather than the direct benchmark command. Keep the existing typed kinds, report keys, fast-but-wrong session/recreation guards, sample counts, and budgets unchanged. This lane does not edit `performance-baseline.json` or invent a second move budget | Review/freeze first; integrate first |
| Swipe Commit/Cancel + `swipe_commit` | Focused swipe state machine, production sidebar wheel/gesture wiring, bounded chrome-only interpolation, commit/cancel snapshot and semantics, Spaces E2E | Sole owner of new typed `SwipeCommit` benchmark kind/request/driver/report key and `performance-baseline.json` seed. At least 30 render-inclusive samples against the unchanged warm 50/100-ms class with an exact destination/selected-chip/session guard | Review/freeze independently; integrate second |

Shared kernel expected in both lanes: focused `bbr-app` sidebar/space composition,
`bbr-testkit` request/snapshot registries, shell snapshot/semantics, and Spaces
E2E registration. The orchestrator merges these as a semantic union in the
fixed order **Drag-to-chip → Swipe+Perf**. Implementations and non-GUI checks
may run concurrently; every GUI, real-app E2E, and performance recipe remains
host-exclusive through the repository GUI lock. No third performance writer is
allowed: `swipe_commit` depends on the real swipe implementation, while drag
reuses and strengthens the settled tab-move performance authority.

## Constraints

- One space-switch authority. The strip, keyboard, and swipe must not fork
  state, ordering, or event emission.
- Warm-switch invariants and existing 50/100 ms budgets, sample counts, and
  fast-but-wrong guards are unchanged; a larger footer never justifies a budget
  change.
- Cross-profile switching keeps the §11.3 command-field profile hint and its
  deterministic interaction-bounded lifetime.
- The strip clips and scrolls; it never wraps, never grows the footer height
  per space count, and never starves `bbr-downloads`/`bbr-plus` or the native
  content geometry.
- Curated identity values only, validated by the portable contract; no free
  emoji entry in this slice and no emoji as a substitute for a missing chrome
  icon.
- Tint is chrome-only, contrast-safe, and derived from tokens; no raw colors,
  no site-adaptive tinting, no gradients.
- Keep new/modified modules below roughly 500 lines; do not materially enlarge
  legacy files.

## Evidence

### Unit and contract

- [ ] Strip composition tests: fixed controls plus flex strip, deterministic
      chip order, overflow scrolling without wrapping, selected chip scrolled
      into view, no toggle state remaining anywhere in the model.
- [ ] Identity/tint tests: only validated curated values render; tinted sidebar
      material still satisfies the §3 contrast contract at standard and
      increased contrast in light and dark.
- [ ] Swipe state machine tests: in-progress emits no switch, below-threshold
      release restores the source with no event, above-threshold release emits
      exactly one switch, and interpolation is bounded to chrome tint.

### Real-app E2E

- [ ] `just e2e-shell-sidebar` proves the revised footer ids, chip activation,
      overflow scrolling, drag-target chips, and the removal of the former
      toggle behavior.
- [ ] `just e2e-shell-spaces` is migrated off the reveal toggle and proves
      pointer, keyboard, and swipe switching with unchanged selected-tab and
      session identity, plus the unchanged cross-profile hint.
- [ ] `just e2e-shell-organize` proves drag-into-space through chips with
      same-profile session preservation and cross-profile single recreation.

### Performance

- [ ] Existing `space_switch` and `space_switch_cross_profile` scenarios stay
      green and unchanged.
- [ ] A swipe-commit scenario measures the committed switch through semantic
      presentation commit with ≥30 samples against the unchanged warm budget
      class, with a guard rejecting a fast-but-wrong commit.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
cargo clippy -p bbr-testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-spaces
just e2e-shell-organize
just e2e-shell-hover
just e2e-perf
just e2e
just roadmap
```

GUI recipes run sequentially under the host-wide lock.

## Delivery sequence

1. Replace the footer composition and remove the toggle model end to end.
2. Add chip identity rendering and the curated tint through central tokens.
3. Add the swipe state machine on the existing input path.
4. Migrate assertions and add the swipe performance scenario.
5. One independent read-only review, one focused remediation for concrete
   findings, then rechecks.

## Iteration policy

Start with the smallest end-to-end path — permanent strip with click switching
and migrated assertions — before tint and swipe. Classify each failure as
composition, token/contrast, gesture threshold, assertion migration, or
performance, and fix the smallest central cause. Never keep a parallel legacy
toggle path "temporarily".

## Blocked stop condition

If the curated tint cannot satisfy the contrast contract, if the strip cannot
hold the fixed controls and native content geometry at compact widths, or if
swipe cannot be expressed on the production input path without weakening
warm-switch budgets, stop with measured evidence, the preserved branch, the
exact blocker, and the smallest product decision required — for example a
narrower tint surface, a chip overflow affordance, or deferring swipe to its
own slice.

## Delivery ledger

```text
Outcome: queued from Patrick's post-U2 product revision; implementation pending
Commits: planning/documentation only
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending; budgets unchanged
Roadmap/documentation updates: DESIGN §11.3/§11.4, interaction contract,
        this packet, roadmap roll-ups, delivery ledger
Open blockers and non-goals: appearance persistence, settings pages, and free
        emoji entry are explicitly out of scope
Worktree state: not created; branches after the appearance-contract integration
```
