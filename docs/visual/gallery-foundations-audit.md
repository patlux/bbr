# Gallery applicability audit — Foundations, Primitives and Core Controls

Acceptance owner: [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md).
Read with the [migration source audit](gallery-source-audit.md).
Inspection base: `4a5fedf9`; IDs 0–20 only. This is source evidence, not new
PNG/Metal/input evidence, measured current page ranges or baseline approval.
No production, Gallery, vendor or asset code changes accompany this audit.

## Classification and common boundaries

- **Represented** means the current source constructs the named sample; it does
  not mean its pixels or framing were verified in a guest.
- **Not applicable to this sample** is narrow: a token table does not have a
  pointer transition, and a static glyph is not a spinner animation. It never
  removes an interaction requirement from the actual production control.
- **Open** includes missing supported states, assets, effective theme propagation,
  nested content/paint extents and real guest evidence. These are not silently
  converted to unsupported cases or omitted from required coverage.

The capture enumerator visits every registered selector across Light/Dark,
standard/increased canvas contrast and Small/Normal/ExtraLarge canvas density.
`select_capture` forces BBR Reduced Motion. It does not send input, change control
recorder values, focus the input or click controls. The runner changes the separate
`gpui_component::Theme` appearance, but does not set that global theme's contrast,
control `Size`, or BBR motion mode. Story labels are not effective-control facts.

Direct story-child wrappers are indivisible pagination units. Their layout bounds
are checked inside the 48pt inset; descendants, glyph paint, painted shadow extents
and focus rings are not measured by that callback. Table text uses ellipsis in
`value_row`, so the outer row fitting cannot establish that its full text is
visible. No current page count is authored from these source observations.

Common source authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_cases.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app/capture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_page.rs`

## Foundations: IDs 0–9

| ID / story | Represented and applicable | Narrow exclusions and open requirements |
|---|---|---|
| 0 Tokens | Five theme material swatches and elevation description rows, ten direct children | Material names are colors, not proof of native glass or shadows. Input states do not apply to these static samples. Fresh full framing/repeat remains open. |
| 1 Typography | Seven real type-role size/weight samples at the selected BBR density, with leading printed as metadata | Sample rows set size and weight, not explicit role-specific line height or per-role mono font. Do not infer all typography/leading/font behavior from labels; nested text bounds need review. |
| 2 Spacing / Radius | Eight 4pt-step bars and five radius samples using theme tokens | Static geometry, not every component's corner composition/hit area. Complete nested sample/text bounds remain open. |
| 3 Motion | Real duration/easing/moment tables; three production illustrations with `key=None` under forced Reduced Motion | Still frames prove no motion, sequence, interruption or standard-motion execution. Long table strings may ellipsize. Rest-frame production output and nested illustration bounds require guest review. |
| 4 Density | Nine token rows showing all three densities side by side | Each capture does not represent only its named density: the table always includes S/N/XL. It is a geometry-token table, not a complete production-component size matrix. |
| 5 Elevation | Actual `elev2`/`lift` values printed, plus a bordered/rounded sample | The sample never applies `.shadow(...)`. Printed shadows are not painted shadow coverage. Light/Dark row asymmetry must remain explicit in the independent plan. |
| 6 Color | Twelve token/contrast rows; five veil-over-rail rows; every space tint at three intensities plus a grained standard sample | This is a selected set of palette/material samples, not every production surface. Nested 84×48pt rail samples intentionally clip to their radius; verify labels/grain inside them. No transparency/system-material behavior claim. |
| 7 States | Eleven selector labels mapped to `TabVisualState`, displayed with one real favicon and vocabulary notes | With `image=None`, only Loading and Error choose different favicon branches; the other nine selectors use the monogram branch. The selected vocabulary text changes, but no row wash, attention dot, drag shadow or focus ring is supplied by this favicon. Those row states remain open, not proven by the selector name. |
| 8 Iconography | Explicit list of 25 glyph samples, each at three explicit icon sizes | Not every current `Icon` variant is listed. Newer glyphs below are absent; no renderer inventory may silently define complete coverage. Glyph paint/nested cell extents remain open. |
| 9 Brand | Production `ChassisMark` at 28pt with BBR/name text | Narrow chassis-mark sample, not the current app icon's whole rendering/export pipeline or all identity glyphs. Pointer/animation states do not apply to this static composition. |

Foundation authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/foundations/tokens.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/foundations/type_motion.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/foundations/motion.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/foundations/paper.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/foundations/states_icons.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/theme.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/favicon.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/icons.rs`

`all_icons()` omits `Search`, `Warning`, `Gear`, `Info`, `Document`, `Person`,
`Shield`, `Eye`, `Keyboard`, both `Camera` and `Microphone` values,
`MediaPrevious`, `MediaPlay`, `MediaPause`, `MediaNext`, both `Speaker` values,
`Split`, and `Bbr(BbrGlyph)` variants. They may appear elsewhere, but that does
not prove a complete explicit Iconography matrix. Existing 25 entries are not
removed. An additive withheld catalog row names this gap without guessed pages.

