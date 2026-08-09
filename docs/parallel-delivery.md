# Parallel delivery and worktree integration

Parent: [Development workflow](development-workflow.md) · [Roadmap](../ROADMAP.md) · [Roadmap maintenance](roadmap/README.md)

BBR phases are product acceptance milestones, not a requirement to implement
all browser capabilities in one serial branch. Parallel work is allowed when
its contracts, ownership, evidence, and integration order are explicit.

## Runtime integration anchor and lane state

This document owns the durable parallel-delivery rules, not a particular
execution wave. The exact verified base SHA, active/frozen lanes, worktrees,
writers, shared integration points, and serial merge order live only in the
[Current delivery](delivery/current.md), reached through [ROADMAP.md](../ROADMAP.md).
Never infer an anchor from a stale worktree, historical example, task-packet
planning SHA, or uncommitted `main`.

A lane is one bounded task packet, one branch, one worktree, and at most one
writer. Multiple phases may have active lanes when their owned files and
contracts do not overlap. Before a product writer starts, the current-delivery
snapshot must record the exact clean base commit, dependencies, provided API,
exclusive paths, shared integration points, and integration order. A
packet-only or research worktree may exist for read-only preparation, but it
makes no product implementation claim and is not an active lane unless the
current-delivery snapshot says so.

## Serial integration kernel

These surfaces have one integration owner at a time. Feature lanes propose
changes in their packet; they do not independently evolve the contract:

- root `Cargo.toml`, `Cargo.lock`, and workspace dependency versions;
- public `bbr-web` actions, events, geometry, and platform adapter contracts;
- `bbr-browser` public controller facade and cross-feature snapshots;
- `bbr-app` composition root and global focus routing;
- testkit protocol version and top-level wire envelope;
- performance baseline policy and canonical scenario registration;
- SQLite migration numbering and domain persistence DTOs;
- the one Current-delivery runtime snapshot and canonical `just e2e` ordering.
  `ROADMAP.md` itself remains the stable entry point/product roll-up and changes
  only for durable pointer, hierarchy, or acceptance updates.

Implementations behind an accepted seam remain lane-owned. Shared-contract
changes land first as small serial slices, after which dependent worktrees
rebase or are recreated from the new clean anchor. Do not solve integration
conflicts by adding compatibility layers or duplicating state.

## Worktree lifecycle

1. Settle every writer on the intended base branch; require clean Git status.
2. Run the anchor's required verification and record the exact commit.
3. Activate a bounded task packet and reserve its exclusive paths.
4. Create `parallel/<lane>` from the recorded commit in a sibling worktree.
5. Keep one writer in that worktree; reviewers remain read-only.
6. Run targeted unit/contract checks freely. Run GUI gates only through the
   host-wide lock described below.
7. Commit coherent, verified groups on the lane branch.
8. Before integration, update onto the latest integration anchor only after
   the lane is settled. Resolve shared-kernel changes in the integration lane,
   not independently in several feature worktrees.
9. Run affected gates after each integration, then one `just verify` and
   canonical `just e2e` after the complete integration wave is stable.
10. Remove a worktree only after its commits and delivery ledger are preserved.

No reset, clean, stash, amend, force-push, or silent ownership replacement is
part of normal handoff or recovery.

## Host-wide GUI gate

AppKit window geometry can interfere across repositories' sibling worktrees,
so per-process sequential execution is insufficient. Every `just e2e-*` and
performance recipe is wrapped by `scripts/with_gui_lock.py`.

The wrapper derives one lock key from `git rev-parse --git-common-dir`, so all
worktrees of this repository serialize on the same advisory lock in `/tmp`.
It records holder PID, Pi session id, and cwd for timeout diagnostics; the OS
releases the lock if the holder exits. Commands retain their own hard timeout
and process-tree cleanup.

Configuration:

```text
BBR_GUI_LOCK_TIMEOUT_SECONDS  wait limit; default 1800 seconds
BBR_GUI_LOCK_DIR              lock directory; default /tmp
BBR_GUI_LOCK_KEY              explicit key for wrapper contract tests only
```

A lock timeout exits 75 and reports the current holder. Do not bypass the lock
for a GUI suite because another worktree appears idle.

## Integration-wave completion

A wave is complete only when:

- every lane has a settled owner and clean or explicitly preserved worktree;
- shared contracts have one integrated definition;
- requirement-specific unit, E2E, and performance evidence is green;
- `just verify` and canonical sequential `just e2e` pass from the integrated
  commit;
- roadmap roll-ups name completed work without claiming unfinished phases;
- the next foundation commit is recorded before another lane branches.
