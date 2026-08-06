# Parallel delivery and worktree integration

Parent: [Development workflow](development-workflow.md) · [Roadmap](../ROADMAP.md) · [Roadmap maintenance](roadmap/README.md)

BBR phases are product acceptance milestones, not a requirement to implement
all browser capabilities in one serial branch. Parallel work is allowed when
its contracts, ownership, evidence, and integration order are explicit.

## Stable integration anchor

The Phase-4 cross-context slice settled on clean `main` at `adb9a5c` after its
writer and independent reviewer completed. It is the historical input to the
parallel-delivery foundation, not a branch from which new feature worktrees
should be created after that foundation lands.

A new feature worktree must name the **current clean foundation commit** in its
task packet and branch from that exact commit. The foundation delivery ledger
records that commit after verification. Never infer the anchor from a stale
worktree or from an uncommitted `main`.

The existing `/Users/patwoz/dev/Personal/macos/bbr-url-input` worktree on
`parallel/url-input` predates the foundation (`5fcebb8`). It remains preserved
and is not an active lane until an explicit owner settles or hands it off and a
separate integration decision records how its work is carried forward.

## Execution lanes

A lane is one bounded task packet, one branch, one worktree, and at most one
writer. Multiple phases may have active lanes when their owned files and
contracts do not overlap.

| Lane | Canonical owner | Parallel-safe owned surface | Shared integration point | State |
|---|---|---|---|---|
| Parallel-delivery foundation | [task packet](roadmap/tasks/parallel-delivery-foundation.md) | workflow docs, portable crate landing zones, GUI lock | workspace manifest, `justfile`, `ROADMAP.md` | active until its completion gates pass |
| Command core | [Phase 5](roadmap/phases/05-command-bar.md) | `crates/bbr-command/**`, portable parser/ranking tests and benchmarks | app composition, history provider, command snapshot | ready for a bounded task packet after foundation |
| Persistence core | [Phase 7](roadmap/phases/07-persistence.md) | `crates/bbr-persistence/**`, schema/repository/migration fixtures | domain export/import DTOs, app startup/shutdown | ready for a bounded task packet after foundation |
| Split core | [Phase 6](roadmap/phases/06-split-view.md) | new split-focused domain/controller modules and tests | `WebEngine` geometry, controller facade, app composition | blocked on a serial contract-extraction packet |
| Performance runner | [performance track](roadmap/tracks/performance-release.md) | runner provisioning, accepted metadata/trends | canonical baseline acceptance | externally provisioned; does not block portable core lanes |
| Phase-8 WebKit capabilities | [Phase 8](roadmap/phases/08-daily-driver.md) | one native adapter module per accepted capability contract | `bbr-web` events/actions, testkit wire, app surfaces | blocked on serial capability contracts |

A lane marked “ready” still needs its own bounded task packet before a branch or
writer starts. The packet must state base commit, dependencies, provided API,
exclusive paths, shared integration points, E2E/performance evidence, and merge
order.

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
- `ROADMAP.md` current-focus roll-ups and canonical `just e2e` ordering.

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
