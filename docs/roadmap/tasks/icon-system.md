# Task packet — Icon system (BBR-drawn §7 chrome glyphs)

Parents: [UI design system track](../tracks/ui-design-system.md) · Phase 4/9
shell craft · Product: [DESIGN.md §7/§14](../../../DESIGN.md)

## Outcome

Replace the Unicode/placeholder chrome glyphs the first shell slices shipped
(tab close `×`, pin `●`/`○`, loading `●`, new-space `+`, crash `!`,
empty-pane brand `B`, and the status-line `⚠`) with a small reusable,
BBR-drawn GPUI geometric instrument glyph set that matches DESIGN.md §7 —
consistent 1.5 pt stroke (1.25 pt at S density), squared geometry, luminance
matched to `ink.secondary` at rest — while preserving every id, hit target,
label, semantic, focus path, listener, performance budget, and Chassis & Glass
token. No emoji/Unicode substitutes remain in the component-owned chrome.

**Source-drift correction (current main):** the core swaps above were merged
via the original lane commit `9f81f71` (`feat(ui): draw native chrome
icons`) and the integration commit `2e10037` (`feat(ui): integrate the
geometric icon system`). The one remaining component-owned placeholder this
packet now closes is the find bar's `‹`/`›`/`✕` step/close controls in
`crates/bbr-app/src/portal/find.rs` (a later find-in-page slice shipped
Unicode instead of drawn glyphs), rendered through the same reusable
`components::icons` catalog — plus the chevron geometry/name/color contract
tests that lock the catalog extension.

## Scope

- **Icon module** `crates/bbr-app/src/components/icons.rs` (merged base): a
typed `Icon` enum (`Close`, `Pin { pinned }`, `Lifering`, `ChassisMark`,
`Plus`, `LoadingArc`, `ChevronLeft`, `ChevronRight`) rendered through GPUI's
vector path/quad paint API (`PathBuilder`/`paint_path`/`paint_quad` inside a
sized non-interactive `canvas`). No external icon dependency, no SVG assets,
no gradients, no emoji.
- **Glyph swaps (bounded, component-owned files only):**
  - `tab_row.rs` — close control `×` → `Icon::Close`; pin control `●`/`○` →
    `Icon::Pin { pinned }` (disabled state keeps `ink.faint`); favicon
    loading `●` → `Icon::LoadingArc` (accent, static). *(Merged base.)*
  - `pane.rs` — crash `!` → `Icon::Lifering` (32 pt, `status.error`); empty
    pane brand `B` → `Icon::ChassisMark` (64 pt chassis frame + glass
    aperture). *(Merged base.)*
  - `space.rs` — new-space `+` → `Icon::Plus`. *(Merged base.)*
  - `shell/presentation.rs` — status-line `⚠` removed (§14: no emoji in
    chrome); the crash line reads "Web content process terminated" and the
    error state keeps its §3.3 non-color companions (drawn lifering + error
    surface + panel copy). *(Merged base.)*
  - `portal/find.rs` — find-bar step `‹`/`›` → `Icon::ChevronLeft` /
    `Icon::ChevronRight` and close `✕` → `Icon::Close`, at the
    `icon:<density>` size inside the unchanged `density:hit-min` /
    `density:hit-min.close` hit squares. *(This lane.)*
- **Tests:** lower-level geometry/name tests for the icon catalog
  (stable names, §7 stroke/density contract, glyphs fit their boxes, chassis
  radius stays in the §5 range, palette-derived rest colors, chevron
  geometry) and a presentation unit test proving the status line carries no
  emoji glyphs. *(Chevron additions and their contract tests: this lane.)*
- **Docs:** this packet, linked from the ui-design-system track as the
  canonical owner.

## Non-goals

- **Command composition surface** — the command-field prefix `▶`, the sidebar
  brand text "BBR", and `app/sidebar_view.rs` stay untouched (command
  composition owns them).
- **New behavior/motion** — the loading arc is drawn statically; the §10.5
  rotation is transient-indication motion for a later polish pass. No new
  testkit wire field, no new performance scenario, no semantic/protocol
  change.
