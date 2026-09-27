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
- Carry-over from S2 (measured on the S2 tokens): at 12 % the brightest dark
  chrome washes (Slate) push the composited dark selection to `ink` 9.69:1
  (D1 bar 10.5) and the footer veil to `ink.secondary` 5.26:1 (bar 5.9).
  The S2 palette gates already composite over every tinted rail, so they
  fail first; correct the dark chrome-wash hues (lower luminance, same hue)
  or the veils, never the bars.

## Evidence

### Unit and contract

- [x] Gate: `ink ≥ 12:1` (IC 14), `ink.secondary ≥ 5.9:1` (IC 9) on every
      tinted rail; chips distinct per theme (`space_tint.rs` gate, chip and
      visibility tests; the S2 palette gates re-prove D1 and the footer on
      the 12 % rails).

### Real-app E2E

- [x] Spaces suite: tint evidence per space, swipe tracking/cancel/commit
      (21/21); sidebar lift scenario asserts the lifted active chip.

### Performance

- [x] Space-switch scenarios unchanged (paint values only); execution
      deferred per the program constraint.

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
Outcome:               12 % / 14 % space tint with recalibrated chrome-wash
                        hues (dark: 1.3× saturation near rail luminance;
                        light: value 1, saturation ≤ 0.45 bounded by
                        accent.ink); active chip = selection + 8 % hue with
                        `elev:lift`, reported in `lifted_ids`.
Commits:               see git log (feat(ui): tint the whole rail …)
Functional evidence:   just verify ok; testkit clippy ok; bbr-ui 182 tests;
                        app bins 288
Real-app E2E:          shell-spaces 21/21, shell-sidebar 18/18, gallery
                        46/46, shell-organize 9/9
Visual evidence:       visual-capture light representative/ordinary (cobalt
                        default space, local /tmp/v2s4/cap)
Performance evidence:   not run — execution deferred
Deviations:            wash hues recalibrated (v1 values failed D1 at 12 %:
                        dark selection ink 9.69, footer 5.26; light ΔE 0.3);
                        the default space follows its profile hue, so a fresh
                        profile now shows a visible cobalt tint.
Review:                serial self-review: bars unchanged, every composited
                        gate recomputed from tokens, IC deepens strictly.
Worktree state:        clean after commit
```
