# Task packet — browser-MVP palette UI (U1)

Parents: [Phase 5](../phases/05-command-bar.md) · [Phase 4](../phases/04-ui-shell.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Performance track](../tracks/performance-release.md) ·
[Performance contract](../../product/performance-contract.md) ·
[Interaction contract](../../product/browser-interactions.md) ·
[DESIGN.md](../../../DESIGN.md) §11.4/§11.5/§13/§15.

## Outcome

`Cmd-T` opens the **centered palette** and `Cmd-L` / a pointer click on the
sidebar command field opens the **field-anchored overlay** — two
presentations of the same production command engine with identical modes,
ranking, results, and activation. Staged semantics stay untouched (New Tab
creates neither a tab nor a native session until Enter commits a non-empty
destination; Escape cancels and restores the prior field text and focus
domain). Both surfaces expose deterministic `bbr-*` ids, typed snapshot
state, and §15 semantic nodes; at most one command surface is open at a time;
opening one clears the §11.3 cross-context profile hint; and
`palette_open`/filter/activate presentation samples are measured through
semantic presentation commit.

## Scope

- **Centered palette** (`bbr-palette`): `elev:2` overlay centered over the
  window content per DESIGN §11.4 — `material:overlay`, `win:palette`
  width (S 400 / N 480 / XL 560), `radius:lg` at S/N, `radius:xl` at XL,
  field row (`bbr-palette-field`/`-text`/`-mode`/`-cursor`), results
  (`bbr-palette-result-<k>`) using the §11.5 row anatomy, `selected`
  visuals, `accent.ink` match highlighting. Opacity fade `dur:fine` on open;
  never translate or scale.
- **Field-anchored overlay** (`bbr-command-overlay`): `elev:2` overlay
  anchored to the sidebar command field (top/leading edges flush, width =
  field width clamped to 320–`win:palette`), same result engine; results
  keep the existing §11.5 ids (`bbr-command-result-<k>`); Location mode
  shows the full URL selected in the field.
- **Entry routing** on the Root contract (`WindowHandle<Root>` +
  `Entity<BbrApp>`): `Cmd-T` → centered palette in New Tab staging mode;
  `Cmd-L` → field focus with full-URL selection + anchored overlay;
  pointer click on the field → same field-focused overlay. Overlay-window
  policy (in-window layer vs child window) follows the Root-anchor
  composition seam and stays a composition detail, not a visual change.
- **Command-surface state:** exactly one open surface at a time;
  open/close transitions clear the §11.3 `command_profile_hint` per the
  interaction contract; Escape/commit/outside press (one pointer action)
  close and restore the origin focus domain.
- **Snapshot and semantic exposure:** `bbr-palette*` and
  `bbr-command-overlay` in `visible_ids`, mode/result state on the typed
  snapshot, §15 semantic nodes mirroring both surfaces, and
  mutual-exclusion assertions.
- **Real-app E2E:** extend the production command-bar suites
  (`just e2e-shell-command`) with palette/overlay scenarios.
- **Performance:** `palette_open` (plus `palette_filter`/`palette_activate`
  when the filter/activation paths differ from the existing measured
  command-bar path) in the gate-enforced perf matrix with semantic
  presentation commit.
- **Docs:** this packet; DESIGN §11.4/§11.5/§17.6 (already prepared by the
  L4 docs lane); interaction-contract note only if a behavioral conflict
  surfaces; ROADMAP/phase/track promotion is parent-owned at integration.

## Non-goals

- No new command-engine features: no new modes, ranking groups, history
  provider (Phase 7), or result grouping polish (DESIGN §16).
- No IME/autocomplete work; no tab-insertion policy change (interaction
  contract open decision 1).
- No sidebar rail changes — region order, Favorites grid, Folders, footer,
  and the plus menu are the [U2 sidebar slice](browser-mvp-sidebar-ui.md).
- No split/window interactions, no downloads/links surface, no peek.
- No platform Accessibility exposure (GPUI 0.2.2 has no layer; Phase 9).

## Parallel-lane contract

```text
Base commit:          f89a4799999d1d43f785da7c767a1df68759ee0b — U-lanes branch only
                      from the reviewed green gpui-ce-components-anchor
                      integration commit (parallel-delivery rule; the
                      argument is recorded when L0 lands). This packet was
                      prepared docs-only at
                      574750bd129df628a0f74ff95e48579f4c12e0ef.
Depends on:           L0 gpui-ce-components-anchor integrated; Phase-5
                      command composition (ranking/activation seams,
                      integrated at 872d222); DESIGN §11.4/§11.5 revision
                      (this wave); existing e2e-shell-command harness.
Provides/API:         palette/overlay presentation module, entry routing,
                      command-surface state, typed snapshot + §15 nodes.
Exclusive paths:      crates/bbr-app/src/app/palette.rs (new) and its unit
                      tests; palette/overlay entry seams in the app command
                      state; additions to the e2e-shell-command suites.
Shared integration points: bbr-app composition root and global focus
                      routing; testkit snapshot wire fields; perf-matrix
                      scenario registration; ROADMAP/phase/track roll-ups.
Integration order:    L0 → L5 → U1 → U2 (serial; orchestrator merges).
Integration base arg: f89a4799999d1d43f785da7c767a1df68759ee0b, recorded after L0 integration and its reviewed testkit shutdown remediation; merge command runs against this recorded anchor.
```

