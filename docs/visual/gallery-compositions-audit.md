# Gallery source audit — Sidebar, Command, Space configuration and menus

Canonical acceptance: [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md).
Inspection base: `eca54f31a41b6c64a0ee7ab81f92a5d0164cbc7d`. This is source and
portable diagnostic evidence, not measured framing, native input, captured pixels
or baseline approval. Product, fixtures, renderers and historical page requirements
are unchanged. See the [audit index](gallery-source-audit.md) for common limits.

The actual registry assigns **44 Sidebar, 45 Command palette / overlay, 46 Space
configuration, 47 combined Tab / Favorite menu, 48 Page menu**. Tab and Favorite
are not separate registered stories; Space configuration must not disappear from
the audit because of that mistaken grouping.

## 44 — Sidebar

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/sidebar.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/sidebar_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/sidebar_fixture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/sidebar_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/sidebar/render.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/sidebar/regions.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/sidebar/strip.rs`

Sixteen selector families build the actual `sidebar_surface`, one rail at a time.
They cover width clamp/wide/zero-width, empty and dense models, long titles,
profile hint, plus menu, staging, download count, projected hover/pressed/focus/
drag, stronger accessibility, titlebar availability and empty Favorites drop band.
Family density remains authoritative: the dense family is Small, others Normal;
canvas appearance and stronger accessibility flow through `fixture_theme`.
Canvas density names therefore do not establish three distinct rail densities.

Concrete defaults exclude current component branches:
- All rows, Favorite tiles and folder members use `closed=false`; image slots
  are absent. Favorite `selected` and `pop`, folder rename/drop and member
  selection are absent. Rows have `split=None`; no Split work is activated here.
- `status_visible=false` in every model means the supplied `StatusModel` never
  paints in this composition, despite the historical full-composition prose.
  `media=None`, `floating=false`, `grain=false`, `tint_fade=None`,
  `drag_proxy=None` and `drag_destination=None` omit those production branches.
  `pin_offer_targeted=false` omits the targeted ghost-row representation.
- Address progress is Idle with default site data and no page mark. A **125%
  zoom chip is represented** in the first family; do not copy the separate
  Address story's no-zoom finding here. Footer Downloads focus remains false.
- The focused-folder family sets `reveal_count=true`, unlike the separate
  Folders story. The production Plus-menu cursor/shadow and production space
  strip are used here, unlike the separate component fixtures. Those earlier
  findings are not grounds to declare these branches absent in Sidebar.

The dense fixture supplies ten pinned rows and only the first 64 of 500 regular
rows, with `scroll_top=0` and a 436-row bottom spacer. Its retained ScrollHandle
is not a browser windowing controller. The 700pt wrapper and outer pagination
cannot prove that all supplied rows, nested folder members, footer and shadows
are visible, nor that scrolling repopulates the window. No measured clipping is
claimed. `SidebarAppState::observe` records and notifies; rebuilding the fixture
from its case does not turn close, navigation, folder or drag callbacks into
model transitions. Actual capture drives no such inputs.

The command-field/Favorites-band anchors are zero-size placeholders; command
results and Downloads are labeled boxes, not app surfaces. Space configuration
and other rect-recording children are absent. Keep these app-owned boundaries
explicit rather than reimplementing them. A width-zero Gallery rail is also not
proof of the app's root-level decision to omit a collapsed rail.

The production Plus menu uses `floating::appear`, which bypasses its fade under
Reduced Motion. Capture forces Reduced Motion; do **not** invent a missing
Sidebar fade wait. Fresh guest endpoint and nested/shadow evidence remain open.

## 45 — Command palette / overlay

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/command.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/command_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/command_fixture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/command_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_cases.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/command.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/command/render.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/command/rows.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/command/moments.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/app/palette.rs`

Twelve selector families render actual `command_card`: Closed/Inactive, palette
and overlay with Results/Loading/Empty/ActivationError, selected destructive rows,
long titles and increased-contrast/reduced-motion. All authored families have
Normal density; all open families set `chrome_focused=true`. Current row anatomy
uses fixed 40pt palette / 36pt overlay rows, not the old density-driven prose.

`build_command_fixture` always uses `EditState::end_of(text)` and `line_sink=None`.
The actual field paints selection, marked composition and insertion-point caret,
but these fixtures never supply selection/marked ranges. In particular, the
comment that the overlay URL is preselected is not what this fixture builds.
App `command_portal_model` passes its actual edit state and line sink.

Every fixture row has `url=None`. Production uses URL data for host monogram
badges and shortened details; a URL-looking subtitle is not that branch. Six
static rows cover tabs, space, actions and fallback, but no `saved_page` drawn
Bookmark or `history` group. Action subtitles are absent and no query starts
with `>`, so shortcut hints and the ACTIONS mode chip are unrepresented. The
current app maps all these row fields through; a missing profile-hint field is
a narrow legitimate exclusion, not a reason to erase URL/content variants.

`entrance=None` and `glide=None` mean static committed rows, not moment evidence.
The callback-free card is legitimate: activation, ranking and Portal placement
stay app-owned. The Gallery recorder summarizes fixture models; it does not
exercise ranking or activation. Semantic tests assert target presence and summary
values, not selected text pixels or full nested framing.

