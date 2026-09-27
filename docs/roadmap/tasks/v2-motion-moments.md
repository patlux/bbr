# Task packet — v2 S7: after-commit moments

Parents: [v2 program](v2-visual-refresh-program.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[DESIGN.md](../../../DESIGN.md) §6, §10, §11.3, §11.4, §11.10, §11.12,
§18.1 (D4), §18.5.

## Outcome

Four bounded moments play only after their state is committed and presented:
the favorite pop with specks, the download check with ring, the ⌘T result
stagger and cursor glide, and the space-switch tint cross-fade — each with a
Reduced Motion variant and a real-app proof that the committed state is
visible on the commit frame.

## Scope

- Favorites tile pop/specks on a new favorite; Downloads row check/ring on
  success; command results stagger and cursor highlight glide; chrome tint
  fade on committed (non-swipe) space switches.
- Testkit transient evidence per moment (running, reduced) and commit-frame
  evidence (the committed model/tint value is reported before the moment
  ends).
- DESIGN sections above; §18.6 row.

## Non-goals

- New interactions or durations beyond §18.5; no motion on selection or
  layout.

## Parallel-lane contract

```text
Base condition:          S3–S6 integrated
Depends on:              v2-motion-core
Provides/API:            none new
Exclusive paths:         favorites/drag landing, downloads rows, command render, space switch presentation
Shared integration points: testkit snapshot, organize/downloads/command/spaces suites
Integration order:       after S6
```

## Constraints

- Every moment ≤ 260 ms single-shot; nothing loops.
- Space switch: geometry, lists, selected tab and native view snap; only the
  chrome tint fades.

## Evidence

### Unit and contract

- [ ] Moment descriptors and reduced variants.

### Real-app E2E

- [ ] Favorite drop: favorite committed in the snapshot immediately; pop
      reported, reduced variant under override.
- [ ] Download success: check moment reported; reduced variant.
- [ ] ⌘T: selection committed on the key press; glide/stagger reported.
- [ ] Space switch: destination tint committed as target, fade reported,
      instant under Reduce Motion; swipe commit does not fade.

### Performance

- [ ] Existing space-switch and command scenarios keep their budgets; scenario
      code for the fade path added; execution deferred.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-organize
just e2e-shell-downloads
just e2e-shell-command
just e2e-shell-spaces
just e2e-gallery
```

## Iteration policy

One moment at a time with its E2E; keep the commit-first proof as the first
assertion of each test.

## Blocked stop condition

If a moment cannot start without delaying its commit, stop with the evidence
and ship the static variant.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:   not run — execution deferred
Worktree state:
```
