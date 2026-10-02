# Gallery source audit — Downloads, Find, Error, Auth and Context menu

Canonical acceptance owner: [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md).
Inspection base: `f9b80c9c0c6d27523e6a5f4e90646a2092a85315`. This is a bounded
source audit of IDs 38–43, not native capture, a measured clipping diagnosis,
input evidence, complete applicability or baseline approval. It continues the
[Gallery audit index](gallery-source-audit.md). Product and fixture sources stay
unchanged; IDs 44–48 remain subsequent source work.

## Shared capture and observation boundary

All six registrations in
`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories.rs` have empty
selector lists: their default page enumerates static matrix families. Capture
changes appearance/contrast/density and forces Reduced Motion, but does not send
pointer, keyboard, scroll or editing events:
`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app/capture.rs`.

The per-story app states build observers which record events and notify the
Gallery entity; subsequent rendering rebuilds the same fixture models. Neither
recorded press counts nor a fixture boolean imply an input-driven transition.
Sources are the `render_context`/`observe` implementations in:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/downloads_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/find_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/error_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/auth_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/context_menu_app.rs`

Existing semantic tests are valuable callback/renderability contracts, not native
pixels. Repeated production IDs often resolve to the last painted family. Outer
family bounds, presence of an ID and laid-out overscan are not proof that every
nested control/text/shadow is visible in the capture viewport. No Gallery GUI test is run or
weakened by this source inspection.

## 38 — Downloads panel

Sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/downloads.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/downloads_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/downloads_panels.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/downloads.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/downloads/render.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/downloads/row_detail.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/downloads/check_moment.rs`

The actual `downloads_panel_surface` renders seven machine/action families, two
Running wash families and five panel families. Waiting unknown/known, Running
fraction, Succeeded Reveal enabled/disabled, Failed Retry and Cancelled
Restart/Delete are represented. Group labels, empty hint, Clear and a static
keyboard-cursor/ring model are present. The current rejected Cancelled-without-
Restart/Delete rule is legitimate; no native transfer/controller is fabricated.

`family_model` always supplies `open=true`, `alpha=1`, `check=None`, and a width of
260pt. Thus the production completion check/ring key and panel closing/fade states
are not driven. The static succeeded check is a legitimate rest sample, not a
completion sequence. The unknown-progress implementation is a **fixed 30% leading
segment**, despite prose calling it an indeterminate sweep; no moving sweep is
proven or required by that source branch. Disabled Reveal carries a reason in the
DTO but the component does not paint a reason tooltip; do not invent one.

Every family borrows the same `ScrollHandle`. Each `family_model` call sets its
offset; the 22-row history panel supplies window `8..22` and a negative offset,
but later cursor and wash families reset that same handle to zero before the
page is painted. Independent scroll state is therefore not established by the
fixture's offset calculation. The source defect is the shared mutable handle;
the actual visible outcome needs guest measurement, not an inferred screenshot.
The production viewport exposes at most six rows while fourteen are laid out,
with spacers and clipping. Existing tests assert overscan bounds and omission of
row 7, not all rows' visibility or scroll traversal. Repeated row IDs are explicitly
handled by painting washes last, contrary to the stale introductory unique-base
comment. A focused model does not focus the shared handle; the app state explicitly
never focuses it. Nested text/actions, viewport, ring and `elev2` extents remain
open. Withheld: `gallery-38-download-scroll-moments`.

## 39 — Find bar

Sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/find.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/find_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/find.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/find.rs`

Six static `FindSurface` DTOs use the real `find_bar`: focused/unfocused, empty/
typed query, pending report, intermediate/last ordinal and zero matches. The
caller-resolved theme reaches the surface. The fixture rect is 360pt by the
density row height, at the origin; it is not a measured native portal placement.
The component renders text and an accent field edge, not a Gallery editor.
Previous/next/close callbacks record only; they do not step counts, edit or close.

`with_find_id_targets` registers all seven production-ID debug selectors on
nested wrappers around the **entire first bar container**. Those debug bounds
are not the individual field, text, count or control bounds. The semantic tests'
ID-presence checks therefore cannot establish descendant geometry; their clicks
use token-derived coordinates instead. Complete text/control/focus/shadow framing
and actual focus/typing remain unproven. Withheld:
`gallery-39-find-wrapper-bounds`.

Narrow exclusions remain truthful: the DTO has no stale flag and no enabled/
disabled control property. The three pointer controls have listeners regardless
of count; the portable session owns no-step-at-zero policy. A rejected matrix
claim is not proof that this card disables Next or implements the session policy.
Do not create a fake stale visual or disabled variant.

## 40 — Error surface

Sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/error.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/error_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/error.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/error.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/app/error_surface.rs`

Sixteen static TLS/navigation/offline families use real `error_surface`, including
origin-present/omitted and Retry/Dismiss copy. TLS bypass/raw-platform-message
rejections remain narrow, legitimate policies. These literal fixture strings are
not proof of native error classification or recovery.

