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

## 1. Visual direction — "Chassis & Glass"

BBR's chrome is a **machined instrument panel** holding an **untampered glass**
(web content). The identity is that of precision drafting equipment — avionics
and camera bodies rather than consumer candy: a quiet, near-monochrome
aluminum/graphite chassis, hairline rules, tabular numerals, geometric icons,
and exactly one colored signal family (the **Phosphor** accent, a teal-green
instrument-light hue) plus a reserved set of semantic indicator hues.

Positioning paragraph: a keyboard-first daily driver for many long-lived
contexts needs a shell that recedes the moment content appears. Chassis
surfaces are matte, flat, layered by luminance and hairline rules — never
glassmorphism, never site-adaptive tinting, never thumbnails composited from
pages. Color is a scarce, meaningful resource: it marks selection, status,
profile identity, and focus, and nothing else. Web content is the only
animated, colorful surface on screen.

### What makes BBR visually distinct

| | Arc | Zen | Safari | Swift reference (PWBrowser) | **BBR** |
|---|---|---|---|---|---|
| Shell | floating pills, candy gradients, auto-hidden toolbar | adaptive per-site tint, glass blur, playful radius | uniform translucent toolbar, ambient page color | ad-hoc dark placeholder chrome | flat machined chassis, hairline rules, constant luminance across sites |
| Color | saturated gradient accents | site-adaptive pastels | ambient translucency | single hardcoded grays | monochrome + one Phosphor accent + semantic indicator hues |
| Tabs | large rounded chips | large rounded cards | compact tabs with favicon | none (text placeholder) | compact rule-based rows, accent rail for selection |
| Geometry | organic, large radii | organic | respectable | none | 4 pt grid, radii 6–10 px, square chips, 3 px rails |
| Motion | playful spring choreography | animated everywhere | subtle | none | commit-first snaps; animation only on transient indication |
| Content relation | sidebar overlays page | sidebar overlays page | toolbar merges | separate | sidebar is a physical rail; content is never overlapped or sampled |

The Swift reference is our functional ancestor (spaces, profiles, pinned tabs)
but intentionally has no visual identity; BBR replaces its placeholder lookup
table (currently hardcoded `rgb(0x…)` values in `crates/bbr-app/src/app/render.rs`)
with this token system in the same commit group that introduces the shell.

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
L0 canvas          — window base, content backdrop (glass area, empty/crash)
L1 sidebar/rail    — chrome surfaces: sidebar, panels      elev-0
L2 controls        — fields, buttons, rows inside chrome   elev-1
L3 floating        — command palette, peek, dialogs        elev-2
```

| Token | Light | Dark | Notes |
|---|---|---|---|
| `material:canvas` | `#F6F6F4` | `#121416` | content backdrop, visible only before attach / empty / crash |
| `material:sidebar` | `#EFEFEC` | `#1A1D21` | the physical rail; all chrome in v1 lives here |
| `material:field` | `#FFFFFF` | `#23272C` | command field, search, hover-raised rows |
| `material:overlay` | `#FFFFFF` | `#202429` | floating surfaces, elevation 2 |
| `material:glass` | `#FFFFFF` | `#151719` | what sits behind native web views; must never be textured or sampled |

### Elevation

- `elev:0` — base surface. No border beyond the sidebar rail's right hairline.
- `elev:1` — raised control (field, button, hover tile). `hairline` border,
  same material family. No shadow on rows, ever (list paint budget).
- `elev:2` — floating. Shadow tokens (GPUI `box_shadow`, bounded blur):

| Token | Light | Dark |
|---|---|---|
| `elev:2.shadow` | `0 1 0 rgba(0,0,0,0.04), 0 12 24 rgba(23,26,29,0.10)` | `0 0 0 1px rgba(255,255,255,0.05), 0 16 32 rgba(0,0,0,0.45)` |

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
| `color:ink` | `#1A1C1F` | `#ECEEF1` | `#000000` | `#FFFFFF` | primary text, titles |
| `color:ink.secondary` | `#555B63` | `#A6ACB5` | `#333A41` | `#D6DAE0` | meta, icons at rest, secondary tab text |
| `color:ink.faint` | `#949AA2` | `#6E747D` | `#5C626A` | `#8A919B` | disabled, placeholders, purely decorative — never normal text |
| `color:hairline` | `#E2E2DF` | `#2A2E34` | `#AFB2AC` | `#454C55` | structural rules |
| `color:border.strong` | `#C7C9C5` | `#3B4149` | `#9FA29E` | `#525A63` | field borders, tile outlines |

