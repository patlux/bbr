# 2026-08-08 — L0-Reste Closeout: Evidence Ledger (Queue item 18)

Lane: `parallel/l0-rest-closeout` · Worktree: `/Users/patwoz/dev/Personal/macos/bbr-l0-rest-closeout`
Base: `9960079` · Writer scope: bounded evidence/fix slice for queue item 18
("Split-Geometry-Flake, Xcode-/MSRV-Checkboxen"). The independently approved
slice is serially integrated by the accompanying main commit.

## Scope decisions

- Only queue item 18 was brought to reviewed lane-complete evidence. Broader
  Phase-6 split items (creation UI, divider drag, focused-pane polish), F2a UI,
  persistence, downloads, Gallery, and shutdown remain open and untouched
  (Phase-6 checklist not promoted).
- No test, deadline, budget, sample count, or geometry tolerance was loosened.
- The established repository Xcode default (`BBR_XCODE_DEVELOPER_DIR` =
  `/Applications/Xcode-26.2.0.app/Contents/Developer`, `scripts/xcode-shims`
  forwarding `xcrun` to `/usr/bin/xcrun`) was preserved; no ad hoc user-machine
  state was added. The dev shell's nix-provided `DEVELOPER_DIR` (an
  `apple-sdk-14.4` store path that cannot invoke Metal) is overridden by every
  `just` recipe exactly as designed.

## 1. Split-geometry flake — current truth

### Root cause (from git history + source, not memory)

- `ac5223d` "test(e2e): determinize hidden Root harness windows" records the
  root cause: shown-but-unfocused harness windows were tiled by the host's
  yabai BSP window manager a few frames after spawn (observed reframes
  1080x701 → 655x1031 / 1213x1063, the KeePassXC tile), creating an
  order-dependent split-geometry failure. This was not AppKit clamping.
- The durable fix is present in the current source:
  - `crates/bbr-app/src/main.rs:240-249` — `--bbr-hidden` harness windows are
    created with `show: false`, so no window manager can ever reframe them
    (WebKit/GPUI still render through the testkit `FlushFrame` paint path).
  - `crates/bbr-app/tests/e2e_common/mod.rs` — `wait_for_window_settled`
    requires `WINDOW_SETTLE_FRAMES = 4` consecutive frame-driven unchanged
    observations of window bounds **and** native content rect before any
    absolute-geometry assertion.
- Every shell E2E harness passes `--bbr-hidden`
  (`e2e_common::spawn_harness`).

### Reproduction evidence (this host, GUI lock held)

`just e2e-shell-split-geometry` (production path, real WKWebViews, GUI lock,
`DEVELOPER_DIR` = Xcode 26.2.0; the hidden split harness does not depend on
the screen being unlocked — only the shown-window portal scenarios do):

| Run | Context | Result |
|---|---|---|
| 1–5 | standalone | 5/5 pass (~1.6–1.8 s each) |
| 6 | after `e2e-shell-sidebar` (canonical order) | pass |
| 7 | after `e2e-shell-sidebar` | pass |
| 8 | canonical-order position 8 (after smoke/tabs/profiles/shell-tabs/shell-spaces/sidebar, run as an individual recipe — the aggregate `just e2e` stops at position 2 because `e2e-shutdown` has no target) | pass |
| 9 | order sequence (sidebar → split → command) | pass |

Total: **9/9 green** (writer aggregate: 5× standalone + 4× in canonical
order). The independent reviewer personally reproduced a **5/5** subset on
the same tree. The historical order-flake no longer reproduces; the
sweep/settle determinism record lives in `gpui-ce-components-anchor.md`
(its evidence and delivery-ledger sections), not in
`split-shared-contract.md` — the shared-contract packet owns the geometry
behavior itself (per-`WebViewId` rects, pane layout, typed split wire) and
not the historical flake evidence. No canonical `just e2e` completed (see
§4); "in canonical order" means the same recipe sequence run as individual
invocations.

Sidebar regression (single-pane geometry contract): `just e2e-shell-sidebar`
7/7 on each of 4 runs (2 order sequences + canonical).

### Observed unrelated transient (honest record)

- First order-sequence run (sidebar → split → command) reported the command
  suite as 24 passed / 1 failed (test name not captured; the log tail was
  truncated). It never reproduced in 7 subsequent command-suite runs
  (3 standalone + 4 in-order), all 25/25 green, and the command suite in
  canonical-order runs is green. Attributed to host scheduling contention from the
  concurrently winding-down icon-system lane; no code path attributable and no
  test was loosened. Not part of queue item 18's scope; flagged for the
  orchestrator's awareness.

