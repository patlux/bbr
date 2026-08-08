# Task packet — Component gallery MVP (bbr-gallery)

Parents: [Phase 4](../phases/04-ui-shell.md) · [Phase 9](../phases/09-quality-release.md) ·
[UI design-system track](../tracks/ui-design-system.md) ·
[Accessibility track](../tracks/accessibility.md) ·
[Testing track](../tracks/testing.md) ·
Visual contract: [DESIGN.md](../../../DESIGN.md) ·
Anchor: [UI library anchor (bbr-ui)](ui-library-anchor.md) ·
Product: [PRODUCT.md](../../../PRODUCT.md) ·
Workflow: [development workflow](../../development-workflow.md) ·
[parallel delivery](../../parallel-delivery.md) ·
Template: [task-packet template](../templates/task-packet.md)

Status: **G1 packet 2.** Bases from the settled [UI library anchor](ui-library-anchor.md)
slice (packet 1). This slice builds the dev-only **BBR Gallery** application:
a separate native GPUI app, keyboard-operable component tree on the left,
real production `bbr-ui` component on the right canvas with mode/state
controls, covering the Foundations pages plus the smallest valuable real
primitives from the anchor's honest inventory.

## Outcome

`bbr-gallery` (dev-only, outside BBR.app) is a keyboard-first component
browser: a deterministic, stable-id component tree on the left; a real
`bbr-ui` story rendered on the right canvas at the selected theme/contrast/
density/reduced-motion/state/variant; and a `just e2e-gallery` in-process
suite proving startup, tree keyboard operation, stable `StoryId`s, mode/state
switching, and that every registered story renders without panicking and
exposes its semantic root/id. The gallery renders production `bbr-ui`
components with plain fixture models and recorder callbacks — never copies,
never a fake engine.

## Scope

- **`crates/bbr-gallery/**`** (bin crate per the anchor's parked decision 1 —
  package `bbr-gallery`, binary `bbr-gallery`): the app window, the catalog
  registry, the tree surface, the canvas surface, the mode/state controls,
  and the fixture builders for this packet's stories.
- **Catalog registry with stable `StoryId`s:** a single declaration-order
  registry assigns `StoryId(u64)` sequentially (never derived from labels);
  groups and stories appear in explicit declaration order (never sorted by
  label); ids are **append-only across G1 packets** — packets 3–4 add
  stories at later ids and never renumber packet-2 ids; a test pins the
  first-N id assignment so reordering cannot silently shift ids.
- **Tree keyboard contract:** `↑`/`↓` move selection; `←` collapses a group
  (or moves to the parent from a story); `→` expands a group (or enters a
  story from a group); `Home`/`End` jump to first/last node; `Enter`/`Space`
  activate (select the story on the canvas); focus and selection semantics
  with stable ids (`bbr-gallery-tree`, `bbr-gallery-node-<id>`,
  `bbr-gallery-story-<id>`, `bbr-gallery-canvas`,
  `bbr-gallery-controls-*`); one tree focus, the selected story drives the
  canvas.
- **Canvas + controls:** the right canvas renders the selected story with
  `Theme::new(mode, contrast, density)`; controls switch `ThemeMode`
  (light/dark), `ContrastMode` (standard/increased), `Density`
  (small/normal/extralarge), Reduced Motion (bool), and the story's own
  State/Variant selector when the story registers one. Mode/state changes
  re-render the canvas and are themselves asserted (snapshot fields mirror
  the §15 mode fields).
