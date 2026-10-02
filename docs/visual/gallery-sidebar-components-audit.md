# Gallery applicability audit — Address and Sidebar components

Acceptance owner: [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md).
Read with the [migration source audit](gallery-source-audit.md).
Inspection base: `83109086334598f96f78c64b7c229bd076901e75`; IDs 31–37 only.
This is source evidence, not a native render, measured clipping/page count,
interaction recording or baseline approval. No product, fixture or renderer edits.

## Common boundary

All seven registry entries have empty state selectors: their story draws the whole
family matrix. Capture changes canvas appearance/contrast/density and forces BBR
Reduced Motion, but does not drive pointer, keyboard or scroll input. Favorites
and Profile-chip families retain their declared density; the other components use
the canvas theme. Family labels and emitted inventory are not independent coverage
or proof of effective state. Historical required rows remain untouched.

Address and Group-4 observers record dispatch; builders reconstruct models from
matrix constants. Notification does not turn those observations into hover, folder
expansion, selected tabs or open menus. Static fragments legitimately omit browser
controller/persistence execution, but that narrow boundary cannot erase missing
production presentation branches. Existing Gallery tests check family bounds,
IDs and dispatch/propagation, not native pixel equivalence or complete framing.

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/address_recorder.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/group4_app.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/group4_recorder.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/e2e/group4.rs`

## Story-by-story applicability

| ID / story | Source-represented content | Open evidence / narrow exclusions |
|---|---|---|
| 31 Address control | Ten production `address_control` samples: domain variants, hover boolean, profile hint, insecure HTTP, Library/Settings and local file marks | Zoom/progress/device marks and History/Welcome are absent. Field anchor is invisible placeholder, not measured overlay geometry. Display-only pill has no editing state; this does not exclude staging editing below. |
| 32 Command staging | Seven production `staging_surface` models: text/placeholder, focused/unfocused, cursor, hint and profile hint | All cursors are at end, no selected/marked range, no layout sink. No real text/IME/focus transition or long-line framing. |
| 33 Favorites grid | Eight grid families including S/N/XL, identity, empty, selected and closed; five extra groups of production drag-offer fragments | Every tile is monogram-only with no pop key. Widths assume a 300pt rail without constraining the story grid to that rail. No real drag/selection/reopen state transition. |
| 34 Folders | Eight production row/member families: collapsed/expanded/empty/count/rename/two groups/selected/closed | Counts always transparent; selected rename, drop target and bitmap branches absent. Member hover close is a real production style, not a modeled capture state. Nested shadows remain unmeasured. |
| 35 Footer | Eight real `footer_surface` families: count/no count, one/many/overflow/active/quiet chips, plus control | Downloads focus ring never supplied. Strip is a fixture using profile chips/text, not production space-strip anatomy. Nested horizontal overflow is not traversed. |
| 36 Profile chip | Twelve real static `profile_chip` samples: three sizes, one/two letters, six hues and distinct profile index | No own callbacks or interactive state: narrow static non-applicability is justified. Not a space-chip/navigation surrogate. Real glyph/text bounds and Light/Dark review remain open. |
| 37 Plus menu | Three state/data families: closed explanation and two open selection positions, two production item identities, pointer recorder | Story constructs its own card/rows rather than current production `plus_menu_surface`; selected styling already differs. No focus, real key dispatch, outside dismissal or production shadow evidence. |

Story sources:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/address.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/favorites.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/folders.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/footer.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/profile_chip.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/menu.rs`

## Address and staging: current fields exceed the matrix

`build_address_fixture` always supplies `zoom=None`, `progress=Idle` and
`site=Default`. Production paints a zoom accessory when supplied; Loading is a
sweep or static Reduced Motion line, and Finishing follows its production motion
plan (absent under Reduced Motion). A static finish label would not establish
that time-dependent branch. The four live/blocked camera/microphone combinations
replace the identity mark and may have independent site-settings press routing.
None is in this matrix; zoom/site callbacks in the story are no-ops. History and
Welcome are production internal-page mark variants absent from the ten fixtures.

`hovered=true` is an authored model, not the result of recorded hover. The story's
"no border" explanatory text is stale: production uses a one-point address border.
The rejected Copy-Link accessory remains a narrow valid exclusion (no such branch
here); it does not imply an empty-only accessory because zoom now exists. The
pill remains noneditable and has no focus-ring branch. Do not promote all of a
historical rejection's prose to current design authority.