## Constraints

- Runs on the one pinned GPUI-CE type world and the Root window contract;
  no crates.io fallback, no dual paths, no `gpui-component::webview`.
- Only chrome allowed to overlap the glass (§13): `elev:2`,
  `material:overlay`; open/close is an opacity fade `dur:fine` only —
  never translate, scale, or spring; Escape and one pointer action always
  dismiss (§13).
- Deterministic ids per DESIGN §11.4 (`bbr-palette*`,
  `bbr-palette-result-<k>`, `bbr-command-overlay`,
  `bbr-command-result-<k>`), §15 semantic mirror, typed snapshots; no pixel
  comparisons, no sleeps, deterministic fixtures and deadlines.
- Staged semantics are contract: Enter creates exactly one tab/session,
  Escape creates nothing and restores prior text + focus domain; pointer
  and keyboard origins keep their focus domain (interaction contract).
- The two surfaces are mutually exclusive; opening either clears the §11.3
  profile hint (never coexists).
- Performance: palette presentation samples through semantic presentation
  commit; absolute warm budgets and canonical-baseline enforcement per the
  performance contract; ≥30 samples in the full gate; fast-but-wrong guards
  reject missing/misplaced surfaces. GUI suites run sequentially under the
  host-wide lock; no weakened budgets, sample counts, or guards.
- Keep files under ~500 lines; no new dependencies.

## Evidence

### Unit and contract

- [ ] §15 semantic-tree tests: `bbr-palette` and `bbr-command-overlay`
      nodes render with the expected roles/names and mirror the visible
      surface; absent when closed.
- [ ] `visible_ids` derivation tests: palette/overlay ids exposed only
      while the respective surface is open.
- [ ] Command-surface state tests: mutual exclusion (opening one closes the
      other), profile-hint clearing, Escape restoration of field text and
      mode, New Tab staging (no tab/session before Enter).

### Real-app E2E

- [ ] Extended `just e2e-shell-command`: Cmd-T opens the centered palette
      in New Tab staging; typing filters results; Enter creates exactly one
      tab/session and moves focus into the page; Escape cancels and
      restores field text, mode, and chrome/web focus domain; Cmd-L and
      field click open the field-anchored overlay with the full URL
      selected; commit/outside press close; web and chrome origins covered;
      mutual exclusion and profile-hint clearing asserted.

### Performance

- [ ] `palette_open` scenario in the gate-enforced matrix: 30 samples of
      Cmd-T open measured through semantic presentation commit; warm
      50/100 ms budget class; driver asserts the palette ids and staged
      mode; guard rejects fast-but-wrong (missing/invented surface).
- [ ] `palette_filter`/`palette_activate` added only if their latency path
      differs from the already-measured command-bar filter/activate path;
      otherwise covered by the existing matrix rows.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-command   # extended palette/overlay suites
just e2e-perf
just e2e
just roadmap
```

## Delivery sequence

1. One exclusive writer implements the slice on the recorded post-anchor
   base and verifies targeted checks without committing.
2. Parent inspects the diff and runs the gates.
3. One independent read-only reviewer maps requirements to evidence.
4. One remediation writer only for concrete findings, then a focused
   recheck.
5. Parent commits and merges serially on the anchor with the recorded
   integration base arg.

## Iteration policy

Start with the smallest end-to-end path: the centered palette on the Root
contract with Cmd-T routing and staged New Tab, verify with targeted
semantic/state unit tests, then add the field-anchored overlay and the
click/Cmd-L entries, then E2E and the perf scenario. After each failing
check, identify the mechanism before changing the implementation; run
targeted feedback while iterating and the canonical sequential gate once the
slice is stable. Do not broaden scope to solve unrelated findings.

## Blocked stop condition

If no defensible path remains, stop with evidence gathered, approaches
attempted, preserved worktree/branch state and owner, the exact blocker, and
the smallest environment, contract decision, or user input required next
(e.g. an overlay-window API gap in the pinned GPUI-CE revision, or a DESIGN
§11.4 presentation ruling that contradicts the interaction contract).

## Delivery ledger

```text
Outcome:       reviewed prep complete; implementation remains pending in U1
Commits:       parent-owned docs integration pending from the reviewed L4 lane;
               implementation commits remain none
Functional evidence: pending
Real-app E2E:  pending
Performance evidence: pending
Roadmap/documentation updates: DESIGN §11.4/§11.5/§17.6 revised by L4;
               ROADMAP/phase/track linking is parent-owned at promotion
Open blockers and non-goals: L0 base is recorded; history provider (Phase 7),
               result grouping, and sidebar rail (U2) remain excluded
Worktree state: reviewed L4 docs diff at base 574750b; parent promotion records
               U1 base f89a4799999d1d43f785da7c767a1df68759ee0b
```

Planning/read-only, implementation, test execution, review, remediation,
waiting/retries, total elapsed, and human intervention fields follow the
template at implementation time; none measured here (docs-only prep).
