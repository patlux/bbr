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

- [ ] Palette width per density; menu row resolution; library selection pair.

### Real-app E2E

- [ ] Command suite asserts palette and overlay widths; menu, find, downloads,
      saved-pages suites stay green.

### Performance

- [ ] Command open scenarios unchanged; execution deferred.

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
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not run — execution deferred
Worktree state:
```
