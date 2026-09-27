# Task packet — v2 S4: space paper and active chip lift

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §11.3, §18.1 (D3), §18.2.

## Outcome

The active space tints the rail at 12 % (IC 14 %) toward its curated chrome
wash, the active footer chip is the lift carrying 8 % of the identity hue, and
the swipe interpolation and snapshot tint evidence follow the new values.

## Scope

- `theme/space_tint.rs` alphas and `chip_active` derivation; contrast gate for
  every curated theme × mode × contrast at the D3 bars.
- Strip chip presentation with the lift; swipe interpolation unchanged in
  mechanism.
- DESIGN §11.3; §18.6 row.

## Non-goals

- The switch cross-fade (S7), intensity steps and grain (S10).

## Parallel-lane contract

```text
Base condition:          S3 integrated
Depends on:              v2-tokens, v2-rail
Provides/API:            v2 SpaceTint values
Exclusive paths:         bbr-ui theme/space_tint.rs, sidebar/strip.rs
Shared integration points: e2e_shell_spaces tint evidence, gallery tint story
Integration order:       after S3, before S5
```

## Constraints

- Web content is never tinted; tint stays on BBR-owned chrome.
- D3 bars recomputed from tokens, never copied.

## Evidence

### Unit and contract

- [ ] Gate: `ink ≥ 12:1` (IC 14), `ink.secondary ≥ 5.9:1` (IC 9) on every
      tinted rail; chips distinct per theme.

### Real-app E2E

- [ ] Spaces suite: tint evidence per space, swipe tracking/cancel/commit.

### Performance

- [ ] Space-switch scenarios unchanged; execution deferred.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-spaces
just e2e-shell-sidebar
just e2e-gallery
```

## Iteration policy

Alpha and gate first, then chip, then spaces E2E.

## Blocked stop condition

If a curated theme cannot hold the D3 bars at 12 %, stop with the per-theme
ratios.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not run — execution deferred
Worktree state:
```
