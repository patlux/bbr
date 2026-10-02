# Gallery applicability audit — Status, Tab Row and Saved Pages Library

Acceptance owner: [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md).
Read with the [migration source audit](gallery-source-audit.md) and
[Foundations audit](gallery-foundations-audit.md).
Inspection base: `7cc720cc`; IDs 21–30 only. These are source observations, not
fresh PNG/input evidence, measured current page ranges or baseline approval.
No story, fixture, production component, asset or renderer is changed here.

## Common classification

**Represented** means the source constructs the named sample. **Open** means
current coverage or runtime evidence is missing. **Not applicable** is restricted
to the narrow sample: static status/count text has no own input transition;
Gallery callback recording is not browser state mutation, persistence or native
navigation. An old matrix's rejected label is not proof that production still
rejects that state.

Capture selects the story, appearance, contrast, density and state through Gallery
controls, forces BBR Reduced Motion, then draws without pointer/keyboard input.
The Tab Row story renders every family at its own density while preserving canvas
appearance/contrast/motion. Library components use the canvas theme. Neither the
image filename nor a state label proves a real interaction or a distinct effective
family density. Required historical rows remain intact; these findings add only
withheld blockers, without page, sequence or moment metadata.

Source authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app/capture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_cases.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_page.rs`

## Story-by-story applicability

| ID / story | Source-represented content | Narrow exclusions and open evidence |
|---|---|---|
| 21 Status line | Five real `status_line` samples: No page, Loading, title, URL and termination text | These are literal `StatusModel` text fixtures, not exercised app derivation from navigation/loading/crash. Static labels have no own input state. Text framing at each effective theme remains unreviewed. |
| 22 Tab Row | Nineteen production `tab_row` families, two mid-list drag ticks, six rejected-matrix explanation rows | Families project state/density/close flags, not pointer/focus/drag transitions. Bitmap rejection is stale; split brackets are never supplied. Focused has no dedicated ring branch. Painted shadows and nested companion glyphs need guest inspection. |
| 23 Library row | Four production rows across default/selected/editing: user label, captured title plus collection, long URL/hostname fallback, missing-favicon monogram; editing substitutes the first row's draft field | Rename/search are presentation chrome with app-owned text input, not native editable entities in this story. Recorder activation/select/rename events do not mutate the fixture. Long text is intentionally truncated; wrapper bounds do not establish correct ellipsis or paint. |
| 24 Library collection row | Work and Recipes with counts 3 and 1, wired action callbacks | Collection create/rename/remove stage surfaces are not rendered by these rows. A recorded callback does not stage them. |
| 25 Library search | Empty placeholder, literal `recipes`, and empty field with `focused=true` | Focus is a boolean presentation, not an acquired focus owner or typed query. Typed+focused, editing/selection/IME/caret and real filtering are not proven by the three selectors. |
| 26 Library sort | All four fixed order labels with a CycleSort recorder callback | Clicking records dispatch, not a reordered model/result list. No real sort transition capture. |
| 27 Library count | Totals 1, 24 and 1000 | `library_count_line` prints total only; the model contains no rows and zero window bounds. No rendered-window or large-list coverage follows from `1000 Saved Pages`. Static count text has no own pointer transition. |
| 28 Library remove confirm | One-target and three-target Remove/Cancel presentations | Callback counts are not deletion, confirmation lifecycle or browser-tab safety evidence. Nested text/button framing remains open. |
| 29 Library state rows | Literal loading, empty and no-results rows | These are static text, not a loading animation or live search transition. Full text bounds remain unreviewed. |
| 30 Library composition | Eleven selectors through production `library_surface` in a 520pt frame, including loading/empty/populated/no-results, row selection, destructive confirm, long, monogram, initial 1k window and row rename | Row flags do not populate the separate selection set; toolbar/move sheet and collection stages are missing. Fixed nested viewport/windowing prevents outer pagination from proving complete content. No app-store, persistence or actual search/scroll transition claim. |

Story authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/browser_components.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/fixtures.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/saved_pages_library.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/library_composition.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/library_fixtures.rs`

## Tab Row: current production versus historical matrix

`TAB_ROW_VALID` has nineteen families: twelve visual-state projections (including
Unresponsive), pinned N/S, regular XL, long title, missing favicon, hover close
reveal and closed pinned. Each builder supplies `show_close=true` and `split=None`.
The long-title fixture applies the shared 32-character title budget before the
component's pixel ellipsis. This is not a measured narrow-rail truncation case.

`TAB_ROW_REJECTED` and `tab_row_case_valid` still reject `bitmap_favicon`, with the
reason that `FaviconSpec` has no bitmap field. Current production has
`image: Option<Arc<Image>>`, uses GPUI `img` with a monogram decode-failure fallback,
and preserves Loading/Error/Unresponsive precedence. The rejection is therefore
an obsolete fixture limitation, not a legitimate production-state exclusion.
The audit does not remove the old row or add a fake acquired image.