Contrast contract (recomputed by tests from tokens, not copied): `ink` ≥
14:1, `ink.secondary` ≥ 5.9:1, `accent-ink` ≥ 5.9:1 (light) / ≥ 9:1 (dark)
on their surfaces; status inks ≥ 4.5:1; `accent` rails/rings ≥ 3:1;
increased-contrast raises `ink.secondary` ≥ 9:1 and `hairline` ≥ 1.8:1.

The light tokens above were verified against the §3.1 contract when the
`crates/bbr-app/src/palette.rs` token gate shipped and corrected where the
authored values fell short: light `accent.ink` `#0B6B5C → #0A5C4E`,
light `attention` `#9C6500 → #8F5C00`, light-IC `ink.secondary`
`#3D434A → #333A41`, light-IC `hairline` `#C9CBC8 → #AFB2AC`. The token
gate recomputes every pair from the stored hex values.

### 3.2 Accent — Phosphor

| Token | Light | Dark | IC Light | IC Dark | Use |
|---|---|---|---|---|---|
| `color:accent` | `#0E8F79` | `#2AD4B6` | `#0B6B5C` | `#3DF0CE` | selected rail, focus ring, drag tick, split preview frame (UI ≥ 3:1) |
| `color:accent.ink` | `#0A5C4E` | `#3DF0CE` | `#0A5A4C` | `#3DF0CE` | accent-colored text (match highlight, mode chip, links) |

Usage rules: the accent marks **one thing at a time** — the selected tab rail,
a focused control, the active command result, the split target. It never
gradients, never glows, never shifts with page content.

### 3.3 Status and indicator hues

Status colors appear only on small glyph/dot surfaces **with a non-color
companion** (glyph, text, or position); color never carries status alone.

| Token | Light | Dark | Non-color companion |
|---|---|---|---|
| `color:status.error` | `#C22E24` | `#FF6A63` | ⚠/lifering glyph + error surface tint + (crash) panel copy |
| `color:status.attention` | `#8F5C00` | `#FFC63F` | pulsing dot (§6.3) + attention reason in status line |
| `color:status.ok` | `#177A55` | `#45C08A` | check glyph where used |
| `color:status.info` | `#2C5CC5` | `#8AA6FF` | ℹ glyph |

Surfaces: `status.*.surface` = status color at 10% over the owning material,
12% in increased contrast. The `error` **row** state and the `attention` dot
are the only two colored dot states; loading uses the accent, never amber.

### 3.4 Increase-contrast mode

Applies when macOS `Increase Contrast` is on (and is assertable via a
`contrast` snapshot field, §15). Changes: text steps per §3.1; hairline
strengthened; `accent` darkened/lightened per §3.2; selected tint deepens
20%; focus ring becomes 3 px with a 1 px material-gap double ring (§9);
tile washes deepen 7%; the crash/error and attention surfaces gain a 1 px
`border.strong` outline so tint is never required.

### 3.5 Selection tint

`color:tint.selected` = `accent` at 10% over material (light
`#D8E5E0` / dark `#1C3333`), 20% in increased contrast. The rail, not the
tint, is the primary selection cue; the tint is confirmation.

## 4. Typography

- **Family:** system UI (SF Pro on macOS) via GPUI's font-kit; monospace
  `SF Mono` for URLs, ids, and keystrokes. No custom fonts in v1.
- **Numerals:** tabular figures (`tnum`) for counts, timestamps, and any
  right-aligned tabular data.
