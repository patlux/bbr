# BBR — Design System: visual direction and semantic tokens

Status: **canonical visual contract** for Phase 4 and every later UI phase.
Product intent lives in [PRODUCT.md](PRODUCT.md); interaction behavior in
[docs/product/browser-interactions.md](docs/product/browser-interactions.md);
performance budgets in [docs/product/performance-contract.md](docs/product/performance-contract.md);
implementation facts in [ARCHITECTURE.md](ARCHITECTURE.md); delivery status in
[ROADMAP.md](ROADMAP.md). This document is the single source of truth for how
BBR looks, states its components, and exposes that state to automation.

## How to read this document

- **Contract** — decided visual behavior. Components, tokens, and E2E
  semantics must follow it. Deviating means updating this document in the same
  group.
- **Planned** — visual direction acknowledged in the roadmap but not yet
  presented in the running app.
- **Open decision** — intentionally unresolved; resolve here before the
  affected surface ships.

Every token has a stable name (`color:surface.sidebar`, `type:title`,
`sp:3`, `radius:md`, `elev:2`, `dur:fine`, `density` variants). Names are the
interface: the GPUI token module must use exactly these names so tests and
docs align. Interactive surfaces carry stable semantic ids prefixed `bbr-`
(§15) so real-app E2E can assert rows, states, and focus without pixels.

## 1. Visual direction — native light rail and untampered glass

BBR's production chrome is a **quiet light macOS rail** holding **untampered
glass** (native web content). Patrick's complete 2× production-shell reference
(`/Users/patwoz/Library/Application Support/CleanShot/media/media_aio6G8pltD/Screenshot_2026-08-12_12-16-59@2x.png`)
is the visual authority for the expanded Sidebar: a softly tinted matte rail
behind the traffic-light safe area, compact controls and address display,
large Favorite tiles, the active Space name as the primary hierarchy, readable
inset tab/folder rows and a compact footer strip. BBR retains its own name,
geometric icons, Phosphor interaction accent, semantic indicator hues and real
browser data/capabilities.

A keyboard-first daily driver for many long-lived contexts needs a shell that
recedes when content appears. Chrome remains matte, flat and deterministic —
never page screenshots, sampled site tint, fake content or private WebKit
composition. Color marks selection, status, profile identity and focus. Dark
and increased-contrast token sets remain supported library variants. The
production shell follows the **macOS system appearance** — a dark system
presents the complete dark token set, a light system presents the shipped
reference-calibrated light rail; neither mode is hardcoded and no user-facing
theme preference exists. Light remains the calibrated reference (the
visual authority above); Patrick's dark reference capture
(`/Users/patwoz/Library/Application Support/CleanShot/media/media_lbaS6F07Ws/Screenshot_2026-08-12_21-45-38@2x.png`)
is the dark composition authority — it shares the light reference's hierarchy
and geometry and only swaps the token set.

### What makes BBR visually distinct

| | Arc | Zen | Safari | Swift reference (PWBrowser) | **BBR** |
|---|---|---|---|---|---|
| Shell | floating pills, candy gradients, auto-hidden toolbar | adaptive per-site tint, glass blur, playful radius | uniform translucent toolbar, ambient page color | ad-hoc dark placeholder chrome | full-height light matte rail, native titlebar-safe integration, constant luminance across sites |
| Color | saturated gradient accents | site-adaptive pastels | ambient translucency | single hardcoded grays | soft neutral hierarchy + one Phosphor accent + semantic indicator hues |
| Tabs | large rounded chips | large rounded cards | compact tabs with favicon | none (text placeholder) | readable inset rows with quiet selected surface and exact favicon/monogram identity |
| Geometry | organic, large radii | organic | respectable | none | 4 pt rhythm, 6–10 px row radii, reference-calibrated tiles and 36–40 pt normal rows |
| Motion | playful spring choreography | animated everywhere | subtle | none | commit-first snaps; animation only on transient indication |
| Content relation | sidebar overlays page | sidebar overlays page | toolbar merges | separate | sidebar is a physical rail; content is never overlapped or sampled |

**App icon (v2 D8).** The packaged app carries "Phosphor b": a lowercase b on
a dark continuous-corner tile whose bowl is the accent loading arc — the
spinner of §10.5 becomes the brand, so the Dock and the rail speak one visual
language. The Dock icon is static; the vector master is
`packaging/icon/bbr-icon.svg` and `scripts/package/gen_icon.py` renders it
deterministically into `AppIcon.icns` (§7).

The Swift reference is a functional ancestor (spaces, profiles, pinned tabs),
not the production visual authority. Patrick's complete-shell screenshot above
supersedes the former dark placeholder interpretation. The implementation uses
this token system and the one production `bbr-ui` composition rather than
hardcoded render colors or a Gallery-local shell.

### Design principles

1. **Native truth.** The chrome must feel like a first-class macOS citizen —
   quiet aluminum, 4 pt rhythm, system text — not a webby dashboard pasted
   into a Metal window.
2. **Machine-readable.** Rows are tables of readable data: uniform row
   heights, tabular numerals, deterministic truncation, stable identity per
   host.
3. **Signal over decoration.** Motion, color, and elevation exist to mark a
   state or a fact, never as ornament. If a state needs no attention at rest,
   it renders at rest.
4. **The glass stays glass.** Web content is the native child; we never
   screenshot, blur, sample, or re-render it. The boundary between chrome and
   glass is one hairline, then silence.
5. **Commit first, indicate after.** Selection and layout changes snap on the
   next frame (budgets in the performance contract); animation is allowed only
   for transient indication (loading, attention, press) after the fact.
6. **Restraint under pressure.** At 500 tabs, dense mode, long titles, missing
   favicons, crashed pages — the system degrades by *removing* detail, never
   by adding noise.
7. **Density is a product of the machine, not a slider.** Small/normal/very
   large collections each get deliberate geometry (§8) rather than a free
   zoom control.

## 2. Composition, materials, and elevation

### Layers (bottom → top)

```text
L0 chassis         — window base the rail and the content card sit in
L0.5 content card  — the seated native slot; canvas shows through before attach
L1 sidebar/rail    — chrome surfaces: sidebar, panels      elev-0
L2 controls        — fields, buttons, rows inside chrome   elev-1
L3 floating        — command palette, peek, dialogs        elev-2
```

| Token | Light | Dark | Notes |
|---|---|---|---|
| `material:canvas` | `#F6F6F4` | `#131517` | content backdrop *inside* the content card, visible only before attach / empty / crash |
| `material:chassis` | rail paint | rail paint | the window base the rail and the card sit in; the rail's own (space-tinted) paint, so the chassis reads as one continuous body rather than a second surface |
| `material:sidebar` | `#E5E5E2` (limestone) | `#1C1E22` (wet slate) | the physical rail (v2 §18.2); the production mode follows the system appearance |
| `material:field` | `#FFFFFF` | `#2A2D33` | command field, search, hover-raised rows |
| `material:overlay` | `#FFFFFF` | `#25282D` | floating surfaces, elevation 2 |
| `material:glass` | `#FFFFFF` | `#151719` | what sits behind native web views; must never be textured or sampled |
| `material:favorite` | white 50 % veil | white 6 % veil | Favorite destination tile, painted over the (tinted) rail |
| `material:footer` | white 62 % veil | white 5 % veil | the media card at the rail bottom (the Downloads/Space/Plus strip sits directly on the rail, S12) |

Veils (§18.2) are translucent tokens painted over the possibly space-tinted
rail, so one value follows every tint; contrast gates composite them over the
untinted and every tinted rail before computing a ratio.

### Elevation

- `elev:0` — base surface. The rail carries no border; the content card's
  own edge supplies the only separation.
- `elev:1` — raised control (field, button, hover tile). `hairline` border,
  same material family. No shadow on rows (list paint budget), with exactly
  one exception (D2): `elev:lift` on the one selected row and the active
  space chip — static, painted with the surface, never animated or on hover.
- `elev:1.content` — the seated content card. One static shadow, painted with
  the card, never animated and never re-evaluated per scroll frame.
- `elev:2` — floating. Shadow tokens (GPUI `box_shadow`, bounded blur):

| Token | Light | Dark |
|---|---|---|
| `elev:1.content.shadow` | `0 1 2 rgba(23,26,29,0.06), 0 8 20 rgba(23,26,29,0.10)` | `0 0 0 1px rgba(255,255,255,0.05), 0 1 2 rgba(0,0,0,0.45), 0 10 24 rgba(0,0,0,0.38)` |
| `elev:2.shadow` | `0 0 0 1px ink 7 %, 0 2 4 ink 5 %, 0 16 36 −6 ink 18 %` | `0 0 0 1px white 8 %, 0 16 32 black 45 %` |
| `elev:lift` | `0 1 3 ink 10 %, 0 0 0 1px ink 4 %` | `0 1 2 black 35 %, 0 0 0 1px white 4 %` |

### The content card

Web content sits in a **content card** (`bbr-content-card`,
`bbr_ui::card`): a rounded, clipped rectangle inset from the window by
`sp:content-inset` on the top, trailing and bottom edges and **flush with the
rail on its leading edge** — the rail and the card are one continuous body
seated in the chassis.

| Property | Value |
|---|---|
| Inset | `sp:content-inset` top / trailing / bottom; `0` leading |
| Radius | 10, all four corners (= `sp:content-inset`; independent of `radius:lg`, which v2 moved to 12) |
| Elevation | `elev:1.content` |
| Behind it | `material:chassis` |
| Inside it | the native WKWebView (its backing layer clips to the radius); `material:canvas` only before attach / empty / crash |

1. **The card is the content, not a card over content.** The card's inner
   box is the native slot; nothing floats above the glass because of it.
2. **The leading edge is never inset** while the rail is visible.
3. **The shadow seats, it does not glow:** short, dark, low-alpha layers; on
   dark a 1 px `rgba(255,255,255,0.05)` ring.
4. **The rail loses its right hairline**, and the resize divider is empty
   paint at rest (accent only under keyboard focus).
5. **The inset is chrome, so it disappears with chrome.** Native full screen
   is full-bleed: inset `0`, radius `0`, no shadow. A collapsed sidebar adds
   the leading inset, and the top inset becomes the native titlebar band
   (`win:titlebar`, the safe titlebar strip height) so the traffic lights
   never sit on web content. The band is empty, draggable chassis
   (`bbr-chassis-band`, double click runs the system titlebar action).
6. **The inset is not free space for chrome:** no control, badge, or
   indicator is placed in the chassis gap.

Clipping uses public AppKit/Core Animation only (`wantsLayer`, the layer's
`cornerRadius`/`masksToBounds` on the WKWebView and its navigation cover);
web content is never sampled or re-rendered. Split panes share the one card
until the [Split composition](docs/roadmap/tasks/split-app-composition.md)
defines pane cards; each pane's layer clips to the card radius meanwhile.

### Hairlines

`hairline` = 1 px structural rule: separates regions within a surface
(profile groups, pinned/regular boundary, sidebar from content). Hairlines are
*presence*, not *content*: they may fall below 3:1 contrast (verified at
≈1.1–1.3) and must never be the only cue for a state. Increased-contrast
strengthens them (§3.4).

**Deferred: vibrancy.** No `NSVisualEffectView` material, no blur behind
chrome in v1. Layered matte paint is cheaper, deterministic for rendering
tests, identical offscreen on CI, and stays in GPUI's ownership; vibrancy is
an open decision for Phase 9 polish.

## 3. Color tokens

### 3.1 Neutral scale and text

| Token | Light | Dark | IC Light | IC Dark | Role |
|---|---|---|---|---|---|
| `color:ink` | `#111315` | `#ECEEF1` | `#000000` | `#FFFFFF` | primary text, titles |
| `color:ink.secondary` | `#4A4F55` | `#A6ACB5` | `#292E34` | `#D6DAE0` | meta, icons at rest, secondary tab text |
| `color:ink.faint` | `#949AA2` | `#6E747D` | `#5C626A` | `#8A919B` | disabled, placeholders, purely decorative — never normal text |
| `color:hairline` | `#D0D0CC` | `#2C2F35` | `#9FA29E` | `#454C55` | structural rules |
| `color:border.strong` | `#C7C9C5` | `#3B4149` | `#9FA29E` | `#525A63` | field borders, tile outlines |

Contrast contract (recomputed by tests from tokens, not copied): `ink` ≥
14:1, `ink.secondary` ≥ 5.9:1, `accent-ink` ≥ 5.9:1 (light) / ≥ 9:1 (dark)
on their surfaces; status inks ≥ 4.5:1; `accent` rails/rings ≥ 3:1;
increased-contrast raises `ink.secondary` ≥ 9:1 and `hairline` ≥ 1.8:1.
On a **space-tinted** rail (D3) the bars are `ink` ≥ 12:1 (IC ≥ 14:1) and
`ink.secondary` ≥ 5.9:1 (IC ≥ 9:1); the selected row has its own D1 bars
(§3.5).

The tokens are verified by `bbr-ui` palette tests against the untinted
rail, and the composited veils (selection, footer, address field) against the
untinted and every tinted rail. Status error/attention were darkened during
reference calibration rather than weakening the 4.5:1 assertion. The v2
rail (`#E5E5E2` / `#1C1E22`) keeps every bar; the dark rail's slightly
lighter value is why the tinted-rail `ink` bar follows D3 from v2 S2 on.

### 3.2 Accent — Phosphor

| Token | Light | Dark | IC Light | IC Dark | Use |
|---|---|---|---|---|---|
| `color:accent` | `#0A7967` | `#2AD4B6` | `#0B6B5C` | `#3DF0CE` | selected rail, focus ring, drag tick, split preview frame (UI ≥ 3:1) |
| `color:accent.ink` | `#0A5C4E` | `#3DF0CE` | `#0A5A4C` | `#3DF0CE` | accent-colored text (match highlight, mode chip, links) |
| `color:on-accent` | `#FFFFFF` | `#151719` | `#FFFFFF` | `#151719` | text and marks painted *on* an accent fill (primary buttons, selection-box mark); equals `material:glass`. Never `accent.ink`, which is text on a neutral surface and fails §3.1 on an accent fill |

Usage rules: the accent marks **one thing at a time** — a focused control,
an active command result, drag target, progress/status or split target. Resting
selected tab/space rows use quiet surfaces + weight rather than a decorative
accent rail. The accent never gradients, glows or shifts with page content.

### 3.3 Status and indicator hues

Status colors appear only on small glyph/dot surfaces **with a non-color
companion** (glyph, text, or position); color never carries status alone.

| Token | Light | Dark | Non-color companion |
|---|---|---|---|
| `color:status.error` | `#B5261E` | `#FF6A63` | ⚠/lifering glyph + error surface tint + (crash) panel copy |
| `color:status.attention` | `#805100` | `#FFC63F` | pulsing dot (§6.3) + attention reason in status line |
| `color:status.ok` | `#177A55` | `#45C08A` | check glyph where used |
| `color:status.info` | `#2C5CC5` | `#8AA6FF` | ℹ glyph |

Surfaces: `status.*.surface` = status color at 10% over the owning material,
12% in increased contrast. The `error` **row** state and the `attention` dot
are the only two colored dot states; loading uses the accent, never amber.

### 3.4 Increase-contrast mode

Applies when macOS `Increase Contrast` is on — observed live through the
public `NSWorkspace` accessibility-display API and its change notification
(v2 S1) — and is assertable via the `contrast` snapshot field (§15). Changes: text steps per §3.1; hairline
strengthened; `accent` darkened/lightened per §3.2; the dark selection
becomes the opaque `#35383F` step (light keeps white + lift) and
`tint.accent` deepens (§3.5); focus ring becomes 3 px with a 1 px
material-gap double ring (§9); row washes deepen to 8 % / 12 %; the address
border strengthens (ink 10 % light, white 10 % dark; the accessory keeps
its 8 %),
fill white 8 % / hover 12 % dark); the crash/error and attention surfaces
gain a 1 px `border.strong` outline so tint is never required.

### 3.5 Selection tint and `tint.accent`

`color:tint.selected` (v2, D1) lifts the selected row **above** the rail:
light is opaque `#FFFFFF`, dark is a white 9 % veil over the (tinted) rail,
and dark increased contrast is the opaque `#35383F` step; light IC keeps
white. The surface carries `elev:lift` (§2) from the rail slice on. The
selected row's text bar is computed on the composited surface over the
untinted and every tinted rail: `ink` ≥ 10.5:1 (IC ≥ 11:1) and
`ink.secondary` ≥ 4.5:1 (IC ≥ 7:1), and the composited selection sits at
least 4 CIE L* above the rail in every mode. (v1's dark selection sat below
the rail because the old 14:1 bar capped it; D1 replaced that bar.)

