# Task packet — v2 visual refresh and motion (program)

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §17.17, §18.

Specification input: the Paper file “BBR — Browser Shell”
(`01KZY8FRMDSAYFJ4TJ9TYTV4AF`) — pages 01–06 after the Arc-informed token
refresh and page 07 “Motion & delight (exploration)”, plus the board
“Study · Arc → BBR v2”. Paper and the generated WebP loops are reference
only; every surface ships as native GPUI drawing from `bbr-ui` tokens.
Patrick adopted decisions D1–D11 (DESIGN §18.1) as recommended on
2026-09-28.

## Outcome

The running app presents the v2 shell: one tinted body with a lifted
selection, translucent veils and washes, softer radii, calmer floating
surfaces, a bounded set of after-commit motion moments with Reduced Motion
variants, drawn empty-state illustrations, a Phosphor-b app icon and a
curated intensity/grain control per space — while every commit-first budget,
the native-glass boundary and profile isolation stay unchanged.

## Scope

The program is twelve bounded slices, each with its own packet, evidence
and ledger:

| Slice | Packet | Depends on |
|---|---|---|
| S0 | [v2 contract](v2-contract.md) | — |
| S1 | [system preferences](v2-system-prefs.md) | S0 |
| S2 | [tokens](v2-tokens.md) | S0 |
| S3 | [rail](v2-rail.md) | S2 |
| S4 | [space paper](v2-space-paper.md) | S2 |
| S5 | [floating surfaces](v2-overlays.md) | S2 |
| S6 | [motion core](v2-motion-core.md) | S1, S2 |
| S7 | [after-commit moments](v2-motion-moments.md) | S3–S6 |
| S8 | [empty states](v2-empty-states.md) | S6 |
| S9 | [brand icon](v2-brand-icon.md) | S0 |
| S10 | [theme picker](v2-theme-picker.md) | S4 |
| S11 | [closeout](v2-closeout.md) | all |

## Non-goals

- No site-adaptive tint, colour wheel, vibrancy or blur (DESIGN §16, D7).
- No idle ambient animation (D6) and no animation on selection, layout,
  sidebar toggle or split changes (§6 principle 1).
- No drawing above or sampling of native web content (§13); the card-edge
  progress bar is withdrawn (D5).
- The media card itself remains owned by the
  [sidebar media control](sidebar-media-control.md); v2 only restyles the
  token it consumes.
- Performance execution stays deferred by Patrick; scenario code is added,
  execution and acceptance remain unpassed.

## Parallel-lane contract

```text
Base condition:          main after the media-session portable contract (RA-MEDIA paused)
Depends on:              DESIGN §18 adoption (S0)
Provides/API:            v2 palette/elevation/motion tokens, motion-mode theme field
Exclusive paths:         per slice packet
Shared integration points: bbr-ui palette/theme, testkit snapshot, e2e suites, gallery stories, DESIGN.md §18.6
Integration order:       S0 → S1/S2 → S3 → S4 → S5 → S6/S9 → S7 → S8/S10 → S11
```

The orchestrator is the only writer; slices run serially on `main` unless the
current-delivery document records a parallel lane.

## Constraints

- AGENTS.md architecture boundaries: GPUI owns chrome, native web views own
  pages; portable crates import no macOS frameworks; `bbr-ui` stays the one
  token authority.
- Contrast bars change only where DESIGN §18.1 says so (D1, D3), in the same
  commit as the values; every other gate is kept.
- Every motion starts after its state is committed and presented, and has a
  Reduced Motion variant; idle requests no frames.
- No weakened tests, budgets, validators or acceptance criteria.

## Evidence

### Unit and contract

- [ ] Each slice's unit/contract evidence (per packet).

### Real-app E2E

- [ ] Each slice's affected suites green (per packet).
- [ ] Final canonical `just e2e` aggregate without `e2e-perf`, plus
      `e2e-peek` and `e2e-gallery`, green from the integrated commit.

### Performance

- [ ] Scenario code for new user-visible timing lands with its slice;
      execution deferred (Patrick, 2026-09-27) and recorded as unpassed.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e              # aggregate minus e2e-perf while performance is deferred
just e2e-peek
just e2e-gallery
just roadmap
```

## Iteration policy

Land slices in the integration order. Each slice starts from the smallest
production path, runs targeted checks after every change, updates its DESIGN
sections and §18.6 row with the implementation, and records its ledger before
the next slice starts.

## Blocked stop condition

Stop with evidence, attempts, preserved state and the smallest required input
if another writer changes `main`, the macOS session is locked for GUI gates,
a slice cannot pass without weakening a test or contract, or a persistence
change needs a schema decision.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   unpassed — execution deferred by Patrick
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:         main, serial
```