- **Size steps:** 11, 12, 13, 14, 16, 24 pt. Bold steps: 400/500/600/700.
  Calibrated from the preserved S1 reference (§8): normal-density title lands
  at 14 pt so the effective tab/space text reads 14–15 pt.

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
| `radius:sm` | 6 | small tile, profile chip, S-density rows, S favorites tiles |
| `radius:md` | 8 | N/XL row tiles, buttons |
| `radius:lg` | 10 | command field, N palette, crash panel |
| `radius:xl` | 14 | XL palette |
| `radius:full` | 50% | dots, avatars |

Rule: chrome radii stay in 6–10 px; `radius:xl` (14) is reserved for the XL
palette sheet. No giant rounded sheets over content and no `radius` on
scrollable list backgrounds. Inset row surfaces (§10.2–§10.4) and footer
identity chips (§11.3) are bounded rounded rectangles from this scale —
never fully circular pills around text. The S1 calibration raised these values
with the rest of the scale but keeps the language.

- **Sidebar widths:** `win:sidebar.S` 232, `.N` 280, `.XL` 340 pt; adjustable
  between 200 and 400 pt via a 4 px drag hairline (cursor `ew-resize`),
  minimum width enforced; collapse (Cmd-B) yields 0 with the glass full-bleed
  (§11.4). **Native-content floor stays strictly positive:** the native slot
  keeps ≥ 64 pt (`CONTENT_MIN`, 2 × `sp:7`) for every window that can host
  the floor — the effective rail width is `min(requested, window_width − 64)`.
  At the shipped 1080 pt window the 280 pt normal rail leaves 800 pt of
  glass; in windows too narrow to host the requested width plus the §13
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
4. **Reduced Motion** (macOS system setting, assertable) removes all
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
| `ease:decelerate` | `cubic-bezier(0, 0, 0, 0.3)` |
| `load:indeterminate` | 580 ms linear loop (§11.5) |

Animated states and their permitted motion are enumerated in §10; anything
not listed there is `dur:instant`.

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
- **Fallback:** every icon context has a text alternative (label, tooltip, or
  aria-label). A missing functional glyph falls back to a 2-character
  `type:badge` chip, never to an arbitrary Unicode emoji. The deliberate
  exception is a space's user-selected identity in §11.3: that value is a
  validated member of the curated `SpaceIdentity` emoji/glyph catalog, not a
  fallback or substitute for a missing chrome icon.

### Favicon tiles (missing favicon rule)

When a tab has no favicon yet, render a deterministic monogram tile instead of
an empty box:

- 18 pt (S: 16, XL: 20) square, `radius:sm`, letter = first character of the
  registrable hostname (uppercase, `type:badge` size), on a desaturated
  slate-hue ramp derived from FNV-1a(hostname) mod 6.
- The slate ramp is separate from accent/status/profile hues so favicon tiles
  never impersonate identity or status.
- While loading, the tile slot shows the `loading` spinner (§10.5); after
  load, the real favicon replaces the monogram. Monograms are static — no
  spin, no pulse, no animation.

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
close control. The table below is the **authoritative scale**, decided from
that reference — not a provisional percentage and not a candidate. The
[global UI scale and density pass](docs/roadmap/tasks/global-ui-scale-density.md)
applies it verbatim through the central `Theme`/`TypeScale`/`Spacing`/`Density`
seams. BBR keeps its own Chassis & Glass materials, color discipline,
semantics, and native-web boundary; the references calibrated scale, rhythm,
and row affordance only. Dense-500 bounded-window rendering, scroll math,
semantic ids, interaction behavior, and every performance budget/sample count
are unchanged by this table (§15); the geometry below only raises token values.

| Token | S (small) | N (normal) | X (very large) |
|---|---|---|---|
| `density:row` | 32 | 36 | 44 |
| `density:row-tile` | 28 | 36 | 44 |
| `density:radius` | `radius:sm` (6) | `radius:md` (8) | `radius:lg` (10) |
| `type:title` | 12 | 14 | 16 |
| `icon:` | 16 | 18 | 20 |
| `density:space-chip` | 32 | 36 | 44 |
| `density:sidebar` | 232 | 280 | 340 |
| `density:hit-min` | 28 (close 24) | 32 (close 28) | 40 (close 32) |
| `density:row-inset` | `sp:1` (4) | `sp:2` (8) | `sp:3` (12) |
| `density:section-gap` | `sp:3` (12) | `sp:4` (16) | `sp:5` (20) |
| favorites grid gap | `sp:1` (4) | `sp:2` (8) | `sp:2` (8) |
| pinned rows | icon-only 32×32, gap 4 | icon + title | icon + title |