`color:tint.accent` — light `#E6F1EE` / dark `#26403F`, IC `#D9EAE5` /
`#2B4A48` — is the single accent moment on white and overlay surfaces:
command, menu, Library and Space-configuration cursors and drop targets (tab
row, Space chip, Favorites band/cell, drag offers). Text on it is `ink` /
`accent.ink` with `ink` ≥ 9:1 and `accent.ink` ≥ 4.5:1 in every mode, and it
stays at least 4 L* off `material:overlay` and `material:field`. The exact
row identity, type weight and inset surface remain non-color selection cues;
the Phosphor focus/interaction accent is not used as decorative site tint.

## 4. Typography

- **Family:** system UI (SF Pro on macOS) via GPUI's font-kit; monospace
  `SF Mono` for URLs, ids, and keystrokes. No custom fonts in v1.
- **Numerals:** tabular figures (`tnum`) for counts, timestamps, and any
  right-aligned tabular data.
- **Size steps:** 11, 12, 13, 14, 16, 24 pt. Bold steps: 400/500/600/700.
  Calibrated from the preserved S1 reference (§8): normal-density title lands
  at 14 pt so the effective tab/space text reads 14–15 pt.
- **v2 weights (§18.3):** row titles stay `type:title` 500 in every state —
  the selected row is carried by the lift, not a heavier title; the
  active-space heading is `type:label` size at 600 in `ink.secondary`; the
  address domain is 500. GPUI text has no tracking control, so documented
  tracking values (`type:badge` +0.06 em, heading +0.01 em) are not rendered.

| Role | Size | Weight | Density S | N | XL | Notes |
|---|---|---|---|---|---|---|
| `type:display` | 24 | 600 | − | 24 | 26 | empty-state headline only, leading 1.2 |
| `type:title` | 14 | 500 | 12 | 14 | 16 | tab title, space name; leading 1.2 |
| `type:label` | 13 | 500 | 12 | 13 | 14 | buttons, nav labels, badges' base |
| `type:meta` | 12 | 400 | 11 | 12 | 13 | status, counts, timestamps, `tnum` |
| `type:mono` | 13 | 400 | 12 | 13 | 14 | URL in location mode and results |
| `type:key` | 12 | 500 | 11 | 12 | 13 | shortcut hints, `SF Mono` |
| `type:badge` | 11 | 600 | 11 | 11 | 12 | section labels "Pinned"/"Tabs", uppercase, tracking +0.06em |

### Truncation contract

- **Tab titles / space names:** one line, end-ellipsis; prefer cutting after
  the first `" – "`, `" - "`, or `" | "` separator when present so the
  site name survives; otherwise plain end-ellipsis. Hover shows the full title
  via native tooltip (Planned until a tooltip surface exists).
- **URLs (location mode, command results):** *mid-ellipsis* — always keep
  `scheme://host` and the final path segment; ellipsize the middle:
  `https://github.com/…/pull/1234`. Host is never truncated.
- **Favicon tile:** never truncates (fixed 18 pt; S 16 / XL 20, §7.2).
- **Dense mode:** identical truncation; nothing wraps to a second line, ever.
  Row heights stay uniform so scroll math and accessibility sizing stay
  deterministic (§11.6).

## 5. Spacing, radius, layout grid

- **Grid:** 4 pt base. All chrome spacing is a multiple of 4 within a row of
  padding; the density rows themselves use 2 px interstitial rules for
  mechanical look (§8).
- **Scale:** `sp:1` 4, `sp:2` 8, `sp:3` 12, `sp:4` 16, `sp:5` 20,
  `sp:6` 24, `sp:7` 32, `sp:8` 40.
- **Radius:**

| Token | Value | Used for |
|---|---|---|
| `radius:sm` | 6 | S rows, small tiles, profile chip, S favorites tiles, menu cursors |
| `radius:md` | 10 | N rows, Favorite tiles, buttons, menus |
| `radius:lg` | 12 | address field, palette field, panels, XL rows, crash panel |
| `radius:xl` | 16 | the ⌘T palette sheet only |
| `radius:full` | 50% | dots, avatars |

Rule (v2 §18.3): chrome radii stay in 6–12 px; `radius:xl` (16) is reserved
for the palette sheet. The content card keeps 10 (= `sp:content-inset`). No
giant rounded sheets over content and no `radius` on scrollable list
backgrounds. Inset row surfaces (§10.2–§10.4) and footer identity chips
(§11.3) are bounded rounded rectangles from this scale — never fully circular
pills around text; `density:radius` is S 6 / N 10 / XL 12.

- **`sp:content-inset` = 10** — the §2 card frame, the only value off the
  4 pt grid; it equals the card radius so the corner arc and the distance to
  the window edge are the same number. Identical at S, N and XL.
- **Sidebar widths:** `win:sidebar.S` 232, `.N` 300, `.XL` 340 pt; adjustable
  between 200 and 400 pt via a 4 px drag handle (cursor `ew-resize`, empty
  paint at rest), minimum width enforced; collapse (Cmd-B) yields 0 and the
  content card takes the freed width, gaining the leading inset and the
  titlebar-band top inset (§2 rule 5, §11.4). **Native-content floor stays
  strictly positive:** the native slot keeps ≥ 64 pt (`CONTENT_MIN`,
  2 × `sp:7`) on the card's inner box for every window that can host the
  floor — the `sp:content-inset` frame is charged to the slot, so the
  effective rail width is `min(requested, window_width − 64 − 10)`.
  At the shipped 1080 pt window the 300 pt normal rail leaves a 770 pt card;
  in windows too narrow to host the requested width plus the §13
  content floor, the rail yields below the minimum so the native slot stays
  strictly positive (never zero or negative).

## 6. Motion

### Principles

1. **Commit first.** Selection, tab switches, sidebar open/close/toggle, and
   split changes snap to their final geometry within the performance contract
   window (≤ 50 ms p95, next display frame). **No cross-fades, slides, or
   elastic settling on state changes.**
2. **Transient indication animates; persistent state snaps.** Hover,
   pressed, attention, loading, drag feedback may animate briefly *after* the
   fact they indicate is already true.
3. **Direction is meaningful.** Attention and loading never translate chrome;
   they pulse/luminesce in place so nothing scrolls or jumps.
4. **Reduced Motion** (macOS system setting, observed live since v2 S1 and
   reported as `reduced_motion`) removes all
   translation and scale; only opacity at `dur:fine` remains for attention.
   State changes stay 0 ms everywhere. No state requires motion to be
   understood — every animated cue has a static counterpart.

### Tokens

| Token | Value |
|---|---|
| `dur:instant` | 0 ms |
| `dur:micro` | 70 ms — hover/active tint, drag feedback |
| `dur:fine` | 120 ms — command palette open (opacity), result fade-in |
| `dur:standard` | 180 ms — attention pulse beat, panel swaps |
| `dur:emphasis` | 260 ms — attention two-beat sequence, peek entrance (opacity) |
| `ease:standard` | `cubic-bezier(0.22, 1, 0.36, 1)` |
| `ease:accelerate` | `cubic-bezier(0.36, 0, 1, 1)` |
| `ease:decelerate` | `cubic-bezier(0, 0, 0.3, 1)` (v2 D11; the former `0, 0, 0, 0.3` was a transposition that read as a steep, almost linear ramp) |
| `load:indeterminate` | 580 ms linear loop (§11.5) |

Animated states and their permitted motion are enumerated in §10 and §18.5;
anything not listed there is `dur:instant`.

**Implementation (v2 S6).** One helper, `bbr_ui::theme::motion`, owns the
easing curves (bezier-evaluated from the tokens above) and one descriptor per
transient moment with its Reduce Motion variant; every animated cue is built
from it, so timing and the reduced variant cannot drift per component. A
moment only decorates a state that is already committed and rendered on the
commit frame. Loops (loading arc, address sweep) exist only while their state
holds — the element leaves the tree with the state — and one-shots rest on
their final frame, so an idle shell builds no animation element and requests
no animation frames (asserted in E2E and the performance idle budget through
the typed `motion` snapshot).

**After-commit moments (v2 S7).** The favorite pop (§11.10), the download
check (§11.12), the ⌘T result entrance and cursor glide (§11.4) and the D4
space-tint fade (§11.3) are started only by the code path that has already
committed their state, keyed per start, bounded by their plan (≤ 260 ms) and
removed by a timer. One instance per kind: a newer commit supersedes a
running decoration, even when the newer start is static under Reduce Motion.
The `motion.started` evidence records every start with its variant.

## 7. Iconography

### Family

- **Style:** geometric instrument glyphs, 1.5 pt stroke (1.25 at S density),
  stroke joins rounded, glyph corners squared; no filled-duotone blocks, no
  gradients. Originals drawn for BBR; luminance-contrast matched to
  `ink.secondary`.
- **Sizes:** `icon:s` 16, `icon:m` 18, `icon:l` 20 — row glyphs pick the
  owning density's size (§8: S 16 / N 18 / XL 20). Hover affordances and
  close buttons: hit area ≥ `density:hit-min` (S 28 / N 32 / XL 40), close
  controls ≥ `density:hit-min.close` (S 24 / N 28 / XL 32), glyph optically
  centered in the hit square.
- **Catalog:** close, pin, lifering, clock, chassis mark, plus, downloads,
  check, stopped, folder (open/closed), disclosure, loading arc, chevrons,
  sidebar toggle, back, forward, reload, **bookmark** (the §11.16 Saved Page
  mark) and **pencil** (rename), plus the §11.3 identity glyphs. The clock
  draws two hands from its center. Identity glyphs are normalized by their
  own farthest point so every glyph fills the same optical box.
- **Fallback:** every icon context has a text alternative (label, tooltip, or
  aria-label). A missing functional glyph falls back to a 2-character
  `type:badge` chip, never to an arbitrary Unicode emoji. The deliberate
  exception is a space's user-selected identity in §11.3: that value is a
  validated member of the curated `SpaceIdentity` emoji/glyph catalog, not a
  fallback or substitute for a missing chrome icon.

### App icon

"Phosphor b" (§1, D8) on the macOS 824 pt icon grid: a superellipse tile
(n = 5, `#1C2127` → `#0B0D10`), a soft Phosphor halo at the bowl, an `ink`
(dark) stem, and the bowl as the §10.5 arc — 62 % of the turn with round caps,
`accent.ink` (dark) → `accent` (light), a faint 12 % white track where the arc
is not, and a soft glow. The mark stays legible at 16 pt (stem and bowl
resolve to distinct pixels; packaging tests assert it). Static in the Dock;
no animated Dock icon.

### Favicon tiles (missing favicon rule)

When a tab has no favicon yet, render a deterministic monogram tile instead of
an empty box:

- 18 pt (S: 16, XL: 20) square, `radius:sm`, letter = first character of the
  registrable hostname (uppercase, `type:badge` size), on a desaturated
  slate-hue ramp derived from FNV-1a(hostname) mod 6. Both modes use mid
  slates with a white letter (S12, Paper: light `#616A75`…`#707A87`, dark
  `#4A5662`…`#606B77`), so every tile holds ≥ 3:1 and reads as a favicon.
- The slate ramp is separate from accent/status/profile hues so favicon tiles
  never impersonate identity or status.
- While loading, the tile slot shows the `loading` spinner (§10.5); after
  load, the real favicon replaces the monogram. Monograms are static — no
  spin, no pulse, no animation.

### Favicon acquisition contract (real-favicon-pipeline)

Every favicon slot — tab rows, pinned rows, Favorites tiles, folder items —
presents the site's real favicon once acquired, with the deterministic
monogram above as the unchanged fallback:

- **Candidates:** after a navigation commit, the engine's isolated
  content-world collector reads the committed DOM's `<link rel~="icon">`
  candidates in document order; `/favicon.ico` derived from the committed
  URL is the final fallback when the page declares none. One best candidate
  is chosen by a deterministic portable rule: among candidates with a
  declared size, the one closest to 2× the density icon size (36 pt at
  normal), ties in document order; unsized candidates rank after every
  sized one.
- **Profile-isolated fetch:** the chosen candidate downloads through a
  session bound to the owning profile's data store (the profile's
  `WKHTTPCookieStore` seeds a private cookie storage), so the request
  carries exactly that profile's cookies and two profiles can never share a
  fetch, a cache entry, or a session. The resolved bytes land in the owning
  profile's bounded host-keyed in-memory cache (per profile, FIFO, no disk
  persistence); every presentation of that host derives from the same
  record — one acquisition, every presentation.
- **Bounded fallback:** downloads are bounded (payload size, deadline,
  concurrent count, FIFO queue). Any failure — no resolvable candidate,
  non-2xx response, over-bound payload, timeout, destroyed or re-registered
  view, decode failure — falls back silently to the §7.2 monogram: never a
  broken row, never a retry storm, never a per-frame decode. Loading/error/
  unresponsive precedence (§10.5/§10.7/§10.11) is unchanged over the tile.

## 8. Density — small / normal / very large collections

Three fixed densities, chosen by the shell from tab-collection size (and
user override later; open decision). Geometry is per-density; **states and
semantics are identical across densities.**

**S1 scale pass — settled from the preserved user reference (2026-08-08).**
Patrick's live U1 review found the complete chrome scale clearly too small,
and the supplied 2× references
(`/Users/patwoz/dev/Personal/macos/bbr-design-references/arc-sidebar-full.png`
and `arc-tab-row-hover.png`) calibrated the target feel: normal rows
approximately 36–40 pt, effective text approximately 14–15 pt,
favicons/glyphs approximately 18–20 pt, generous section spacing, large rounded
Favorite tiles, and inset rounded hover/active row surfaces with a trailing
close control. Patrick's later complete-shell reference is authoritative for
material and hierarchy as well as scale. The table below remains the central
density authority, with the normal rail widened to 300 pt and Favorite tiles
split into explicit width/height tokens by the
[reference-faithful shell packet](docs/roadmap/tasks/visual-shell-reference.md).
BBR keeps its own semantics, icons and native-web boundary. Dense-500
bounded-window rendering, scroll math,
semantic ids, interaction behavior, and every performance budget/sample count
are unchanged by this table (§15); the geometry below only raises token values.

| Token | S (small) | N (normal) | X (very large) |
|---|---|---|---|
| `density:row` | 32 | 36 | 44 |
| `density:gap` | 4 | 4 | 4 |
| `density:pitch` (row + gap) | 36 | 40 | 48 |
| `density:row-tile` | 28 | 36 | 44 |
| `density:radius` | `radius:sm` (6) | `radius:md` (8) | `radius:lg` (10) |
| `type:title` | 12 | 14 | 16 |
| `icon:` | 16 | 18 | 20 |
| `density:space-chip` | 32 | 36 | 44 |
| `density:sidebar` | 232 | 300 | 340 |
| `density:hit-min` | 28 (close 24) | 32 (close 28) | 40 (close 32) |
| `density:row-inset` | `sp:1` (4) | `sp:1` (4) | `sp:2` (8) |
| `density:section-gap` | `sp:3` (12) | `sp:4` (16) | `sp:5` (20) |
| Favorite tile width × height | 40×36 | 64×44 | 88×52 |
| favorites grid gap | `sp:1` (4) | `sp:1` (4) | `sp:2` (8) |
| pinned rows | icon-only 32×32, gap 4 | icon + title | icon + title |

Transition rule: density changes when the collection crosses 200 regular tabs
(S) or 600 (X) only after user confirm at first occurrence or a follow-up
preference (open decision); the shell never flips density mid-scroll in a way
that changes row height — row heights are stable for the whole session view.

**Row pitch (reference polish, settled 2026-08-13):** the normal rail ships
the reference's ~40 pt pitch — 36 pt row surfaces separated by 4 pt
inter-row gaps. `density:gap` is the fixed 4 pt inter-row gap at every
density and `density:pitch` = `density:row` + `density:gap` is the exact
step every regular/pinned/folder/nested/New Tab row renders on and the
single unit the dense scroll math, the drag-slot geometry, and the testkit
snapshot use (§11.6). Hit minima, row surfaces, and the state vocabulary are
unchanged; only the inter-row pitch grew.

## 9. Focus

- **Ring:** `color:accent` 2 pt outline inside the focused control's bounds,
  radius `+1 pt`, painted as GPUI outline (no blur). In increased contrast:
  3 pt `accent.ink` with a 1 pt `material` gap between control and ring
  (double ring) so the ring reads on any surface.
