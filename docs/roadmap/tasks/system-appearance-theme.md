# Task packet — System-appearance production theme (light + dark)

Parents: [Phase 4](../phases/04-ui-shell.md) · [UI design-system track](../tracks/ui-design-system.md) · [DESIGN.md](../../../DESIGN.md) · [Reference-faithful production shell](visual-shell-reference.md).

Decision authority: Patrick settled on 2026-08-13 that the production shell
follows the **macOS system appearance** — a dark system appearance presents
the complete dark token set, a light system appearance presents the shipped
reference-calibrated light rail. Neither mode is hardcoded. Patrick's dark
reference capture
(`/Users/patwoz/Library/Application Support/CleanShot/media/media_lbaS6F07Ws/Screenshot_2026-08-12_21-45-38@2x.png`)
is the composition authority for the dark variant; it shares the light
reference's hierarchy and geometry and only swaps the token set.

## Outcome

BBR's production shell derives its `ThemeMode` from the actual macOS system
appearance through a public GPUI API, presents the complete dark token set on
a dark system, keeps the shipped light rail on a light system, and re-presents
a live appearance change commit-first without restart. Both variants pass the
unchanged §3 contrast contracts, and E2E can force either mode
deterministically.

## Scope

- One production theme-selection authority in `bbr-app` that maps the GPUI
  window appearance to `ThemeMode`, replacing the hardcoded
  `ThemeMode::Light` in the render composition. No second theme source.
- Live appearance-change observation: a system appearance flip re-renders the
  complete chrome with the other token set on the next presentation commit —
  no cross-fade, no intermediate mixed-token frame (§6 commit-first).
- Dark counterparts for every reference-calibrated surface introduced by the
  light shell calibration (favorite tile, footer strip, address pill,
  selection tint, row washes) verified against the unchanged §3.1–§3.5
  contracts; missing or stale dark values are corrected in the `bbr-ui` token
  module, never inline in render code.
- Testkit: a deterministic appearance override request/response so real-app
  E2E can force light and dark regardless of host appearance; the existing
  §15 `theme (light|dark)` snapshot field reports the effective mode.
- DESIGN.md update in the same group: §1 production-default wording changes
  from "settled default is Light" to system-appearance-driven with light as
  the calibrated reference; the dark reference capture above is recorded as
  the dark composition authority.
- Gallery: existing mode-matrix stories prove the changed dark surfaces; new
  stories only where a changed production surface has no dark story yet
  (append-only StoryIds).

## Non-goals

- No vibrancy/`NSVisualEffectView` (§2 deferred).
- No user-facing theme preference or per-space light/dark override; the
  system appearance is the only selector in this slice.
- No space-tint contract change (§11.3 wash math already defines both modes).
- No favicon, row-pitch, folder-anatomy, footer-glyph, or media work — owned
  by the sibling alignment packets.

## Parallel-lane contract

```text
Base condition:          integrated VS-REFERENCE shell on local main
Depends on:              none
Provides/API:            production ThemeMode selection authority; testkit appearance override
Exclusive paths:         bbr-app theme selection/composition seam; bbr-ui dark token values
Shared integration points: bbr-ui theme module, testkit protocol, Gallery registry, justfile, DESIGN.md
Integration order:       before sidebar-reference-polish (polish calibrates both modes)
```

## Constraints

- `bbr-ui` remains the only token/renderer authority; no hardcoded render
  colors and no Gallery-local shell.
- Contrast ratios are recomputed from tokens by the existing gate, never
  copied; no §3 assertion is weakened for a dark value.
- Appearance flip is a state change: `dur:instant`, same presentation-commit
  mechanism, no animation.
- Existing ids, states, interaction routes, dense bounds and performance
  budgets/sample counts are unchanged. Performance execution remains blocked
  under the active override; performance code must keep compiling unchanged.
- No private AppKit/WebKit API; the web content itself is never tinted or
  sampled (§13).

## Evidence

### Unit and contract

- [ ] Theme-selection authority maps both appearances and the override
      deterministically.
- [ ] Dark palette pairs for every changed surface pass the recomputed §3
      contrast gate (standard + increased contrast).

### Real-app E2E

- [ ] Forced-dark launch: snapshot reports `theme == dark`; sidebar, selected
      row, hover wash, footer and address surfaces assert their §10/§11
      states unchanged.
- [ ] Live flip mid-session: appearance override change re-presents the full
      chrome; `theme` field flips; selection/session identity preserved.
- [ ] Forced-light launch remains identical to the integrated VS-REFERENCE
      evidence.

### Performance

- [ ] Not executed: performance execution remains blocked under the active
      override. Scenario code, budgets and sample guards compile unchanged.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr-ui -p bbr-browser
cargo test -p bbr --features testkit --bins
cargo clippy -p bbr-ui --all-targets -- -D warnings
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just e2e-shell-sidebar
just e2e-shell-hover
just e2e-shell-spaces
just e2e-gallery
just roadmap
```

## Delivery sequence

1. One exclusive writer implements and verifies the bounded slice.
2. One independent read-only reviewer maps requirements to evidence.
3. One remediation writer only for concrete findings, then a focused recheck.
4. No separate final audit; phase closeout owns the full sequence.

## Iteration policy

Start with the selection authority and forced-mode E2E, then correct dark
token values in descending visual impact against the dark reference capture.
After each failing check, identify the mechanism before changing the
implementation.

## Blocked stop condition

Stop fail-closed if GPUI exposes no public appearance signal usable for live
observation, if a dark surface cannot meet the unchanged contrast contract
without a light-side regression, or if a required gate fails without a
defensible bounded fix. Preserve exact fingerprints, findings and the
smallest input needed.

## Delivery ledger

```text
Outcome:
Commits:
Functional evidence:
Real-app E2E:
Performance evidence:
Roadmap/documentation updates:
Open blockers and non-goals:
Worktree state:
```
