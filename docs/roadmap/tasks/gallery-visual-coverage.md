# Task packet — Native Gallery visual coverage

Parents: [Phase 4](../phases/04-ui-shell.md) · [Testing track](../tracks/testing.md) · [Design contract](../../../DESIGN.md).

## Outcome

Every registered Gallery story and its finite supported visual-state inventory
is discoverable and captured through the pinned GPUI offscreen Metal renderer,
with real macOS text shaping and no native window or focus takeover, complete
component framing, exact comparisons, HTML review and explicit
baseline acceptance. Coverage failures cannot silently remove cases.

## Scope

- Audit the existing catalog, state selectors, valid/rejected fixture matrices
  and interactive controls. Reuse those authorities rather than copying them.
- Add a dev-only windowless capture entrypoint to the Gallery, using the pinned
  HeadlessAppContext with real macOS text and Metal (not no-op test pixels).
  Capture real production components without the catalog sidebar or recorder noise;
  preserve component anatomy, fonts, shadows and callback/model boundaries.
- Capture Light/Dark and applicable contrast/density states. Inventory records
  represented states and explicit exclusions, including unsupported states.
- Reuse the unchanged comparator and source/environment sealing, add bounded
  recipes and completeness contracts, and review independent captures.
- Provide one `just visual` command for Gallery and the existing eight browser
  snapshots, with both suite reports even when one comparison needs review.
  Provide `just visual-update <batch> <reviewed-case>...` to route explicit
  selections from either suite through unchanged upstream acceptance. No implicit
  latest-run selection, accept-all or automatic baseline writes. This command
  integration does not claim the pending all-state/framing audit is complete.
- Correct Gallery-only fixture wiring exposed by native captures (appearance,
  Reduced Motion and undersized presentation bounds), with regression tests.
  This does not authorize changes to bbr-ui or browser production behavior.
- Deliver in coherent increments: inventory/native capture foundation first,
  then audited full Gallery coverage. Real browser compositions require their
  own subsequent packet before implementation.

## Non-goals

No browser product rendering or behavior changes, native WebKit/traffic-light
pixels, renderer fork, new dependency graph, network service, automatic
acceptance, tolerance, masks or invented native pixels. No Split work, media
capability implementation, performance execution, arbitrary state cartesian
product, or animation-timing claim. Static endpoints do not prove motion.
Titlebar/Portal acceptance stays owned by the product functional gates, separate
from visual baseline review.

## Parallel-lane contract

Base condition: committed visual snapshot adapter and integrated Gallery catalog.
Depends on: real production bbr-ui and gpui-component stories and their matrices.
Provides/API: native Gallery capture/inventory feeding the exact comparator.
Exclusive paths: crates/bbr-gallery, scripts/visual, tests/visual, visual configs.
Shared integration points: justfile, roadmap/testing/Phase 4, Current delivery,
E2E_ARCHITECTURE.md and visual-tooling documentation.
Integration order: product/Paper first; tooling/Gallery second; fresh combined-
tree inventory, captures and explicit baseline re-review after integration.

## Constraints

Keep Gallery's five normal dependencies unchanged (bbr-ui, bbr-domain, gpui,
gpui_platform, gpui-component). Do not import app/browser/WebKit/testkit,
fixtures, persistence or command runtimes. Keep normal Gallery behavior and
semantic E2E intact. Capture state must be deterministic and semantically
ready; no sleeps/pixel-stability loop. Detect clipping rather than accept a
scroll viewport as full coverage. Use mbx, hard deadlines and the existing
host-wide GUI lock through descendant cleanup, conservatively retained for GPU
serialization even though the producer opens no native window. No mutation of
other worktrees. Do not hide/offscreen-position an NSWindow as a headless fallback.
Prove locked-session operation on the host before claiming it; do not change the
user's lock state. Native browser/window/WebKit E2Es remain separate.

## Evidence

### Unit and contract

- [ ] Registry-driven story/state inventory and explicit applicability/exclusions.
- [ ] Missing/extra/duplicate cases, unsupported states and incomplete framing
  fail closed; additions cannot silently disappear from coverage.
- [ ] Existing Gallery dependency and matrix contracts remain green.
- [ ] Adapter provenance, seal and acceptance contracts remain green.

### Real-app E2E

- [ ] Offscreen optimized Gallery captures use production story renderers, real
  macOS text/Metal and deterministic appearance, state, content bounds and scale.
- [ ] No native window/focus takeover; locked-session operation and repeatability
  have runtime evidence, with differences from saved native references explained.
- [ ] Each supported inventory case is fully framed and visually reviewed.
- [ ] Two independent captures match exactly; missing baselines initially fail
  and every accepted case is explicitly named after review.
- [ ] Existing semantic Gallery E2E and affected browser capture gates pass.
- [ ] Stable functional aggregate passes, or previously recorded unrelated
  blockers remain explicit and completion remains unclaimed.

### Performance

Capture throughput is not product performance. No browser interaction timing
changes; performance execution remains deferred in Current delivery. Static
Reduced Motion captures do not prove animation behavior or timing.

## Completion gates

Bounded mbx Gallery unit tests and all-target Clippy (`-D warnings`), existing
`just e2e-gallery`, native Gallery capture/repeat comparison, `just visual-test`,
`just visual-check`, `just verify`, normal testkit Clippy, the canonical
functional sequence excluding deferred performance plus Peek, and `just roadmap`.
Run targeted gates before intermediate commits; no full-coverage or stable-
acceptance checkbox until its evidence exists.

## Delivery sequence

One exclusive local writer, requirement-to-evidence self-review (no subagents
authorized), concrete remediation and focused recheck. Commit verified bounded
increments; retain ownership until explicit handoff. Integration requires an
explicit user request and combined-tree verification. A merge does not authorize
push or acceptance of combined-tree baselines; baseline review remains separate.

## Iteration policy

Begin with audited inventory and the smallest native capture path. Diagnose
framing/readiness/nondeterminism before expanding the matrix. Re-capture after
source changes, inspect real PNGs, and leave unsupported claims open. Add the
next component family only when the capture contract proves the prior one.

## Blocked stop condition

Stop if native rendering, framing, deterministic readiness, GUI ownership,
dependency boundaries or a required gate has no defensible path. Preserve the
exact branch/tree/run paths, evidence, failed approaches and smallest required
input. Do not alter production pixels or tests to make the comparator pass.

## Delivery ledger

First bounded increment corrects Gallery-only theme propagation and Tab/Favorite
menu fixture capacity. The canvas now supplies actual appearance and Reduce
Motion; family-specific density and named accessibility requirements remain
explicit. Regression coverage checks model tint/geometry and natural production
menu layout rather than metadata alone. No bbr-ui or browser product change.

The next bounded increment supplies the windowless optimized Metal producer and
`just gallery-capture [story-id]` with a completeness-checked but unsealed probe
adapter. Registry state selectors are not yet an audited all-supported-state
inventory; nested framing, interactive-state applicability and locked-session
proof remain open. The subsequent command increment connects the current matrix
to source/environment seals and unchanged comparator reports in `just visual`,
alongside existing browser snapshots. Explicit `visual-update` routes reviewed
selections from either suite; no Gallery baseline is accepted by implementation. Static endpoint waits do not prove
animation. Volatile owner/base/status and headless runtime evidence live in
[Current delivery](../../delivery/current.md).
Patrick approved the windowless path after the visible-window producer stalled
with a locked macOS session. The existing pinned renderer is reused unchanged;
no GPUI upgrade, renderer fork, simulated text or synthesized pixels is allowed.