Every staging model uses `EditState::end_of(text)` and `line_sink=None`.
Production `edit_line` has byte-indexed insertion, selected-range wash, marked-text
underline and scroll-to-caret within a content mask. The fixture's short `Reading`
or empty text never supplies selected/marked ranges or a mid-string caret. The
painted caret is not proof of platform text input, IME candidate placement or focus
ownership. Staging profile hints and suffixes compete for width; direct parent
bounds cannot prove their descendant text fits. No guessed editing sequence,
new input authority or screenshot synthesis is proposed.

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/address_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/address.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/edit_line.rs`

## Favorites and Folders: labels do not establish rendered branches

All Favorite fixtures have `image=None`; all `favorite_tile` calls use `pop=None`.
Production supports GPUI image rendering/decode fallback (including closed opacity)
and pop surface/speck layers. Static offer/target fragments are real components,
but do not exercise a production drag, commit or pop timing. Hover/active/focus
are not fields on this tile DTO and no such dedicated tile branches exist here;
that narrow fact is not an exclusion for app-owned drag/keyboard composition.

The story passes `favorite_tile_width(theme, 300.0)` to each tile. Production
`favorites_grid` uses `flex_wrap`, not a fixed four-column grid. The story family
has no matching 300pt width constraint, so the "four columns" text and column-count
helper do not establish actual wrapping on the wider capture page. This is a
specific geometry mismatch to measure, not a claim of a reproduced pixel defect
or permission to change layout. Representative rail widths need an independent
framing contract; count labels cannot substitute.

Every Folder row has `reveal_count=false` and `drop_target=false`; rename uses
`selected=false`, and every member image is None. Production count opacity is
one only when `reveal_count && rename.is_none()`. Thus the "count tnum" family
carries 12 in its model but paints the count at zero opacity. The story's folder
hover callback is a no-op; clicking the disclosure records an index without
changing the fixture's expanded state. The rename selection wash and drop-target
outline/fill are absent, as are image/decode-fallback samples.

The nested close control uses production `group_hover` even without a hover DTO;
it is opaque when selected and otherwise transparent until hover. Therefore
"the DTO has no full state field" is not enough to declare every hover visual
inapplicable. The selected member's `elevation::lift` and rename row's 2pt-spread
halo are additional painted extents, not proven by a direct story-child rectangle.
The "hairlines between groups" fixture simply appends rows/members, and current
`folder_row` adds no group-separator hairline; its label is not paint evidence.
Selected-closed and collapsed-with-visible-members remain legitimate narrow
rejections. No browser persistence/rename/selection behavior is changed here.

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/favorites_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/folders_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/favorites.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/folders.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/folders/rename.rs`

## Footer, Profile chip and Plus menu: identity versus renderer

Every Footer call passes `downloads_focused=false`; the shared tracked focus
handle is deliberately not focused. Production has a conditional two-point ring
and Enter/Space route, neither established by count/plus pointer recording. The
story's hairline description is stale: current footer intentionally paints no band
or rule. `build_footer_strip` composes profile monograms plus space/profile text,
with an accent border for active chips. It does not call production
`space_chip`/`space_strip`. The twelve-chip overflow sample is a horizontal scroll
viewport without authored scroll positions or descendant visibility evidence.
Outer pagination cannot prove every chip is seen; the active label also does not
supply the production chip's selected surface/lift. Keep it classified as an
in-context fixture, not a second production strip authority.

Profile-chip samples do use the actual stateless production symbol. Its profile
index, hue and one/two-letter fixtures are directly represented; no new blocker
invented for nonexistent chip callbacks. Text bounds still need real inspection,
and fixed family densities are not twelve unique canvas-derived variants.

The Plus-menu story shares `plus_menu_items`/`PlusMenuState`, not the production
menu element tree. It paints selected rows with `border_l_2` and an unrounded
accent wash; its card uses a hairline border, vertical padding and no shadow.
Current production `sidebar::regions::plus_menu_surface` uses rounded
`floating::menu_cursor` **without an accent rail**, all-side menu padding, `elev2`,
tracked focus, key callbacks and an inside-press callback. These are concrete
source differences despite shared labels/IDs. The pure `move_selection` test
checks up/down wrapping only; it is not Enter commit/Escape dismissal input
coverage. The closed family renders explanatory text, not a hidden production
menu transition. Correcting renderer reuse requires a separate bounded follow-up;
this audit neither adds nor legitimizes a second renderer.

Authorities:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/footer_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/profile_chip_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/menu_matrix.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/footer.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/space.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/menu.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/sidebar/regions.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/sidebar/strip.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/floating.rs`

## Result and continuation

Six additive withheld entries expose the Address, staging, Favorites, Folders,
Footer and Plus-menu gaps. All 936 historical required pairs and fifteen prior
blockers are preserved unchanged; no page, sequence or moment is invented.
Negative-first catalog tests and read-only discovery assertions keep these gaps
visible, not repaired. No VM, native capture, real PNG/HTML review or functional
GUI gate occurred. Actual guest runtime, approved network/asset route, pilot,
full same-source/environment repeats and explicit acceptance remain required.

IDs 38–48 remain a subsequent bounded source slice; none of their all-state or
framing gates is implied complete by this note. The migration and final
requirement-to-artifact audit remain open.