Transition rule: density changes when the collection crosses 200 regular tabs
(S) or 600 (X) only after user confirm at first occurrence or a follow-up
preference (open decision); the shell never flips density mid-scroll in a way
that changes row height — row heights are stable for the whole session view.

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
| Web-content focus | chrome renders "at ease": no ring anywhere, command cursor hidden, row hover washes active but no accent tint on the field; the selected-tab rail stays as the only accent |

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
Row wash transparent; text per role. **Assert:** `visual_state == default`.

### 10.2 `hover` (pointer over control)
Wash: light `rgba(0,0,0,0.045)` (`#E4E4E1`), dark `rgba(255,255,255,0.05)`
(`#25282C`); IC 7%. The wash paints an inset rounded row surface — inset
`density:row-inset` from the rail edges (S `sp:1` 4 / N `sp:2` 8 / XL `sp:3`
12), radius `density:radius` (S `radius:sm` / N `radius:md` / XL
`radius:lg`) — rather than a full-bleed band, so the trailing close
control sits inside that surface. Close button reveals on tab rows (fade-in
opacity `dur:micro`, allowed). Tint animates `dur:micro`. **Assert:** `hover`.

### 10.3 `active` (pointer down / pressed)
Wash deepens (light 8%, dark 8%) plus 1 px inner `hairline` on the same inset
rounded surface as §10.2 (`density:row-inset`, `density:radius`); `dur:micro`
tint. No translation or "press down" motion. **Assert:** `active`.

### 10.4 `selected`
`tint.selected` paints the inset rounded row surface of §10.2
(`density:row-inset`, `density:radius`) plus the accent rail 3 pt at the row's
leading edge (4 pt IC); title `ink`, meta `ink.secondary`. Snap
(`dur:instant`). In web-focus domain the rail remains the sole accent (see
§9). **Assert:** `selected`
(plus the domain via `chrome_focused`).

### 10.5 `loading`
Favicon slot swaps to an 18 pt circular arc spinner (S 16 / XL 20, accent,
rotating,
`dur` 640 ms linear — allowed, it is transient indication); tab title drops
to `ink.secondary`; a 2 pt indeterminate progress bar sweeps the top edge of
the content pane (`load:indeterminate`, opacity 1→0.6 loop). Selection/
hover visuals unchanged. **Assert:** `loading`.