- **Hit-target sizing** — the §7 ≥ 24 pt (N/XL) / ≥ 20 pt (S) close hit
  square is a layout change that rides a later polish pass; this slice
  preserves the current hit targets exactly.
- **Icon catalog expansion** — no speculative icons beyond the replaced
  placeholders; the favicon-slot error lifering for crashed rows (§10.7) is a
  separate visual change owned by a later row-state slice.
- **GUI E2E / performance runs** — visual replacement with no behavior change
  needs no new scenario; the existing real-app gates run at integration under
  the host-wide GUI lock.

## Parallel-lane contract

```text
Base commit:      9960079 (current main; the core icon slice merged earlier
                  at 2e10037 already carried the components/* + presentation
                  swaps, so this lane is a bounded find-bar completion)
Depends on:       the merged components::icons catalog, DESIGN §7/§14
Provides/API:     Icon::{ChevronLeft, ChevronRight} added to the existing
                  catalog; find-bar controls render catalog glyphs (no API
                  change for other callers)
Exclusive paths:  crates/bbr-app/src/components/icons.rs,
                  crates/bbr-app/src/portal/find.rs
Shared integration points: components/icons.rs (F2a may add its own footer/
                  space-strip catalog variants — enum-union merge expected)
Integration order: after this lane is reviewed; canonical `just e2e` under
                  the host-wide GUI lock
```

## Constraints

- Only files on the lane's exclusive allowlist are modified; `app.rs`,
  `app/render.rs`, `app/sidebar_view.rs`, testkit protocol, justfile, root
  manifests, `ROADMAP.md`, and portable crates stay untouched.
- Stable element ids (`bbr-tab-close-<n>`, `bbr-tab-pin-<n>`,
  `bbr-tab-favicon-<n>`, `bbr-crash-panel`, `bbr-empty-new-tab`,
  `bbr-space-new`, `bbr-status`, `bbr-find-previous`, `bbr-find-next`,
  `bbr-find-close`), labels, semantics, focus, and listener paths are
  preserved byte-for-byte where the tree asserts them.
- Colors come only from the palette tokens (`ink.secondary`, `ink.faint`,
  `accent`, `status.error`); no raw arbitrary colors, gradients, glow, or
  emoji.
- The semantic tree and testkit snapshot are untouched (no protocol change);
  the status node derives its name from `shell::status_line`, so render and
  semantics stay aligned automatically.
- Files stay under ~500 lines; no weakened tests, budgets, or validators.

## Evidence

### Unit and contract

- [x] `crates/bbr-app/src/components/icons.rs` tests: catalog names stable,
      §7 stroke weight per density (1.25 S / 1.5 N·XL), glyph geometry fits
      its box at every shipped size, chassis frame radius stays in the §5
      4–8 px range with the aperture inside the frame, chevron arm fits at
      every shipped size, and rest colors resolve from the palette.
- [x] `crates/bbr-app/src/shell/presentation.rs` test: the crash status line
      is exactly "Web content process terminated" and ASCII-only (no
      emoji-presentation glyphs), and non-crash statuses are unchanged.

### Real-app E2E

- [x] `just e2e-shell-find` stays green — 6/6 (find-bar visual swap over
      the real portal + WKWebView). No new scenario: this is visual
      replacement with no behavior change (AGENTS.md browser-feature rule).
- [ ] `just e2e-shell-portal` is **5/7**, not green. Both failures
      reproduce identically on clean base `9960079` and are pre-existing,
      not lane-caused: `host_hides_on_minimize_and_restores` (minimize
      hit-test environment blocker — `CGSSessionScreenIsLocked` / foreign
      frontmost-window interference) and
      `host_tears_down_when_parent_closes_with_open_presentation`
      (parent-close Broken-pipe base regression from the current
      quit-on-owning-browser-window behavior introduced by the auth merge
      `53a9590` and re-landed by F2c `26e19c1`). Neither failure is called
      passed.
- [ ] Canonical `just e2e` is **not green**: the run passed `e2e-smoke`
      and then stopped because the current tracked justfile references the
      missing test target `e2e_shutdown`; no such test file currently exists
      in any worktree. Consequently `e2e-perf` did not run as part
      of the canonical e2e; no integration evidence is claimed.

### Performance