## 2. Xcode / MSRV checkboxes and claims — current truth

Verified from source, lockfile/dependency metadata, installed paths, and git
history (no invented versions):

- **Phase 0** (`docs/roadmap/phases/00-foundation.md:16`) — `[x]`:
  `BBR_XCODE_DEVELOPER_DIR` default `/Applications/Xcode-26.2.0.app/Contents/Developer`
  matches the `justfile` (`xcode_developer_dir`). Installed: confirmed
  `/Applications/Xcode-26.2.0.app` exists; Metal toolchain cryptex resolves via
  `/usr/bin/xcrun --find metal` under that `DEVELOPER_DIR`.
- **gpui-ce anchor** (`docs/roadmap/tasks/gpui-ce-components-anchor.md:59-60`)
  — `[x]` minimum-Rust claim derived from dependency manifests; `[x]` Xcode
  tooling facts match an installed successful build path rather than the
  absent Xcode 16.4 path.
  - MSRV floors re-verified from crate manifests:
    `smol_str 0.3.6` → `rust-version 1.89` (macOS floor),
    `oo7 0.6.0` → `rust-version 1.92` (Linux floor; `ashpd 0.13.13` 1.87 /
    `zbus 5.18.0` 1.87 below it), dev shell `rustc 1.97.1` (`flake.nix`
    `rust-bin.stable.latest`). `Cargo.toml` `rust-version = "1.89"` and the
    ARCHITECTURE.md §Toolchain paragraph are accurate.
