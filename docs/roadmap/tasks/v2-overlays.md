# Task packet — v2 S5: floating surfaces

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §8, §11.4, §11.12, §11.13, §11.16, §18.4.

## Outcome

Menus, find bar, auth sheet, Space configuration and Downloads use 1 pt edges
and `elev:2` v2; menu and Library cursors are inset `tint.accent` surfaces
without an accent rail; the ⌘T palette is 508/600/716 pt wide with a 17 pt UI
query, 56 pt field, 40 pt results and radius 16; the ⌘L overlay is at least
440 pt wide with 40 pt field and 36 pt results.

## Scope

- `bbr-ui` surfaces: menu, tab menu, find, auth, space config, downloads,
  command render/geometry; Library rows.
- `Density::palette_width`; app overlay width clamp.
- DESIGN sections above; §18.6 row.

## Non-goals

- Result stagger and cursor glide (S7); new menu rows or behaviour.

## Parallel-lane contract

```text
Base condition:          S4 integrated
Depends on:              v2-tokens
Provides/API:            v2 floating-surface anatomy
Exclusive paths:         bbr-ui surfaces/*, saved_pages_library rows
Shared integration points: app palette portal sizing, command/menu E2E suites, gallery
Integration order:       after S4
```

## Constraints

- Keyboard ownership, dismissal and Portal placement unchanged.
- Library selection text meets `ink ≥ 9:1` on `tint.accent`.

## Evidence

### Unit and contract

- [x] Palette width per density (`theme.rs`), ⌘T/⌘L anatomy
      (`command/render.rs`), shared menu edge/padding/cursor/field helper
      (`surfaces/floating.rs`), library selection pair (`rows.rs`, palette
      gate `tint.accent` ink ≥ 9:1).

### Real-app E2E

- [x] Command suite asserts palette (600 pt at normal density) and overlay
      (440 pt–`win:palette`) widths; tab-menu, context-menu, find,
      downloads, saved-pages-library, auth and gallery suites stay green.

### Performance

- [x] Command open scenarios unchanged in code; execution deferred per the
      program constraint.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-command
just e2e-shell-tab-menu
just e2e-shell-context-menu
just e2e-shell-find
just e2e-shell-downloads
just e2e-shell-saved-pages-library
just e2e-auth
just e2e-gallery
```

## Iteration policy

Menu family first, then panels, then command geometry; the affected suite
after each group.

## Blocked stop condition

If the wider palette cannot fit the minimum window with the Portal margins,
stop with the geometry evidence.

## Delivery ledger

```text
Outcome:               menus (page, tab + submenu, plus) share the 1 pt edge,
                        4 pt padding, elev:2 v2 and the inset tint.accent
                        cursor; the in-rail and palette/overlay result cursors
                        drop the accent rail; find/auth fields are 1 pt at rest;
                        ⌘T 508/600/716, radius 16, 56 pt field, 17 pt query,
                        40 pt rows in an 8 pt list; ⌘L ≥ 440 pt, 40 pt field,
                        36 pt rows.
Commits:               see git log (feat(ui): adopt the v2 floating-surface …)
Functional evidence:   just verify ok; testkit clippy ok; bbr-ui 185 tests
Real-app E2E:          shell-command 25/25, shell-tab-menu 12/12,
                        shell-context-menu 11/11, shell-find 6/6,
                        shell-downloads 11/11, shell-saved-pages-library
                        12/12, auth 5/5, gallery 46/46
Performance evidence:   not run — execution deferred
Deviations:            the 17 pt ⌘T query and the ⌘L `type:mono` query both
                        render in the system UI face (the shell loads no
                        mono family yet — §4 role only); ⌘L card radius read
                        as `radius:lg` 12 with rows at 10; Downloads, Space
                        configuration and the auth sheet already had 1 pt
                        card edges — only their resting fields changed.
Review:                serial self-review: keyboard ownership, dismissal and
                        Portal placement untouched; ids unchanged; widths fit
                        the 1080 pt default window with 8 pt margins.
Worktree state:        clean after commit
```
