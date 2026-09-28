# Task packet — Blueprint D20: New folders open in rename

Parents: [Phase 4](../phases/04-ui-shell.md) ·
[complete browser blueprint](paper-browser-blueprint.md) ·
[DESIGN.md](../../../DESIGN.md) §11.11, §19.

Status: implemented 2026-09-29 (DESIGN §19); position 2 of 24. Paper boards: S-4 · Folders, PR-06 · Organize — folders & drag rules (file
“BBR — Browser Shell”).

## Outcome

A new folder opens in inline rename immediately; Return commits, Escape keeps 'New Folder'.

## Scope

- Implement the target as drawn on the boards above, light and dark, with
  the variants of page 40 that apply (increased contrast, density, Reduce
  Motion).
- Land the decision in DESIGN.md §11.11 in the same group and flip the
  D20 row of §19 to adopted.
- Expose the new surface's stable ids and typed snapshot evidence through
  the testkit.

## Non-goals

- Other blueprint deltas, even where a board shows them next to this one.
- Behaviour the boards mark as Native OS (drawn by macOS or WebKit).

## Parallel-lane contract

Not parallelized.

## Constraints

- Today on main: New folders are created as 'New Folder' and keep that name until renamed from the menu.
- GPUI owns the chrome; floating surfaces over web content use the one
  native Portal; web content is never sampled or redrawn.
- Colours, type and spacing come from the palette and theme tokens that
  the Paper file mirrors; no new literal colours.
- No weakened tests, budgets, bars or acceptance criteria.

## Evidence

### Unit and contract

- [x] FolderStore::rename_folder (trim, EmptyTitle, unknown folder) in bbr-domain; OrganizationController::rename_folder + FolderRenamed in bbr-browser; FolderRename::apply_key (replace-selection, Backspace, arrows, Return, command pass-through) in bbr-app; folders_ids stability and the snapshot's bbr-folder-rename gating; gallery 'inline rename' family.

### Real-app E2E

- [x] `just e2e-shell-organize` — e2e_shell_folder_rename::new_folders_open_in_inline_rename_and_commit_or_keep_the_title (plus-menu New Folder opens rename with the title selected; typing/Backspace/Return commit; Escape and an emptied draft keep 'New Folder'; ChromeBackgroundPress and Cmd-T focus-out commit); sidebar, persistence, dense, tab-menu and gallery suites green.

### Performance

- [ ] Performance scenario: none added — the rename is keystroke text editing on an existing row with no new presentation milestone; measurement is deferred with the rest of the performance evidence (Patrick, 2026-09-27).

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-perf
just e2e
```

## Iteration policy

Start with the smallest production path of the first board frame, then add
states in the order the boards list them. Compare the running app with the
boards after each state (visual capture), fix the mechanism behind each
difference, and keep DESIGN.md and this packet in step.

## Blocked stop condition

If Patrick rejects or changes the D20 proposal, or the target conflicts
with an existing contract, stop with the evidence, the conflict and the
decision needed, and update the §19 row instead of implementing.

## Delivery ledger

```text
Outcome:                New folders open in inline rename; Return/outside press/focus-out commit, Escape keeps the title.
Commits:                see git log (feat/fix commit carrying this packet)
Functional evidence:    FolderStore::rename_folder (trim, EmptyTitle, unknown folder) in bbr-domain; OrganizationController::rename_folder + FolderRenamed in bbr-browser; FolderRename::apply_key (replace-selection, Backspace, arrows, Return, command pass-through) in bbr-app; folders_ids stability and the snapshot's bbr-folder-rename gating; gallery 'inline rename' family.
Real-app E2E:           `just e2e-shell-organize` — e2e_shell_folder_rename::new_folders_open_in_inline_rename_and_commit_or_keep_the_title (plus-menu New Folder opens rename with the title selected; typing/Backspace/Return commit; Escape and an emptied draft keep 'New Folder'; ChromeBackgroundPress and Cmd-T focus-out commit); sidebar, persistence, dense, tab-menu and gallery suites green.
Performance evidence:   unpassed — execution deferred (Patrick, 2026-09-27)
Roadmap/documentation updates: DESIGN §11.4 plus-menu text, §11.11 rename paragraph + bbr-folder-rename id, §19 D20 implemented
Open blockers and non-goals: none
Worktree state:         main, clean after commit
```
