# Task packet — Folder membership pins tabs

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · [DESIGN.md](../../../DESIGN.md).

## Outcome

Folder membership and pinned-tab state express one product concept end to end: every tab shown inside a folder is pinned, loose pinned rows and folder rows form one organized area, and no lifecycle, command, drag, menu, restore, or dense-session path can leave an unpinned folder member.

## Scope

- Establish one portable invariant: `folder member => pinned` for every tab identity.
- Route folder insertion, removal/ungrouping, explicit unpinning, drag/drop, keyboard organization, reconciliation, and restore through one transition matrix. Moving a tab into a folder pins it atomically; removing a member from a folder returns it to the loose pinned rows; explicitly unpinning a member removes its folder membership atomically. Container deletion is settled as Unpack (Patrick, 2026-09-27): the [Folder-menu packet](folder-context-menu.md) adds the row and consumes the ungroup transition; no folder row closes tabs.
- Keep loose pinned rows ordered before folder rows inside the common organized area; this packet does not introduce arbitrary mixed row ordering or folder nesting.
- Make lifecycle budgeting preserve every folder member through the same pinned authority rather than a UI-only exception.
- Add lower-level invariant tests, production-path typed snapshots, real-app organization/dense/persistence E2E, and the matching roadmap/architecture updates.

## Non-goals

- No visual redesign, new drag affordance, context-menu expansion, destructive folder-delete decision, nested folder model, folder persistence redesign, Split work, media work, or unrelated tab-order change.
- No compatibility layer for states that violate the current invariant; invalid restored/session-local membership is reconciled through the one canonical transition policy.
- The preserved Paper design branch `design/browser-shell-paper` remains design evidence, not implementation authority for this slice.

## Parallel-lane contract

```text
Base condition:          current delivery explicitly activates the slice from a clean verified main
Depends on:              integrated Sidebar reference polish and existing organization controller
Provides/API:            one folder-member/pinned transition authority and invariant
Exclusive paths:         bbr-domain tab/folder invariants; bbr-browser organization transitions
Shared integration points: bbr-app drag/menu/testkit wiring, persistence projection, Sidebar snapshots, organize/dense/persistence E2E, DESIGN.md
Integration order:       before any later Folder-menu behavior that assumes grouped pinned tabs
```

## Constraints

- `TabId`, `SpaceId`, `ProfileId`, native `WebViewId`, profile isolation, active selection, and live-session identity remain unchanged by grouping alone.
- The domain/browser layer owns the invariant; `bbr-ui`, Gallery, and snapshots may present it but may not repair or duplicate it.
- Same-profile and cross-profile tab moves retain their existing session-reuse/recreation rules.
- No weakened dense-session cap, restore validation, E2E assertion, performance budget, or architecture boundary.

## Evidence

### Unit and contract

- [x] Folder insertion pins atomically; explicit ungroup returns members to loose pinned rows; explicit unpin removes membership; failed transitions leave both authorities unchanged. Folder deletion (Unpack) is left to the folder-menu packet, which consumes the ungroup transition.
- [x] Reconciliation and restore cannot produce an unpinned folder member, including unknown/deleted tabs and cross-space rejection paths.
- [x] Session budgeting protects folder members because their canonical tab state is pinned, with no UI- or folder-store-only keep exception.

### Real-app E2E

- [x] `just e2e-shell-organize` proves pointer and keyboard grouping, ungrouping, unpinning, legal/illegal boundaries, focus, exact identities, and profile isolation through the production path.
- [x] `just e2e-shell-dense` proves folder members remain live under the bounded-session policy and exact row/window geometry remains unchanged.
- [x] `just e2e-shell-persistence` proves the invariant after graceful and force-quit restore without eager session creation.

### Performance

