# Gallery visual source audit — migration prerequisite

Canonical acceptance owner: [Gallery visual coverage](../roadmap/tasks/gallery-visual-coverage.md).
This document records a bounded **source inspection**, not complete visual
coverage, measured page ranges, real input evidence or approved reference images.
Initial inspection base: `b9228dce`; the [Foundations / Primitives / Core Controls
continuation](gallery-foundations-audit.md) audits IDs 0–20 at `4a5fedf9`;
the [Status / Tab Row / Library continuation](gallery-library-audit.md) audits
IDs 21–30 at `7cc720cc`. No product or Gallery renderer changes accompany
these inspections.

## Registry and historical-plan mismatch

The catalog in
`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories.rs` declares
53 append-only identities (0–52). The independent retained plan at
`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/catalogs/gallery-plan.json`
contains only IDs 0–48: 630 appearance-free selector/contrast/density cases and
936 page pairs (1,872 images). Those numbers describe the historical requirement,
not current-source completeness. Required entries stay unchanged until they are
audited; emitted inventory cannot silently regenerate the expected plan.

Four explicit `withheld` entries in
`/Users/patwoz/dev/Personal/macos/bbr/scripts/visual/catalogs/gallery.json` now name
IDs 49–52 and their concrete gaps. They are not `unsupported`: the missing audit
and capture work cannot be made green by classifying it as inapplicable. They
have no invented page, sequence or moment metadata. Existing interaction,
framing and locked-session blockers remain visible.

## What current producer selection proves

Source authority:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_cases.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/app/capture.rs`

The producer accepts one numeric `--story` and filters cases before creating its
rendering context. Inventory always lists the complete registry. The internal
Rust consumer now binds an upstream Selection to an independent schedule and
original host request, invokes that existing numeric command per authored story,
and validates returned scope before upstream sealing. The producer itself remains
unchanged and does not consume `GPUI_VISUAL_SELECTION`. Portable fixtures prove
this composition, not actual guest/native selection parity. No real story catalog
is enabled; normal consumer capture remains blocked.

The producer enumerates each registered selector across two appearances, two
contrast settings and three density settings. This is **canvas settings**, not
proof of twelve distinct production component variants. For example,
`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/sidebar_fixture.rs`
and
`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/command_fixture.rs`
retain each family's intrinsic density and stronger accessibility requirements.
The future applicability contract must record effective component settings and
explicit non-applicability, not infer it from image names. Selector labels such
as hover/pressed/focus do not establish that pointer/focus events were driven.
Capture selection forces Reduced Motion but never drives those input events.

## Pilot source findings

| Story | Represented source content | Required evidence still missing |
|---|---|---|
| 0 — Tokens | Ten direct children: five material swatches plus section/value rows from real theme tokens | Fresh Light/Dark guest frames, exact full framing and independent repeat. The labels describe elevation; they do not demonstrate painted shadows or all theme states. |
| 16 — Checkbox | Real vendored Checkbox, recorder-backed initial checked value and production callback | Actual pointer down/up through production callback, checked and unchecked frames, semantic assertions and complete appearance pairs. Current capture never clicks. The empty asset source cannot supply the required checked SVG; the legitimate asset-source change still requires approval. |
| 44 — Sidebar | Selected production `sidebar_surface`, family fixture theme, metadata rows, 700pt rail and rejected-matrix explanations | Independent current selector/page requirements, actual multipage guest frames, nested clipping/shadows and effective density audit. The windowed 500-tab family is not a picture of every tab. App-owned command/download slots are explicitly labeled placeholders, not those real browser surfaces. |

Pilot references:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/foundations/tokens.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/core_controls.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/sidebar.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/vendor/gpui-component/src/checkbox.rs`

The Tokens/Elevation distinction matters: the Elevation story's final sample has
background, border and radius but no `.shadow(...)`; preceding rows print shadow
token values. It cannot substitute for shadow-extent coverage on real overlays.
No pixel change or proposed restyle is authorized by this observation.

## Added stories (49–52)

| Story | Source-represented states | Scope exclusions / open work |
|---|---|---|
| 49 — Delete Space sheet | Two exact production cards with Cancel/Delete keyboard-focus model values and two counted-body fixtures | These are static focus representations, not a real Tab transition. App placement, veil and commit are excluded. Production card/focus-ring shadow bounds and both current page sets require measurement and review. |
| 50 — Peek presentation | Production `peek_wash` and `PeekFrame` around GPUI stand-in origin/preview pages, plus a reserved empty bar rectangle | Stand-ins are not native WKWebView or actual portal pixels. Do not promote this composition to required production-page/portal evidence or synthesize replacements. A narrower legitimate wash/geometry capture contract must be decided before claiming visual coverage. |
| 51 — Toast and zoom readout | Three production toasts (Accent/Ok/Neutral, with/without action), one 140% readout | Toast entrance opacity survives Reduced Motion, but story 51 currently gets zero settling duration. Thus the source path cannot prove a settled toast endpoint. No runtime nondeterminism is claimed without capture. Page/shadow evidence and legitimate endpoint handling remain open; timers, actions, exits and portal placement are not proven. |
| 52 — Media card | Six production static cards: playing/seekable, paused, live, unknown duration, muted, several sessions | Callbacks are no-ops. No live media, WebKit capability, seek or playback transition evidence. Measure all six fragments, pagination and nested bounds before authored capture requirements; no guessed page counts. |

Source references:
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/space_delete.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/peek.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/toast.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/stories/media_card.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/space_delete.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/surfaces/toast.rs`
- `/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-ui/src/components/media_card.rs`

## Framing boundary

`/Users/patwoz/dev/Personal/macos/bbr/crates/bbr-gallery/src/capture_page.rs`
measures/paginates **direct story-child layout bounds**, validates those bounds
inside a 48pt inset, and refuses an indivisible child taller than 856pt. This
proves neither descendant clipping nor painted shadow extents. The 48pt padding
is not a measured all-shadow guarantee. Nested scroll viewports, virtualized
lists and placeholders must be accounted for independently; missing coverage
cannot become success because the outer wrapper fits. The plan's retained page
ranges remain historical diagnostics until fresh current-source evidence.

## Next bounded steps and stop boundaries

1. Audit remaining IDs 31–48 story/matrix families and effective-state
   applicability from their source authorities. Preserve an explicit open item
   where interaction, framing or production identity is not proven.
2. Verify the now internally composed upstream-selection → guest-request →
   existing producer filtering → upstream-seal path with real approved guest
   tests. Selected reports remain explicitly partial and non-accepting; complete
   appearances/pages are required. Synthetic contract passes are not that proof.
3. Obtain network-route and legitimate Checkbox asset approval; verify actual
   guest runtime availability without installation/repair/download fallback.
4. Run Tokens/real Checkbox/multipage Sidebar pilot, measure complete framing,
   resolve the Toast endpoint through production duration rather than sleeps,
   and author current page requirements independently of emitted output.
5. Only then prove the supported full matrix with two independent exact RGBA8
   captures and actual PNG/HTML review. Baseline/environment approval is separate.

No production or Gallery edits, producer/native execution, VM boot, baseline
updates, runtime installation or performance claim belong to this audit slice.
This audit is intentionally incomplete for all-state/framing acceptance.