`build_error_fixture` sets `illustration=None` for **every** case. Production
`ErrorSurfaceState::model` maps Offline to `Some(None)` (rest illustration), and
`error_surface` chooses the Offline drawing for `Some(key)` versus Lifering for
`None`. All three Gallery Offline cases therefore select the wrong production
branch. The foundation's separate Offline illustration sample cannot prove this
error composition. Withheld: `gallery-40-offline-illustration-gap`.

Each full-size surface is nested in a 240pt frame. Wrapped copy, origin and button
visibility need actual descendant evidence. Repeated IDs and first-family clicks
at fixed offsets are not complete geometry assertions. Recorder action/dismiss
counts are not recovery/re-navigation or disappearance evidence.

## 41 — Crash / Empty panes

Sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/error_panel_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/error.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/pane.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/app/render.rs`

Two families call actual `crash_panel` and `empty_space_pane`, each with illustration
key `None` and a 260pt frame. Rest illustrations, Reload/Close and New Tab/profile
copy are legitimate static presentations. The matrix rejects a crash panel for a
healthy tab. It does not exercise native-view replacement, actual crash/reload,
new-tab creation, input focus, appearance motion or all nested extents.

Current production also calls `unresponsive_panel`, with Wait/Reload/Close and
three focus handles. The Gallery has no call to that component and its pane
matrix only enumerates Crash/Empty. This missing recovery-presentation family
cannot be declared inapplicable merely because the older matrix predates it.
Withheld: `gallery-41-unresponsive-pane-gap`. This observation does not authorize
product changes or manufacture a new capture/page requirement.

## 42 — Auth sheet: valid narrow card, not an entire challenge flow

Sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/auth.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/auth_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/auth.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/auth.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/portal/probe.rs`

Eight authored examples represent realm present/absent, origin/proxy, empty/typed
synthetic username, empty/masked password flag and username/password focus. The
builder's 32-combination unit test proves construction, **not 32 captured families**;
no arbitrary cross-product is added. Real `auth_sheet` takes the resolved theme
and renders at fixture width 360pt. Production now follows the shell theme too,
despite older module prose referring to dark-only presentation.

The password DTO has no raw password and its true flag draws **six fixed bullets**,
not an observed password length. Username Debug is redacted and no real credentials
are used. Raw-password/unmasked/serialization exclusions remain valid; they must
not be converted into visual cases. Focus is a model edge, not text input or Tab
traversal. Submit/Cancel only record. Measured meta-wrapper/button dimensions in
semantic tests are useful targeting evidence, not all-family text containment.

The Gallery deliberately renders just the shared card. The app's Portal additionally
owns content-card `wash_active` veil, placement, propagation policy and `appear`.
The Gallery's lack of an outside-dismiss listener does not prove that policy, native
challenge lifecycle, keyboard submit/cancel or redaction across the browser. Those
remain app-owned composition/functional evidence; do not rebuild them in the Gallery.
No new omitted-card-branch blocker is invented for this legitimate narrow boundary.
The existing all-state/framing requirement still covers its unproven runtime bounds.

## 43 — Context menu

Sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/context_menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/context_menu_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/context_menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/theme.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-app/src/app/context_menu.rs`

Eight page-menu models call real `context_menu_card`, representing Back/Forward
availability, missing/present link, Bookmark/Remove/omitted bookmark, selected row
and focused/unfocused edge. Disabled rows have no hover/press listeners. Recorder
notifications do not mutate selected rows, execute page actions or dismiss the card.
WebKit element menus and source-less Tab/Favorite/Folder claims are legitimately
outside this page-menu model; no native menu pixels are synthesized.

Every family uses rect `(8,8,220,148)` inside a 260×180pt container, for both four
and five rows and every density. App `context_menu_placement` instead derives
height as `items * (label.size + 2*sp2) + 2*sp1`: current Small/Normal/XL four-row
heights are 120/124/128pt and five-row heights 148/153/158pt. These are app formula
values, not measured natural paint bounds. Production rows use text with vertical
padding, not `density.row`; the Gallery semantic-test click helper still uses the
latter. The fixed fixture matches only one formula case, not general production
placement. Native text line-height, last-row containment and `elev2` extents must
be measured rather than asserting a clipping defect from arithmetic alone.
Withheld: `gallery-43-context-menu-geometry`.

## Independent coverage and verification boundary

Five additive withheld rows record the concrete gaps above. All 936 historical
required page pairs and the preceding 21 withheld rows remain unchanged; the
independent page plan is untouched. No guessed page, sequence or moment metadata,
unsupported relabeling, renderer/fixture fixes, asset changes or baseline edits.
A negative-first consumer contract requires these rows and their concrete reasons;
read-only CLI discovery asserts their visibility without executing producers.
Exact commands/results and local ownership are recorded in
[Current delivery](../delivery/current.md). Portable tests prove diagnostic
completeness only. Fresh guest capture, legitimate input/endpoints, nested framing,
independent pixel repeats, PNG/HTML review and functional Tart gates remain open.