- **Foundations pages** (from the anchor's extraction table): Tokens
  (surfaces/materials/elevation), Typography (type roles per density),
  Spacing/Radius, Motion (durations + reduced-motion behavior), Density
  (S/N/XL table), Elevation (`elev:1`/`elev:2` shadow pair), Color
  (neutral/accent/status/tint per mode+contrast with the derived contrast
  ratio shown), States (the §10 vocabulary), Iconography (every glyph ×
  S/N/XL), Favicon/monogram tile, Labels/Section, Brand.
- **Smallest valuable real primitives** (the "repeated same-intent anatomy"
  bar from the anchor): Favicon tile, Profile chip, Section label,
  Status line, Loading arc — each as a real `bbr-ui` component story with
  fixture models.
- **`just gallery`** (dev run) and **`just e2e-gallery`** (in-process,
  non-network, dev-only test recipe) — both GUI-locked, both dev-only, never
  part of `just e2e`.
- **Perf sanity:** a report-only render/switch sanity measure (frame-time
  around story switches and mode changes) recorded to `target/bbr-gallery-perf/`
  — sanity only, never a product budget, never in `just e2e-perf`.

## Non-goals

- **No browser-component or composition stories here.** Tab Row, address,
  Favorites/Folders/Footer/Space chip, Downloads, Find, Error/Crash, Auth,
  Context Menu, Split Preview are [packet 3](component-gallery-browser-components.md)
  and [packet 4](component-gallery-compositions.md).
- **No product UI change.** The gallery consumes `bbr-ui`; it does not add,
  restyle, or re-architect any product surface.
- **No `bbr-app` import, no controller, no engine.** The gallery builds
  plain fixture models only; there is no `BrowserController`, no fake
  `WebEngine`, no portal host, no WebKit, no persistence, no network, no
  bbr-testkit socket, no fixture server (per the anchor's invariant list).
- **No pixel assertions.** Per the anchor's parked decision 2, `just
  e2e-gallery` asserts semantic ids/typed state/stable StoryIds; a dev-only
  screenshot command may exist for human review but is never an assertion.
- **No macOS-gating of `bbr-ui`** (the crate stays portable); the gallery
  app itself is macOS-gated for v1 per the anchor's parked decision 3.
- **No packaging.** The gallery is never in the `.app`, never in
  `just package-verify`, never in the canonical `just e2e`.

## Parallel-lane contract

```text
Base commit:          the settled packet-1 (UI library anchor) slice — the
                      bbr-ui foundations + primitives cutover, verified on
                      main after F2a with `just verify` + affected suites.
Depends on:           packet 1 (crate bbr-ui, its DTOs, ids, and the
                      forbidden-dependency gate); F2a integrated main.
Provides/API:         binary `bbr-gallery`; the gallery catalog registry
                      (StoryId assignment), tree/canvas surfaces, mode/state
                      controls, fixture builders for Foundations stories;
                      `just gallery`, `just e2e-gallery`; gallery-only
                      perf-sanity artifacts.
Exclusive paths:      crates/bbr-gallery/** (new). No bbr-ui module is
                      modified by this packet except where a Foundations
                      story needs a pure fixture builder that legitimately
                      lives in the gallery crate.
Shared integration points: root Cargo.toml/Cargo.lock (workspace member),
                      justfile (gallery recipes — proposed by the lane,
                      landed by the orchestrator), UI track and ROADMAP
                      roll-ups, ARCHITECTURE.md facts.
Integration order:    packet 1 → this packet → packet 3 → packet 4 → serial
                      post-G1 integration. Runs parallel-safe against F3 /
                      U3 / non-UI lanes (disjoint paths).
```

## Constraints

- **Gallery dev-only, outside BBR.app:** no WebKit, no listener/socket/
  testkit gateway, no fixture server, no persistence store, no native Portal
  host, no network driver of any kind. E2E is in-process or a dedicated
  non-network dev-only test (see Evidence) — never the bbr-testkit socket.
- **Stable `StoryId`:** assigned by the registry in declaration order,
  never derived from labels, append-only across packets 2–4; a test pins the
  first-N assignment so a label edit or a reorder can never silently shift
  an id. The story registry is the single declaration point.
- **Deterministic catalog/group order:** explicit declaration order, never
  alphabetical sorting; the tree order, the canvas default, and the E2E
  expectations all derive from that one declaration.
- **Real production components only:** the canvas renders `bbr-ui` symbols
  with fixture DTOs and recorder callbacks; no gallery-local reimplementation
  of anatomy, no copied tokens, no fake engine.
- **Keyboard-operable tree:** arrows, collapse/expand, Home/End,
  Enter/Space, focus/selection semantics and stable ids per Scope; pointer
  operation may exist but never replaces keyboard completeness.
- **`bbr-ui` tests already cover contrast/ids/models/callbacks** (packet 1);
  this packet adds gallery-side tests only for gallery-owned logic (registry,
  tree focus/selection, controls, fixture builders).
- **Gallery perf is sanity only, never a product budget:** measured values
  are recorded and labeled report-only; no budget, baseline, or gate is
  derived from them.
- **Gallery tests supplement, never replace, affected browser E2E/perf.**
- **GUI recipes use the host lock** (`python3 scripts/with_gui_lock.py --`);
  `just e2e-gallery` is a GUI-locked, bounded, dev-only recipe with a hard
  timeout.
- **Every gate list contains `cargo fmt --all -- --check` and states
  `CARGO_BUILD_JOBS=4`.**
- **Module size budget:** gallery modules stay below roughly 500 lines
  (catalog registry, tree, canvas, controls, fixtures each separated by
  responsibility).
- **No weakened tests, budgets, validators, or acceptance criteria** — and
  none of the product's existing gates are touched by this packet.

## Evidence

### Unit and contract

- [ ] Registry tests: `StoryId` assignment is sequential by declaration
      order, stable across label edits, and append-only (packet-2 ids are a
      pinned prefix; packets 3–4 append without renumbering).
- [ ] Tree tests: focus/selection transitions for `↑`/`↓`/`←`/`→`/`Home`/
      `End`/`Enter`/`Space` over a mixed group/story catalog, with stable
      node ids and one-focus semantics.
- [ ] Controls tests: mode/contrast/density/reduced-motion/state switching
      resolves to the exact `Theme::new(...)` + story state and re-renders
      the canvas (asserted via the semantic canvas state).
- [ ] Fixture-builder tests: every Foundation story's fixture model is
      valid for its `bbr-ui` component (fields non-empty, ids stable).
- [ ] Forbidden-dependency test for the gallery crate: its manifest
      `[dependencies]` contains `bbr-ui`, `gpui`, `gpui-component` (Root,
      optional), `gpui_platform` — and nothing from the forbidden set
      (bbr-app, bbr-browser, bbr-web, bbr-web-macos, bbr-testkit,
      bbr-fixtures, bbr-persistence, objc2*, webkit, serde, network
      crates).

### Real-app E2E (dev-only, non-network)

- [ ] `just e2e-gallery` (GUI-locked) proves: startup presents the tree and
      the default story on the canvas; real keyboard events move the tree
      selection and Enter/Space activate the story; every registered story
      renders without panic and exposes its semantic root + stable id;
      mode/contrast/density/reduced-motion/state switching changes the
      canvas state fields and keeps ids stable; a label edit on a fixture
      does not change any `StoryId`.
- [ ] The harness is in-process or a dedicated non-network dev-only test
      (mirrors the product's "drive the exact production methods" precedent,
      never the bbr-testkit socket; no network, no listener, no fixture
      server).

### Performance (sanity only)

- [ ] Report-only story-switch/mode-change frame-time observations recorded
      under `target/bbr-gallery-perf/`; labeled sanity-only, never compared
      to any product budget, never part of `just e2e-perf`.

## Completion gates

```bash
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo check --workspace --all-targets
CARGO_BUILD_JOBS=4 cargo test --workspace
CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p bbr-gallery --all-targets -- -D warnings
just ui-deps                    # forbidden-dependency audit gate
just e2e-gallery                # GUI-locked dev-only suite (host lock)
just verify                     # full repo verify after the slice is stable
python3 scripts/check_roadmap.py
```

Every cargo invocation states `CARGO_BUILD_JOBS=4`. GUI recipes run
sequentially under `python3 scripts/with_gui_lock.py --`. The canonical
`just e2e` is **not** part of this packet's gates (the gallery is dev-only);
affected product suites re-run per the [anchor's serial cutover](ui-library-anchor.md#serial-cutover).

## Delivery sequence

1. Confirm the packet-1 base is settled and `bbr-ui` Foundations/primitives
   are integrated and green.
2. One exclusive writer creates `crates/bbr-gallery/**`: registry, tree,
   canvas, controls, fixtures, the in-process harness, and the just recipes
   (proposed to the orchestrator).
3. Run targeted gallery tests while iterating; run `just e2e-gallery` when
   stable; run `just verify` once after the slice is stable.
4. One independent read-only reviewer maps every story back to the anchor's
   inventory and checks stable ids, keyboard completeness, and the no-network/
   no-copy constraints.
5. One remediation writer only for concrete findings, then a focused recheck.

## Iteration policy

Start with the registry + tree + one token story end to end (smallest
vertical slice), then add the remaining Foundations pages and primitives.
For each failure, classify it as registry/id, tree-focus, canvas/theme
resolution, fixture validity, harness, or gate environment; change the
smallest central cause and re-run the focused test. Never sort the catalog,
derive ids from labels, or fall back to pixel assertions.

## Blocked stop condition

If a story cannot render without panic or without its semantic root/id, stop
with the story, the failing `bbr-ui` component, the panic/assertion, the
preserved worktree state, and the smallest `bbr-ui` or fixture change
required. If the packet-1 base is not settled or F2a has not integrated,
stop before starting and report the missing base.

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

Planning/read-only:
Implementation:
Test execution:
Review:
Remediation:
Waiting/retries:
Total elapsed:
Human intervention:
```

Use measured timing only when runtime/command timestamps support it;
otherwise label values as estimates.