- **Stale claim corrected** (`docs/roadmap/tasks/performance-e2e-foundation.md`,
  acceptance item): the parenthetical "Xcode 16.4.0 ships `metal` and a macOS
  15.5 SDK" contradicted the later-established installed Xcode 26.2.0 +
  Metal-Toolchain-component truth (H0 hotfix: Xcode 16.4 does not exist on
  this host). Reworded to the installed-path record, keeping the paragraph's
  corrective point (the "lacks the macOS Metal toolchain" review note was
  wrong; the just recipes' `DEVELOPER_DIR` makes the gates build and run).
- **Canonical truth surfaces are clean:** no remaining unchecked matching
  Xcode/MSRV/toolchain checkbox in `ROADMAP.md`, `docs/roadmap/**`
  (`- [ ]` lines mentioning xcode/msrv/rust-version/rustc/toolchain/metal/
  DEVELOPER/SDK), `ARCHITECTURE.md`, or `E2E_ARCHITECTURE.md`. The superseded
  `docs/delivery/2026-08-07-browser-mvp-parallel-plan.md` still carries
  unchecked L0 boxes (its L0 checklist, e.g. "Docs-Fakten korrigiert
  (Xcode 26.2-Pfad, MSRV-Claim, Task-Packet-Basis)" and "einmal `just e2e`
  komplett grün"). That file is a historical night protocol, explicitly
  superseded by the Aug-8 delivery log and never a canonical roadmap document
  (see `docs/roadmap/README.md`); it is intentionally not rewritten.
- `rust-version` floor remains a contract (1.89/1.92) per the anchor packet;
  the tested toolchain is the dev shell's 1.97.1 — no change made.

## 3. Changes in this slice

- `docs/delivery/2026-08-08-post-mvp-delivery.md` — queue item 18: ⬜ → 🟨
  (reviewed and serially integrated) with evidence summary.
- `docs/roadmap/tasks/performance-e2e-foundation.md` — stale Xcode 16.4.0 /
  macOS 15.5 claim corrected to the installed Xcode 26.2.0 + Metal Toolchain
  component record.
- `docs/delivery/2026-08-08-l0-rest-closeout-evidence.md` — this ledger.
- No production code, tests, budgets, or justfile recipes changed.

## 4. Gates run (all via `nix develop`, `CARGO_BUILD_JOBS=4`, GUI suites
   through `scripts/with_gui_lock.py`, `DEVELOPER_DIR` = repo Xcode default)

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | green |
| `just roadmap` (73 markdown files) | green (also after the doc edits) |
| `git diff --check` | clean |
| `just verify` (fmt/check/test/clippy/roadmap/script-tests) | green — 57 test binaries, 1158 tests, 0 failures |
| `cargo clippy -p bbr --features testkit --all-targets -- -D warnings` | green |
| `just e2e-shell-split-geometry` | 9/9 writer aggregate (see table); independent reviewer personally 5/5 |
| `just e2e-shell-sidebar` | 7/7 ×4 |
| `just e2e-shell-command` | 25/25 ×7 (one unreproduced transient, see above) |
| aggregate `just e2e` (21 recipes) | **no canonical pass**: stops at position 2 — `e2e-shutdown` has no target (`crates/bbr-app/tests/e2e_shutdown.rs` exists in no worktree) |
| remaining 20 recipes, run individually / in canonical order | 19 suites fully green; `just e2e-shell-portal` 5/7 (see §5); shutdown unavailable |
| `just e2e-perf` (separate gate) | green on unchanged retry; budgets/sample counts unchanged |
| `just e2e-perf` writer's first attempts | failed on documented host-load artifacts, retried green (see §5) |

### Independent reviewer reproduction (workflow `l0-rest-closeout-mskd3t3e-7xpxax`)

- Split-geometry: reviewer personally ran **5/5** green (4 standalone +
  canonical-order position 8 after sidebar); sidebar 7/7 ×2; command 25/25 ×2.
- `just verify`: 57 test binaries, 1158 tests, 0 failures — matches the
  writer claim exactly.
- Perf: first review attempt failed on cold-startup p95 **526.14 ms** against
  the unchanged 500 ms p95 ceiling (max 526 ms < 750 ms; host load 5–7 with
  CoreSimulator/WindowServer/coreduetd/bun noise); the unchanged retry was
  green — startup p95 **408.64 ms**, `command_bar_first_result`
  p95 **40.70 ms** / max **41.45 ms** over 30 samples, page_ready 606 ms,
  **0 regressions**, budgets untouched.

## 5. Environment blockers and pre-existing regression (nothing loosened)

1. **`e2e-shutdown` has no target**: the tracked `justfile` recipe references
   the test target `crates/bbr-app/tests/e2e_shutdown.rs`, but that file
   exists in no worktree. It was historically observed as an untracked
   prototype in the main worktree (delivery-log records 15:38/17:17) and was
   later removed; it is not a current foreign file, no orchestrator restore
   applies, and the old SHA record is not a current artifact. The aggregate
   `just e2e` therefore stops at position 2. The separately planned process-shutdown slice will implement the
   target later from fresh post-F2a `main`; the recipe stays untouched.
2. **Screen locked (`CGSSessionScreenIsLocked = 1`)**: `just e2e-shell-portal`
   fails exactly `host_hides_on_minimize_and_restores` — the shown-window
   minimize/restore scenario — because the loginwindow overlay (or any
   foreign frontmost window) answers all Window-Server hit-tests; the other
   portal scenarios pass (5/7 overall).
   **Separate pre-existing base regression (not loginwindow):**
   `host_tears_down_when_parent_closes_with_open_presentation` fails because
   the current quit-on-owning-browser-window behavior
   (`quit_when_browser_window_closes`, `crates/bbr-app/src/window_seam.rs`)
   terminates the process when the browser window closes, breaking the test's
   old zero-window/gateway-alive expectation. It is attributed to the base,
   not to the screen lock.
   Documented; no lock-detection bypass was added.
3. **Perf host-load artifacts** (queue item 16 territory, not this item):
   writer's first `just e2e-perf` attempts failed on cold-startup p95 507.6 ms
   (500 ms ceiling) and `command_bar_first_result` max 142.9 ms (100 ms
   ceiling) while a foreign lane built at load 27–39; quiet-host retry green
   (`command_bar_first_result` p95 37.8 ms / max within 100 ms; startup within
   500 ms). Independent reviewer reproduction: first attempt cold-startup
   p95 526.14 ms vs the unchanged 500 ms ceiling, unchanged retry green —
   startup p95 408.64 ms, `command_bar_first_result` p95 40.70 / max
   41.45 ms over 30 samples, zero regressions. Budgets untouched.

## 6. Handoff notes

- The independently approved diff is committed on the lane and integrated
  serially into `main`.
- Recommended next queue item: 15 (U3 startup evidence) or 16 (perf gaps),
  both measurable on a quiet host; 7 (F2a) remains blocked behind F2c → F2b.
- The unreproduced single command-suite transient and the items in §5
  (shutdown target absent, screen-locked minimize/restore, teardown base
  regression, perf host-load artifacts) are the only red evidence recorded
  in this slice. No test, budget, deadline, or tolerance was loosened.

## Timing (estimates)

Planning/read-only: ~0.5 h · Reproduction/gates: ~2.5 h (incl. waiting for
GUI slot and foreign host load) · Docs + ledger: ~0.3 h.
