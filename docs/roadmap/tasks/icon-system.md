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

## Scope

- **New icon module** `crates/bbr-app/src/components/icons.rs`: a typed
  `Icon` enum (`Close`, `Pin { pinned }`, `Lifering`, `ChassisMark`, `Plus`,
  `LoadingArc`) rendered through GPUI's vector path/quad paint API
  (`PathBuilder`/`paint_path`/`paint_quad` inside a sized non-interactive
  `canvas`). No external icon dependency, no SVG assets, no gradients, no
  emoji.
- **Glyph swaps (bounded, component-owned files only):**
  - `tab_row.rs` — close control `×` → `Icon::Close`; pin control `●`/`○` →
    `Icon::Pin { pinned }` (disabled state keeps `ink.faint`); favicon
    loading `●` → `Icon::LoadingArc` (accent, static).
  - `pane.rs` — crash `!` → `Icon::Lifering` (32 pt, `status.error`); empty
    pane brand `B` → `Icon::ChassisMark` (64 pt chassis frame + glass
    aperture).
  - `space.rs` — new-space `+` → `Icon::Plus`.
  - `shell/presentation.rs` — status-line `⚠` removed (§14: no emoji in
    chrome); the crash line reads "Web content process terminated" and the
    error state keeps its §3.3 non-color companions (drawn lifering + error
    surface + panel copy).
- **Tests:** lower-level geometry/name tests for the icon catalog
  (stable names, §7 stroke/density contract, glyphs fit their boxes, chassis
  radius stays in the §5 range, palette-derived rest colors) and a
  presentation unit test proving the status line carries no emoji glyphs.
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
Base commit:      ec52bf07aba70a3071aa98f3b4d635b252c68b20
                  (parallel/icon-system, clean at activation)
Depends on:       the Phase-4 shell slice components (tab_row/space/pane),
                  the §7/§14 DESIGN contract
Provides/API:     crate::components::icons::{Icon, icon_stroke, ...}
                  — canvas-based glyphs; callers keep their own ids/hit
                  squares/listeners/labels on the wrapping divs
Exclusive paths:  crates/bbr-app/src/components/icons.rs (new),
                  crates/bbr-app/src/components/{mod,tab_row,space,pane}.rs,
                  crates/bbr-app/src/shell/presentation.rs
Shared integration points: none beyond the unchanged component signatures
                  (`close_control`/`pin_control`/`favicon_tile`/
                  `space_new_row`/`crash_panel`/`empty_space_pane` keep their
                  exact call shapes, so render.rs/sidebar_view.rs compile
                  unchanged)
Integration order: after the icon slice is reviewed; the canonical `just e2e`
                  gate runs at integration (not in this lane)
```

## Constraints

- Only files on the lane's exclusive allowlist are modified; `app.rs`,
  `app/render.rs`, `app/sidebar_view.rs`, testkit protocol, justfile, root
  manifests, `ROADMAP.md`, and portable crates stay untouched.
- Stable element ids (`bbr-tab-close-<n>`, `bbr-tab-pin-<n>`,
  `bbr-tab-favicon-<n>`, `bbr-crash-panel`, `bbr-empty-new-tab`,
  `bbr-space-new`, `bbr-status`), labels, semantics, focus, and listener
  paths are preserved byte-for-byte where the tree asserts them.
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
      4–8 px range with the aperture inside the frame, and rest colors
      resolve from the palette.
- [x] `crates/bbr-app/src/shell/presentation.rs` test: the crash status line
      is exactly "Web content process terminated" and ASCII-only (no
      emoji-presentation glyphs), and non-crash statuses are unchanged.

### Real-app E2E

- [ ] Existing shell suites must stay green — run at integration under the
      host-wide GUI gate (not run in this lane by design). No new scenario:
      this is visual replacement with no behavior change (AGENTS.md
      browser-feature rule).

### Performance

- [ ] No new scenario: glyphs are static paths/quads and only replace text
      children on existing elements; the gate-enforced matrix runs at
      integration.

## Completion gates

```bash
cargo fmt --all -- --check
cargo test -p bbr --features testkit
cargo clippy -p bbr --features testkit --all-targets -- -D warnings
just roadmap
git diff --check
```

GUI E2E (`just e2e-*`) is deliberately excluded from this lane and runs at
integration.

## Delivery sequence

1. One exclusive writer (this lane) implements and verifies the bounded
   slice.
2. One independent read-only reviewer maps requirements to evidence.
3. One remediation writer only for concrete findings, then a focused recheck.
4. Integration of `parallel/icon-system` runs the canonical `just e2e` under
   the host-wide GUI lock.

## Iteration policy

Start from the smallest end-to-end path (the icon module + the two row
controls), verify with targeted unit tests and clippy, then expand to the
panes/space/status swaps. After each failing check, identify the cause and
change approach before retrying; never weaken a check to pass.

## Blocked stop condition

If the GPUI 0.2.2 vector API cannot express a required glyph, report the exact
primitive gap and the alternative (e.g. div-composed shapes) with evidence
before proceeding; the slice is blocked only if no token-conformant drawn
glyph is achievable for a listed placeholder.

## Delivery ledger

```text
Outcome:       All listed placeholders replaced by drawn §7 glyphs; ids/
               semantics/focus/listeners unchanged; unit + clippy + testkit
               green.
Commits:       left uncommitted on parallel/icon-system for review
               (no commit/push by lane policy in this run).
Functional evidence: cargo test -p bbr --features testkit (64 unit +
               all non-GUI integration binaries green); targeted
               `icons`/`presentation` tests green.
Real-app E2E:  not run (GUI gate at integration); existing suites unchanged.
Performance:   not run (no new scenario; presentation-only change).
Roadmap/docs:  this packet + ui-design-system track link (planning/evidence;
               no checkbox claimed complete).
Open blockers and non-goals: command-prefix/sidebar brand untouched; loading
               arc static; §7 hit-target minimums deferred to a layout slice.
Worktree state: clean base; implementation diff pending review.
```