- **Never color-only:** the focused element also brightens by one text step
  (`ink.faint → ink.secondary → ink`) and, where it has one, its label
  becomes `ink`.
- **Focus domains (interaction contract):**

| Domain | Chrome visual state |
|---|---|
| Chrome focus | ring on the focused control; command field shows cursor; hover washes active |
| Web-content focus | chrome renders "at ease": no ring anywhere, command cursor hidden, row hover washes active but no accent tint on the field; selected rows retain their quiet surface + weight with no decorative accent |

- **Order:** command field → favorites grid (row-major) → pinned → folders
  (expanded rows, then their nested rows) → regular tabs → the planned media
  card controls when an eligible session exists (§11.15) → collapse control →
  Downloads → space identity chips in footer order → Plus → (after the last
  chrome control) focus moves into the native web view; Shift-Tab from web
  content returns to the first sidebar control. Section labels, brand, and the
  non-interactive space-strip viewport are skipped. An absent media card adds
  no focus stop. Both directions are verified by real-app focus E2E (a11y
  track).
- **Focus after tab mutation:** governed by the browser interaction contract;
  the visual system only guarantees the ring lands on the correct control in
  the same presentation frame as the selection change.

## 10. Reusable states

Each state lists its static visuals (light / dark), permitted motion, and the
assert token used by E2E (§15). States are composable: `selected+loading`,
`hover+drag-target`, etc., with the more specific state winning the visual.

### 10.1 `default`
Row surface paints nothing, so the (space-tinted) rail shows through; text
per role. **Assert:** `visual_state == default`.

### 10.2 `hover` (pointer over control)
Wash (v2 §18.2): a translucent veil over the (tinted) rail — light ink 5 %,
dark white 5 %; increased contrast 8 % in both modes. The wash paints an inset
rounded row surface — inset `density:row-inset` from the rail edges (S `sp:1`
4 / N `sp:1` 4 / XL `sp:2` 8, §8), radius `density:radius` (S `radius:sm` 6 /
N `radius:md` 10 / XL `radius:lg` 12) — rather than a full-bleed band, so the
trailing close control sits inside that surface. Hover never lifts. Close button reveals on tab rows (fade-in
opacity `dur:micro`, allowed). Tint animates `dur:micro`. **Assert:** `hover`.

### 10.3 `active` (pointer down / pressed)
Wash deepens (light ink 9 %, dark white 8 %; IC 12 %) plus 1 px inner
`hairline` on the same inset rounded surface as §10.2 (`density:row-inset`, `density:radius`); `dur:micro`
tint. No translation or "press down" motion. **Assert:** `active`.

### 10.4 `selected`
`tint.selected` (§3.5) paints the inset rounded row surface of §10.2
(`density:row-inset`, `density:radius`) with the static `elev:lift` (D2) — the
only row surface that shadows. The title keeps `type:title` 500 in `ink` (the
lift carries selection, v2 §18.3); meta stays `ink.secondary`. A selected
folder member lifts the same way on its nested row. No decorative Phosphor
rail appears at rest; keyboard focus, drag targets and active command
surfaces retain the accent. Snap (`dur:instant`). **Assert:** `selected`
(plus the domain via `chrome_focused`) and the typed `lifted_ids` evidence —
exactly the selected tab's row ids plus the active space chip.

### 10.5 `loading`
Favicon slot swaps to an 18 pt circular arc spinner (S 16 / XL 20, accent)
rotating at 640 ms per turn, linear (static arc under Reduce Motion); tab
title drops to `ink.secondary`. Page progress is the D5 line in the address
field (§11.5), never a bar on the content pane — GPUI does not paint above
the native web view. Selection/hover visuals unchanged. **Assert:**
`loading`, plus the `loading-arc` moment in the `motion` snapshot.