### 10.6 `attention`
Amber dot (8 pt at the row's trailing state slot) + one two-beat opacity
pulse (`dur:emphasis`, 1→0.45→1) + a one-frame amber surface flash
(`status.attention.surface`, `dur:fine`). Non-color companions: the badge
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
`dragging`: the dragged row renders at 92% opacity with `elev:2` shadow and
a 1 pt `border.strong` outline (`dur:micro` opacity, allowed).
`drag-target`: insertion tick = 2 pt vertical accent rule at the target
index spanning the row tile, plus target tile outline 1 pt accent and
`tint.selected`. Cross-profile destinations additionally show the
destination profile chip replacing the insertion rule (§12.3). Split zones
reuse the accent outline language with the strip fills of §11.7.
**Assert:** `dragging`, `drag-target`.

### 10.10 `focused` (keyboard)
Ring per §9, label brightens one step. Same presentation frame as the
selection change (snap). **Assert:** `focused`.

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
- **States:** all §10 states apply; `selected` rail is the leading edge;
  `loading` spinner replaces the favicon; `attention` dot in the state slot
  (pinned rows include the state slot except at S).
- **Long title:** §4 truncation; tooltip on hover (Planned).
- **Assert ids:** `bbr-tab-row-<n>`, `bbr-tab-title-<n>`,
  `bbr-tab-favicon-<n>`, `bbr-tab-close-<n>`, `bbr-tab-state-<n>`.

### 11.2 Pinned tab

- **Anatomy:** pinned region renders above the regular region, separated by a
  `hairline` + `type:badge` "Pinned" label (label hidden when region empty).
  S density: icon-only 32×32 rows with 4 pt gap. N/XL: same row anatomy as
  regular rows.
- **Behavior:** pinned rows never shift order on selection; close is
  hover-reveal only; drag within the pinned region reorders; dragging to the
  regular region unpins (contract); a drag from the regular region into
  pinned pins it.
- **States:** same vocabulary; pinned rows additionally read a "locked
  position" identity: the favicon tile is never replaced by scroll chrome,
  and at S density the tile gets a subtle 1 pt `hairline` outline so
  icon-only rows are legible.
- **Assert ids:** `bbr-tab-pinned-<n>` (shared sub-ids `-title/-favicon/
  -close`).

### 11.3 Space identity chips and profile context

- **Placement:** spaces are always available in the §11.4 footer strip. There
  is no revealable space-row region above Favorites and no `[Spaces]` toggle.
- **Anatomy:** each space is one keyboard-focusable identity chip containing
  exactly one validated `SpaceIdentity` value from the curated appearance
  contract: either a selected emoji or a BBR icon glyph. The chip has a stable
  accessible name containing the space and profile names; color or emoji alone
  never carries that meaning. Normal S1 geometry is calibrated with the same
  generous hit-target family as the rest of the footer.
- **Behavior:** pointer click or keyboard activation performs the existing warm
  space switch. The active space is unmistakable through a persistent selected
  surface plus focus-independent outline/rail; inactive chips remain quiet.
  Activation keeps the selected tab/session warm and preserves every existing
  50/100 ms budget and fast-but-wrong guard.
- **Appearance:** each space owns one curated theme-color token and one identity.
  The active token tints the BBR-owned sidebar material through a contrast-safe,
  desaturated chrome variant. Web content is never sampled or tinted. During a
  direct horizontal swipe over the sidebar/space strip, the sidebar tint may
  interpolate between source and candidate target tokens in direct proportion
  to gesture progress; cancellation returns to the source token and commit
  selects the target. The native web view itself never cross-fades or
  interpolates.
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

### 11.4 Sidebar

- **Anatomy (top → bottom):** brand block (app mark + name) `sp:4`
  padding (`bbr-brand`); command field (§11.5) directly under; Favorites grid
  (§11.10); Pinned region (§11.2); Folders (§11.11); Tabs region (§11.1);
  status line (`bbr-status`, copy unchanged); footer row at the very bottom —
  exactly `[Downloads icon] [horizontally scrollable Spaces strip, flex:1]
  [Plus icon]` (`bbr-downloads`, `bbr-space-strip`, `bbr-plus`), separated
  from the list above by a `hairline`. The former `[Spaces]` reveal-toggle and
  its upper space-row region do not exist in the target composition. The
  current pre-G1a shell keeps its existing trailing status-row collapse control
  while fresh F2a lands; the bounded §11.14 titlebar packet later performs one
  atomic cutover to `bbr-titlebar-sidebar-toggle` and removes the duplicate
  status-row control. F2a does not implement or anticipate that later cutover.
- **Regions:** section labels use `type:badge` and are hidden when their
  region is empty (`bbr-section-favorites` / `bbr-section-pinned` /
  `bbr-section-folders` / `bbr-section-tabs`); profile groups and regions
  stack under hairlines separated by `density:section-gap` breathing (S
  `sp:3` / N `sp:4` / XL `sp:5`), with no repeated headers. The Favorites grid and
  the Pinned region are fixed-height blocks above the scroll view; Folders
  and Tabs scroll together in `bbr-scroll` (label behavior per §11.6).
- **Spaces strip:** `bbr-space-strip` is the permanent space navigation
  surface defined by §11.3. It takes all remaining footer width, clips to one
  line, scrolls horizontally, and exposes one `bbr-space-chip-<n>` per space.
  It is not a disclosure control and never changes the vertical region stack.
  Pointer click, keyboard activation, and horizontal trackpad swipe route
  through the same space-switch authority and preserve warm-session budgets.
- **Footer:** `[Downloads]` opens the downloads surface (Phase 8; the fixed
  icon control exposes a text alternative and a `type:meta` `tnum` count when
  content exists). The middle flex region is the permanent Spaces strip.
  `[+]` opens the **plus menu** (Planned) — a real context menu
  (`bbr-plus-menu`, `elev:2`, `material:overlay`, `radius:md`) with exactly
  two items in v1: **New Space** (`bbr-plus-new-space`) and **New Folder**
  (`bbr-plus-new-folder`). New Space creates a real space through the
  controller; New Folder creates a session-local folder (§11.11). The menu
  is keyboard-operable (`↑`/`↓`, Enter commits, Escape dismisses without
  action), closes on outside pointer press (one pointer action, §13), and
  the chosen action preserves the invoking focus domain per the
  interaction contract.
- **Behavior:** width per §5 (`win:sidebar.*`), draggable 4 pt hairline
  (`bbr-sidebar-divider`), collapse Cmd-B to 0 **instantly** (performance
  contract next-frame; the opening glass gets no animation), expand
  restores the previous width. Collapse memory is **global in v1** — one
  last-expanded width per window, restored on expand; per-space memory is
  an open decision (§17). Divider keyboard focus provides `←`/`→` resize
  and `0` reset (DESIGN §9 pointer/keyboard parity). Resize commits
  geometry on the next frame; drag previews no intermediate chrome states.
- **Command surface launch:** the command field owns two presentations of
  the same §11.5 engine:
  - **Cmd-T → centered palette** (Planned, `bbr-palette`): `elev:2`
    overlay centered over the window content, `material:overlay`, width
    `win:palette` (S 440 / N 520 / XL 620 pt), `radius:lg` at S/N /
    `radius:xl` at XL per §5. Field row on top (`bbr-palette-field`):
    prefix glyph, `type:mono` text (`bbr-palette-text`), mode chip
    (`bbr-palette-mode` — New Tab staged per the interaction contract),
    block cursor (`bbr-palette-cursor`). Results below
    (`bbr-palette-result-<k>`) use the §11.5 row anatomy `[icon] [title]
    [url mono trunc] [key hint]`, `selected` visuals (accent rail inside
    the result list), and `accent.ink` match highlighting.
  - **Cmd-L / pointer click on the address control → palette over the
    address** (Planned, `bbr-command-overlay`): `elev:2` overlay that
    **covers** the §11.5 address control rather than dropping below it —
    top edge flush with the control's top edge, leading edge flush with
    the control's leading edge, width = control width clamped to
    360–`win:palette`, `material:overlay`, `radius:lg`. The address
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
- **Assert ids:** `bbr-sidebar`, `bbr-sidebar-column`, `bbr-brand`,
  `bbr-scroll`, `bbr-status`, `bbr-sidebar-toggle`, `bbr-sidebar-divider`,
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
- **Anatomy:** a borderless pill, height `density:row` (S 32 / N 36 /
  XL 44), `radius:lg`, `type:title` domain text (`bbr-address-domain`,
  mid-ellipsis), and a **trailing accessory slot** (`bbr-address-accessory`)
  inset at the pill's trailing edge. The slot reserves space for one to
  three §14 glyphs at `density:hit-min`; when a glyph appears, the domain
  text yields width by ellipsis — the address never pushes an accessory out
  of the pill. v1 ships the slot empty or with Copy Link only (§17.14).
- **Fill:** translucent, not a bordered field — **white-translucent on dark
  chrome, black-translucent on light chrome**, following the theme and the
  space tint through the token pair, never a hardcoded color. **No border
  and no focus ring**, at any time.
- **Hover:** the fill moves one step brighter on dark chrome and one step
  darker on light chrome, using the §10.2 wash mechanics at `dur:fine`.
- **Activation:** pointer press or `Cmd-L` opens the §11.4 palette over the
  control. The control **never accepts keyboard text**; its semantic role is
  a button that opens address entry, and it holds no editing state, no
  cursor and no selection.
- **Focus:** the control participates in the chrome focus domain as a
  button. There is no field-focus appearance, because there is no field.
  After the palette closes, focus returns where it was before opening — web
  content or the sidebar — per the interaction contract.
- **Assert ids:** `bbr-address`, `bbr-address-domain`,
  `bbr-address-accessory`; the palette's own input carries
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
  `material:canvas` with a centered 64 pt chassis mark (drawn glyph, not
  emoji), headline `type:display` "No tabs in this space", a line naming the
  profile ("in Profile · Work"), and one primary button "New Tab  ⌘T" that
  focuses the command field in NewTab mode; Escape returns to the glass pane.
  Closing the final tab already opens the empty New-Tab command surface
  (contract); this pane is what the user returns to after Escape.
  **Assert:** `bbr-empty-new-tab`, `bbr-crash-*` absent.
- **Crash panel:** replaces the glass area of the crashed tab (native view
  hidden) with `material:field`, `elev:1`: 32 pt lifering glyph in
  `status.error`, title "This page crashed", copy "The tab stays open; reload
  to restore it.", buttons `[Reload]` (primary, accent) and `[Close tab]`
  (secondary); focus order: Reload → Close tab → sidebar. The tab remains
  selected and listed per the interaction contract; no auto-reload in v1
  (open decision). **Assert:** `bbr-crash-panel`, `bbr-crash-reload`,
  `bbr-crash-close`; crashed tab row carries `error` state.
- Both panes paint flat GPUI surfaces; they never render web content or
  screenshots.

### 11.9 Missing favicon and long titles

Covered by §7.2 (monogram tile) and §4 (truncation). Additional edge rules:
empty title falls back to the hostname; empty host falls back to the URL
scheme; a title consisting only of whitespace is treated as empty.

### 11.10 Favorites grid (Planned)

- **Anatomy:** a `type:badge` "Favorites" label (`bbr-section-favorites`,
  hidden when empty) over a grid of square destination tiles
  (`bbr-favorites`): the §7.2 favicon/monogram tile centered in a
  `radius:sm` tile at `density:row-tile` size (S 28 / N 36 / XL 44), gap
  `sp:1` (S) / `sp:2` (N/XL), fixed per-density column count (S 5 / N 4 /
  XL 3) so the tile →
  index mapping is deterministic at every sidebar width.
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
  `bbr-fav-<n>`, `bbr-fav-icon-<n>`.

### 11.11 Folders (Planned)

- **Anatomy:** a `type:badge` "Folders" label (`bbr-section-folders`,
  hidden when empty) over folder rows (`bbr-folders`): `[disclosure glyph
  `icon:m` (18; S `icon:s` 16) §7] [name, trunc] [count, `type:meta`
  `tnum`]` at `density:row` tall,
  hairlines between folder groups; nested rows indent one `sp:3` step.
- **Behavior:** a folder is a session-local container of tab identities
  (in-memory in v1, never persisted; Phase 7 owns restoration). Toggling
  (`bbr-folder-toggle-<n>`, pointer or keyboard Enter/Space)
  expands/collapses the nested row list and snaps (`dur:instant`). Nested
  rows reuse the §11.1 row anatomy and the full §10 vocabulary; index
  `<m>` is the nested position within folder `<n>` (0-based,
  deterministic). A folder whose tabs were all closed remains as an empty
  collapsible row.
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
  `-favicon`/`-close`).

