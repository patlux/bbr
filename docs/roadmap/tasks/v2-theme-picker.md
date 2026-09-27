# Task packet — v2 S10: curated intensity and grain

Parents: [v2 program](v2-visual-refresh-program.md) ·
[space appearance contract](space-appearance-contract.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3, §18.1 (D6, D7).

## Outcome

Each space owns a curated tint intensity (subtle / standard / rich) and a
static-grain flag next to its theme and identity; the Space configuration
surface edits both with the existing draft/commit/cancel rules; the rail
renders the chosen intensity and an optional static grain; both persist.

## Scope

- Portable appearance model: closed intensity enum and grain flag with
  deterministic defaults (standard, off).
- Space configuration picker rows with stable ids; draft/commit through the
  existing exact-SpaceId authority.
- Space tint resolution per intensity within the D3 bars; static grain overlay
  on BBR-owned rail chrome only.
- Persistence columns with a schema-version bump under the closed
  discard-on-mismatch policy ([schema v2 decision](schema-v2-migration-decision.md)).
- DESIGN §11.3; §18.6 row.

## Non-goals

- No free colour field (D7), no animated grain (D6), no per-tab tint.

## Parallel-lane contract

```text
Base condition:          S4 integrated
Depends on:              v2-space-paper
Provides/API:            SpaceAppearance intensity/grain
Exclusive paths:         bbr-domain space appearance, bbr-persistence spaces, space_config surface
Shared integration points: SQLite schema version, testkit snapshot, e2e_shell_spaces_config, persistence suite
Integration order:       parallel-capable with S8
```

## Constraints

- Every intensity holds the D3 bars for every theme and mode.
- Grain never paints over web content or controls' text.

## Evidence

### Unit and contract

- [ ] Domain round-trip, persistence round-trip and version-bump discard.
- [ ] Gate across intensities.

### Real-app E2E

- [ ] Configuration edits intensity and grain, commit applies, cancel keeps
      the previous values; restart restores them.

### Performance

- [x] Not applicable: configuration surface only.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-shell-persistence
just e2e-gallery
```

## Iteration policy

Domain and persistence first, then tint resolution, then the surface and E2E.

## Blocked stop condition

If the persistence change needs anything beyond the closed version-bump
policy, stop with the schema diff and ask.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not applicable
Worktree state:
```
