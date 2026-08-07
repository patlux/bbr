# Task packet — global UI scale and density pass

Parents: [Phase 9](../phases/09-quality-release.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Testing track](../tracks/testing.md) ·
[Performance track](../tracks/performance-release.md) ·
[DESIGN.md](../../../DESIGN.md) §4/§5/§8/§11/§15.

Status: **queued as the first bounded slice after browser-MVP closeout.** It
must not delay U1 integration, U2 performance/integration, canonical `just e2e`,
or final `just package-verify` and delivery of the first clickable `.app`.

## Outcome

BBR chrome is comfortably readable at its normal density after one coherent
DESIGN-token revision: typography, leading, row heights, spacing, control/hit
target sizes, sidebar geometry, and palette/overlay geometry increase together
without changing product behavior. The typed `Theme`/`Density` system remains
the single sizing authority; no surface receives an isolated local enlargement.

## User evidence — calibrated reference, not provisional guess

Patrick's live U1 palette test: the presentation already looks good, but the
entire UI is clearly too small. This is a global shell-scale finding, not a
palette-only defect. Preserve the visual direction and interaction model while
raising the complete chrome scale as one system.

Patrick then supplied reference screenshots, preserved at
`/Users/patwoz/dev/Personal/macos/bbr-design-references/`:

- `arc-sidebar-full.png` — whole-sidebar target proportion (2× capture);
- `arc-tab-row-hover.png` — row hover/active detail with the inset rounded row
  surface and trailing close control (2× capture).

His instruction: the rest of the sidebar should feel approximately like this —
noticeably more room, larger text, less compact than today. The resulting
target values for the revised table are therefore **calibrated against a user
reference** and are no longer a provisional percentage:

- normal row height approximately 36–40 pt;
- effective title text approximately 14–15 pt;
- favicon/glyph approximately 18–20 pt;
- hover/active as an inset rounded row surface with the close control inside it;
- generous section spacing between regions;
- Favorites as large rounded icon tiles.

These are the calibration target, not literal token values: the slice still
authors one internally consistent S/N/XL table that satisfies them at normal
density. BBR keeps its own materials, color discipline, iconography, semantics,
and native-web boundary — the reference calibrates scale, rhythm, and row
affordance only, never branding or behavior. Dense-500 windowing, scroll math,
and every performance budget remain unchanged and gate this slice.

## Scope

- Decide and record one revised, internally consistent scale in `DESIGN.md`
  before production code changes, calibrated against the preserved reference
  screenshots above rather than a blanket multiplier:
  - typography sizes and line-height/leading;
  - base spacing rhythm where chrome readability depends on it;
  - tab, space, favorite, folder, footer, menu, field, result, and button rows;
  - icon/control sizes and minimum pointer hit targets;
  - sidebar normal/minimum/default/maximum geometry where required by the new
    control scale;
  - centered palette and field-anchored overlay width/height/padding geometry.
- Apply that decision through the existing central `Theme`, `TypeScale`,
  `Spacing`, and `Density` seams. Move any size literal that blocks coherent
  scaling into the nearest canonical token seam instead of tuning its component
  locally.
- Preserve S/N/XL as deliberate collection densities. Revise their token table
  coherently; do not replace density with arbitrary per-surface scaling.
- Update token/unit tests and every affected real-app geometry assertion to
  derive expectations from the revised contract.
- Rebuild the packaged app for a manual readability smoke after automated gates.

## Non-goals

- No reopening or delaying the MVP U1 → U2 → packaging sequence.
- No browser-page zoom, site zoom, WKWebView scaling, or native webpage geometry
  change; this slice scales BBR-owned chrome only.
- No runtime scale slider, preference persistence, macOS Accessibility setting,
  or automatic display-DPI policy.
- No information-architecture, command behavior, ranking, focus-routing,
  sidebar region-order, tab/session, profile/space, or Portal-host change.
- No color, material, radius-language, motion, icon-design, copy, or broad visual
  redesign unless a size token mechanically requires a matching optical size.
- No scattered component-specific overrides and no blanket render transform.

## Parallel-lane contract

Not parallelized. This is the first serial slice from the verified browser-MVP
closeout commit.

```text
Base commit:          final MVP closeout commit after U1, U2, canonical E2E,
                      package-verify, and delivery-ledger settlement
Depends on:           integrated U1 NativeAboveWeb palette/overlay; integrated
                      U2 sidebar; final packaged-app baseline
Provides/API:         revised DESIGN sizing contract and central GPUI token
                      values; no public behavior or wire API change
Allowed scope:        DESIGN.md; bbr-app central theme/density/geometry token
                      seams and direct token consumers; affected token,
                      snapshot, geometry, E2E, perf-guard, roadmap, and
                      architecture documentation only
Shared integration points: sidebar/native-content geometry, Portal placement,
                      typed snapshots, performance fast-but-wrong guards
Integration order:    MVP closeout → this slice → deferred P8 integrations and
                      other post-MVP feature work
```

## Constraints

- `DESIGN.md` is changed first and remains canonical. Production constants and
  tests must name or derive from its tokens; no unexplained magic sizes.
- One scale decision: preserve relative hierarchy and the 4 pt chassis rhythm
  while raising the system as a whole. Do not enlarge only the U1 palette or
  only selected sidebar controls.
