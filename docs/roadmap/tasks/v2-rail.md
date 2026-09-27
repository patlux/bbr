# Task packet — v2 S3: rail (lift, veils, heading, footer)

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §4, §10.2–§10.4, §11.1, §11.4, §11.5,
§11.10, §18.2, §18.3.

## Outcome

The selected tab, pinned and folder-item row presents the lifted white (dark:
white 9 %) surface with the `elev:lift` shadow and a 500-weight title; hover
and press paint translucent washes; the active-space heading is 13 pt / 600;
the address domain is 500 weight over the v2 veil with a 4 % edge; Favorite
tiles use the veil at `radius:md`; footer glyphs render at 16 pt and the space
strip centres.

## Scope

- `bbr-ui` tab row, favorites, address, space heading, footer/strip
  components; the one lift helper from S2.
- Snapshot evidence: the selected row reports its lift (typed field), weights
  are asserted through existing typed row evidence where available.
- DESIGN sections listed above; §18.6 row.

## Non-goals

- Space tint alpha and the active chip lift (S4); motion (S6/S7); media card
  anatomy (owned by the sidebar media control packet).

## Parallel-lane contract

```text
Base condition:          S2 integrated
Depends on:              v2-tokens
Provides/API:            lifted row presentation
Exclusive paths:         bbr-ui tab_row/favorites/address/footer/space components
Shared integration points: testkit snapshot, e2e_shell_sidebar, gallery sidebar stories
Integration order:       S3 → S4 → S5
```

## Constraints

- Exactly the D2 shadow exception: only the selected row, never hover.
- Row geometry, ids, density table and dense-500 math unchanged.

## Evidence

### Unit and contract

- [x] Row visual resolution: selected → lift + 500; hover/press → washes;
      rest → no paint (`tab_row.rs` tests, `app/testkit/lift.rs`).
- [x] Footer glyph size and strip alignment tokens (`footer.rs`,
      `favorites.rs` tile radius, `sidebar/render.rs` heading).

### Real-app E2E

- [x] Sidebar suite asserts the lifted selected row (exactly one), the washes
      on hover/press and the folder-member lift (`e2e_shell_sidebar/lift.rs`);
      the footer geometry (16 pt glyphs, centred strip) is asserted in the
      gallery's production sidebar (`bbr-gallery` e2e sidebar); hover,
      tabs, organize, dense and spaces suites stay green.

### Performance

- [x] Existing selection/hover scenarios cover the path (paint-only change,
      no new interaction); execution deferred per the program constraint.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-tabs
just e2e-shell-organize
just e2e-shell-dense
just e2e-gallery
```

## Iteration policy

One component at a time with its unit test, then the sidebar suite.

## Blocked stop condition

If GPUI cannot paint the lift inside the dense bounded window without a
geometry change, stop with the measured evidence.

## Delivery ledger

```text
Outcome:               selected tab rows and folder members lift (white /
                        white 9 % + `elev:lift`), every title 500, rest rows
                        and inactive chips paint nothing so the tint shows,
                        veiled Favorites at radius:md, 13/600 heading, 16 pt
                        footer glyphs, safe-centred strip, typed `lifted_ids`.
Commits:               see git log (feat(ui): lift the selected rail row …)
Functional evidence:   just verify ok; testkit clippy ok; bbr-ui 179 tests
Real-app E2E:          shell-sidebar 18/18 (new lift scenario), shell-hover
                        4/4, shell-tabs 3/3, shell-organize 9/9, shell-dense
                        4/4, shell-spaces 21/21, gallery 46/46 (new footer
                        geometry scenario)
Visual evidence:       visual-capture release build, representative +
                        ordinary light captures (/tmp/v2s3/cap, local only)
                        read against the Paper v2 light shell.
Performance evidence:   not run — execution deferred
Deviations:            heading tracking (+0.01 em) not expressible in GPUI;
                        footer geometry asserted in the gallery (real-app
                        snapshot has no element bounds); folder-member tabs
                        also render as pinned rows, so a selected member
                        lifts on both presentations of the one tab.
Review:                serial self-review: diff read against §18.2–§18.4,
                        D2 exception limited to `row_lift` + selected member,
                        geometry/ids unchanged (dense 500 green).
Worktree state:        clean after commit
```