The production DTO also has `SplitMark`, but every current fixture supplies None.
This is an observed missing branch, not authorization to activate the deferred
Split composition or inspect its worktree. A later authorized coverage contract
must own any such scope. The `focused` family passes the enum only: current
`tab_row` has no focus tracking or dedicated focused ring, and `row_wash` returns
the transparent rest surface for it. Source labels cannot manufacture focus
coverage.

Selected rows apply `elevation::lift`; Dragging applies a 12pt-offset/24pt-blur
shadow. Unresponsive places both Lifering and Clock with a 2pt gap inside the
nominal icon-sized favicon container. These are specific nested paint/layout
surfaces to inspect, not a claim that a runtime clipping defect was reproduced.
Outer child bounds alone cannot prove their extents. The two drag ticks are
explicitly `first=false`; slot-zero/app drag geometry is not represented here.

Story 22 does have a finite endpoint policy: `AttentionPulse`'s Reduced Motion
duration, checked against layout after first prepaint. The production dot retains
its opacity pulse in Reduced Motion; loading uses the static arc. This source path
is stronger than a zero-duration assumption but is not current guest repeat or
animation-timing evidence. No sleeps or changed settling rules are proposed.

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/tab_row_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/tab_row.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/favicon.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/attention.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/status.rs`

## Library: selection is two separate model fields

`library_model("selection")` marks the first row selected; `"multi"` marks two.
Both call `populated`, which leaves `SavedPagesLibraryModel.selected` empty.
Production `library_selection_strip` immediately returns an empty vector when
that set is empty. Thus selected-row fills/boxes are represented, but neither
selector renders the selection count/Clear/Remove/Move toolbar. This is not a
synonym or harmless selector naming difference: an actual production branch is
absent.

All eleven fixtures keep `move_sheet_open=false`, `collection_create_staged=false`
and `collection_action=None`. Consequently the move options and collection
create-input/rename/remove-confirm presentations never appear. The destructive
selector represents **Saved Page removal**, not collection removal. The row
rename selector does not cover collection rename. Repeated monogram content in
`"missing favicon"` is deliberate for Library: production `library_row_spec`
derives monograms and Saved Pages store no favicon blob. Unlike the Tab Row's
stale bitmap rejection, this is a narrow legitimate Library fixture boundary.

The callbacks are genuine component callbacks, but `observe_library` updates only
`LibraryRecorder`, then notifies. `render_library` reconstructs `library_model`
from the selector, never the recorder. Dispatch counters do not stage rename,
change sort/selection, move records or advance the model's window. Existing
semantic tests explicitly set the editing selector after checking rename dispatch;
composition tests check nonempty surface bounds for the eleven labels. These
remain valuable dispatch/renderability tests, not native visual completeness.

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/library_observer.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/library_recorder.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/library.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/tab_row.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/saved_pages_library/model.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/saved_pages_library/header.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/saved_pages_library/rows.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/saved_pages_library/toolbar.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/saved_pages_library/collection_surface.rs`

## Library: nested viewport is not full framing

`large_1k_model` builds 1000 deterministic records, then supplies only indices
`[0, 24)` (bookmark IDs 1–24), `scroll_offset=0` and `viewport_rows=24`. Production
`library_surface` contains a flexing, `min_h_0`, `overflow_y_scroll` list below its
header. The Gallery wraps that entire surface in a 520pt-high direct story child.
The capture paginator can move the whole child to a page; it does not measure
its descendant rows, enumerate scroll positions or expand the bounded model.

Accordingly, neither all 1000 records nor full visible framing of the 24 supplied
rows is proven by the parent fitting. A legitimate independently authored viewport
contract must distinguish required representative windows from intentionally
virtualized records; no demand to flatten the production list or guess additional
pages follows. Native rendering/inspection is needed to locate actual clipping,
ellipsis and visible boundaries. The source audit alone does not assert a measured
row count or current page count.

Authority:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/saved_pages_library/surface.rs`

## Result and continuation

Three additive withheld catalog entries name the stale Tab Row applicability,
Library selection/collection omissions and Library nested-window framing. All
936 historical required page pairs and twelve earlier blockers remain unchanged.
The negative contract and read-only doctor assertions verify those gaps stay
visible, not that the production omissions are repaired. No real E2E or capture
ran for this audit; historical passes are not reused as current image evidence.

IDs 31–48 remain the next bounded source slice (some stories have preliminary
inspection, not a completed audit). Real guest runtime, network/asset approvals,
legitimate endpoint/input handling, current independent requirements, exact
complete repeats and PNG/HTML review remain required. Baseline approval and the
full migration completion audit remain separate and open.
