# Task packet — profile-scoped singleton favorite runtime (deferred)

Parents: [Browser-orchestration track](../tracks/browser-orchestration.md) ·
[Phase 8](../phases/08-daily-driver.md) ·
[browser interaction contract](../../product/browser-interactions.md) §Favorites.

Status: **deferred post-MVP architecture option; not an MVP or Phase-8 exit blocker.**

## Outcome

BBR can optionally promote a favorited concrete tab into one profile-bound
singleton runtime whose live native session remains the same while the user
changes between spaces of that profile. The window-global favorite tile still
identifies one concrete persistent browser object, but presenting it in another
same-profile space no longer requires moving, copying, reloading, or creating a
second tab/session. Cross-profile reuse remains impossible.

This deliberately replaces part of the current invariant that every `TabId`
belongs to exactly one space. It must land only with an explicit product and
migration decision; the current MVP contract remains authoritative until then.

## Scope

- Decide and model a distinct profile-bound singleton-tab/favorite identity,
  including its relationship to ordinary space-owned `TabId` values.
- Define visibility, selection, ordering, split, close, move, suspend, crash,
  recently-closed, and persistence semantics without weakening profile
  isolation.
- Preserve exactly one live `WebViewId` for the singleton runtime across
  same-profile space switches; never attach that session to another profile's
  `WKWebsiteDataStore`.
- Define migration from the current window-global concrete-tab favorite model:
  opt-in promotion, persisted schema/version behavior, rollback/rejection, and
  what happens to the former owning-space position.
- Add portable domain/controller contracts, persistence migration, native
  composition, typed testkit state, real-app E2E, and optimized performance
  evidence.
- Update the canonical browser interaction contract before changing shipped
  behavior.

## Non-goals

- No URL-shortcut favorites. Favorites remain concrete persistent browser
  identities, not bookmarks that create arbitrary new tabs.
- No cross-profile session reuse or shared website data.
- No change to the MVP implementation while this packet is deferred.
- No implicit conversion of every favorite; migration and user intent must be
  explicit.
- No cloud sync or multi-window sharing.

## Parallel-lane contract

Not parallelized. This changes shared domain identity, controller session
ownership, persistence schema, native composition, and testkit contracts.

## Constraints

- One singleton runtime belongs to exactly one profile and therefore one
  persistent website-data store.
- At most one live native session exists for the promoted favorite in the
  browser window.
- Same-profile space presentation must not reload or recreate the session.
- Cross-profile activation must switch to or create a valid presentation in
  the owning profile; it must never carry the native session across the
  container boundary.
- Ordinary tabs retain exactly-one-space ownership unless explicitly promoted.
- No compatibility shim that silently supports both contradictory ownership
  models indefinitely. Use one versioned migration and one canonical model.
- No new dependency unless existing domain/persistence capabilities are proven
  insufficient.
- Keep files below roughly 500 lines; do not weaken lifecycle, persistence,
  profile-isolation, E2E, or performance gates.

## Evidence

### Unit and contract

- [ ] Domain invariants distinguish ordinary space-owned tabs from promoted
      profile-bound singleton runtimes and reject duplicate/cross-profile
      ownership.
- [ ] Controller tests prove exactly one `WebViewId`, same-profile reuse,
      suspend/crash recreation, split/close semantics, and deterministic
      activation from every space.
- [ ] Persistence tests prove current-model migration, round-trip identity,
      corruption rejection, and closing/removal semantics.

### Real-app E2E

- [ ] Real app promotes one concrete favorite, switches between two spaces of
      its profile, and proves stable `TabId`/`WebViewId`, unchanged session
      count, no reload, and correct focus/presentation.
- [ ] Cross-profile activation proves the original website-data store remains
      authoritative and no cookie/session data crosses profiles.
- [ ] Restart restores the singleton runtime contract; closing it removes every
      presentation and leaves no native orphan.

### Performance

- [ ] Add an optimized same-profile favorite-space-switch scenario with at
      least 30 accepted samples and semantic/native presentation commit.
- [ ] Same-profile presentation stays in the warm-session budget; any
      cross-profile recreation remains in the session-recreation class.

## Completion gates

```bash
just verify
cargo clippy -p bbr-domain --all-targets -- -D warnings
cargo clippy -p bbr-browser --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-profiles
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. Resolve and record the product model and migration before implementation.
2. Implement domain identity/invariants and persistence migration.
3. Implement controller/native session ownership with fake-engine evidence.
4. Compose the production UI/testkit path and real-app E2E.
5. Run independent architecture and profile-isolation review, focused
   remediation, and final recheck.

## Iteration policy

First prove the model with pure identities and a fake engine. Then prove
persistence. Only then attach a real native session and expose it across spaces.
After a failure, identify whether the contradiction is identity ownership,
selection/order, persistence, native session mapping, or profile isolation
before changing the contract.

## Blocked stop condition

Stop with the current MVP model preserved if no coherent single-owner model can
satisfy ordinary-tab ordering, close/recently-closed semantics, split behavior,
persistence, and exactly-one native session. Report the conflicting invariants,
models attempted, migration evidence, and the smallest product decision needed.

## Delivery ledger

```text
Outcome: deferred post-MVP architecture option documented
Commits: planning only
Functional evidence: pending
Real-app E2E: pending
Performance evidence: pending
Roadmap/documentation updates: task packet + parent links
Open blockers and non-goals: current concrete TabId / owning-space MVP contract remains authoritative
Worktree state: no implementation lane active
```