Both presentations receive a fixed `(0,0,600,420)` rect inside a 460pt wrapper;
the card uses `max_h` and `overflow_hidden`, not a measured app placement. The
six-row samples do not establish arbitrary result-list capacity. Long text,
field caret/selection, all rows and elevation still need guest measurement.
The overlay retains its actual `Motion::fine` opacity animation even with the
BBR theme Reduced Motion, and **story 45 already has a matching settling duration**.
Do not reuse Toast's zero-duration finding. The duration mechanism is source
support, not current guest endpoint/repeat proof or animation acceptance.

## 46 — Space configuration

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/space_config.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/space_config_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/space_config_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/space_config.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/space_config/model.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/space_config/render.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/space_config/paper.rs`

Four selectors build fresh actual surface models: defaults, emoji, BBR glyph and
renamed draft. Both focus booleans, all three paper intensities and grain on/off
are represented. The real surface iterates the complete curated theme/emoji/glyph
option sets; selected samples are not an arbitrary Cartesian-product requirement.

Every draft name is nonempty. `name_row` has a concrete missing branch: empty
names paint `Space name` in faint ink rather than the supplied name in normal
ink. This is a component coverage gap, not permission to implement an editor.
App-owned keyboard editing, persistence, exact SpaceId operations and outside
press dismissal remain excluded from the narrow static surface.

The state owns a focus handle but never focuses it during normal Gallery capture.
`focused` draws a two-point ring independently. Callback wiring only appends to
the recorder and notifies; theme/identity/paper/name models are rebuilt from the
selected case, so clicking an option does not establish a changed selection frame.
Existing semantic tests simulate real surface presses and assert events, including
paper/grain; they do not assert resulting model mutation. Their selected debug
bounds are useful, but are not an exhaustive native pixel/framing proof.

The 320pt wrapper lets the curated groups wrap naturally; it does not cap the
surface to an app rail viewport. Direct-child pagination still needs descendant
text/options/ring/Done/Cancel containment checks at each effective density. No
source-only conclusion about clipping or missing options is made.

## 47 — Tab / Favorite menu

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/tab_menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/tab_menu_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/tab_menu_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/tab_menu_framing.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/tab_menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/app/tab_menu.rs`

Five simultaneous static families use the real `tab_menu_card`: no-destination
Tab, Space submenu, Folder submenu, Favorite and Bookmark Page. Three rejected
claims cover disabled selection, invalid submenu and empty menu. Rejection tests
prove those model constraints, not current completeness or source availability.

All five models are focused; the UI's unfocused hairline edge is absent. All Pin
rows say `Pin Tab`; no `Unpin Tab` variant is supplied. Only Bookmark Page is
supplied, not Remove Bookmark. Every Close Tab is enabled; production `build_rows`
sets it from `availability.close`, including the disabled closed-source branch.
Copy Link and Suspend are always enabled in fixtures as well. App row construction
can contain Folder and Space parents together, in that order; Gallery supplies
them in separate families only. These are plain presentation gaps, not a request
to import a controller. Duplicate Tab remains absent from **this app row builder**;
its availability elsewhere in the product is not evidence for inventing a menu row.

The old 240pt constant is not the effective height: `fixture_rects` computes
natural line/icon maxima, rounds each row before summing, and adds current card
padding/borders. The existing ignored semantic test measures the actual production
card with auto height at all densities and checks parent/submenu rectangles against
the 480pt wrapper. Preserve this stronger regression, rather than repeating the
old underallocation claim. It does not independently measure submenu row contents,
painted shadows, long labels, edge flipping or current native page framing.

Both submenu fixtures are fixed on the right at `(228,8)` and contain two rows;
the app owns edge placement. Models rebuild after recorder events; hover/press
callbacks do not update selection, open a new submenu or dismiss a card. Repeated
production IDs across families and wrapper bounds do not prove each descendant.
The real current focus cue is a one-point accent edge, not the stale two-point
ring prose. No renderer fix or guessed page requirement accompanies these facts.

## 48 — Page menu

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/page_menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/context_menu_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/context_menu_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/app/context_menu.rs`

This is the curated matrix subset `[0,3,4,5]`, not a second menu renderer. It
legitimately demonstrates bookmark omitted/save/remove and fully enabled rows,
using the same recorder callbacks. Bookmark commits, URL gating, source binding,
focus restoration and dismissal remain app-owned, not simulated by fixture text.

It inherits story 43's fixed `(8,8,220,148)` card and 260×180 wrapper. The app
computes height from row count, label size and padding: for four rows, 120/124/128pt;
for five, 148/153/158pt across Small/Normal/XL. As in the
[surface audit](gallery-surfaces-audit.md), that formula mismatch is **not** a
measured clipping diagnosis. Last-row text and shadow containment require actual
measurement. Four positive parent bounds cannot close the production-geometry gap.

## Portable evidence and remaining boundary

Five additive withheld entries expose these concrete findings through read-only
diagnostics. All 936 historical required pairs and all 26 prior withheld rows
are preserved as JSON values; no page, sequence or moment is guessed. The new
negative-first contract checks these entries remain withheld with concrete reasons
and no invented capture metadata; the actual CLI doctor tripwire checks visibility
without producer/build/probe/write side effects. These tests do not test fixed
renderers or establish visual coverage. Gate results live in
[Current delivery](../delivery/current.md).

Source continuations now touch every registered ID 0–52, but they are **not** a
completed supported-state contract: effective-state decisions, legitimate fixture
corrections, independent current page requirements, nested framing, native input,
endpoint behavior, fresh guest environment and two exact full captures remain
unproven. No stale source label, historical semantic test, generated inventory or
portable green status can substitute for that evidence or explicit acceptance.
