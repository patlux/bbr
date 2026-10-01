# Task packet — Favicon disk cache

Parents: [Phase 4](../phases/04-ui-shell.md) · [Native integration track](../tracks/native-integration.md) · [DESIGN.md §7.2](../../../DESIGN.md). Follows the [real favicon pipeline](real-favicon-pipeline.md), whose non-goal "no favicon persistence across restarts" this packet delivers.

## Outcome

Patrick (2026-10-01, screenshot of grey monogram Favorites after a
relaunch): favicons are cached so they still show after the browser was
closed and opened again. A tab that owns no session after a relaunch — a
deferred tab, a closed pinned row, a Favorite — shows the icon acquired in
an earlier run instead of the monogram, without fetching it again.

## Scope

- `bbr-browser`: the bounded per-profile favicon store can be seeded
  (`restore_favicons`, ignored for unknown profiles) and emptied per
  profile (`forget_favicons`); deleting a profile drops its records.
- `bbr-app` `favicon_cache`: `favicons/<profile id>/index` (format `1`,
  hex hosts in insertion order) and one file per host in the profile
  directory. Seeded at launch before the first frame; written as a diff
  (one file per new or changed record, one deletion per eviction, the
  index on any change, temp file + rename) whenever the store version
  changes and in the quit flush. Unknown profiles' directories are removed
  at launch; unreadable, oversized (> 1 MiB) or extra (> 64) records are
  skipped.
- Clear Browsing Data with "cached images and files" drops the active
  profile's records (memory and disk) for any range.
- DESIGN §7.2 contract text.

## Non-goals

- No favicon blob in the SQLite store or in Saved Pages (§11.16).
- No time per record (a ranged clear drops all of the profile's records).
- No refresh policy beyond "the next acquisition of a host replaces it".

## Parallel-lane contract

Not parallelized: serial on `main`.

## Constraints

- Profile isolation: one directory per profile, never a shared record.
- The store stays the only favicon authority; the disk is a cache of it,
  and losing it is harmless (the monogram renders until the next visit).
- No weakened assertion, budget or E2E step.

## Evidence

### Unit and contract

- [x] `bbr-browser` favicon store: restore order, empty hosts/payloads
      skipped, `forget_profile`; controller: kept records present an image
      without a session, unknown profiles ignored, forget and profile
      deletion drop records.
- [x] `favicon_cache_tests`: round trip per profile in order, diff writes
      and eviction deletes, forgotten/unknown profile directories removed,
      damaged/oversized/other-version records skipped, host file names.

### Real-app E2E

- [x] `e2e_shell_persistence::favicons`: an icon acquired in leg 1 is kept
      in `favicons/1/`; after a graceful restart the session-less Favorite
      tile shows `favicon == "image"` and the fixture served the icon once.
      With the seeding removed the same test fails with `"monogram"`.
- [x] `e2e-shell-favicons`, `e2e-profiles`, `e2e-web-data`, `e2e-history`
      stay green.

### Performance

- [ ] The launch reads the kept records before the first frame (at most 64
      small files per profile); the startup scenario covers it, run with
      the performance override's end-of-queue execution.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-persistence
just e2e-shell-favicons
just e2e-profiles
just e2e-web-data
just e2e-history
just e2e-perf          # deferred (performance override)
```

## Iteration policy

Store API first, then the disk module with unit tests, then the app wiring
and the restart E2E (red without the seeding, green with it).

## Blocked stop condition

Stop with the evidence if keeping favicons would need a second favicon
authority or a store schema change.

## Delivery ledger

```text
Outcome:            acquired favicons survive a relaunch per profile
Functional evidence: bbr-browser favicon/controller_favicon_tests;
                    bbr favicon_cache_tests; testkit clippy; just verify
Real-app E2E:       e2e-shell-persistence (new favicons leg pair; red with
                    the seeding removed: "monogram"), e2e-shell-favicons 3/3,
                    e2e-profiles, e2e-web-data, e2e-history on the host
Performance:        not run (override)
Review:             self-review against this packet
Worktree state:     main, serial
```
