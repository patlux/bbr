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
| Geometry | organic, large radii | organic | respectable | none | 4 pt grid, radii 4–8 px, square chips, 3 px rails |
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
- **Size steps:** 10, 11, 12, 13, 15, 22 pt. Bold steps: 400/500/600/700.

| Role | Size | Weight | Density S | N | XL | Notes |
|---|---|---|---|---|---|---|
| `type:display` | 22 | 600 | − | 22 | 24 | empty-state headline only, leading 1.2 |
| `type:title` | 13 | 500 | 11 | 13 | 15 | tab title, space name; leading 1.2 |
| `type:label` | 12 | 500 | 11 | 12 | 13 | buttons, nav labels, badges' base |
| `type:meta` | 11 | 400 | 10 | 11 | 12 | status, counts, timestamps, `tnum` |
| `type:mono` | 12 | 400 | 11 | 12 | 13 | URL in location mode and results |
| `type:key` | 11 | 500 | 10 | 11 | 12 | shortcut hints, `SF Mono` |
| `type:badge` | 10 | 600 | 10 | 10 | 11 | section labels "Pinned"/"Tabs", uppercase, tracking +0.06em |

### Truncation contract

- **Tab titles / space names:** one line, end-ellipsis; prefer cutting after
  the first `" – "`, `" - "`, or `" | "` separator when present so the
  site name survives; otherwise plain end-ellipsis. Hover shows the full title
  via native tooltip (Planned until a tooltip surface exists).
- **URLs (location mode, command results):** *mid-ellipsis* — always keep
  `scheme://host` and the final path segment; ellipsize the middle:
  `https://github.com/…/pull/1234`. Host is never truncated.
- **Favicon tile:** never truncates (fixed 16 px, §7.2).
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
| `radius:sm` | 4 | small tile, profile chip, S-density rows |
| `radius:md` | 6 | N/XL row tiles, buttons |
| `radius:lg` | 8 | command field, N palette, crash panel |
| `radius:xl` | 12 | XL palette |
| `radius:full` | 50% | dots, avatars |

Rule: chrome radii stay in 4–8 px; no pill-shaped cards, no giant rounded
sheets over content, no `radius` on scrollable list backgrounds.

- **Sidebar widths:** `win:sidebar.S` 216, `.N` 260, `.XL` 320 pt; adjustable
  between 200 and 360 pt via a 4 px drag hairline (cursor `ew-resize`),
  minimum width enforced; collapse (Cmd-B) yields 0 with the glass full-bleed
  (§11.4). In windows too narrow to host the requested width plus the §13
  content floor, the rail yields below the minimum so the native slot never
  goes zero or negative.

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
- **Sizes:** `icon:s` 14, `icon:m` 16, `icon:l` 20. Hover affordances and
  close buttons: hit area ≥ 24 pt at N/XL, ≥ 20 pt at S, glyph optically
  centered in the hit square.
- **Fallback:** every icon context has a text alternative (label, tooltip, or
  aria-label). A glyph missing from the set falls back to a 2-character
  `type:badge` chip, never to a Unicode emoji.

### Favicon tiles (missing favicon rule)

When a tab has no favicon yet, render a deterministic monogram tile instead of
an empty box:

- 16 pt (S: 14) square, `radius:sm`, letter = first character of the
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

| Token | S (small) | N (normal) | X (very large) |
|---|---|---|---|
| `density:row` | 28 | 32 | 40 |
| `density:row-tile` | 26 (gutter 1) | 28 (gutter 2) | 36 (gutter 2) |
| `density:radius` | `radius:sm` | `radius:md` | `radius:md` |
| `type:title` | 11 | 13 | 15 |
| `icon:` | 14 | 16 | 18 |
| `density:space-row` | 30 | 34 | 44 |
| `density:sidebar` | 216 | 260 | 320 |
| `density:hit-min` | 28 (close 20) | 32 | 40 |
| pinned rows | icon-only 28×28, gap 2 | icon + title | icon + title |

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