## Primitives: IDs 10–13

| ID / story | Represented and applicable | Narrow exclusions and open requirements |
|---|---|---|
| 10 Favicon tile | Four fixed-host monograms in Default/Loading/Error, plus a missing-spec `?` sample always at Default | Fixtures always have `image=None`: acquired image/decode fallback and Unresponsive branches are not represented. Reduced Motion supplies a static loading arc; no loading transition or real favicon pipeline evidence. |
| 11 Profile chip | Six hue/monogram specimens through production `profile_chip` and current BBR theme | Static identity chips, no app profile picker/selection/management. Geometry and text paint remain to be reviewed. |
| 12 Section label | Pinned/Tabs/Favorites at rest and Pinned with `focused=true` | Focus appearance is an explicit model projection, not an actual keyboard focus transition. Do not classify it as driven focus evidence. |
| 13 Loading arc | Production static `Icon::LoadingArc` at all S/N/XL sizes | It does not call `loading_spinner`; its story note about a later rotation pass does not negate the actual production spinner implementation. Static glyph sample only, no timing/rotation claim. |

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/primitives.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/fixtures.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/favicon.rs`

## Core Controls: IDs 14–20

All seven stories have no explicit state selector. The shared recorder starts
with zero clicks, unchecked Checkbox/Switch and Radio index zero. Lazy production
entities create an empty Input with `Type here…` placeholder and a 0–100 Slider
(step 1) at 25. The capture loop never invokes their event path. BBR page labels
follow its density/contrast; this does not establish corresponding vendored control
variants. Calls use the controls' default sizes, with explicit 280pt Input/Slider/
Progress containers. Vendored control colors use `cx.theme()` rather than the
`StoryContext` BBR palette. Light/Dark is explicitly changed by the capture runner.

| ID / story | Current initial sample | Open supported-state / framing evidence |
|---|---|---|
| 14 Button | Primary and ghost, click counter zero | Press/hover/focus/disabled and recorder transitions are not captured; variants beyond the two samples are not implicitly covered. |
| 15 Input | Real empty `InputState`, placeholder, value row | Focus/editing/selection/IME/caret/disabled/error states and clipped text/caret geometry are not captured. No claim of complete editor coverage. |
| 16 Checkbox | Unchecked production Checkbox and its recorder state | Real pointer down/up and checked/unchecked endpoint frames required by pilot. The checked SVG dependency and production 250ms icon transition require legitimate handling, not state injection or arbitrary sleeps. |
| 17 Switch | Unchecked production Switch | No on-state or pointer/keyboard/focus/disabled/transition evidence. BBR Reduced Motion alone does not establish the vendored transition policy. |
| 18 Radio | Three controls, first selected because `radio_index=0` | Selected first Radio already calls `checkbox_check_icon` and therefore needs `icons/check.svg` on the initial frame. The asset gap is not confined to a future Checkbox click. Selection changes, focus/disabled behavior and nested shadows are open. |
| 19 Slider | Real horizontal Slider at 25 in 280pt container | No endpoints, drag, keyboard/focus/disabled/range-mode or value propagation capture. |
| 20 Progress | Four fixed bars at 25/50/75/100 and a fifth mirroring initial Slider 25 | Static read-only bars have no own pointer transition. Mirroring after a real Slider action, other production modes and nested label/track framing remain open. |

The vendored Checkbox and Radio add a shadow when their global theme permits it
and a focus ring when focused. Neither extent is proven by direct-child bounds.
`checkbox_check_icon` chooses `IconName::Check.path()` for checked controls; when
checked state changes it uses a 250ms animation plus background timer. The current
producer only assigns nonzero settling durations to stories 22 and 45. A future
Checkbox pilot must establish its actual endpoint and callback state; forcing BBR
Reduced Motion or merely draining the executor is not that evidence.

Core authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/core_controls.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/recorder.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/control_observer.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/theme/mod.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/checkbox.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/radio.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/styled.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/icon.rs`

The existing semantic interaction scenarios in
`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/controls.rs`
exercise these callbacks through input and snapshots. They are a reusable behavior
verification seam, not current native image evidence. No E2E/native producer was
executed for this source audit, and historical semantic passes are not promoted
to verified visual assets or timing.

## Result and next boundary

Four additive withheld rows expose States projection, missing Iconography,
Core-Control applicability and initial Radio asset dependency through the existing
catalog/doctor path. All 936 historical required page pairs and eight earlier
blockers are preserved. These rows do not resolve full coverage, activate a story
catalog, change pages, authorize production changes or accept baselines.

Continue the same bounded source inspection for IDs 21–48 (browser components,
fixture matrices and compositions); pilot/additions 44 and 49–52 already have
initial source notes but still need runtime framing. All real capture, assets,
network permissions, guest runtime, exact repeats and image/report review remain
open. No masks, resampling, tolerance or synthesized native pixels are proposed.