### 11.12 Downloads panel (Planned)

The fixed footer `[Downloads]` control (§11.4) opens the downloads surface. It
is an **anchored popover** (`bbr-downloads-panel`), not a sidebar region and
not an above-glass surface: `elev:2` over `material:overlay` with the palette's
shadow class (§2), `radius:lg`, anchored so its bottom edge is flush with the
footer control's top edge and its leading edge with the sidebar column's
leading edge. Width = the sidebar column's current width − 2 ×
`density:row-inset` (S `sp:1` 4 / N `sp:2` 8 / XL `sp:3` 12), clamped to stay
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
    succeeded shows the `status.ok` check glyph; failed the §10.7
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
- **Focus domain:** while open the panel is chrome focus (ring per §9; the
  selected-tab rail stays the only accent). Opening preserves the active
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
uses its own Chassis & Glass tokens, copy, product capabilities, profile model,
and command authorities; it does not copy Zen/Firefox features or styling.

- **Material and geometry:** production BBR menus use `material:overlay`,
  `elev:2`, `radius:md`, `type:label`, and hairline group separators. Native
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

### 11.14 Unified titlebar navigation controls (Planned)

The post-F2a/G1a titlebar target is
`[traffic lights] [sidebar collapse/expand] … [Back] [Forward] [Reload]`
inside the actual macOS safe titlebar layout guide.

