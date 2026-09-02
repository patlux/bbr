# Task packet — Folder membership pins tabs

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Browser orchestration track](../tracks/browser-orchestration.md) · [DESIGN.md](../../../DESIGN.md).

## Outcome

Folder membership and pinned-tab state express one product concept end to end: every tab shown inside a folder is pinned, loose pinned rows and folder rows form one organized area, and no lifecycle, command, drag, menu, restore, or dense-session path can leave an unpinned folder member.

## Scope

- Establish one portable invariant: `folder member => pinned` for every tab identity.
- Route folder insertion, removal/ungrouping, explicit unpinning, drag/drop, keyboard organization, reconciliation, and restore through one transition matrix. Moving a tab into a folder pins it atomically; removing a member from a folder returns it to the loose pinned rows; explicitly unpinning a member removes its folder membership atomically. The separate Folder-menu packet must settle container-delete semantics before deletion may consume this transition authority.
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

- [ ] Folder insertion pins atomically; explicit ungroup returns members to loose pinned rows; explicit unpin removes membership; failed transitions leave both authorities unchanged. Folder deletion remains unavailable until its own packet settles retain/close semantics.
- [ ] Reconciliation and restore cannot produce an unpinned folder member, including unknown/deleted tabs and cross-space rejection paths.
- [ ] Session budgeting protects folder members because their canonical tab state is pinned, with no UI- or folder-store-only keep exception.

### Real-app E2E

- [ ] `just e2e-shell-organize` proves pointer and keyboard grouping, ungrouping, unpinning, legal/illegal boundaries, focus, exact identities, and profile isolation through the production path.
- [ ] `just e2e-shell-dense` proves folder members remain live under the bounded-session policy and exact row/window geometry remains unchanged.
- [ ] `just e2e-shell-persistence` proves the invariant after graceful and force-quit restore without eager session creation.

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