- **Order:** command field → spaces (only while the §11.4 spaces region
  is revealed) → favorites grid (row-major) → pinned → folders (expanded
  rows, then their nested rows) → regular tabs → footer row → (after the
  last chrome control) focus moves into the native web view; Shift-Tab from
  web content returns to the first sidebar control. Section labels and the
  brand are skipped. Both directions verified by real-app focus E2E (a11y
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
(`#25282C`); IC 7%. Close button reveals on tab rows (fade-in opacity
`dur:micro`, allowed). Tint animates `dur:micro`. **Assert:** `hover`.

### 10.3 `active` (pointer down / pressed)
Wash deepens (light 8%, dark 8%) plus 1 px inner `hairline`;
`dur:micro` tint. No translation or "press down" motion. **Assert:** `active`.

### 10.4 `selected`
Accent rail 3 pt at the row's leading edge (4 pt IC) + `tint.selected`;
title `ink`, meta `ink.secondary`. Snap (`dur:instant`). In web-focus
domain the rail remains the sole accent (see §9). **Assert:** `selected`
(plus the domain via `chrome_focused`).

### 10.5 `loading`
Favicon slot swaps to a 14 pt circular arc spinner (accent, rotating,
`dur` 640 ms linear — allowed, it is transient indication); tab title drops
to `ink.secondary`; a 2 pt indeterminate progress bar sweeps the top edge of
the content pane (`load:indeterminate`, opacity 1→0.6 loop). Selection/
hover visuals unchanged. **Assert:** `loading`.

### 10.6 `attention`
Amber dot (6 pt at the row's trailing state slot) + one two-beat opacity
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

- **Anatomy:** `[favicon 16] · [title flex, trunc] · [state slot 8] ·
  [close 16, hover-reveal]`, padding-x `sp:2` (S) / `sp:2` (N) / `sp:3` (XL),
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
  S density: icon-only 28×28 rows with 2 pt gap. N/XL: same row anatomy as
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

### 11.3 Space and profile rows

- **Anatomy:** `[profile chip 20 §12] · sp:2 · [name, trunc] · [count,
  `type:meta` `tnum`]`, `density:space-row` tall.
- **Behavior:** click activates the space (contract: preserves warm selected
  tab); the active space shows the accent rail like `selected` tabs; profile
  groups stack under a `hairline` with no repeated profile header in v1 (the
  chip is always present on each row — identity must be readable per row).
- **Cross-context rule:** when the shell presents tabs/spaces of more than
  one profile in the same sidebar, each row keeps its chip and groups are
  separated by hairlines; switching spaces renders the destination profile
  chip at the top of the command field for one interaction (a `type:label`
  "in Profile · Work" hint) — the interaction contract requires communicating
  the destination profile.
- **Assert ids:** `bbr-space-row-<n>`, `bbr-profile-chip-<n>`.

### 11.4 Sidebar

- **Anatomy (top → bottom):** brand block (app mark + name) `sp:4`
  padding (`bbr-brand`); command field (§11.5) directly under; spaces
  region (§11.3) — hidden by default in the compact rail, revealed by the
  footer `[Spaces]` control; Favorites grid (Planned, §11.10); Pinned
  region (§11.2); Folders (Planned, §11.11); Tabs region (§11.1); status
  line (`bbr-status`, copy unchanged) carrying the collapse control at its
  trailing edge (`bbr-sidebar-toggle`); footer row at the very bottom —
  exactly `[Downloads] [Spaces] [+]` (`bbr-downloads`, `bbr-spaces`,
  `bbr-plus`), separated from the list above by a `hairline`.
- **Regions:** section labels use `type:badge` and are hidden when their
  region is empty (`bbr-section-favorites` / `bbr-section-pinned` /
  `bbr-section-folders` / `bbr-section-tabs`); profile groups and regions
  stack under hairlines with no repeated headers. The Favorites grid and
  the Pinned region are fixed-height blocks above the scroll view; Folders
  and Tabs scroll together in `bbr-scroll` (label behavior per §11.6).
- **Spaces region:** the footer `[Spaces]` control toggles the §11.3 space
  rows between hidden (compact default) and revealed (rendered directly
  under the command field, above Favorites). Toggling snaps
  (`dur:instant`); row ids, §12 profile chips, and activation behavior are
  unchanged. Escape or a second activation hides it; while hidden, space
  rows are not part of the §9 traversal order.
- **Footer:** `[Downloads]` opens the downloads surface (Phase 8; the
  control shows a §7 glyph + label and a `type:meta` `tnum` count when the
  surface has content). `[Spaces]` toggles the spaces region (above).
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
    `win:palette` (S 400 / N 480 / XL 560 pt), `radius:lg` at S/N /
    `radius:xl` at XL per §5. Field row on top (`bbr-palette-field`):
    prefix glyph, `type:mono` text (`bbr-palette-text`), mode chip
    (`bbr-palette-mode` — New Tab staged per the interaction contract),
    block cursor (`bbr-palette-cursor`). Results below
    (`bbr-palette-result-<k>`) use the §11.5 row anatomy `[icon] [title]
    [url mono trunc] [key hint]`, `selected` visuals (accent rail inside
    the result list), and `accent.ink` match highlighting.
  - **Cmd-L / pointer click on the field → field-anchored overlay**
    (Planned, `bbr-command-overlay`): `elev:2` overlay anchored to the
    field — top edge flush with the field's bottom edge, leading edge
    flush with the field's leading edge, width = field width clamped to
    320–`win:palette`, `material:overlay`, `radius:lg`. Same result
    engine and row anatomy; results keep the §11.5 ids
    (`bbr-command-result-<k>`). Location mode shows the full URL selected
    in the field (`bbr-command-text`).
  - Both open with an opacity fade `dur:fine` only — never translate or
    scale — close on Escape/commit/outside press (one pointer action,
    §13), and are the only chrome allowed to overlap the glass (§13). At
    most one command surface is open at a time; opening either clears the
    §11.3 profile hint (interaction contract). Result selection snaps
    (`dur:instant`).
- **500-tab rule:** the sidebar remains responsive with 500 persisted tabs
  (§11.6); sidebar open/close/toggle and the palette surfaces submit to
  the same latency budgets.
- **Assert ids:** `bbr-sidebar`, `bbr-sidebar-column`, `bbr-brand`,
  `bbr-scroll`, `bbr-status`, `bbr-sidebar-toggle`, `bbr-sidebar-divider`,
  `bbr-section-favorites` / `bbr-section-pinned` / `bbr-section-folders` /
  `bbr-section-tabs`, `bbr-favorites`, `bbr-fav-<n>` (+ `-icon`),
  `bbr-folders`, `bbr-folder-row-<n>` (+ `-name`/`-count`/`-toggle`),
  `bbr-folder-item-<n>-<m>` (+ `-title`/`-favicon`/`-close`),
  `bbr-downloads`, `bbr-spaces`, `bbr-plus`, `bbr-plus-menu`,
  `bbr-plus-new-space`, `bbr-plus-new-folder`, `bbr-palette`
  (+ `-field`/`-text`/`-mode`/`-cursor`), `bbr-palette-result-<k>`,
  `bbr-command-overlay`; space rows keep `bbr-space-list`,
  `bbr-space-row-<n>`, `bbr-space-new`.

### 11.5 Command bar

- **Anatomy:** `material:field`, `radius:lg`, `elev:1`; prefix glyph (mode:
  `►` location / `+` new tab / `⌘` commands — Planned), mono text
  (`type:mono`, mid-ellipsis), mode chip (`type:badge`, accent.ink) when in
  a transient mode, block cursor 2 pt accent when chrome-focused.
- **Behavior:** modes per the interaction contract (`Inactive`, `Location`,
  `NewTab` staged); typing never scrolls or shifts the field; Enter commits,
  Escape restores (§contract). Results (Planned surface): rows `[icon] [title]
  [url mono trunc] [key hint]`; matched substring in `accent.ink`; selected
  result row uses `selected` visuals (accent rail inside the result list).
- **Focus in/out:** `Cmd-L`/`Cmd-T` focus the command field with ring; focus
  out returns to web content or sidebar per contract. The two palette
  presentations — the centered Cmd-T palette and the Cmd-L/click
  field-anchored overlay — are specified in §11.4: both open as `elev:2`
  overlays with an opacity fade `dur:fine` — never translate or scale — and
  they are the only chrome allowed to overlap the glass (§13).
- **Assert ids:** `bbr-command-bar`, `bbr-command-text`, `bbr-command-mode`,
  `bbr-command-cursor`, `bbr-command-result-<k>`.

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
  `radius:sm` tile at `density:row-tile` size (S 26 / N 28 / XL 36), `sp:1`
  gap, fixed per-density column count (S 5 / N 4 / XL 3) so the tile →
  index mapping is deterministic at every sidebar width.
- **Behavior:** clicking a tile activates that destination (open-vs-focus
  semantics is an open decision); hover/active washes follow §10.2/§10.3;
  keyboard traversal follows the §9 order row-major; tiles never reorder
  during selection.
- **States:** `default`, `hover`, `active`, `focused` (§10). Favorites are
  destinations, not tabs, so tab lifecycle states (§10.5–§10.7) do not
  apply; the destination name is the tile's semantic name (tooltip is
  Planned per §17.2).
- **Data source:** how favorites are added/removed (tab context menu, drag
  into the grid) is an open decision; the grid contract covers rendering
  and activation only. Data ownership (global vs per-profile vs per-space)
  is resolved by the favorites domain slice, not by this visual contract.
- **Assert ids:** `bbr-section-favorites`, `bbr-favorites`,
  `bbr-fav-<n>`, `bbr-fav-icon-<n>`.

### 11.11 Folders (Planned)

- **Anatomy:** a `type:badge` "Folders" label (`bbr-section-folders`,
  hidden when empty) over folder rows (`bbr-folders`): `[disclosure glyph
  14 §7] [name, trunc] [count, `type:meta` `tnum`]` at `density:row` tall,
  hairlines between folder groups; nested rows indent one `sp:2` step.
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

## 12. Profile identity

- **Chip:** 20 pt square (18 at S), `radius:sm`, filled with the profile's
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
sheets; no emoji in chrome; no glow shadows; no cover-flow/thumbnail tabs;
no site-adaptive chrome tinting; no animated content behind chrome; no
emoji/filled-duotone icon substitutes; no micro-interaction choreography at
60 fps. Where a generic pattern is tempting, the machine answer is always a
hairline, a numeral, or silence.

## 15. Automated semantic assertions

Real-app E2E and the testkit snapshot must assert the visual system
semantically — never by pixel comparison.

- **Stable ids:** the `bbr-*` ids enumerated in §11 (plus `bbr-root`,
  `bbr-sidebar`, `bbr-command-bar`, already present in the placeholder
  shell). Every interactive control carries one.
- **State vocabulary:** snapshot field `tab_states: Vec<VisualState>` with
  `VisualState ∈ {default, hover, active, selected, loading, attention,
  error, disabled, dragging, drag-target, focused}` — exactly the §10 names.
  During a drag report both the source row (`dragging`) and any target
  (`drag-target`); the command-bar domain derives from the existing
  `chrome_focused` + `command_bar_mode` fields.
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
custom fonts; emoji iconography; page thumbnails/previews; spring physics;
auto-reload of crashed tabs; sidebar auto-collapse; density auto-switch;
palette result grouping polish; peek choreography (its own task).

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