### 10.6 `attention`
Amber dot (8 pt at the row's trailing state slot) playing one two-beat pulse
when the row enters the state (§18.5: two `dur:fine` beats, dot scale
1→1.45→1 plus one ring 4→16 pt fading out; Reduce Motion: opacity-only
1→0.45→1), then resting on the static dot. The former one-frame amber surface
flash is withdrawn (§18.5). Non-color companions: the badge
only appears with a reason line in the status area, and the row keeps its
title in `ink`. Used for e.g. a tab update the user flagged, a
requested-but-unfocused action, or a failed background open — never for
errors (those use `error`) and it must never beat longer than 520 ms.
**Assert:** `attention`.

### 10.7 `error`
Row: `status.error` glyph replaces the favicon (lifering), title stays
`ink`, tile outline `border.strong` (IC: `status.error.surface` + outline).
Crash panels use the dedicated §11.8 surface. **Assert:** `error`.

### 10.8 `disabled`
Glyph + label at `ink.faint`, wash suppressed, no hover, default arrow
cursor, tooltip explains the reason ("No back history"). Contrast of
disabled UI is exempt from AA but must remain perceivable at IC.
**Assert:** `disabled`.

### 10.9 `dragging` / `drag-target`
A row press **arms** a drag; the dragging presentation (dragged-row state,
insertion tick, drag offers, drag proxy, chip targets) begins only after the
pointer travels the 4 pt activation distance. Below it the press is a
click — nothing shifts the rail layout on a plain click.
`dragging`: the dragged row renders at 92% opacity with `elev:2` shadow and
a 1 pt `border.strong` outline (`dur:micro` opacity, allowed); it stays in
the list as the placeholder while the **drag proxy** — the lifted row
(favicon + title, same 92% / `elev:2` / `border.strong` language) — follows
the pointer with the press point kept under it.
`drag-target`: insertion tick = 2 pt vertical accent rule at the target
index spanning the row tile, plus target tile outline 1 pt accent and
`tint.accent` (§18.2 drop target). Cross-profile destinations additionally show the
destination profile chip replacing the insertion rule (§12.3). Split zones
reuse the accent outline language with the strip fills of §11.7.

**Drag offers.** At activation the rail offers the destinations a tab can
be dropped into without a precise position: the Favorites band or drop cell
(§11.10) and, for an unpinned tab, the Pinned ghost row (§11.2). They appear
together and disappear together on drop, cancel or Escape. The growth they
need is committed **once, at activation** (`activation_growth`): the ghost
row adds one `density:pitch` above the regular rows, a full last Favorites
row adds one tile row; the drag resolver subtracts that growth from the
pointer travel, so slot targets stay on the rows the render shows and
nothing reflows while the pointer moves. The activation event itself keeps
the source slot (a drop there is a no-op): the offers render in the next
frame, and resolving against the pre-growth geometry could commit a drop
the user never saw.

**One live target at a time.** Offers in different regions are visible at
the same time because each names a different destination, but only the one
under the pointer takes the `drag-target` language; the others stay in
their offered (dashed) state. Region offers are hit through the rects their
anchors recorded in the last frame — never a parallel layout model.

**Assert:** `dragging`, `drag-target`, `bbr-tab-drag-proxy` (present only
during an active drag), `bbr-drag-tick-<n>` (present only when a concrete
index is resolved; the design branch's `bbr-drag-insertion-tick` is this
indexed id).

### 10.10 `focused` (keyboard)
Ring per §9, label brightens one step. Same presentation frame as the
selection change (snap). **Assert:** `focused`.

### 10.11 `unresponsive`
A renderer that misses the bounded liveness deadline keeps its tab identity
and live session but enters a state distinct from `loading` and `error`.
The row keeps the lifering error-shape companion plus a clock glyph in the
trailing state slot; title remains `ink`, no pulsing or movement is allowed,
and the active slot is replaced by §11.8a. A completing probe clears the state
immediately; a crash takes precedence and uses §10.7/§11.8. **Assert:**
`unresponsive`.

## 11. Component behavior specifications

Each spec is precise enough to implement and to assert with real-app E2E
using the IDs of §15. All rows use `density` geometry from §8.

### 11.1 Vertical tab row

- **Anatomy:** `[favicon 18] · [title flex, trunc] · [state slot 10] ·
  [close 18, hover-reveal]` (S density: favicon/close 16, state slot 8;
  XL: favicon/close 20), padding-x `sp:2` (S) / `sp:3` (N) / `sp:4` (XL),
  tile per §8.
- **Behavior:** click selects (pointer keeps the focus domain per contract);
  middle-click closes (Planned affordance until wired, open decision on
  background semantics); close button appears on row `hover` or when the
  close control has keyboard focus; keyboard: `↑/↓` move selection, `⌫` or
  `⌘W` close, Enter activates.
- **States:** all §10 states apply; `selected` lifts (§10.4);
  `loading` spinner replaces the favicon; `attention` dot in the state slot
  (pinned rows include the state slot except at S).
- **Long title:** §4 truncation; tooltip on hover (Planned).
- **Assert ids:** `bbr-tab-row-<n>`, `bbr-tab-title-<n>`,
  `bbr-tab-favicon-<n>`, `bbr-tab-close-<n>`, `bbr-tab-state-<n>`.

### 11.2 Pinned tab

- **Anatomy:** loose pinned rows and folder rows (§11.11) are two
  presentations inside one *organized area*. A folder is the grouped form of
  pinned tabs; a loose pinned row is the ungrouped form. They share one
  uninterrupted `density:pitch` rhythm with no section gap, label, or
  hairline between them; exactly one `bbr-organized-hairline` closes the area
  against New Tab (§11.4). Loose pinned rows remain above folders until a
  separate ordering-model decision says otherwise. S density: icon-only
  32×32 loose rows with 4 pt gap. N/XL: same row anatomy as regular rows.
- **Behavior:** pinned rows never shift order on selection; close is
  hover-reveal only; drag within the pinned region reorders; dragging to the
  regular region unpins (contract); a drag from the regular region into
  pinned pins it through the ghost row below.
- **Drag offer (`bbr-pinned-drop-row`):** pinning is only discoverable by
  dragging, so once a drag of an **unpinned** tab passes the 4 pt activation
  distance (§10.9) the organized area appends one ghost row after its loose
  rows and folders: `density:row` surface plus the row gap (one pitch), a
  dashed `accent` outline on the §10.2 inset row surface, no fill, a leading
  plus glyph and `type:meta` “Pin tab here” in `accent.ink`. As the live
  target the outline turns solid over `tint.accent`. A drop pins the tab
  at the end of the pinned block through the one pin authority (same
  `TabId`, same session). It is an offer, not a precision target, and it
  appears in an empty organized area too — the only way a user without
  pinned tabs learns the region exists; at rest an empty organized area
  still consumes no space. A pinned tab is never offered the row.
- **States:** same vocabulary; pinned rows additionally read a "locked
  position" identity: the favicon tile is never replaced by scroll chrome,
  and at S density the tile gets a subtle 1 pt `hairline` outline so
  icon-only rows are legible.
- **Assert ids:** `bbr-tab-pinned-<n>` (shared sub-ids `-title/-favicon/
  -close`), and `bbr-pinned-drop-row` while it is offered.

### 11.3 Space identity chips and profile context

- **Placement:** spaces are always available in the §11.4 footer strip. There
  is no revealable space-row region above Favorites and no `[Spaces]` toggle.
- **Anatomy:** each space is one keyboard-focusable **icon-only** identity chip
  containing exactly one validated `SpaceIdentity` value from the curated
  appearance contract: a selected emoji renders directly; a curated glyph
  name renders its drawn §7 geometric path (the catalog ships; a non-curated
  value — nothing in the curated catalog — falls back to the §7 two-letter
  badge, never arbitrary Unicode). No Space title text appears in the footer at rest. The chip has a stable accessible
  name containing the space and profile names; color or emoji alone never
  carries that meaning. Normal S1 geometry is calibrated with the same generous
  hit-target family as the rest of the footer.
- **Behavior:** pointer click or keyboard activation performs the existing warm
  space switch. The active space is unmistakable through a persistent selected
  surface plus stronger identity weight: v2 paints the active chip as the
  `tint.selected` surface over the tinted rail carrying 8 % of the identity
  hue, with the static `elev:lift` (D2); keyboard focus supplies the accent
  outline and inactive chips paint no surface of their own.
  Activation keeps the selected tab/session warm and preserves every existing
  50/100 ms budget and fast-but-wrong guard.
- **Appearance:** each space owns one curated theme-color token, one identity,
  and a curated paper: one of three tint intensity steps and a static-grain
  flag (D6/D7; defaults `standard`, grain off).
  The active token tints the BBR-owned sidebar material (the whole chassis
  body) by blending a curated chrome-wash hue into it and pinning the result
  to the untinted rail's relative luminance (S12, Paper model): each step
  adds hue, never lightness, so every luminance bar below holds exactly as
  on the untinted rail, and the neutral Ink token reads as the untinted rail.
  The steps are chroma shares — light `subtle` 18 %, `standard` 36 %,
  `rich` 54 %; dark 12 / 24 / 36 % (the same share reads louder on a dark
  rail); increased contrast adds 4 % to each. Light cobalt `standard` is
  `#E0E5F4` (Paper `#DFE4F3`). The grain is one static
  128 px noise tile (one texel per device pixel, anchored to the window
  origin) painted on the tint beneath every child of the rail and the
  chassis — never over the card, web content or control text, never
  animated. Its specks are white on light chrome (≤ 28 %) and black on dark
  chrome (≤ 36 %), about half the sheet clear, so every speck only moves
  the paper away from its text and no text bar can drop. Every composited
  bar holds on every tinted rail and step: `ink` ≥ 12:1 (IC 14:1),
  `ink.secondary` and light `accent.ink` ≥ 5.9:1 (IC 9:1 for
  `ink.secondary`), the D1 selection (`ink` ≥ 10.5:1). Dark washes are the
  identity hue at 1.3× the v1 saturation; light washes are the identity hue
  at HSV value 1 and saturation ≤ 0.45 (Slate half-saturated, Ink neutral
  grey). Web
  content is never sampled or tinted. During a
  direct horizontal swipe over the sidebar/space strip, the sidebar tint may
  interpolate between source and candidate target tokens in direct proportion
  to gesture progress; cancellation returns to the source token and commit
  selects the target. The native web view itself never cross-fades or
  interpolates. A committed non-swipe switch (pointer, keyboard, command)
  paints the destination tint on the commit frame and cross-fades only the
  chrome tint (rail and chassis) from the source over `dur:standard` with
  `ease:standard` (D4); a swipe commit is already at the destination and
  never fades; Reduce Motion switches instantly.
- **Overflow and ordering:** chips follow the canonical space-list order in one
  single-line horizontal scroll viewport. The viewport consumes all footer
  width left after Downloads and before Plus; it scrolls without wrapping or
  shrinking the fixed controls. The selected chip is brought fully into view
  without changing order.
- **Cross-context rule:** a cross-profile activation still renders the
  destination profile chip at the top of the command field for one interaction
  (`type:label` "in Profile · Work"). During drag-to-space, the targeted identity
  chip gains the existing drag-target treatment plus a visible destination
  profile companion so isolation is never communicated by color alone.
- **Assert ids:** `bbr-space-strip`, `bbr-space-chip-<n>`,
  `bbr-space-identity-<n>`, and the existing transient
  `bbr-command-profile-hint` / `bbr-profile-chip-<n>` where profile context is
  presented.

#### Configuration surface

- **Lifecycle:** creating a Space commits + activates it before any
  configuration; the optional rename/identity surface then opens over the
  usable Space (both the plus-menu New Space and the keyboard Cmd-Shift-N
  paths share the same create→activate→configure authority). Escape/Cancel
  closes the surface without applying the draft — the created Space and its
  deterministic defaults stay.
- **Surface:** one BBR-owned in-rail panel above the footer row
  (`bbr-space-config`) with a header (`bbr-space-config-title`), a rename
  field (`bbr-space-config-name`), the curated theme picker
  (`bbr-space-config-theme-<name>`), the paper row
  (`bbr-space-config-paper-label`, one `bbr-space-config-intensity-<step>` per
  step — each previewing its rail at the draft theme — and the
  `bbr-space-config-grain` toggle), the curated identity picker (emoji
  `bbr-space-config-emoji-<index>` and BBR-glyph `bbr-space-config-glyph-<name>`),
  and Done/Cancel (`bbr-space-config-done` / `bbr-space-config-cancel`).
  Only curated emoji/BBR-glyph values, curated theme tokens and the three
  intensity steps exist — no free-form icon, colour field or strength. Keyboard: printable chars edit the draft,
  Backspace deletes, Enter commits, Escape cancels.
- **Exact-SpaceId contract:** the surface binds the exact persistent SpaceId
  at open and never re-derives it; every commit targets that id through the
  controller (which rejects unknown spaces whole), so a stale surface can
  never mutate another Space after a switch.
- **Persistence (schema v4):** the committed rename and the whole curated
  appearance — theme, identity, intensity and grain — are durable. The
  browser-state write stores each space's effective appearance in its row;
  restore applies it after the structural restore and keeps an explicit
  record only where it differs from the derived default. A store stamped
  with an older schema is discarded under the closed version-bump policy.

### 11.4 Sidebar

- **Anatomy (top → bottom):** the full-height light rail is painted directly
  on the chassis and carries **no right hairline** — the §2 content card's
  edge is the boundary. It starts behind the native traffic-light safe area; the implemented §11.14 titlebar row owns
  Sidebar toggle + Back/Forward/Reload, followed by the compact address
  display (§11.5). There is **no standalone BBR app-mark/name brand block**.
  The Favorites grid (§11.10) sits directly under the address display, exactly
  as in the reference composition. The active Space's heading
  (`bbr-active-space-title`) is then the primary list heading for the
  scrolling list: `[identity bbr-active-space-identity] [exact title]`, where
  the identity is the **same validated value the §11.3 footer chip renders**
  (emoji, drawn §7 glyph, or two-letter badge — one space never shows two
  marks) at `icon:m` on the tab rows' favicon column, so the title starts on
  the same x as every tab title below it. The heading text is `type:label`
  size (N 13 pt) at 600 in `ink.secondary` (v2 §18.3; the +0.01 em tracking
  is not expressible in GPUI and is omitted). The heading is followed by the organized area
  of loose Pinned rows (§11.2) and grouped Folder rows (§11.11), then a visible
  New Tab row routed through the existing
  staged-new-tab authority, and regular Tabs (§11.1). The Favorites region is
  omitted entirely when a Space has no favorites; nothing is fabricated to keep
  the region present. Settled redundant status copy consumes no row; loading,
  crash/error and unresponsive status remains visible. The footer is exactly
  `[Downloads icon] [horizontally scrollable Spaces strip, flex:1] [Plus icon]`
  (`bbr-downloads`, `bbr-space-strip`, `bbr-plus`) directly on the rail — no
  band and no rule (S12, Paper). The
  former `[Spaces]` reveal-toggle and upper space-row region do not exist.
- **Regions:** Favorites, Pinned, Tabs and Folder wrappers retain stable ids
  and truthful empty-region behavior, but the production rail does not repeat
  visible `Favorites`/`Pinned`/`Tabs` labels; the active Space title supplies
  the primary hierarchy. Folder names remain visible content, not a generic
  section heading. Profile groups and regions stack under quiet separation
  with `density:section-gap` breathing (S `sp:3` / N `sp:4` / XL `sp:5`), but
  the gap never separates loose pinned rows from their grouped folder form.
  The Favorites grid and loose pinned rows are fixed-height blocks above the
  scroll view; Folders and Tabs scroll together in `bbr-scroll` (label
  behavior per §11.6). One
  1 px hairline (`bbr-organized-hairline`) separates the organized
  Pinned/Folders area from the visible New Tab row (reference composition):
  it renders exactly while the organized area has content (pinned rows or a
  folders list) and is truthfully absent when both are empty.
- **Spaces strip:** `bbr-space-strip` is the permanent space navigation
  surface defined by §11.3. It takes all remaining footer width, clips to one
  line, scrolls horizontally, and exposes one `bbr-space-chip-<n>` per space.
  The chips start at the strip's leading edge right after Downloads, whether
  they fit or overflow, so scrolling never hides a chip behind the scroll
  origin and the strip reads as one row with Downloads (Blueprint D01; the
  v2 §18.3 centring is withdrawn). It is
  not a disclosure control and never changes the vertical region stack.
  Pointer click, keyboard activation, and horizontal trackpad swipe route
  through the same space-switch authority and preserve warm-session budgets.
- **Footer:** the glyph-only `[Downloads]` control opens the downloads surface
  and exposes the full text alternative plus a `type:meta` `tnum` count when
  content exists. The middle flex region is the permanent icon/badge-only
  Spaces strip; `[+]` is likewise glyph-only. Both fixed glyphs render at
  16 pt in every density (v2 §18.3); inactive chips paint no surface of their
  own, so the footer veil and the rail tint show through.
  `[+]` opens the **plus menu** (Planned) — a real context menu
  (`bbr-plus-menu`, `elev:2`, `material:overlay`, `radius:md`) with exactly
  two items in v1: **New Space** (`bbr-plus-new-space`) and **New Folder**
  (`bbr-plus-new-folder`). New Space creates a real space through the
  controller; New Folder creates a session-local folder that opens in
  inline rename (§11.11). The menu is keyboard-operable (`↑`/`↓`, Enter
  commits, Escape dismisses without action), closes on outside pointer
  press (one pointer action, §13), and New Space preserves the invoking
  focus domain per the interaction contract (New Folder hands chrome focus
  to the rename, §11.11).
- **Behavior:** width per §5 (`win:sidebar.*`), draggable 4 pt hairline
  (`bbr-sidebar-divider`), collapse Cmd-B to 0 **instantly** (performance
  contract next-frame; the opening glass gets no animation), expand
  restores the previous width. On collapse the content card keeps its
  trailing and bottom inset, gains the leading inset, and its top inset grows
  to the titlebar band (§2 rule 5); inset, radius and shadow land in the same
  frame as the width. Collapse memory is **global in v1** — one
  last-expanded width per window, restored on expand; per-space memory is
  an open decision (§17). Divider keyboard focus provides `←`/`→` resize
  and `0` reset (DESIGN §9 pointer/keyboard parity). Resize commits
  geometry on the next frame; drag previews no intermediate chrome states.
- **Command surface launch:** the command field owns two presentations of
  the same §11.5 engine:
  - **Cmd-T → centered palette** (`bbr-palette`): `elev:2`
    overlay centred horizontally over the seated content card, 110 pt below
    its top edge (S12, Paper), `material:overlay`, width
    `win:palette` (D9: S 508 / N 600 / XL 716 pt), `radius:xl` (16) at every
    density. Field row on top (`bbr-palette-field`, 56 pt, a hairline below):
    the drawn search magnifier, the query at 17 pt in the UI sans
    (`bbr-palette-text`) with the caret (`bbr-palette-cursor`, 2 pt `accent`
    at the query's text height) at the insertion point — after the query,
    before the placeholder while empty — and the mode chip
    (`bbr-palette-mode`, an uppercase `type:badge` pill on `tint.accent` in
    `accent.ink` — New Tab staged per the interaction contract). Results
    below (`bbr-palette-result-<k>`, 40 pt rows at `radius:md` inside an
    8 pt padded list) use the §11.5 row anatomy `[badge] [title]
    [url mono trunc] [kind hint]`: an 18 pt host-monogram badge (§7.2; the
    drawn bookmark for a Saved Page, the group letter without a URL), the
    title with the query's match in `accent.ink`, the shortened URL
    (`host/…/last`) unless a cross-space identity or shortcut subtitle
    takes the column, and a trailing hint — `↵` on the selected row, the
    kind (`Tab`, `History`, `Saved`, `Space`, `Action`, `URL`, `Search`)
    otherwise; the `selected` result is the inset `tint.accent` row
    surface (no accent rail, §18.4). A freshly published result set fades in with a
    20 ms stagger and a 4 pt rise (rows past the seventh share the last
    slot, ≤ 260 ms); Up/Down commits the selection at once and the cursor
    highlight glides to it in `dur:micro` (Reduce Motion: fade only, the
    cursor snaps). The ⌘L overlay renders at rest.
  - **Cmd-L / pointer click on the address control → palette over the
    address** (Planned, `bbr-command-overlay`): the resting control uses the
    reference-calibrated 4 pt rail inset; `elev:2` overlay that
    **covers** the §11.5 address control rather than dropping below it —
    top edge flush with the control's top edge, leading edge flush with
    the control's leading edge, width = control width clamped to
    440 pt–`win:palette` (D9), `material:overlay`, `radius:lg` (12), a 40 pt
    field keeping the `type:mono` size, and 36 pt result rows at
    `radius:md` inside a 4 pt padded list. The address
    control is not visible while the overlay is open: the overlay takes
    its place, so the address never appears twice and the eye stays where
    the pointer clicked. Same result engine and row anatomy; results keep
    the §11.5 ids (`bbr-command-result-<k>`). The overlay's own field
    (`bbr-command-overlay-field` / `-text` / `-cursor`) opens with the
    **full current URL selected**, so one keystroke replaces it.
  - Both open with an opacity fade `dur:fine` only — never translate or
    scale — close on Escape/commit/outside press (one pointer action,
    §13), and are the only chrome allowed to overlap the glass (§13). At
    most one command surface is open at a time; opening either clears the
    §11.3 profile hint (interaction contract). Result selection snaps
    (`dur:instant`).
- **500-tab rule:** the sidebar remains responsive with 500 persisted tabs
  (§11.6); sidebar open/close, space-strip scrolling/switching, direct swipe,
  and palette surfaces submit to their existing latency budgets. Larger S1
  geometry never changes bounded-window rendering or dense scroll math.
- **Assert ids:** `bbr-sidebar`, `bbr-sidebar-column`, `bbr-active-space-title`,
  `bbr-active-space-identity`,
  `bbr-new-tab`, `bbr-scroll`, conditional `bbr-status`,
  `bbr-titlebar-sidebar-toggle`, `bbr-sidebar-divider`, `bbr-organized-hairline`,
  `bbr-section-favorites` / `bbr-section-pinned` / `bbr-section-folders` /
  `bbr-section-tabs`, `bbr-favorites`, `bbr-fav-<n>` (+ `-icon`),
  `bbr-folders`, `bbr-folder-row-<n>` (+ `-name`/`-count`/`-toggle`),
  `bbr-folder-item-<n>-<m>` (+ `-title`/`-favicon`/`-close`),
  `bbr-downloads`, `bbr-space-strip`, `bbr-space-chip-<n>`,
  `bbr-space-identity-<n>`, `bbr-plus`, `bbr-plus-menu`,
  `bbr-plus-new-space`, `bbr-plus-new-folder`, `bbr-palette`
  (+ `-field`/`-text`/`-mode`/`-cursor`), `bbr-palette-result-<k>`,
  `bbr-command-overlay` (+ `-field`/`-text`/`-cursor`), `bbr-address`
  (+ `-domain`/`-accessory`).

### 11.5 Address control

The sidebar address is a **display control, not a text field**. BBR has
exactly one real URL input — the one inside the palette (§11.4). Editing an
address in two places invites two editing states, two focus rings and two
definitions of what the current address is; this section removes that class
of problem instead of styling it.

- **Content:** the **domain only** — host without scheme, path or query, a
  leading `www.` elided, IDN shown human-readable. The full address exists
  in the palette, never here. Internal pages show their short name (the
  exact naming is open, §17.13).
- **Anatomy:** a quiet field surface, height `density:row` (S 32 / N 36 /
  XL 44), `radius:lg` (12), `type:title` domain text (`bbr-address-domain`,
  mid-ellipsis) at 500 in `color:ink`, and a **trailing accessory slot**
  (`bbr-address-accessory`) inset at the field's trailing edge. The slot
  reserves one §14 glyph at `density:hit-min`; content in it never pushes
  the domain out of the field (the domain yields by ellipsis).
- **Fill:** a bordered field one step off the rail — a **white veil over the
  rail material** (v2 §18.2: light white 55 %, dark white 6 %), so the field
  follows the theme and the space tint, plus a 1 px presence edge (light
  ink 4 %, dark white 4 %; increased contrast deepens it to 10 %) and **no
  shadow**. The non-address staging surface (`bbr-command-bar`) uses the
  same veil at rest. Deliberately not `material:field` white: the address is the one
  address affordance, not the loudest element in a rail whose job is to
  recede. Colors come from the palette tokens, never the component.
- **Zoom accessory (§17.14):** the slot shows the page zoom chip
  (`bbr-address-zoom`, e.g. `110%`, `type:meta` at 500 in `ink`) only while
  the active site's zoom is not 100 %, on an inset wash (light ink 8 %, dark
  white 8 %; S12 Paper). Pressing it resets
  the site to 100 % through the site-zoom authority and never opens the
  palette. Copy Link and permission indicators stay future accessories.
- **Load progress (D5):** while the active tab loads, a 2 pt `accent` line
  on the field's inner bottom edge (inset by the corner radius) sweeps
  (`load:indeterminate`); when the load ends it fills (`dur:standard`) and
  fades (`dur:standard`), then leaves the tree. Reduce Motion: a static line
  while loading, removed on finish. Id `bbr-address-progress` while shown.
- **Hover:** the veil moves one step darker on light chrome (white 40 %) and
  one step brighter on dark chrome (white 10 %), using the §10.2 wash mechanics at `dur:fine`; the
  border never changes weight or color.
- **Activation:** pointer press or `Cmd-L` opens the §11.4 palette over the
  control. The control **never accepts keyboard text**; its semantic role is
  a button that opens address entry, and it holds no editing state, no
  cursor and no selection.
- **Focus:** the control participates in the chrome focus domain as a
  button; the field look communicates what the control is about, never that
  it is being edited (no caret, no selection). After the palette closes, focus returns where it was before opening — web
  content or the sidebar — per the interaction contract.
- **Assert ids:** `bbr-address`, `bbr-address-domain`,
  `bbr-address-accessory`, `bbr-address-zoom` and `bbr-address-progress` (only while shown); the palette's own input carries
  `bbr-command-overlay-field` / `-text` / `-cursor`, and results keep
  `bbr-command-result-<k>`.

### 11.6 Dense list (500 tabs)

- Uniform row heights per density (scroll math stays exact); render only the
  visible range plus 8-row overscan; section labels (Pinned/Tabs) scroll
  with content but the pinned region itself is a fixed-height block above the
  scroll view (no sticky-scroll behavior in v1).
- Selection visibility: after any selection/space activation, the selected
  row is scrolled into view **instantly** (no animated scroll) within the
  same frame.
- Scrollbar: 8 pt track-less thumb at `ink.faint` 40% (60% hover); keyboard
  scroll `↑/↓/PgUp/PgDn` move both selection and viewport.
- Identity in density: rows never collapse headers, never change height, and
  never drop their state slot (only S-density pinned rows omit titles).

### 11.7 Split preview

- **Invocation:** dragging a tab toward the glass area; the drop zone strips
  are the edges (top / bottom / left / right) at 24% of the pane dimension,
  center = positional move. (V1 supports exactly one two-pane split;
  keyboard split creation is a Phase-6 open decision.)
- **Preview rendering:** the target zone gets a 2 pt `accent` inner frame at
  `radius:lg` + 12% accent fill + a corner glyph ("split"); the rest of the
  glass shows no overlay. `drag-target` remains at the row list for the
  positional tick. Preview snaps; no animation.
- **Commit:** opening the split keeps the pre-drag selected tab as the
  focused pane and shows the dropped tab in the second pane (contract);
  both pane slots render chrome frame hairlines on the shared edge only
  (2 pt divider with `hairline` etch), and the focused pane's chrome frame
  shows a quiet 2 pt inset outline in `hairline` → `accent` only when it
  receives command focus (Pane-6 polish).
- **Assert ids:** `bbr-split-zone-<top|right|bottom|left>`,
  `bbr-pane-<0|1>`, `bbr-split-divider`.

### 11.8 Empty and crash states (glass panes)

- **Empty space** (no tabs, no staged tab): the glass area shows
  `material:canvas` with the D10 illustration (drawn, not emoji; S12
  follows the Paper `empty_space` poster on a 400 × 240 pt board): six tab
  cards (`material:overlay`, radius 11, `elev:2`, an 18 pt favicon square
  in a curated space hue — Cobalt, Terracotta, Pine, Amber, Plum — or
  `ink`, and a `border.strong` title bar) floating around an 80 × 60 pt ⌘
  key cap (radius 14, `material:canvas` face with an 8 % `ink` edge on a
  `hairline` depth step, the ⌘ drawn as a 2 pt path). The cards stay
  axis-aligned (GPUI divs do not rotate; the poster's ±4° tilt is
  omitted). Headline `type:display` "No tabs in this space", a line naming the
  profile ("in Profile · Work"), and one primary button "New Tab  ⌘T" that
  focuses the command field in NewTab mode; Escape returns to the glass pane.
  Closing the final tab already opens the empty New-Tab command surface
  (contract); this pane is what the user returns to after Escape.
  **Assert:** `bbr-empty-new-tab`, `bbr-crash-*` absent.
- **Crash panel:** replaces the glass area of the crashed tab (native view
  hidden) with `material:field`, `elev:1`: the D10 heartbeat illustration
  (Paper `empty_crash`: a 400 × 90 pt band; a 2.4 pt line whose beat — one
  damped 38 pt cycle, 26 pt wide — travels just behind an `accent` head dot
  with a 15 % halo; the trail is `accent` fading over 260 pt behind the
  head, the rest of the line `ink.tertiary` at 12 %; the crashed row's
  lifering stays the status companion), title "This page crashed", copy "The tab stays open; reload
  to restore it.", buttons `[Reload]` (primary, accent) and `[Close tab]`
  (secondary); focus order: Reload → Close tab → sidebar. The tab remains
  selected and listed per the interaction contract; no auto-reload in v1
  (open decision). **Assert:** `bbr-crash-panel`, `bbr-crash-reload`,
  `bbr-crash-close`; crashed tab row carries `error` state.
- **Offline load error:** the error surface of the network (offline)
  class replaces its lifering with the D10 illustration (Paper
  `empty_offline`, 310 × 58 pt) — a laptop and a globe stroked in `ink`
  joined by a dotted `border.strong` link with a gap; three `accent`
  packets run from the laptop into the gap and fade there, and the broken
  link (two severed `status.attention` ends on a 10 % disc) pulses.
  TLS and navigation errors keep the lifering (their cause is not a
  network link).
- **Motion (D10):** each illustration plays two 1400 ms cycles (`linear`)
  from the moment its pane appears — a new kind or a fresh appearance
  restarts it — then rests on its final frame, the Paper poster (cards in
  place with the key up; the head mid-line after a full sweep and a
  decelerating second one; packets a third apart) and builds no animation
  element afterwards. Reduce
  Motion shows only the rest frame. **Assert:** `motion.illustration`
  (`kind` `empty-space` / `crashed` / `offline`, `state` `playing` / `rest`).
- Both panes paint flat GPUI surfaces; they never render web content or
  screenshots.

### 11.8a Unresponsive renderer pane

- **When shown:** only over the active tab's rendered pane slot while its
  session is `unresponsive`; never over a transient Peek, a non-focused split
  sibling, the Library, or a classified error surface. The stale native view
  is hidden, not destroyed.
- **Copy:** title “This page is not responding”; calm explanatory text; no
  automatic reload or termination.
- **Actions/focus:** `Wait` → `Reload` → `Close tab` → sidebar. Wait keeps the
  same session and resumes bounded monitoring. Reload uses the sole
  destroy→recreate authority (fresh `WebViewId`, same tab/profile). Close uses
  ordinary tab close/neighbor selection.
- **Assert ids:** `bbr-unresponsive-panel`, `bbr-unresponsive-wait`,
  `bbr-unresponsive-reload`, `bbr-unresponsive-close`.

### 11.9 Missing favicon and long titles

Covered by §7.2 (monogram tile) and §4 (truncation). Additional edge rules:
empty title falls back to the hostname; empty host falls back to the URL
scheme; a title consisting only of whitespace is treated as empty.

### 11.10 Favorites grid (Implemented; reference-calibrated)

- **Anatomy:** when real Favorites exist, `bbr-favorites` presents destination
  tiles without a repeated visible section label. The §7.2 favicon/monogram is
  centered on the `material:favorite` veil (v2 §18.2: white 50 % light /
  6 % dark over the tinted rail) at `radius:md` (10; `radius:sm` at S), gap
  4/4/8 pt; the empty slot and the drop cell share the tile radius. A newly
  committed favorite's tile surface pops 0.86 → 1 with overshoot while a
  1.5 pt `accent` outline breathes 0 → 3 pt out of the tile (60 % → clear)
  and ten specks in `accent` and the curated space hues burst 18 → 44 pt
  past the tile's edge (the burst 0.8 as tall as wide), shrinking 3 → 0.6 pt
  as they fade (Paper `micro_favorite`, compressed into ≤ 260 ms; the tile
  box, icon and listeners never move); Reduce Motion flashes `tint.accent`
  for `dur:fine`.
- **Columns and wrap:** the grid is **4 columns at every density and every
  width** (`grid_columns`), so tile `n` always lands in the same cell. Tile
  *width* derives from the rail — `(content − 3·gap) / 4` inside the region's
  `sp:2` inline padding — so the grid always spans the full content width;
  tile *height* stays fixed per density (36 / 44 / 52 pt), keeping
  `favorites_region_height` a pure function of density, count and the drag
  offer (the E2E compares it with the measured region). Overflow
  wraps (`rows = ceil(n / 4)`); a partly filled last row keeps its columns.
- **Empty (`bbr-favorites-empty`):** with zero favorites the region renders
  one full-width drop band (S 48 / N 56 / XL 64 pt) with a 1 px **dashed**
  hairline outline, `radius:md`, no fill, a leading download-into glyph and
  two lines: `bbr-favorites-empty-label` “Favorites” (`type:meta` 600,
  `ink.secondary`) and `bbr-favorites-empty-hint` “Drop a tab here to
  favorite it” (`type:meta`, `ink.faint`). This is the deliberate exception
  to the truthful-empty-region rule (§11.4): the band teaches the only gesture
  that creates a favorite and renders an affordance, never fabricated
  content. As a drag target it takes the §10.9 language (solid `accent`
  outline, `tint.accent` fill; label “Add to Favorites” in `accent.ink`,
  hint “Reachable from every space and profile”) with **unchanged geometry**,
  so nothing reflows under the pointer. Once favorites exist the band
  shrinks to a glyph-only slot in the next free cell; when the last row is
  exactly full the slot is omitted.
- **Drop cell (`bbr-favorites-drop-cell`):** during an active tab drag a
  populated grid offers one additional cell at the end of the wrap order
  instead of the rest slot: tile geometry, dashed `accent` outline, no fill,
  centered plus glyph in `accent`; with the pointer over the region it is
  the live target (solid `accent`, `tint.accent`). The region is never
  outlined as a whole — a cell names the slot, and an outline would reflow
  the region mid-drag. When the last row is full the cell opens one more row
  (`favorites_offer_growth`), committed once at activation. The whole
  populated region resolves to the cell and the drop appends there;
  per-tile insert-before targets need an insert-at-index favorite capability
  and are deferred.
- **Behavior:** clicking a tile activates that exact favorited tab: switch
  to its owning space in the same browser window (thereby selecting the
  space's profile/container), then focus its stable `TabId`. Never open a
  copy, navigate a different tab, or move the tab. Hover/active washes
  follow §10.2/§10.3; keyboard traversal follows the §9 order row-major;
  tiles never reorder during selection.
- **States:** `default`, `hover`, `active`, `focused` (§10). A favorite is an
  additional global presentation of a concrete tab, so its title/favicon and
  lifecycle availability mirror that tab; the tile's semantic name includes
  the owning space/profile when they differ from the current context.
- **Data source:** favorites are global across all spaces and profiles in the
  browser window. Each tile resolves `TabId + SpaceId + ProfileId`; the tab
  remains canonically owned by one space and derives its profile from that
  space. Dragging a tab into the grid toggles that same tab as a favorite;
  closing it removes the tile, and moving it makes the favorite follow its
  stable `TabId` to the new space/profile. No second web session is created.
- **Assert ids:** `bbr-section-favorites`, `bbr-favorites`,
  `bbr-fav-<n>`, `bbr-fav-icon-<n>`, `bbr-favorites-empty` (+ `-label`/
  `-hint` in the zero-favorites band), and `bbr-favorites-drop-cell` while a
  tab drag is active over a populated grid.

### 11.11 Folders (Implemented; pin invariant enforced)

A folder is the grouped presentation of pinned tabs inside the common
organized area (§11.2/§11.4), not a separate product region. Folder rows
render without a gap before the single organized hairline, and the portable
invariant `folder member => pinned` holds
([folder-membership invariant packet](docs/roadmap/tasks/folder-pinned-invariant.md)):
grouping a regular tab pins it in the same transition (it moves to the end of
the pinned region, reported as an ordinary pin); removing a member from its
folder leaves a loose pinned tab; unpinning a member through any path ends
its membership; a rejected grouping changes neither. Folders are
session-local, so a restore never recreates a membership. A grouped tab
renders inside its folder only: the loose pinned block lists the ungrouped
pinned tabs, a loose row's drag resolves among the loose rows (past the last
one is the pinned region's end), and ↑/↓ traverse the rendered rows — loose
pinned, the expanded folders' members, then the regular region.

- **Anatomy:** a `type:badge` "Folders" label (`bbr-section-folders`,
  hidden when empty) over folder rows (`bbr-folders`): `[folder glyph §7,
  open/closed variant carrying the disclosure state, `icon:m` 18; S
  `icon:s` 16] [name, trunc] [count, `type:meta` `tnum`,
  hover/focus-revealed]` at `density:row` tall, stepping on the §8 pitch
  (surface + gap) like every row; nested rows indent one `sp:3` step and
  the expanded folder's extent stays an exact pitch multiple.
- **Disclosure:** the leading glyph is the §7 folder with the open variant
  for the expanded state (never a separate chevron, never color-only).
  Toggling (`bbr-folder-toggle-<n>`, pointer or keyboard Enter/Space)
  expands/collapses the nested row list and snaps (`dur:instant`). Nested
  rows reuse the §11.1 row anatomy and the full §10 vocabulary; index
  `<m>` is the nested position within folder `<n>` (0-based,
  deterministic). A folder whose tabs were all closed remains as an empty
  collapsible row.
- **Count reveal:** the `type:meta` count paints only while the pointer
  hovers the row or the row is the keyboard focus anchor while the folders
  region owns chrome focus; at rest every count stays quiet (zero-opacity,
  the `bbr-folder-count-<n>` id stays in the tree). The typed snapshot
  reports `count_revealed` so E2E proves the reveal semantically.
- **Rename on create (blueprint D20):** a folder created through the plus
  menu's New Folder opens in inline rename at once: the row reads as a
  focused field (`surface:field`, a 1 pt `accent` edge and the
  `tint-accent` halo, never color-only — the field shape and the 1.5 pt
  accent caret carry it too), its name slot shows the draft
  (`bbr-folder-rename`) with the default title "New Folder" selected, the
  count stays quiet, and the folders region owns chrome focus. The first
  printable character replaces the selection; Backspace deletes; a bare
  arrow collapses the selection to the caret. Return, a primary press
  outside the field, or focus leaving the folders region (Cmd-T, a tab row,
  the web view) commits a changed, non-empty draft through the
  organization's rename command (trimmed; the folder keeps its identity,
  members and expansion); Escape — and an emptied or unchanged draft —
  keeps the committed title. At most one folder is in rename. The typed
  snapshot reports `rename_draft`/`rename_selected` per folder row.
- **Drag:** dragging a tab over a folder row renders the folder as a
  `drag-target` (§10.9); a drop routes through the move-into-folder
  command of the favorites/folders domain slice, and keyboard staging
  (`Cmd-Shift-M`) covers folders per the interaction contract.
- **States:** folder rows take `default`/`hover`/`active`/`focused` plus an
  expanded/collapsed value in the semantic tree; nested rows take the full
  §10 vocabulary.
- **Assert ids:** `bbr-section-folders`, `bbr-folders`,
  `bbr-folder-row-<n>`, `bbr-folder-name-<n>`, `bbr-folder-count-<n>`,
  `bbr-folder-toggle-<n>`, `bbr-folder-item-<n>-<m>` (+ `-title`/
  `-favicon`/`-close`), `bbr-folder-rename` (only while a rename is open).

### 11.12 Downloads panel (Planned)

The fixed footer `[Downloads]` control (§11.4) opens the downloads surface. It
is an **anchored popover** (`bbr-downloads-panel`), not a sidebar region and
not an above-glass surface: `elev:2` over `material:overlay` with the palette's
shadow class (§2), `radius:lg`, a 1 pt `hairline` edge (the §9 focus ring
stays the 2 pt accent, §18.4), anchored so its bottom edge is flush with the
footer control's top edge and its leading edge with the sidebar column's
leading edge. Width = the sidebar column's current width − 2 ×
`density:row-inset` (S `sp:1` 4 / N `sp:1` 4 / XL `sp:2` 8, §8), clamped to stay
fully inside the window; the popover grows upward and its height clamps to the
window. Because the popover is geometrically confined to the sidebar column it
never overlaps the native web view, so it stays ordinary GPUI sidebar chrome
(`GpuiDeferred`/ordinary composition) and never uses the Portal host — the
Portal host remains reserved for chrome that must render above the glass
(find-in-page bar). The anchored-popover form is the user-approved downloads
draft and is **not** provisional; only the open questions named in §17 remain.

- **Open/close:** activating the footer control (pointer or keyboard) toggles
  the popover with an opacity fade `dur:fine` only — never translate or scale
  (§6). Dismiss: Escape, outside pointer press (one pointer action, §13), or
  re-activating the footer control. The panel never auto-opens; the footer
  control keeps its §11.4 text alternative and `type:meta` `tnum` count at
  rest. **The badge counts only in-flight (active) attempts** (settled
  §17.11): it returns to zero when nothing transfers and terminal history
  never inflates it.
- **Anatomy (top → bottom):** two groups separated by a `hairline` —
  **Active** (`bbr-downloads-active`, non-terminal attempts) above, **History**
  (`bbr-downloads-history`, terminal attempts) below; `type:badge` labels
  "Active"/"History" are hidden when a group is empty (§11.4 region-label
  rule). Rows: `[status glyph `icon:<density>`] [name line + secondary line]
  [action]` at `density:row` tall, padding-x `sp:2` (S) / `sp:3` (N) / `sp:4`
  (XL) like §11.1; hover/active inset surfaces per §10.2/§10.3
  (`density:row-inset`, `density:radius`); the action's hit area ≥
  `density:hit-min`.
  - **Status glyph** (§7 icon system): active rows (Waiting/Running) show the
    §10.5 spinner at `icon:<density>` (accent, transient indication — allowed);
    succeeded shows the `status.ok` check glyph (right after the success
    commits it draws its stroke with one expanding, fading `status.ok` ring
    within 260 ms; Reduce Motion: it simply appears); failed the §10.7
    `status.error` glyph; cancelled a neutral glyph at `ink.secondary`. §3.3
    non-color companions apply.
  - **Name line:** filename, one line, end-ellipsis (§4); tooltip on hover
    (Planned, §4).
  - **Secondary line** (`type:meta`, `tnum`): active — a 2 pt progress bar
    (the §10.5 bar geometry; track `hairline`, fill `color:accent`) plus
    transferred/total bytes; a known total fills `fraction()`, an unknown
    length runs the §10.5 indeterminate sweep (`load:indeterminate`) with
    transferred bytes only; terminal — size + status copy naming the machine
    outcome ("Completed" / the `DownloadFailure` reason / "Cancelled"; copy
    follows the §11.8 tone).
  - **Action:** exactly one contextual primary per row — active → **Cancel**,
    failed → **Retry**, succeeded → **Reveal-in-Finder** (conventional macOS
    "Show in Finder" copy; enabled only when the terminal record's resolved
    path exists, otherwise §10.8 `disabled` with a reason tooltip),
    cancelled → **Restart** (settled §17.8: a fresh attempt linked via
    `retry_of` — the same machine path as a failed retry, labelled
    "Restart"). A cancelled row additionally offers the **Delete** control
    (`bbr-download-delete-<n>`), which removes exactly that one terminal
    record durably (settled §17.8).
  - **Clear:** an explicit **Clear** control (`bbr-downloads-clear`) renders
    below the scroll area while any history exists; it empties the
    controller history and the persisted terminal history durably (settled
    §17.7) and keeps live attempts.
- **Machine fidelity (L6 contract):** the panel never invents transitions.
  Active = `Waiting { destination: None }`, `Waiting { destination: Some(..) }`,
  `Running { .. }`; History = `Succeeded`, `Failed { failure }`, `Cancelled` —
  terminal and immutable. Cancel is request/confirmation based: the row stays
  active until the engine's terminal event lands and the first terminal event
  wins (no optimistic state change). Retry is offered on `Failed` and restart
  on `Cancelled` (settled §17.8) — both start a fresh attempt id linked via
  `retry_of`; the panel labels the same machine path differently by state.
  Progress is monotonic; a known total is stable and never reverts to unknown;
  an unknown total may become known (determinate replaces indeterminate); a
  zero-byte download succeeds with a full bar (`fraction()` is never `NaN`).
  History records (final paths, timestamps, attempt counts, retry linkage)
  live in the controller; **terminal history persists through restarts**
  (settled §17.10) — live attempts are never persisted and never resurrected.
- **Group transitions:** a running attempt moves to History on its terminal
  event within the same frame (snap, `dur:instant`, §6). Ordering: Active in
  attempt order (ascending `DownloadId`), History most-recent-terminal first —
  deterministic, so row index `<n>` is stable for E2E.
- **Overflow/scroll:** at most 6 visible rows (`density:row` each); beyond
  that, bounded rendering (visible range + 8-row overscan, §11.6) and the
  §11.6 scrollbar (8 pt track-less thumb, `ink.faint` 40% / 60% hover). The
  full history is never laid out; a clamped popover height scrolls.
- **Keyboard and pointer:** `↑`/`↓`/`PgUp`/`PgDn` move focus and scroll; Enter
  activates the focused row's action; Tab/Shift-Tab move focus between rows;
  Escape dismisses and returns focus to the footer `[Downloads]` control (§9
  order: Downloads precedes the space chips). Pointer: click activates the
  row's action directly; hover/press paint §10.2/§10.3. No persistent
  selection — focus is the keyboard cursor; no multi-select in v1.
- **Focus domain:** while open the panel is chrome focus (ring per §9;
  resting selected rows keep their quiet surface + weight). Opening preserves the active
  tab/session; dismissing returns focus to the footer control per the
  interaction contract.
- **Empty state:** when the panel has no attempts at all, a single hint line
  in `ink.secondary` ("No downloads yet") — no empty box, no icon graveyard.
- **Modes:** tokens per §3; IC deepens washes and strengthens hairlines
  (§3.4); reduced motion removes everything except `dur:fine` opacity (§6).
- **Assert ids:** `bbr-downloads-panel`, `bbr-downloads-active`,
  `bbr-downloads-history`, `bbr-download-row-<n>` (+ `-status`/`-name`/
  `-meta`/`-progress`/`-action`/`-delete` on cancelled rows),
  `bbr-downloads-clear`, `bbr-downloads-empty`; the footer control keeps
  `bbr-downloads` (§11.4).

### 11.13 Shared context-menu language and owned families (Planned)

The Zen references in
`/Users/patwoz/dev/Personal/macos/bbr-design-references/zen-*-context-menu*.png`
are information-architecture and native-macOS behavior references only. BBR
uses its own BBR tokens, copy, product capabilities, profile model and command
authorities; it does not copy Zen/Firefox features or styling.

- **Material and geometry:** production BBR menus use `material:overlay`,
  `elev:2` (v2), `radius:md`, `type:label`, and hairline group separators;
  the card has a 1 pt edge (`hairline`, `accent` while the menu owns
  keyboard focus) and 4 pt inner padding, and the keyboard/pointer cursor is
  an inset `radius:sm` row filled with `tint.accent` — the 2 pt accent rail
  is gone (§18.4). The Plus menu, page menu, tab/favorite/folder menus and
  their submenus share this anatomy through one `bbr-ui` helper. Native
  platform/accent selection is allowed; Zen blue, oversized cards, gradients,
  and copied translucent slabs are not.
- **Interaction:** secondary-click opens at the source pointer, clamped to the
  browser window. Up/Down traverse; Right/Left enter/leave submenus; Return
  commits; Escape or one outside press dismisses. Disabled rows remain visible
  only when their surrounding group is useful and expose a deterministic
  reason. Danger state has a text/glyph companion; color never carries it
  alone.
- **Source identity:** a menu stays bound to the exact `TabId`, Favorite-backed
  `TabId`, `FolderId`, or page/pane that opened it. Opening, traversing, or
  dismissing never implicitly selects, activates, expands, moves, reloads,
  closes, suspends, or recreates that source.
- **Ownership:** sidebar tab/Favorite/folder menus are ordinary GPUI chrome.
  Only BBR page-background menus overlap WebKit and use the existing one-per-
  window `NativeAboveWeb` Portal. Link/image/selection/editable/media and
  JavaScript/unclassified WebKit element menus remain native and untouched.
- **Commands:** callbacks route through an existing or explicitly added
  portable command/controller authority. A missing capability omits its row or
  renders a useful disabled state; the renderer never invents behavior.
- **Explicit omissions:** no Send-to-Mobile, Firefox Container Tab, route-for-
  domain, extension-contributed rows, QR creator, block-element command, video
  downloader, or bulk Select All without their own BBR product contracts.
  After integrated B2, Tab/Favorite and BBR page-background menus may show
  capability-gated `Bookmark Page` / `Remove Bookmark` rows; both invoke the
  same app-owned Saved Page command as Cmd-D and never own duplicate policy.
- **Stable family ids:** `bbr-tab-menu`, `bbr-favorite-menu`,
  `bbr-folder-menu`, and `bbr-page-menu`, with action/submenu ids owned by
  their bounded task packets.
- **Gallery:** G2/G3 stories register closed/open/submenu/disabled/danger/
  overflow states from the exact production `bbr-ui` menu components. Gallery
  fixtures never rebuild menu anatomy and never replace real-browser E2E.

Canonical packets: [Tab/Favorite menus](docs/roadmap/tasks/tab-favorite-context-menus.md),
[Folder menu](docs/roadmap/tasks/folder-context-menu.md), and
[Page-background follow-up](docs/roadmap/tasks/page-background-context-menu.md).

### 11.14 Unified titlebar navigation controls (Implemented; reference-calibrated)

The production titlebar is
`[traffic lights] [sidebar collapse/expand] … [Back] [Forward] [Reload]`
inside the actual macOS safe titlebar layout guide.

- The window uses the public transparent/full-size macOS titlebar so the
  expanded Sidebar owns the complete left rail from the top window edge to the
  footer behind the native traffic-light area. Its collapse control sits immediately after the traffic lights,
  uses the same sidebar-width/collapse authority as Cmd-B, changes glyph for
  collapsed versus expanded, and replaces rather than duplicates the pre-G1a
  status-row control.
- Back/Forward/Reload form a compact trailing cluster and target the selected
  tab in single-pane mode or the focused split pane. Enablement derives from
  real navigation state.
- Glyphs use the BBR geometric 18 pt family with normal-density hit areas at
  least 32 pt, transparent rest state, §10.2 hover wash, and §10.8 disabled
  treatment. No bordered toolbar capsule and no duplicate controls in the
  address display.
- Layout uses the native safe area; traffic-light x/y values are never
  hardcoded. Empty titlebar space remains draggable. Fullscreen,
  multi-display, and narrow-window states must preserve reachability without
  overlap.
- Collapsing the Sidebar removes the complete BBR-owned titlebar/navigation
  row together with the Sidebar. No compact content toolbar or floating
  Back/Forward/Reload replacement remains over the webpage; native traffic
  lights remain available and Cmd-B restores the Sidebar.
- Stable ids: `bbr-titlebar`, `bbr-titlebar-sidebar-toggle`, `bbr-nav-back`,
  `bbr-nav-forward`, `bbr-nav-reload`.

Canonical packets: [Unified titlebar navigation controls](docs/roadmap/tasks/titlebar-navigation-controls.md) and the superseding visual authority [reference-faithful production shell](docs/roadmap/tasks/visual-shell-reference.md).

### 11.15 Sidebar media control (Planned; capability-first)

The media card is conditional ordinary GPUI sidebar chrome above the footer
hairline and therefore above `[Downloads] [Space strip] [+]`. It consumes no
layout or focus space when no eligible session exists. The Zen media reference
sets composition only; BBR capabilities come exclusively from the portable
[media-session contract](docs/roadmap/tasks/media-session-capability-contract.md)
using public WebKit APIs or an isolated content-world bridge — never SPI or
HTML heuristics alone.

- **Identity and capability:** the card is bound to one opaque media session
  plus its exact TabId/SpaceId/ProfileId. Every command revalidates that
  identity. Controls `{play,pause,seek,previous,next,mute}` appear or enable
  only when explicitly advertised; Previous/Next and duration are never
  assumed.
- **Anatomy:** title + source/meta + dismiss; elapsed/duration and a determinate
  scrubber only for known, seekable duration; live/unknown media has no fake
  total or scrubber; source activation, capability-gated transport controls,
  and mute. Dismiss hides the current media identity until it changes and does
  not stop playback.
- **Multiplicity:** when several tabs play, show only the most recently user-
  controlled eligible session plus a compact count/switch affordance — never a
  vertical card stack.
- **Focus:** controls enter §9 before Downloads. Source activation selects the
  exact owning tab/space/profile and visibly communicates cross-profile
  identity; other media commands do not select or focus the tab.
- **Visuals:** `material:field`, `elev:1`, `radius:lg`, BBR text/glyph tokens,
  and `color:accent` progress. No copied Zen blue. Playback/progress indication
  honors Reduced Motion and never hides static state.
- **Stable ids:** `bbr-media-card`, `-title`, `-source`, `-progress`,
  `-elapsed`, `-duration`, `-previous`, `-play-pause`, `-next`, `-mute`,
  `-dismiss`, `-session-count`.
- **Gallery:** static capability/state stories use the same production card;
  they never claim real WebKit control or replace the deterministic media E2E.

Canonical UI packet: [Sidebar media control](docs/roadmap/tasks/sidebar-media-control.md).

### 11.16 Saved Pages result and Library (Implemented)

Saved Pages are durable global URL records, not live-tab presentations. Pin
keeps one concrete Space-owned tab prominent; Favorite presents one concrete
living `TabId` globally and disappears with it; a Saved Page carries no
`TabId`, Space/Profile identity, WebView, native session, cookie, credential,
or page snapshot and survives tab close and restart.

- **Palette result row:** `[bookmark glyph] [title + URL] [collection label]`;
  the mark is the drawn §7 bookmark in the palette, the overlay and the
  in-rail result list alike.
  Title uses `type:title` with the user label when present, otherwise the
  captured title, then hostname/URL fallback; URL uses `type:mono` and the §4
  mid-ellipsis; the optional one-level collection label uses `type:meta`.
  Default/hover/selected/focused states use
  the existing command-row language; the selected result is the inset
  `tint.accent` row (§18.4) and no Saved Page-specific color family is
  introduced.
- **Activation:** opening a Saved Page creates exactly one fresh regular tab in
  the currently active Space and therefore its current Profile. It never
  silently selects an existing matching tab, switches Space/Profile, pins,
  favorites, splits, or reuses a native session.
- **Save/remove:** conventional macOS `Cmd-D` and the BBR-owned menu rows
  `Bookmark Page` / `Remove Bookmark` call one command. Canonical-URL save is
  idempotent; removing a Saved Page never closes or mutates an open tab, and
  closing a tab never removes its Saved Page. The §11.5 address control remains
  a display button; a later saved-state accessory is not required by B2.
- **Library topology:** the full collection lives in one dedicated ordinary
  GPUI Library surface, never as a permanent Sidebar region and never
  as a WebKit page or Portal. The integrated entry point is Cmd-Shift-L; the
  internal chrome surface covers the native view while open and Escape
  dismisses it. The production surface supports search, deterministic sort,
  rename, remove, multi-select, and one-level flat collections through shared
  `bbr-ui` primitives.
- **Library row:** the drawn §7 bookmark glyph and the normal favicon presentation,
  user label or captured-title fallback, mid-ellipsized URL, optional
  collection label, and selection state. A selected row paints
  `tint.accent` with `ink` text (§18.4); its rename control uses the §7 pencil.
  No favicon blob is stored in the record; missing favicon uses the §7.2
  monogram. Hundreds/thousands of cold records use bounded/windowed rendering.
- **States:** empty, loading, populated, no results, single/multi-selection,
  destructive removal confirmation, long title/URL, missing title, missing
  favicon, flat collection, and 1k bounded rendering. State changes snap;
  transient opacity follows §6 and Reduced Motion.
- **Stable ids:** `bbr-saved-page-result-<bookmark-id>`,
  `bbr-saved-page-result-title-<bookmark-id>`,
  `bbr-saved-page-result-url-<bookmark-id>`, `bbr-saved-pages-library`,
  `bbr-saved-pages-search`, `bbr-saved-pages-sort`,
  `bbr-saved-pages-collection-<collection-id>`,
  `bbr-saved-pages-row-<bookmark-id>`, and
  `bbr-saved-pages-remove-confirm`.
- **Gallery:** G2/G3 append only the exact integrated B2/B3 production
  components and models (Library rows/controls/surface and the Library
  composition registered at story ids 23–30 through the recorder/observer
  seam; gallery registry id 22 is the integrated G2-TAB Tab Row story).
  Gallery renderability
  never proves persistence, restart, command activation, or bounded product
  performance.

Canonical packets: [B1 contract/persistence](docs/roadmap/tasks/saved-pages-portable-persistence.md),
[B2 actions/command provider](docs/roadmap/tasks/saved-pages-actions-command.md),
and [B3 Library](docs/roadmap/tasks/saved-pages-library.md).

### 11.17 Peek preview bar (Link Preview / Peek)

A **transient** source-profile preview surface, not a modal, a tab, or a
persisted row. An eligible direct activation (user-initiated `target="_blank"`
/ `window.open`, previewable scheme, no explicit size or tool-style chrome)
opens the preview in the source profile's data store, raised above the warm
origin in the native z-order; the origin stays selected and warm underneath and
never becomes the pointer/keyboard/accessibility-focus target while Peek is
open. Exactly one preview per pane: a new eligible request from inside the
preview navigates it, any other source replaces it (the old session is
destroyed first).

- **Surface:** a full web-content-area-width bar anchored to the top of the web
  content area (right of the sidebar), `density:row`-tall, in the §11.4
  command-surface family — `material:overlay`, `radius:lg`, `elev:2`.
- **Anatomy:** label (preview title, mid-ellipsized, with URL fallback), a
  state token (`Peek` / `Loading` / `Crashed`), and the controls
  Back / Forward / Reload / Open as Tab / Close. Controls route through the
  production app methods via the portal root's deferred update (the same
  `NativeAboveWeb` mechanism as the find bar); Escape and the close control
  dismiss, Open as Tab promotes the same session.
