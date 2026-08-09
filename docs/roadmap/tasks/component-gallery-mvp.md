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

Program position: **G1b — Gallery MVP**, activated only from the settled [UI library anchor](ui-library-anchor.md) state recorded in [Current delivery](../../delivery/current.md)
slice (G1a). This slice builds the dev-only **BBR Gallery** application:
a separate native GPUI app, keyboard- and pointer-operable collapsible
component tree on the left, real production `bbr-ui` or established
`gpui-component` control on the right canvas with mode/state controls,
covering Foundations plus a bounded set of real core primitives. It is
truthfully labeled **Foundations / Primitives** until G2/G3 append browser
components and compositions; it is not yet the complete component explorer.

## Outcome

`bbr-gallery` (dev-only, outside BBR.app) is a keyboard-first component
browser: a deterministic, stable-id component tree on the left with visible
pointer disclosure controls and keyboard collapse/expand; a real `bbr-ui`
story or established `gpui-component` control rendered on the right canvas at
the selected theme/contrast/density/reduced-motion/state/variant; and a
`just e2e-gallery` in-process
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
  label); ids are **append-only across Gallery packets** — G2/G3 add
  stories at later ids and never renumber G1b ids; a test pins the
  first-N id assignment so reordering cannot silently shift ids.
- **Tree interaction contract:** `↑`/`↓` move selection; `←` collapses a
  group (or moves to the parent from a story); `→` expands a group (or enters
  a story from a group); `Home`/`End` jump to first/last node; `Enter`/`Space`
  activate. Every group row has a visible disclosure affordance whose pointer
  target toggles the same single expansion authority, and the child rows plus
  disclosure direction visibly update after collapse/expand. Pointer and
  keyboard paths both preserve one tree focus and stable ids
  (`bbr-gallery-tree`, `bbr-gallery-node-<id>`,
  `bbr-gallery-disclosure-<group-id>`, `bbr-gallery-story-<id>`,
  `bbr-gallery-canvas`, `bbr-gallery-controls-*`); the selected story drives
  the canvas. A disclosure icon without a working group-row pointer handler
  is a product failure, not a future-packet item.
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
- **Real BBR primitives from integrated G1a:** Favicon tile, Profile chip,
  Section label, and Loading arc as real `bbr-ui` stories with fixture models.
  Status line is not a G1b story: G1a left its only production implementation
  app-owned at `bbr-app/src/shell/presentation.rs`, while this packet forbids
  both a `bbr-app` import and a gallery-local copy. Its production extraction
  and first real Gallery story therefore belong to G2's serial cutover.
- **Real framework core controls from the established production anchor:**
  Button, Text Input (`gpui_component::input::{Input, InputState}`), Checkbox,
  Switch, Radio, Slider, and Progress, rendered directly from the vendored
  `gpui-component` symbols already used by BBR's production Root. The Gallery
  initializes `gpui_component` and uses the required `Root`/entity state so
  Button clicks, text entry/focus/editing, and each stateful control exercise
  real handlers; no Gallery wrapper, copied anatomy, or second component
  implementation. Stories cover representative normal/disabled/state
  variants and recorder/state evidence without claiming every upstream widget.
- **Honest current navigation/copy:** title the application/catalog scope as
  **BBR Gallery — Foundations / Primitives** and visibly point to the planned
  Browser Components (G2) and Compositions (G3) expansion. Do not label G1b as
  the complete BBR component explorer.
- **`just gallery`** (dev run) and **`just e2e-gallery`** (in-process,
  non-network, dev-only test recipe) — both GUI-locked, both dev-only, never
  part of `just e2e`.
- **Perf sanity:** a report-only render/switch sanity measure (frame-time
  around story switches and mode changes) recorded to `target/bbr-gallery-perf/`
  — sanity only, never a product budget, never in `just e2e-perf`.

## Non-goals

- **No browser-component or composition stories here.** Tab Row, address,
  Favorites/Folders/Footer/Space chip, Downloads, Find, Error/Crash, Auth,
  Context Menu, Split Preview are [G2](component-gallery-browser-components.md)
  and [G3](component-gallery-compositions.md).
- **No product UI change.** The gallery consumes `bbr-ui` and the established
  vendored `gpui-component` production controls; it does not add, restyle, or
  re-architect any product surface. In particular, it does not
  invent a Gallery-only Status line while that production component remains
  app-owned; G2 must first extract and migrate the real component.
