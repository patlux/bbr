# Task packet — v2 S2: palette, radius and elevation tokens

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §2, §3, §5, §18.2, §18.3.

## Outcome

`bbr-ui` resolves the v2 material, wash, veil, hairline, selection and
`tint.accent` values for light/dark × standard/increased contrast, the radius
scale 6/10/12/16, the `elev:lift` and `elev:2` v2 shadow helpers, and the
contrast gate proves the DESIGN §18.1/§18.2 bars on composited surfaces over
the untinted and every tinted rail.

## Scope

- `palette.rs`: v2 values; translucent washes/veils as alpha tokens;
  `tint_accent`; tests composite translucent tokens before computing ratios.
- `theme.rs` radius scale; `theme/elevation.rs` `lift()` and `elev2` v2.
- Every consumer keeps compiling with unchanged semantics; visible adoption of
  the lift and cursors lands in S3/S5.
- DESIGN §2, §3, §5 updated to the v2 values; §18.6 row.

## Non-goals

- No component anatomy change (S3–S5), no space-tint alpha change (S4).

## Parallel-lane contract

```text
Base condition:          S0 integrated
Depends on:              v2-contract
Provides/API:            Palette::tint_accent, elevation::lift, v2 values
Exclusive paths:         bbr-ui palette/theme/elevation
Shared integration points: gallery token stories, testkit snapshot hex fields
Integration order:       before S3
```

## Constraints

- Bars change only per D1/D3; all other gates keep their thresholds.
- Snapshot hex evidence keeps reporting the rendered colour.

## Evidence

### Unit and contract

- [ ] Palette gate: every documented pair per mode, composited.
- [ ] Radius and elevation token tests.

### Real-app E2E

- [ ] Sidebar, hover and spaces suites green on the new values (colour
      evidence updated only where DESIGN v2 changed the value).

### Performance

- [x] Not applicable: paint values only.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-spaces
just e2e-gallery
```

## Iteration policy

Values and gate first, then fix consumers, then E2E; if a v2 value misses its
bar, correct the value (never the bar) and record it in DESIGN in the same
commit.

## Blocked stop condition

If no value satisfies a bar and the Paper intent together, stop with the
computed ratios and the two candidate values.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not applicable
Worktree state:
```
