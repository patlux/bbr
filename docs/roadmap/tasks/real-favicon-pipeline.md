# Task packet — Real favicon pipeline

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [Native integration track](../tracks/native-integration.md) · [DESIGN.md §7.2](../../../DESIGN.md).

## Outcome

Every favicon slot — tab rows, pinned rows, Favorites tiles and folder
items — presents the site's real favicon, acquired through the owning
profile's isolated session, with the deterministic §7.2 monogram as the
unchanged fallback. The whole rail stops reading as a monogram wall and
matches the reference's identity density.

## Scope

- Portable favicon contract in `bbr-web`: a typed per-page favicon event
  (candidate URLs or resolved image bytes) on the engine boundary,
  engine-agnostic and adapter-implemented.
- macOS adapter in `bbr-web-macos`: after navigation commit, an isolated
  content-world script collects `<link rel~="icon">` candidates with
  `/favicon.ico` as the final fallback; one best candidate is chosen by a
  simple deterministic rule (preferred declared size closest to 2× icon
  size, then document order). Download runs through a session bound to the
  owning profile's data store so cookies/credentials never cross profiles.
  Bounded size and deadline; any failure falls back silently to the
  monogram.
- In-memory favicon cache per profile, host-keyed and bounded; a repeated
  host renders instantly and never refetches per tab. No disk persistence
  in this slice.
- `bbr-browser`: favicon state joins the tab presentation model keyed by
  stable `TabId`/host; Favorites and folder items derive from the same
  record — one acquisition, every presentation.
- `bbr-ui` favicon component renders the decoded image in the existing tile
  geometry; monogram (§7.2) and loading-spinner precedence (§10.5) are
  unchanged. Snapshot gains a `favicon ∈ {monogram, image}` presentation
  field for E2E.
- Deterministic E2E fixtures serving declared favicons (and one host
  without) so real-app evidence never depends on the network.
- DESIGN.md §7.2 update in the same group: the acquisition contract
  (candidates, profile-isolated fetch, bounded fallback) becomes contract
  text.

## Non-goals

- No favicon persistence across restarts (Phase 7 owns persisted state;
  Saved Pages explicitly store no favicon blob, §11.16).
- No touch-icon/manifest/SVG-mask heuristics beyond the single best-candidate
  rule; no per-site overrides.
- No page screenshotting, texture composition or private WebKit SPI.
- No theme, row-pitch or footer-glyph work — owned by the sibling packets.

## Parallel-lane contract

```text
Base condition:          integrated VS-REFERENCE shell on local main
Depends on:              none
Provides/API:            bbr-web favicon event contract; per-profile favicon store; ui image tile
Exclusive paths:         bbr-web favicon contract, bbr-web-macos acquisition, favicon store, bbr-ui favicon component
Shared integration points: controller snapshot/testkit protocol, tab-row/favorites components, Gallery registry, justfile, DESIGN.md
Integration order:       independent of system-appearance-theme; before sidebar-reference-polish captures final evidence
```

## Constraints

- Profile isolation is invariant: acquisition uses the owning profile's data
  store; two profiles never share a fetch, cache entry or session.
- The engine boundary stays portable: `bbr-web` defines the contract,
  platform adapters implement it; portable crates import no macOS framework.
- Downloads are bounded (size, deadline, concurrent count); failure is
  silent monogram fallback — never a broken row, never a retry storm.
- Existing ids, §10 states, dense-500 bounded rendering, scroll math and all
  performance budgets/sample counts are unchanged. Performance execution
  remains blocked under the active override.
- No fake favicons in production; fixtures own deterministic test icons.

## Evidence

### Unit and contract

- [ ] Candidate parsing/selection rule is deterministic across declared
      sizes, rel variants and missing links.
- [ ] Cache bounds, host keying and per-profile separation are proven at the
      store level.
- [ ] Fallback precedence (loading spinner → image → monogram) is proven at
      the component level.

### Real-app E2E

- [ ] Fixture with a declared favicon: tab row and its Favorite tile report
      `favicon == image` after load; a host without one stays `monogram`.
- [ ] Same host in two profiles acquires through two isolated sessions
      (fixture asserts separate requests) and never shares cache entries.
- [ ] Crash/error and loading states keep their §10.5/§10.7 precedence over
      the image tile; dense-500 scrolling stays within existing bounds.

### Performance

- [ ] Not executed: performance execution remains blocked under the active
      override. Acquisition must remain off the presentation-commit path;
      scenario code, budgets and sample guards compile unchanged.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-web -p bbr-web-macos -p bbr-browser -p bbr-ui
cargo test -p bbr --features testkit --bins
cargo clippy -p bbr-ui --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-tabs
just e2e-shell-sidebar
just e2e-shell-dense
just e2e-gallery
just roadmap
```

## Delivery sequence

1. One exclusive writer implements and verifies the bounded slice
   (contract → adapter → store → presentation → fixtures).
2. One independent read-only reviewer maps requirements to evidence, with
   explicit attention on the profile-isolation boundary.
3. One remediation writer only for concrete findings, then a focused recheck.
4. The native-boundary character of this slice justifies review depth, not a
   second renderer or a relaxed contract.

## Iteration policy

Land the portable contract with a fake-engine test first, then the adapter
against fixtures, then the presentation join. After each failing check,
identify the mechanism before changing the implementation; never widen the
acquisition surface to fix a presentation bug.

## Blocked stop condition

Stop fail-closed if favicon acquisition cannot be bound to the owning
profile's data store through public API, if the isolated-content-world
script cannot observe link candidates reliably, or if a required gate fails
without a defensible bounded fix. Preserve exact fingerprints, findings and
the smallest input needed.

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
```