- The collapse control sits immediately after the traffic lights, uses the same
  sidebar-width/collapse authority as Cmd-B, changes glyph for collapsed versus
  expanded, and replaces rather than duplicates the pre-G1a status-row control.
- Back/Forward/Reload form a compact trailing cluster and target the selected
  tab in single-pane mode or the focused split pane. Enablement derives from
  real navigation state.
- Glyphs use the BBR geometric 18 pt family with normal-density hit areas at
  least 32 pt, transparent rest state, §10.2 hover wash, and §10.8 disabled
  treatment. No bordered toolbar capsule and no duplicate controls in the
  address display.
- Layout uses the native safe area; traffic-light x/y values are never
  hardcoded. Empty titlebar space remains draggable. Fullscreen,
  multi-display, narrow-window, and collapsed-sidebar states must preserve
  reachability without overlap.
- Stable ids: `bbr-titlebar`, `bbr-titlebar-sidebar-toggle`, `bbr-nav-back`,
  `bbr-nav-forward`, `bbr-nav-reload`.

Canonical packet: [Unified titlebar navigation controls](docs/roadmap/tasks/titlebar-navigation-controls.md).

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

- **Palette result row:** `[bookmark glyph] [title + URL] [collection label]`.
  Title uses `type:title` with the user label when present, otherwise the
  captured title, then hostname/URL fallback; URL uses `type:mono` and the §4
  mid-ellipsis; the optional one-level collection label uses `type:meta`.
  Default/hover/selected/focused states use
  the existing command-row language; the selected result keeps the accent rail
  and no Saved Page-specific color family is introduced.
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
- **Library row:** bookmark/fallback glyph or normal favicon presentation,
  user label or captured-title fallback, mid-ellipsized URL, optional
  collection label, and selection state.
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
  seam; gallery registry id 22 is reserved for the frozen G2-TAB Tab Row
  story and is released during the semantic union). Gallery renderability
  never proves persistence, restart, command activation, or bounded product
  performance.