- Stable `bbr-*` semantic IDs, roles, names, hierarchy, focus domains, typed
  events, and interaction semantics remain unchanged.
- Performance scenario names, sample counts, semantic presentation commits,
  absolute budgets, and fast-but-wrong guards remain unchanged. Fix performance
  regressions; never relax budgets because geometry is larger.
- Uniform per-density row heights and deterministic dense-list scroll math remain
  exact. The 500-tab bounded-window lifecycle and render-window contract remain
  unchanged.
- Native content stays strictly positive at compact widths. Sidebar enlargement
  may not starve, overlap, screenshot, texture-compose, or reparent WKWebView.
- `NativeAboveWeb` Portal ownership, AppKit z-order, pointer hit-testing, and
  dismissal behavior remain unchanged; only token-derived chrome geometry moves.
- No new dependency. Keep modified/new modules below roughly 500 lines and do
  not materially enlarge legacy files.

## Evidence

### Unit and contract

- [ ] Token tests prove the exact revised S/N/XL typography, leading, row,
      spacing, icon, hit-target, sidebar, and palette geometry from `DESIGN.md`.
- [ ] Static/review evidence finds no new local scale multiplier or duplicated
      palette/sidebar/row size literals in direct token consumers.
- [ ] Dense-list and truncation tests prove uniform row math, bounded rendering,
      and one-line title/URL behavior at all three densities.

### Real-app E2E

- [ ] Affected shell suites prove the larger normal-density sidebar, command
      field, centered palette, anchored overlay, Favorites/Folders/footer,
      tab/space rows, pointer targets, focus rings, truncation, and compact-window
      behavior through the production GPUI path.
- [ ] Portal E2E proves the revised palette/overlay bounds remain inside the
      window, above native WebKit, pointer-reachable, correctly anchored/centered,
      and dismissed by the existing routes.
- [ ] Geometry assertions use shared token-derived expectations where possible;
      obsolete literals are replaced, not loosened. Stable semantic IDs and
      typed snapshots remain byte-for-byte compatible except documented numeric
      geometry values.
- [ ] Manual packaged-app smoke confirms the normal-density palette and sidebar
      are comfortably readable while the website remains unscaled, and that the
      sidebar proportion, row generosity, and row hover affordance match the
      calibrated reference intent.

### Performance

- [ ] Existing startup and interaction scenarios run unchanged, including
      command/palette, sidebar, tab/space, organize, dense, Favorites, Folders,
      plus-menu, and Portal presentation paths.
- [ ] `just e2e-perf` passes the existing absolute budgets and sample-count
      gates; no budget, timeout, or semantic-commit change is part of the slice.

## Completion gates

```bash
just verify
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
cargo clippy -p bbr-testkit --all-targets -- -D warnings
just e2e-shell-tabs
just e2e-shell-spaces
just e2e-shell-sidebar
just e2e-shell-organize
just e2e-shell-dense
just e2e-shell-hover
just e2e-shell-states
just e2e-shell-command
just e2e-shell-portal
just e2e-perf
just e2e
just package-verify
just roadmap
```

Run additional U2-specific bounded recipes present on the integrated MVP commit.
All GUI recipes run sequentially under the host-wide lock.

## Delivery sequence

1. Preserve the shipped MVP `.app` as the before-baseline and inventory every
   chrome size from `DESIGN.md`, central tokens, and direct geometry consumers.
2. Propose one revised S/N/XL table; settle it in `DESIGN.md` before code.
3. Change central tokens and only the direct consumers that cannot already derive
   from them; update token and geometry tests in the same group.
4. Run targeted real-app suites, then unchanged optimized performance evidence,
   canonical sequential E2E, and a fresh package verification.
5. One independent read-only review maps every changed visual dimension back to
   the single token decision and checks for local exceptions.
6. One focused remediation only for concrete findings, then affected rechecks.

## Iteration policy

Start with normal density because that is the live-tested failure, but author the
complete S/N/XL relationship before implementation so the system cannot drift.
For each failure, classify it as token contract, direct-consumer literal,
truncation/layout, native-content geometry, Portal placement, E2E expectation,
or performance. Change the smallest central cause and re-run the affected suite;
never patch individual controls until they look approximately right. Run the
canonical gates once the whole scale is stable.

## Blocked stop condition

If one coherent token table cannot make normal density readable without breaking
compact-window native-content geometry, deterministic dense-list math, Portal
placement, or unchanged performance budgets, stop with the candidate table,
measured failing geometry/timings, affected densities and windows, preserved
branch state, and the smallest product decision required. Do not ship mixed
local scales, overlap web content, or weaken assertions/budgets.

## Delivery ledger

```text
Outcome: queued from Patrick's live U1 feedback; implementation intentionally
        deferred until the browser-MVP .app is closed out
Commits: planning/roadmap commit only; implementation none
Functional evidence: pending post-MVP slice
Real-app E2E: pending post-MVP slice
Performance evidence: pending post-MVP slice; budgets remain unchanged
Roadmap/documentation updates: task packet, Phase 9, UI track, root roadmap,
        browser-MVP delivery ledger
Open blockers and non-goals: exact revised token values are deliberately decided
        inside the slice; no MVP blocker and no page zoom/runtime preference
Worktree state: not created; first serial branch from final MVP closeout
```
