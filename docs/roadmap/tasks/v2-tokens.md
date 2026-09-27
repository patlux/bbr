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

- [x] Palette gate: every documented pair per mode, composited over the
      untinted and every tinted rail (`palette/tests.rs`: D1 selection,
      footer veil, `tint.accent`, address veils, washes; `space_tint.rs`
      at the D3 tinted-rail bars).
- [x] Radius and elevation token tests (`theme.rs`, `density.rs`,
      `elevation.rs`).

### Real-app E2E

- [x] Sidebar, hover and spaces suites green on the new values (colour
      evidence updated only where DESIGN v2 changed the value: the
      swipe-tracking half-way chrome now asserts the exact computed
      interpolation instead of v1's quantized-equal source value).

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
Outcome:               v2 palette/radius/elevation tokens resolved for all
                        four palettes; `tint.accent`, `elev:lift`, `over()`
                        composite; overlay cursors and drop targets moved to
                        `tint.accent` so no selection disappears on white.
Commits:               see git log (feat(ui): adopt the v2 palette, radius
                        and elevation tokens)
Functional evidence:   just verify ok (bbr-ui 175 tests); testkit clippy ok
Real-app E2E:          shell-sidebar 17/17, shell-hover 4/4, shell-spaces
                        21/21 (after the tracking fix), gallery 45/45;
                        consumer suites shell-command 25/25, shell-tab-menu
                        12/12, shell-saved-pages-library 12/12,
                        shell-organize 9/9
Performance evidence:   not applicable
Deviations:            dark footer veil 5 % (Paper 7 % gave 5.73:1 < 5.9 on
                        the Slate rail); tinted-rail ink bar follows D3
                        (12:1) from S2 because the lighter v2 dark rail
                        measured 13.85:1 at the unchanged 4 % Slate tint;
                        address IC values chosen and recorded in §3.4/§18.2.
Review:                serial self-review (no subagents authorised): diff
                        read against §18.2/§18.3, every `tint_selected`
                        consumer classified (rail selection vs overlay
                        cursor/drop target), bars unchanged except D1/D3.
Carry-over:            S4 must retune dark chrome washes or veils at 12 %
                        (recorded in the space-paper packet).
Worktree state:        clean after commit
```