- [x] No new scenario and no new perf kind: glyphs are static paths/quads
      and only replace text children on existing elements.
- [ ] Full performance remains an integration gate, **pending**: `e2e-perf`
      did not run as part of the canonical `just e2e` (stopped at the
      missing `e2e_shutdown` target).

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr --features testkit
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just verify
just e2e-shell-find
just e2e-shell-portal
just e2e
just roadmap
git diff --check
```

GUI E2E (`just e2e-*`) runs under the host-wide GUI lock.

Gate status: `just e2e-shell-portal` is 5/7 with two pre-existing base
failures (see Real-app E2E) and canonical `just e2e` stops at the missing
`e2e_shutdown` test target, so `e2e-perf` has not run; both remain open
integration gates before this roadmap item can be marked complete.

## Delivery sequence

1. One exclusive writer (this lane) audits the merged base for remaining
   component-owned Unicode chrome glyphs, implements the bounded find-bar
   completion, and verifies the slice.
2. One independent read-only reviewer maps requirements to evidence.
3. One remediation writer only for concrete findings, then a focused recheck.
4. Integration runs the canonical `just e2e` under the host-wide GUI lock.

## Iteration policy

Start from the smallest end-to-end path (the two catalog chevron variants +
find-bar step controls), verify with targeted unit tests and clippy, then
confirm the close control and the no-Unicode presentation contract. After
each failing check, identify the cause and change approach before retrying;
never weaken a check to pass.

## Blocked stop condition

If the GPUI 0.2.2 vector API cannot express a required glyph, report the exact
primitive gap and the alternative (e.g. div-composed shapes) with evidence
before proceeding; the slice is blocked only if no token-conformant drawn
glyph is achievable for a listed placeholder.

## Delivery ledger

```text
Outcome:       The merged icon catalog covers every component-owned chrome
               glyph; the find bar's `‹`/`›`/`✕` placeholders are replaced
               by `Icon::ChevronLeft`/`ChevronRight`/`Close` at the
               `icon:<density>` size; ids/hit squares/listeners/semantics
               unchanged; chevron geometry/name/color contract tests added.
Commits:       `bf42a8a feat(ui): draw find bar control icons` — committed
               by the orchestrator after REVIEW_PASS and final gates; not
               merged into `main`.
Functional evidence: cargo test -p bbr --features testkit --bin bbr
               252/252 (5 icon-catalog tests incl. the new chevron
               arm/name/rest-color assertions); `cargo fmt --all --
               --check`, `cargo clippy -p bbr --features testkit
               --all-targets -- -D warnings`, `just verify`, `just
               roadmap`, and `git diff --check` green.
Real-app E2E:  `just e2e-shell-find` 6/6 green. `just e2e-shell-portal`
               **5/7** — the two failures reproduce identically on clean
               base 9960079 (pre-existing, not lane-caused):
               `host_hides_on_minimize_and_restores` (minimize hit-test
               environment blocker) and
               `host_tears_down_when_parent_closes_with_open_presentation`
               (parent-close Broken-pipe base regression from the current
               quit-on-owning-browser-window behavior). Neither is called
               passed. Canonical `just e2e` not green: e2e-smoke passed,
               then stopped at the missing `e2e_shutdown` test target.
Performance:   no new scenario and no new perf kind (static paths replacing
               text children); full performance remains an integration
               gate, pending — `e2e-perf` did not run as part of canonical
               `just e2e`.
Roadmap/docs:  this packet corrected for source drift + truthful gate
               status; ui-design-system track item left unchecked with an
               awaiting-integration note — the code lane is REVIEW_PASS and
               frozen, but the roadmap item is not complete before its
               declared integration gates.
Open blockers and non-goals: command-composition surface (`▶` prefix, `↵`/
               `→` result marks), sidebar brand text, and the `⌘T` key cue
               stay with their owning surfaces; loading arc static; §7
               hit-target minimums already settled via density tokens.
Worktree state: clean at `bf42a8a`; frozen pending serial orchestrator
               integration.
File sizes:    icons.rs 544 now vs 508 at base — bounded +36 catalog
               extension (chevron variants + contract tests); planned
               single-authority enum-union merge with F2a catalog variants.
```
