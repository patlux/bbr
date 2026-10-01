# Host GUI/GPU gate

## Contract and scope

`gate::Options::host()` selects `/tmp/gpui-visual-host-<effective-uid>.lock`
through the canonical `/tmp` directory, independent of repository/worktree and
`TMPDIR`. Participating same-user consumers must share it, including offscreen
Metal. This is cooperative local filesystem exclusion, not a sandbox or a lock
against unrelated applications, other users or escaped/daemonized producers.

`capture::capture_guarded` holds the gate across both environment probes, capture,
validation and process-group cleanup. CLI `check --host-gate` uses it. Portable
contract fixtures may use the explicit unguarded API; native/GUI/GPU consumer
adapters must opt into guarded capture when adopted. No real host gate has been
created/acquired/recovered by this implementation's verification.

## Lifetime and crash safety

- Acquire Unix `flock` on a stable inode; refuse symlinks, hard links, non-regular
  files, foreign ownership, writable-by-others files and replacement while held.
  Never unlink a lock. Open is nonblocking even for malformed special files.
- The primary file stores strict schema-1 evidence: active state, owner PID,
  pending spawn intent, observed process groups and compatibility paths.
- Persist an active lease before production; persist pending intent before spawn
  and PGID immediately afterward. The spawn-to-PGID-write window is still active
  and pending, so a crash cannot silently allow another guarded producer.
- `Gate::execute` clears pending/PGIDs only after the bounded runner proves group
  cleanup. Ordinary timeout, nonzero exit and cancellation can leave a clean
  host; failed cleanup or evidence writes poison the lease.
- Only explicit `finish` makes a proven-clean lease idle. Drop/panic/SIGKILL leave
  the active marker, even though the OS releases flock. New guarded capture fails
  closed on that marker; it never guesses that a dead/old PID is enough.
- Recovery verifies the exact evidence SHA-256 while holding all recorded gates,
  requires a dead owner and absent recorded PGIDs, and retains hash/reason in the
  next record. Invalid/malformed evidence, EPERM, live/reused PIDs and live groups
  refuse recovery. No process is killed by recovery.

This is interruption safety, not a power-loss durability guarantee: filesystem
sync is used, but hardware/filesystem crash behavior and escaped groups are not
proven. Missing/unrecorded descendants require actual external inspection.

## Existing consumer gates

`--compatibility-gate <absolute-path>` also acquires an existing consumer flock,
without changing its metadata. Primary host lock is always first, then unique
compatibility paths in sorted order. Do **not** wrap this command in that same
legacy lock: it is non-reentrant. Paths may not alias the primary, each other or
symlinks. The library offers the same list on `Options`.

This bridges overlap only while both locks are held. An old caller that checks
only its legacy flock cannot see the persistent primary lease after a crashed
holder releases OS locks. Therefore full mixed-caller crash exclusion still needs
consumer integration/ownership coordination; it is not claimed by these library
tests. BBR's current Python gate remains untouched, and no new native capture is
authorized by this change.

## Explicit recovery

No automatic stale-lock removal or implicit retry. Recovery is currently a Rust
API, not a CLI convenience command:

1. Inspect the exact lease path and its recorded compatibility paths.
2. Confirm the owner is gone and investigate recorded **and unrecorded** producer
   descendants, including the spawn-intent gap and any other consumer host work.
3. With separately authorized recovery, pass the exact observed file hash and a
   nonempty concrete inspection reason to `Gate::recover` with the same options.
   It rechecks owner/groups/hash under flock. Wrong paths or changed evidence fail.
4. Recapture after recovery; do not reseal/accept artifacts from the aborted run.

Malformed evidence is deliberately not recoverable through this API because its
owner/groups cannot be proven. Preserve it for manual investigation rather than
inventing clean state. No production lease was recovered in this slice.

## Verification boundary

Tests use isolated temporary lock paths and disposable shell processes. They
exercise real flock contention/legacy metadata preservation, timeout/cancellation,
unchanged inode, unfinished Drop, symlink/hardlink/malformed evidence rejection,
file replacement poisoning, exact-hash recovery and a SIGKILLed supervisor whose
producer survives. Recovery is refused until that owned test producer is killed
and its group is absent. Guarded raw capture matches unguarded disposable-fixture
pixels/manifests; stale lease blocks probes/producer/seals. These are process and
artifact contracts, not GPUI/native UI evidence. Exact fresh gates and timing are
recorded in [migration.md](migration.md).