- **States:** loading (`LoadingArc`) and crash (`Lifering`) are surfaced
  honestly through `bbr-peek-state`; a crashed preview cannot promote and still
  dismisses cleanly. The preview WKWebView renders the page natively — never a
  screenshot or texture-composed copy in GPUI.
- **Focus:** while open, the preview owns browser-command focus; Back /
  Forward / Reload / Stop / Navigate route to the preview session. Dismissal
  restores focus to the exact warm origin control/web view.
- **Motion:** present and dismiss bound to the portal frame with no
  intermediate chrome geometry (reduced-motion safe by construction); an
  entrance/exit transition remains later polish.
- **Theme:** the bar follows the shell appearance (light/dark) like every
  other portal surface; the increased-contrast/compact/reduced-motion matrix
  stays open (§16).
- **Assert ids:** `bbr-peek-bar`, `bbr-peek-url`, `bbr-peek-state`,
  `bbr-peek-back`, `bbr-peek-forward`, `bbr-peek-reload`,
  `bbr-peek-open-as-tab`, `bbr-peek-close`.

Canonical packet: [Link Preview / Peek](docs/roadmap/tasks/link-preview-peek.md).

## 12. Profile identity

- **Chip:** 24 pt square (20 at S, 28 at XL), `radius:sm`, filled with the profile's
  hue tile, containing a 1–2 character monogram (`type:badge`, white or
  near-black letter per tile). Static — no glow, no animation, no gradient.