- **No `bbr-app` import, no controller, no engine.** The gallery builds
  plain fixture/control state only and may consume the established vendored
  `gpui-component` framework controls directly; there is no `BrowserController`, no fake
  `WebEngine`, no portal host, no WebKit, no persistence, no network, no
  bbr-testkit socket, no fixture server (per the anchor's invariant list).
- **No pixel assertions.** Per the anchor's parked decision 2, `just
  e2e-gallery` asserts semantic ids/typed state/stable StoryIds; a dev-only
  screenshot command may exist for human review but is never an assertion.
- **No macOS-gating of `bbr-ui`** (the crate stays portable); the complete
  Gallery v1 binary and its in-process harness/E2E modules are macOS-gated per
  the anchor's parked decision 3. Non-macOS workspace evaluation must not
  compile an unused Gallery harness or require lint allowances for dead test
  support.
- **No packaging.** The gallery is never in the `.app`, never in
  `just package-verify`, never in the canonical `just e2e`.

## Parallel-lane contract

```text
Base condition:       the settled packet-1 (UI library anchor) slice — the
                      bbr-ui foundations + primitives cutover, verified on
                      main after F2a with `just verify` + affected suites.
Depends on:           G1a (crate bbr-ui, its DTOs, ids, and the
                      dependency-boundary gate); F2a integrated main.
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
Integration order:    G1a → G1b → G2 → G3 → serial
                      post-G1a integration. Runs parallel-safe against F3 /
                      U3 / non-UI lanes (disjoint paths).
```

## Constraints

- **Gallery dev-only, outside BBR.app:** no WebKit, no listener/socket/
  testkit gateway, no fixture server, no persistence store, no native Portal
  host, no network driver of any kind. E2E is in-process or a dedicated
  non-network dev-only test (see Evidence) — never the bbr-testkit socket.
- **Stable `StoryId`:** assigned by the registry in declaration order,
  never derived from labels, append-only across G1b/G2/G3; a test pins the
  first-N assignment so a label edit or a reorder can never silently shift
  an id. The story registry is the single declaration point.
- **Deterministic catalog/group order:** explicit declaration order, never
  alphabetical sorting; the tree order, the canvas default, and the E2E
  expectations all derive from that one declaration.
- **Real production components only:** the canvas renders `bbr-ui` symbols
  with fixture DTOs or the established vendored `gpui-component` controls with
  their real entity state and handlers; no gallery-local reimplementation of
  anatomy, no copied tokens, no fake engine. The Gallery crate's allowed
  direct dependency set expands only by `gpui-component`; `bbr-ui` itself
  remains exactly `gpui + bbr-domain`.
- **Keyboard-operable tree:** arrows, collapse/expand, Home/End,
  Enter/Space, focus/selection semantics and stable ids per Scope; pointer
  operation may exist but never replaces keyboard completeness.
- **`bbr-ui` tests already cover contrast/ids/models/callbacks** (G1a);
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
      order, stable across label edits, and append-only (G1b ids are a
      pinned prefix; G2/G3 append without renumbering).
- [ ] Tree tests: focus/selection transitions for `↑`/`↓`/`←`/`→`/`Home`/
      `End`/`Enter`/`Space` over a mixed group/story catalog, plus pointer
      activation of the visible disclosure/group row; both paths assert
      child visibility, disclosure state, stable ids, and one-focus semantics.
- [ ] Controls tests: mode/contrast/density/reduced-motion/state switching
      resolves to the exact `Theme::new(...)` + story state and re-renders
      the canvas (asserted via the semantic canvas state).
- [ ] Fixture/control tests: every Foundation story's fixture model is valid
      for its `bbr-ui` component; core-control stories instantiate the exact
      vendored Button/Input/InputState/Checkbox/Switch/Radio/Slider/Progress
      symbols, and recorder/state assertions prove Button click, real text
      focus/input/editing, and stateful-control interaction paths.
- [ ] Dependency-boundary test for the gallery crate: its direct manifest
      `[dependencies]` contains only `bbr-ui`, `bbr-domain`, `gpui`,
      `gpui-component`, and `gpui_platform`; its source imports no forbidden
      first-party/runtime authority (`bbr-app`, `bbr-browser`, `bbr-web`,
      `bbr-web-macos`, `bbr-testkit`, `bbr-fixtures`, `bbr-persistence`) and
      declares no direct native bridge, WebKit, persistence, or network crate.
      Third-party transitive packages of the explicitly allowed GPUI stack are
      audited as framework closure, not rejected by a flat package-name scan.

### Real-app E2E (dev-only, non-network)

- [ ] `just e2e-gallery` (GUI-locked) proves: startup presents an honestly
      labeled Foundations/Primitives scope, the tree, and the default story;
      real keyboard events move selection and collapse/expand groups;
      a real pointer click on the visible disclosure/group-row hit target
      collapses and expands the same group with child visibility and icon state
      changing; Enter/Space activate stories; every registered story renders
      without panic and exposes its semantic root + stable id; Button and
      stateful-control callbacks fire; real Input receives focus and entered
      text through production input handling; mode/contrast/density/reduced-
      motion/state switching changes the canvas fields and keeps ids stable;
      a label edit does not change any `StoryId`.
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
just ui-deps                    # direct-dependency + approved-closure audit gate
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