- [ ] Every affected optimized organization/lifecycle scenario retains its existing budget and semantic presentation commit; canonical execution is required when the current delivery removes the performance override.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-dense
just e2e-shell-persistence
just e2e-perf
just e2e
just roadmap
```

GUI suites run sequentially through the repository lock. If the active delivery still forbids performance execution, the slice cannot be marked complete; it remains open rather than weakening this packet.

## Delivery sequence

1. Freeze the transition matrix in portable tests before changing production wiring.
2. Implement the invariant in the smallest domain/browser authority and route every existing mutation through it.
3. Extend typed snapshots and real-app E2E for organization, dense lifecycle, and restore.
4. Run one independent read-only review; remediate only concrete findings; recheck the exact candidate before integration.

## Iteration policy

After each change, search every folder-membership and pin/unpin mutation and map it to the canonical transition authority. Run the smallest causal unit/contract test first, then the affected real-app suite. If a path can still create split-brain state, fix the owning portable authority rather than adding presentation repair.

## Blocked stop condition

Stop fail-closed if the transition matrix conflicts with an existing explicit product decision, if restore data cannot be reconciled without deleting unrelated browser state, if profile/session identity would change merely from grouping, or if a required gate cannot run without an external unblocker. Preserve exact branch/tree/status fingerprints, evidence, attempted paths, the unresolved transition, and the smallest decision or environment change needed.

## Delivery ledger

```text
Outcome: `folder member => pinned` holds end to end. FolderStore owns the one
  grouping transition (validate, then pin in the Browser, then record the
  membership); reconciliation drops unpinned members; the checker detects an
  unpinned member; lifecycle budgeting keeps members through their pinned
  state with no folder-only exception. Presentation is unchanged by design:
  the loose pinned block still also lists grouped tabs (follow-up item in
  the UI design-system track).
Commits: this implementation commit on main (after 90eaf4f7).
Functional evidence: bbr-domain 127 tests (grouping pins atomically, the
  membership primitive rejects unpinned facts, rejected groupings leave
  browser and store unchanged, ungroup keeps the pin, unpin ends
  membership); bbr-browser 265 tests incl. 26 organization tests (pin
  index reporting, id-keyed grouping in a non-active space, controller
  unpin ends membership, budget spares the LRU member). All new tests were
  run against a simulated pre-slice store (no pin, pinned ignored): 4
  domain, 5 browser and 3 E2E tests failed.
Real-app E2E: e2e-shell-organize 2/2 + drag-chip 4/4 + new folder-pins 1/1
  (pointer and keyboard Tab-menu grouping with TabPinned evidence, focus,
  ungroup through the testkit organization seam because no user entry
  exists before the folder menu, keyboard and pin-control unpin, unknown
  folder, cross-space and cross-profile rejection, exact TabId/WebViewId/
  space/profile); e2e-shell-dense 3/3 + edge 1/1 (the LRU member stays warm,
  the next LRU tab is suspended, window bounded); e2e-shell-persistence 6/6
  (graceful and force-quit restore keep both grouping pins, no restored
  membership, no eager session); sidebar 15/15 and tab-menu 12/12 assert
  the pin; full `just e2e` aggregate 30/30 in 304 s, plus e2e-peek 7/7 and
  e2e-gallery 45/45.
Performance evidence: open. The performance-execution override forbids
  running `just e2e-perf`; static check only: the U2 perf driver groups
  tab 0, which stays at index 0 when pinned, and the following dense seed
  pins 0..10 anyway, so the scenario state stays coherent.
Roadmap/documentation updates: DESIGN §11.11 and §17.15, Phase 4 prose, the
  UI design-system track (folder-menu deletion wording, loose-pinned
  presentation follow-up), current delivery, ROADMAP.
Open blockers and non-goals: performance gate blocked by the override;
  loose-pinned-block deduplication and the folder menu stay separate.
Worktree state: main, serial writer; the e2e_shell_hover dark-hover race
  (row 0 still loading) got a title wait in a separate test commit.

Planning/read-only: ~25 min (estimate)
Implementation: ~60 min (estimate)
Test execution: unit/contract seconds; targeted E2E ~1 min; affected suites
  ~6 min; two full aggregates 316 s and 304 s (measured)
Review: orchestrator self-review; no independent subagent review without
  Patrick's opt-in
Remediation: none
Waiting/retries: one hover-suite race diagnosed and synchronized
Total elapsed: ~2 h (estimate)
Human intervention: none
```