| Hue | Light tile | Light letter | Dark tile | Dark letter |
|---|---|---|---|---|
| Cobalt | `#4062E8` | white | `#6E8BFF` | white |
| Violet | `#7C4DF0` | white | `#8A63F0` | white |
| Fuchsia | `#C63B8F` | white | `#B02E7E` | white |
| Amber | `#B57912` | `#1A0D00` | `#C0861F` | white |
| Terracotta | `#B4582C` | white | `#B85A30` | white |
| Slate | `#5B6470` | white | `#8B94A2` | white |

Tile/letter pairs are computed to ≥ 3:1 from the tokens (the same bar as the
§7.2 favicon monogram tiles; the dark tiles are darkened from the original
authored values because those topped out at ≈2.1–3.1:1 against white, and
identity never relies on color alone — the monogram is the non-color
companion). Profiles get hues round-robin from this fixed set; hue assignment
is stable for the profile's lifetime.

- **Identity rules:** color never alone — chip always carries the monogram;
  profile hue is never used for status, accent, or favicons; profile identity
  appears on every space row and whenever tabs/spaces cross contexts (move
  previews, space switching per §11.3); at most one profile's identity is
  "loud" at a time (the active space's chip), the rest stay at rest.
- **Assert ids:** `bbr-profile-chip-<n>` with a `profile_hue` snapshot value
  (allowed values = the six hue names).

## 13. Native web-content boundary ("the glass")

- Chrome is a left rail; the remainder is the seated content card (§2) whose
  inner box is the native WKWebView child. At rest there is exactly one
  boundary: the card edge with its static `elev:1.content` shadow, painted
  outside the slot. No chrome shadows, glows, or gradients may intrude over
  web content.
- **Never sampled:** BBR never screenshots, blurs, or textures web content
  into GPUI (product principle + ARCHITECTURE §1). Consequently: no page
  thumbnails, no live tile previews, no cross-fade "peek behind chrome".
  Tab/space switching presents the native view directly on the next frame
  (performance contract), which is also the design commitment: **the glass is
  never interpolated**.
- Floating surfaces (palette, peek) are the only chrome allowed to overlap
  the glass, are `elev:2`, and must be dismissible with Escape/one pointer
  action; peek's temporary state is covered by its own roadmap task.
- When the sidebar collapses, the slot re-flows instantly (§11.4) — no
  intermediate chrome geometry is ever shown; the card's inset, radius and
  clip change in the same frame.

## 14. Anti-generic checklist

The system must never drift toward these (each is a review gate):
no purple-blue gradient heroes; no glass-blur cards; no oversized rounded
sheets; no arbitrary emoji as functional chrome; no glow shadows; no
cover-flow/thumbnail tabs; no site-adaptive chrome tinting; no animated content
behind chrome; no emoji/filled-duotone substitutes for missing functional
icons; no micro-interaction choreography at 60 fps. A curated user-selected
SpaceIdentity emoji in §11.3 is identity data, not a fallback icon. Where a
generic pattern is tempting, the machine answer remains a hairline, a numeral,
a bounded row surface, or silence.

## 15. Automated semantic assertions

Real-app E2E and the testkit snapshot must assert the visual system
semantically — never by pixel comparison.

- **Stable ids:** the `bbr-*` ids enumerated in §11 (plus `bbr-root`,
  `bbr-sidebar`, `bbr-command-bar`, already present in the placeholder
  shell). Every interactive control carries one. The downloads surface
  registers `bbr-downloads-panel`, `bbr-downloads-active`,
  `bbr-downloads-history`, `bbr-download-row-<n>` (+ `-status`/`-name`/
  `-meta`/`-progress`/`-action`), and `bbr-downloads-empty` (§11.12); the
  footer control keeps `bbr-downloads` (§11.4). Planned owned menu families
  register the §11.13 ids; the titlebar registers the §11.14 ids; an eligible
  media session registers the §11.15 `bbr-media-card*` ids and an absent card
  registers none. Integrated Saved Pages register the §11.16 result/Library
  ids only for real records/surfaces; an absent Library/result registers none.
  The S1 scale table (§8) changes geometry tokens only — it
  does not change ids, the §10 state
  vocabulary, interaction behavior, dense-500 bounded-window rendering,
  scroll math, or any performance budget/sample count. The integrated Peek
  preview bar registers `bbr-peek-bar`, `bbr-peek-url`, `bbr-peek-state`,
  `bbr-peek-back`, `bbr-peek-forward`, `bbr-peek-reload`,
  `bbr-peek-open-as-tab`, and `bbr-peek-close` (§11.17) only while a preview is
  presented; an absent preview registers none.
- **State vocabulary:** snapshot field `tab_states: Vec<VisualState>` with
  `VisualState ∈ {default, hover, active, selected, loading, attention,
  error, disabled, dragging, drag-target, focused}` — exactly the §10 names.
  During a drag report both the source row (`dragging`) and any target
  (`drag-target`); `lifted_ids` lists the §15 ids of the surfaces painted
  with `elev:lift` (the selected tab's row and nested folder row, then the
  active space chip); the command-bar domain derives from the existing
  `chrome_focused` + `command_bar_mode` fields. Downloads rows expose
  `download_rows: Vec<DownloadRowSnapshot>` with `machine_state ∈ {Waiting,
  Running, Succeeded, Failed, Cancelled}` (the L6 contract states), `group ∈
  {active, history}`, `progress_fraction: Option<f64>`, `visual_state` (the
  applicable §10 names), and `action ∈ {cancel, retry, reveal, none}` with
  `action_enabled: bool`.
- **Mode fields:** snapshot gains `theme (light|dark)`,
  `contrast (standard|increased)`, `density (small|normal|extralarge)`,
  `reduced_motion (bool)`, `profile_hue` — so every state assertion can
  distinguish modes and every mode change is itself asserted.
- **Contrast gate:** tests recompute WCAG ratios from the tokens in §3 for
  the documented pairs and fail below: text ≥ 4.5 (≥ 9 in IC), accents/rails/
  rings ≥ 3.0, hairlines ≥ 1.0 (≥ 1.8 in IC). No ratio may be copied into
  tests; it is derived from `color` tokens.
- **Motion gate:** a "no-motion on state change" assertion is triggered by
  the performance harness: state-change samples must present the final
  geometry with no intermediate paint (matches the existing presentation-commit
  mechanism in `performance.rs`). Under `reduced_motion`, the attention pulse
  is asserted as opacity-only via the visual_state sequence.
- **Rule:** a UI feature is not done until its §11 spec row has passing
  real-app E2E asserting ids, states, mode fields, and (where the perf
  contract applies) presentation commit.

## 16. Non-goals and deferred (v1)

Vibrancy/NSVisualEffect materials; site-adaptive or theme-marketable color;
custom fonts; arbitrary emoji iconography outside the curated SpaceIdentity
catalog; page thumbnails/previews; spring physics unrelated to direct gesture
cancellation; auto-reload of crashed tabs; sidebar auto-collapse; density
auto-switch; palette result grouping polish; Saved Page cloud/mobile sync,
offline copies, tags, nested collections, or page snapshots; peek entrance/exit
animation and the peek increased-contrast/reduced-motion presentation matrix (the
production Peek surface itself is integrated, §11.17; Gallery stories follow
production integration).

## 17. Open decisions

1. **Density triggers:** at what collection sizes do 200/600 S/X flips fire,
   and does the first flip ask the user?
2. **Tooltips:** native NSTooltip-driven titles on rows and truncated URLs —
   platform-native vs GPUI-drawn.
3. **Pinned S-density titles:** icon-only rows are memorable for ≤ ~12 pins;
   decide a "pinned overflow" fold.
4. ~~**Sidebar collapse memory**~~ **Resolved (2026-08-06): global.** One
   last-expanded width per window, restored on expand; collapse preserves it
   and hidden width mutations never clobber it. Per-space memory would
   require a space-scoped geometry map and is explicitly out of scope for
   the Phase-4 sidebar slice.
5. **Crash auto-reload policy** (currently none).
6. ~~**Command palette opening surface**~~ **Resolved (2026-08-07):** Cmd-T
   opens the centered palette, Cmd-L and pointer focus on the field open the
   field-anchored overlay (§11.4); both run the §11.5 engine. Result
   grouping polish stays deferred (§16).
7. ~~**Downloads clear-history control**~~ **Resolved (2026-08-08):** an
   explicit **Clear** control in the panel (`bbr-downloads-clear`, below the
   scroll area while any history exists) empties the **controller history**
   and the persisted terminal history durably — cleared history can never
   return from the store. Live attempts are kept.
8. ~~**Downloads cancelled-row action**~~ **Resolved (2026-08-08):** a
   cancelled row offers **Restart** (a fresh attempt linked via `retry_of`,
   the same machine path as a failed retry) **and Delete** (removes exactly
   that one terminal record durably).
9. ~~**Downloads destination policy**~~ **Resolved (2026-08-08):** silent
   **suggested-destination** downloads (the engine's standard folder); no
   save panel. An explicit per-download destination stays a later,
   non-breaking option.
10. ~~**Downloads history persistence**~~ **Resolved (2026-08-08):** terminal
    history **persists through restarts** (schema field; no migration — a
    legacy v2 shape is discarded and immediately rewritable). Live attempts
    are never persisted and never resurrected.
11. ~~**Downloads badge count**~~ **Resolved (2026-08-08):** the §11.4 footer
    `tnum` count feeds **in-flight (active) attempts only** — it returns to
    zero when nothing transfers.
12. ~~**Downloads open-file action**~~ **Resolved (2026-08-08):** completed
    files open **only** through the per-row Reveal-in-Finder action
    (§11.12) — no auto-open, no open-on-double-click, no save panel.

13. **Internal page naming in the address control:** §11.5 shows a domain,
    but `about:blank` and future internal pages have no domain. Decide the
    short name they present (product name, a neutral label, or an empty
    pill) so the control never shows a raw scheme.
14. **Address accessory slot contents** — **resolved (2026-09-27),
    implemented:** the slot shows the page zoom chip
    only while zoom is not 100 %; pressing it resets to 100 %. Copy Link and
    a permissions indicator remain later accessories. Owned by the
    [address field and space identity packet](docs/roadmap/tasks/address-field-space-identity.md),
    landed 2026-09-27 (§11.5).
15. ~~**Folder membership versus pinned state**~~ **Resolved (2026-08-14),
    invariant implemented (2026-09-27):** folders are grouped pinned tabs in
    the common organized area. The domain `FolderStore` pins while grouping,
    reconciliation drops unpinned members, lifecycle budgeting protects
    members as pinned tabs, and deleting a folder is Unpack (members stay
    loose pinned tabs; the menu row belongs to the
    [folder context menu](docs/roadmap/tasks/folder-context-menu.md)). The
    [folder-membership invariant packet](docs/roadmap/tasks/folder-pinned-invariant.md)
    keeps only its performance gate open.

16. **Design-branch adoption** — **resolved (2026-09-27); the three
    adopted packets are implemented, only their performance evidence is
    deferred.** The Paper exploration preserved on
    `design/browser-shell-paper@414c97c7` was reviewed row by row against
    main. Adopted: the seated content card with the collapsed titlebar band
    (option B) and no rail hairline; a bordered address field whose fill
    follows the rail/space tint; the zoom accessory rule (§17.14); the space
    identity in the heading; the fixed four-column Favorites grid; the empty
    Favorites drop band; the drag offers with one live target and a drag
    proxy. Rejected: the compact media transport bar (superseded by §11.15)
    and footer identity glyphs outside the curated catalog. The sections
    above stay the shipped contract until each packet lands its text:
    [seated content card](docs/roadmap/tasks/seated-content-card.md)
    (landed as §2 "The content card", 2026-09-27),
    [address field and space identity](docs/roadmap/tasks/address-field-space-identity.md)
    (landed in §11.4/§11.5/§17.14, 2026-09-27),
    [Favorites grid and drag offers](docs/roadmap/tasks/favorites-drag-offers.md)
    (landed in §10.9/§11.2/§11.10, 2026-09-27).

17. **v2 visual refresh and motion** — **resolved (2026-09-28).** After the
    Arc study in Paper (file “BBR — Browser Shell”, pages 01–07) Patrick
    adopted decisions D1–D11 as recommended. The adopted target is §18; the
    numbered sections above keep describing the running app until the owning
    slice lands its text and flips its §18 status row. Program packet:
    [v2 visual refresh and motion](docs/roadmap/tasks/v2-visual-refresh-program.md).
18. **Complete-browser blueprint proposals** — **resolved (2026-09-28).**
    Patrick adopted all 24 deltas and the drawn answers to the D-1 questions
    (§19); only keyboard split creation stays open with Phase 6.

Resolving an item means moving its decision into the relevant section above
and updating the affected roadmap track in the same group.

## 18. v2 visual refresh and motion (adopted 2026-09-28)

**Status:** adopted contract, landing slice by slice. Until a slice lands,
§1–§15 describe the running app; each slice updates the sections it changes in
the same commit as its implementation and flips its row in §18.6. Where §18
and an unchanged older section disagree, §18 is the target and the older
section is the shipped fact.

**Why.** v1 was correct but read as lifeless: a flat grey rail, favorite tiles
and address field in three unrelated greys, a pale-blue selection that did not
lift, and a space tint (4 %) too quiet to notice. Arc feels alive for three
reasons that need none of its candy gradients, site tint or spring
choreography: the window is one tinted body, only the selection is solid, and
everything else is a quiet veil. v2 adopts those three moves, keeps BBR's
Phosphor accent, commit-first motion and native-glass boundary, and adds a
small, bounded set of after-commit moments.

### 18.1 Decisions

| # | Decision |
|---|---|
| D1 | The dark selection lifts **above** the rail (white 9 % + lift) instead of sitting below it. The selected-row text bar becomes `ink ≥ 10.5:1` (IC ≥ 11:1) and `ink.secondary ≥ 4.5:1` (IC ≥ 7:1) on the composited selected surface. |
| D2 | Exactly one static-shadow exception to “rows never shadow” (§2): the **lift** on the one selected row (tab, pinned, folder item) and on the active space chip. Painted with the surface, never animated, never on hover. |
| D3 | Space tint on the rail rises from 4 % to **12 %** (IC 14 %); S12 replaces the alpha with a luminance-pinned chroma share (§11.3). The text bars on every tinted rail become `ink ≥ 12:1` (IC ≥ 14:1) and `ink.secondary ≥ 5.9:1` (IC ≥ 9:1), recomputed for every curated theme in both modes. |
| D4 | A committed space switch cross-fades **only the chrome tint** from source to destination over `dur:standard` with `ease:standard`. Geometry, lists, the selected tab and the native view snap on the commit frame. A swipe commit is already at the destination tint and does not fade again. |
| D5 | Load progress lives in the **address field** (a 2 pt accent line on its inner bottom edge), not on the content card edge: GPUI never paints above the native web view. Supersedes the card-top bar of §10.5. |
| D6 | No idle “living paper” drift. The space paper is a static tint with an optional static grain; nothing animates while the window is idle. |
| D7 | The theme picker stays curated: hue chips, three intensity steps and a grain toggle. No free colour field or colour wheel. |
| D8 | App icon concept C, “Phosphor b”: a lowercase b whose bowl is the loading arc. The Dock icon is static; motion, if any, is limited to the About panel. |
| D9 | ⌘T palette width S 508 / N 600 / XL 716 pt (was 440/520/620); the ⌘T query renders in the 17 pt UI sans. The ⌘L overlay keeps `type:mono` for URLs and widens to at least 440 pt. |
| D10 | Empty-state illustrations play two cycles after the pane appears, then rest on their final frame. |
| D11 | `ease:decelerate` = `cubic-bezier(0, 0, 0.3, 1)` (§6 corrected). |

### 18.2 Materials, colour and elevation

Translucent tokens are painted over the (possibly space-tinted) rail, so one
value follows every tint. Contrast gates composite them over the untinted and
every tinted rail before computing ratios.

| Token | v1 light / dark | v2 light | v2 dark |
|---|---|---|---|
| `material:sidebar` · chassis | `#E0E0E0` / `#1A1D21` | `#E5E5E2` (limestone) | `#1C1E22` (wet slate) |
| `material:canvas` | `#F6F6F4` / `#121416` | `#F6F6F4` | `#131517` |
| `material:field` | `#FFFFFF` / `#23272C` | `#FFFFFF` | `#2A2D33` |
| `material:overlay` | `#FFFFFF` / `#202429` | `#FFFFFF` | `#25282D` |
| `material:favorite` | `#D2D2D2` / `#23272C` | white 50 % veil | white 6 % veil |
| `material:footer` (media card, footer surfaces) | `#F3F8FE` / `#202429` | white 62 % veil | white 5 % veil ¹ |
| address fill / hover | white 48 % / 30 %; dark white 3 % / 7 % | white 55 % / 40 % | white 6 % / 10 % |
| address border | ink 10 % / white 7 % | ink 4 % | white 4 % |
| address accessory | ink 5 % / white 6 % | ink 8 % | white 8 % |
| address IC | — | border ink 10 %, accessory 8 % | fill 8 % / hover 12 %, border 10 %, accessory 8 % |
| `color:hairline` | `#C7C7C7` / `#2A2E34` | `#D0D0CC` | `#2C2F35` |
| `color:tint.selected` | `#EEF2F9` / `#0F1318` | `#FFFFFF` + lift | white 9 % + lift |
| `tint.selected` IC | `#E1E8F3` / `#0A0D12` | `#FFFFFF` + lift | `#35383F` + lift |
| `wash.hover` / `wash.active` | opaque `#D7D7D7`/`#CFD0D0`; dark `#25272B`/`#2B2E32` | ink 5 % / 9 % | white 5 % / 8 % |
| washes IC | 7 % / 10 % | ink 8 % / 12 % | white 8 % / 12 % |
| `color:tint.accent` **(new)** | — (used `tint.selected`) | `#E6F1EE` | `#26403F` |
| `tint.accent` IC | — | `#D9EAE5` | `#2B4A48` |

¹ Paper proposed white 7 %; composited over the Slate-tinted dark rail it
gave `ink.secondary` 5.73:1 against the unchanged 5.9:1 footer bar, so S2
ships 5 % (6.08:1). Per the iteration policy the value moved, not the bar.

- **`tint.accent`** is the single accent moment on white/overlay surfaces:
  command results, menu and library cursors, drop targets. It replaces the
  2 pt accent rail and the accent fill on selected Library rows (§3.2 “one
  thing at a time” unchanged). Text on it is `ink` / `accent.ink`; gates:
  `ink ≥ 9:1`, `accent.ink ≥ 4.5:1` in every mode.
- **Lift** (`elev:lift`): light `0 1 3 ink 10 %` + `0 0 0 1px ink 4 %`;
  dark `0 1 2 black 35 %` + `0 0 0 1px white 4 %`. IC adds nothing beyond the
  IC surface values.
- **`elev:2` v2:** light `0 0 0 1px ink 7 %`, `0 2 4 ink 5 %`,
  `0 16 36 −6 ink 18 %`; dark `0 0 0 1px white 8 %`, `0 16 32 black 45 %`.
  Floating cards lose their 2 pt inset hairline edge (§18.4).
- Ink, accent, status and profile hues are unchanged. Increased contrast keeps
  its §3.4 rules on the v2 values.
- **Space tint (D3):** first shipped as a rail wash alpha 0.12 (IC 0.14),
  with the S4 wash hues recalibrated to hold D1 — the values moved, not the
  bars. The 2026-09-28 Paper comparison showed that rail as nearly grey, so
  S12 pins every step to the untinted rail's luminance and raises the
  chroma share (light 36 %, dark 24 % at `standard`, §11.3); the bars now
  hold by construction. The active
  chip becomes the lift carrying 8 % of the identity hue (so each space's
  chip stays distinguishable).

### 18.3 Radius and type

| Token | v1 | v2 | Used for |
|---|---|---|---|
| `radius:sm` | 6 | 6 | S rows, small tiles, menu cursors |
| `radius:md` | 8 | 10 | N rows, Favorite tiles, buttons, menus |
| `radius:lg` | 10 | 12 | address field, palette field, panels, XL rows |
| `radius:xl` | 14 | 16 | the ⌘T palette sheet only |
| content card | 10 | 10 | stays equal to `sp:content-inset` (§2) |

Chrome radii stay within 6–12; 16 is reserved for the palette sheet. Type
changes: the selected title drops from 600 to **500** (the lift carries
selection); the active-space heading is 13 pt / 600 with +0.01 em tracking in
`ink.secondary`; the address domain is 500; the ⌘T query is 17 pt UI sans.
Footer glyphs render at 16 pt; the Space strip starts at its leading edge (§11.4, D01).

### 18.4 Floating surfaces

- **Menus (§11.13):** 1 pt edge (`hairline`, `accent` while the menu owns
  keyboard focus), 4 pt inner padding, `elev:2` v2. The cursor is an inset
  row surface (radius 6) filled with `tint.accent`; the 2 pt accent rail is
  removed.
- **Find bar, auth sheet, Space configuration, Downloads:** 1 pt edges
  instead of 2 pt insets; field focus keeps its 2 pt accent ring (§9).
- **⌘T palette:** width per D9, radius 16, field 56 pt with the 17 pt query,
  result rows 40 pt with radius 10 inside an 8 pt padded list.
- **⌘L overlay:** width = field width clamped to 440 pt–`win:palette`, field
  40 pt, result rows 36 pt, radius 10; the query stays `type:mono`.
- **Library selection:** `tint.accent` with `ink` text (fixes the accent-fill
  contrast deviation).

### 18.5 Motion (supersedes the listed parts of §6/§10)

Principles 1–4 of §6 stand: every item below starts only **after** the state
it decorates is committed and presented, never delays a commit, and has a
Reduced Motion variant. Single-shot motion is ≤ `dur:emphasis` (260 ms);
loops exist only for loading and progress. When nothing is transient the
shell requests **no animation frames** (asserted).

| Moment | Motion | Reduced Motion |
|---|---|---|
| Loading arc (§10.5) | the favicon-slot arc rotates, 640 ms/turn linear | static arc |
| Address progress (D5) | 2 pt accent line at the field's inner bottom edge: indeterminate sweep on `load:indeterminate`; on finish it fills (`dur:standard`) then fades (`dur:standard`) | static 2 pt line while loading, removed on finish |
| Attention (§10.6) | two beats (each `dur:fine`, total ≤ 260 ms): dot scale 1→1.45→1 plus one ring 4→16 pt fading out | opacity-only 1→0.45→1 (the §10.6 contract) |
| Favorite added | after the drop commits: the new tile pops 0.86→1 with overshoot, an `accent` outline breathes out 3 pt, and ten specks in `accent` and the curated space hues burst past the tile's edge, all within 260 ms | 120 ms `tint.accent` flash on the tile |
| Download finished | the row's status glyph draws the check stroke and one `status.ok` ring expands and fades, within 260 ms | check appears, no ring |
| ⌘T results | results fade in with a 20 ms stagger (each `dur:fine`, 4 pt rise); the cursor highlight glides between rows in `dur:micro` while the selected result is committed immediately | opacity only, no rise; the cursor snaps |
| Space switch (D4) | chrome tint cross-fade, `dur:standard`, `ease:standard` | instant |
| Empty states (D10) | three drawn illustrations — empty space, crashed page, offline — play two cycles, then rest | final frame only |

The §10.5 content-pane progress bar and the “one-frame amber surface flash”
of §10.6 are withdrawn (the first cannot paint above native glass; the
second is replaced by the ring).

### 18.6 Slice status

| Slice | Packet | Lands in | Status |
|---|---|---|---|
| S0 contract | [v2 contract](docs/roadmap/tasks/v2-contract.md) | §6 D11, §17.17, §18 | landed 2026-09-28 |
| S1 system preferences | [system preferences](docs/roadmap/tasks/v2-system-prefs.md) | §3.4, §6, §15 | landed 2026-09-28 |
| S2 tokens | [tokens](docs/roadmap/tasks/v2-tokens.md) | §2, §3, §5 | landed 2026-09-28 |
| S3 rail | [rail](docs/roadmap/tasks/v2-rail.md) | §4, §10.2–§10.4, §11.1, §11.4, §11.5, §11.10 | landed 2026-09-28 |
| S4 space paper | [space paper](docs/roadmap/tasks/v2-space-paper.md) | §11.3 | landed 2026-09-28 |
| S5 floating surfaces | [overlays](docs/roadmap/tasks/v2-overlays.md) | §8 palette width, §11.4, §11.12, §11.13, §11.16 | landed 2026-09-28 |
| S6 motion core | [motion core](docs/roadmap/tasks/v2-motion-core.md) | §6, §10.5, §10.6, §11.5 | landed 2026-09-28 |
| S7 after-commit moments | [motion moments](docs/roadmap/tasks/v2-motion-moments.md) | §6, §10, §11.3, §11.4, §11.10, §11.12 | landed 2026-09-28 |
| S8 empty states | [empty states](docs/roadmap/tasks/v2-empty-states.md) | §11.8 | landed 2026-09-28 |
| S9 app icon | [brand icon](docs/roadmap/tasks/v2-brand-icon.md) | §1, §7 | landed 2026-09-28 |
| S10 theme picker | [theme picker](docs/roadmap/tasks/v2-theme-picker.md) | §11.3 configuration surface | landed 2026-09-28 |
| S11 closeout | [closeout](docs/roadmap/tasks/v2-closeout.md) | §18 status, Evidence | landed 2026-09-28 |
| S12 Paper alignment | [Paper alignment](docs/roadmap/tasks/v2-paper-alignment.md) | §2, §7, §11.3–§11.5, §11.8, §11.10, §11.11, §18.2, §18.5 | landed 2026-09-28 |

## 19. Complete-browser blueprint (2026-09-28)

The Paper file “BBR — Browser Shell” holds the complete browser as a target
design ([packet](docs/roadmap/tasks/paper-browser-blueprint.md)): page 00
index and coverage, 10 foundations, 11 building blocks, 20–26 every surface
in light and dark, 30 the 23 process storyboards, 40 the variant matrices and
the motion specification, 90 the decisions and the delta ledger. The Paper
tokens mirror the palette, space-tint and theme code and are checked against
it (`scripts/paper_blueprint/verify_tokens.py`).

**Authority.** This document stays the contract. Boards marked *Shipped*
restate it; boards marked *Change* or *New · proposal* are targets. Patrick
adopted every row below on 2026-09-28 (“setz alles von Paper um”). An adopted
target becomes the specification of its owning section when its packet lands
that text with the implementation; until then the shipped sections still
describe the running app. Implementation order: D01 → D20 → D05 → D14 → D03 → D04 → D08 → D07 → D02 → D21 → D22 → D09 → D10 → D06 → D16 → D17 → D19 → D11 → D12 → D13 → D15 → D18 → D24 → D25 → D23.

| Delta | Target | Boards | Sections | Status | Packet |
|---|---|---|---|---|---|
| D01 | Footer chips start after Downloads | S-7 W-1 | §11.3 §11.4 §18.3 | implemented | [blueprint-footer-strip-leading](docs/roadmap/tasks/blueprint-footer-strip-leading.md) |
| D02 | Hover reveal of a hidden rail | W-3 V-6 | §2 §11.4 §11.14 | adopted | [blueprint-sidebar-hover-reveal](docs/roadmap/tasks/blueprint-sidebar-hover-reveal.md) |
| D03 | Address control states | S-1 M-8 | §11.5 §17.13 | adopted | [blueprint-address-states](docs/roadmap/tasks/blueprint-address-states.md) |
| D04 | ⌘T recents and the > actions mode | C-1 C-4 L-9 | §11.4 | adopted | [blueprint-palette-recents-actions](docs/roadmap/tasks/blueprint-palette-recents-actions.md) |
| D05 | Space menu and delete confirmation | P-4 PR-07 | §11.3 §11.13 | adopted | [blueprint-space-menu-delete](docs/roadmap/tasks/blueprint-space-menu-delete.md) |
| D06 | Create, edit and delete profiles | P-6 L-4 PR-08 | §12 | adopted | [blueprint-profile-management](docs/roadmap/tasks/blueprint-profile-management.md) |
| D07 | Cross-profile move feedback | P-7 PR-08 | §11.3 §12 §10.9 | adopted | [blueprint-cross-profile-move-feedback](docs/roadmap/tasks/blueprint-cross-profile-move-feedback.md) |
| D08 | Toasts and uniform surface transitions | B-7 G-7 V-6 PR-12 | §6 §18.4 §18.5 | adopted | [blueprint-feedback-toasts](docs/roadmap/tasks/blueprint-feedback-toasts.md) |
| D09 | Peek presentation | G-8 PR-04 | §11.17 | adopted | [blueprint-peek-presentation](docs/roadmap/tasks/blueprint-peek-presentation.md) |
| D10 | History and Clear Browsing Data | M-6 M-7 PR-22 | §11.16 | adopted | [blueprint-history-clear-data](docs/roadmap/tasks/blueprint-history-clear-data.md) |
| D11 | Site settings and permission prompts | M-8 M-9 L-6 PR-14 | §11.5 | adopted | [blueprint-site-settings-permissions](docs/roadmap/tasks/blueprint-site-settings-permissions.md) |
| D12 | Client certificate chooser | M-11 PR-15 | §18.4 | adopted | [blueprint-client-certificates](docs/roadmap/tasks/blueprint-client-certificates.md) |
| D13 | Open-in-app, pop-up and link-routing UI | M-12 M-13 PR-04 PR-20 | §11.13 §18.4 | adopted | [blueprint-link-destinations](docs/roadmap/tasks/blueprint-link-destinations.md) |
| D14 | Back / Forward history menu | W-7 M-3 | §11.14 §11.13 | adopted | [blueprint-navigation-history-menu](docs/roadmap/tasks/blueprint-navigation-history-menu.md) |
| D15 | First run and onboarding | L-1 L-2 PR-01 | §11.8 | adopted | [blueprint-first-run](docs/roadmap/tasks/blueprint-first-run.md) |
| D16 | Settings window | L-3 L-4 L-5 L-6 L-7 L-8 L-9 PR-23 | §3.4 §6 §8 §9 | adopted | [blueprint-settings-window](docs/roadmap/tasks/blueprint-settings-window.md) |
| D17 | Restore notice and recovery panes | L-10 L-11 PR-18 | §11.8 | adopted | [blueprint-restore-recovery-ui](docs/roadmap/tasks/blueprint-restore-recovery-ui.md) |
| D18 | About and updates | L-12 | §1 §7 | adopted | [blueprint-about-updates](docs/roadmap/tasks/blueprint-about-updates.md) |
| D19 | Menu bar, Dock menu and window lifecycle | L-13 L-14 W-9 PR-21 | §11.13 | adopted | [blueprint-menus-windows](docs/roadmap/tasks/blueprint-menus-windows.md) |
| D20 | New folders open in rename | S-4 PR-06 | §11.11 | implemented | [blueprint-folder-rename-on-create](docs/roadmap/tasks/blueprint-folder-rename-on-create.md) |
| D21 | Keyboard entry into the rail | PR-19 B-2 | §9 | adopted | [blueprint-rail-focus-shortcut](docs/roadmap/tasks/blueprint-rail-focus-shortcut.md) |
| D22 | Full-screen card rules under test | W-5 | §2 | adopted | [blueprint-fullscreen-evidence](docs/roadmap/tasks/blueprint-fullscreen-evidence.md) |
| D23 | Split view | G-9 G-10 PR-09 | §11.7 | adopted | [split-app-composition](docs/roadmap/tasks/split-app-composition.md) |
| D24 | Sidebar media card | S-8 PR-17 | §11.15 | adopted | [sidebar-media-control](docs/roadmap/tasks/sidebar-media-control.md) |
| D25 | Folder context menu (ledger correction: M-2 was marked Shipped, but no folder menu exists) | M-2 S-4 | §11.13 §11.11 | adopted without the closing row (question 13) | [folder-context-menu](docs/roadmap/tasks/folder-context-menu.md) |

Questions raised by the blueprint (board D-1), answered by the adoption as
drawn: 1 yes, 300 ms dwell · 2 yes · 3 leading edge · 4 short name + glyph ·
5 per profile with a filter · 6 last active space, 'Ask' optional · 7 yes ·
8 read-only in v1 · 9 yes, 9 % wash · 10 native · 11 still open (phase 6) ·
12 Safari and Chrome.

| # | Question | Delta |
|---|---|---|
| 1 | Adopt the 300 ms edge dwell (D02), or keep ⌘B-only? | D02 |
| 2 | Show recents on empty input (D04), or keep the empty palette quiet? | D04 |
| 3 | Leading edge as in Paper (D01), or keep the shipped centring and update Paper? | D01 |
| 4 | Short name + glyph in the address (D03 closes §17.13)? | D03 |
| 5 | History per profile with a profile filter (D10), or one merged list? | D10 |
| 6 | Default to the last active space, with 'Ask' as an option (D13)? | D13 |
| 7 | Introduce a toast for reversible actions (D08), or keep banners only? | D08 |
| 8 | Read-only shortcut list in v1 (D16)? | D16 |
| 9 | Dim the card with a 9 % wash under blocking sheets (B-6)? | D08 |
| 10 | Stay native NSToolTip everywhere? | — |
| 11 | Which shortcut opens a split (phase 6 open decision)? | D23 |
| 12 | Safari and Chrome only in v1 (D15)? | D15 |
| 13 | M-2 draws "Close Folder and Its Tabs…"; the 2026-09-27 settlement says no folder row closes tabs. Keep the row omitted? (Until answered, D25 ships Rename…, New Tab in Folder, Open All Tabs and "Delete Folder — keep its tabs".) | D25 |

## Evidence

- Token contrast ratios verified during authoring (WCAG 2.x relative
  luminance; §3 tables match the computed pairs).
- `just roadmap` and `just verify` green on the authoring commit (evidence in
  ROADMAP.md and the commit body).
- This document is contract; the placeholder chrome in `render.rs` is
  replaced by tokenized GPUI surfaces in the Phase-4 shell slice, with E2E
  asserting §15.