Canonical packets: [B1 contract/persistence](docs/roadmap/tasks/saved-pages-portable-persistence.md),
[B2 actions/command provider](docs/roadmap/tasks/saved-pages-actions-command.md),
and [B3 Library](docs/roadmap/tasks/saved-pages-library.md).

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

- Chrome is a left rail; the remainder is the native WKWebView child. At
  rest there is exactly one boundary: the sidebar's right `hairline`. No
  chrome shadows, glows, or gradients may intrude over web content.
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
  intermediate chrome geometry is ever shown.

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
  scroll math, or any performance budget/sample count.
- **State vocabulary:** snapshot field `tab_states: Vec<VisualState>` with
  `VisualState ∈ {default, hover, active, selected, loading, attention,
  error, disabled, dragging, drag-target, focused}` — exactly the §10 names.
  During a drag report both the source row (`dragging`) and any target
  (`drag-target`); the command-bar domain derives from the existing
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
offline copies, tags, nested collections, or page snapshots; peek choreography
(its own task).

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
14. **Address accessory slot contents:** §11.5 defines the trailing slot and
    its geometry. Decide what v1 actually shows — nothing, Copy Link only,
    or Copy Link plus a permissions indicator once the §P8a contract has a
    surface.

Resolving an item means moving its decision into the relevant section above
and updating the affected roadmap track in the same group.

## Evidence

- Token contrast ratios verified during authoring (WCAG 2.x relative
  luminance; §3 tables match the computed pairs).
- `just roadmap` and `just verify` green on the authoring commit (evidence in
  ROADMAP.md and the commit body).
- This document is contract; the placeholder chrome in `render.rs` is
  replaced by tokenized GPUI surfaces in the Phase-4 shell slice, with E2E
  asserting §15.
