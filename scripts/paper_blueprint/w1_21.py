import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from shell import *

which = sys.argv[1:]


def frag(t, html, label, note="", w=300, pad="12px 0 10px 0", h=None):
    hh = f"height:{h}px;" if h else ""
    return (f'<div layer-name="{esc(label)}" style="display:flex;flex-direction:column;gap:8px;width:{w}px;flex-shrink:0">'
            f'<div style="display:flex;flex-direction:column;gap:2px;min-height:34px">{text(label, "badge", DOC.c("ink-secondary"), clamp=False)}'
            + (text(note, "meta", DOC.c("ink-faint")) if note else "")
            + f'</div><div style="position:relative;display:flex;flex-direction:column;{hh}padding:{pad};border-radius:12px;overflow:hidden;background-color:{t.c("sidebar")};'
            f'box-shadow:0 0 0 1px {DOC.c("hairline")}">{html}</div></div>')


def frag_rows(b, items, gap=28, wrap=True):
    """items: fn(t) -> list of frag html; one row per theme."""
    for t in (LIGHT, DARK):
        b.caption(b.column(b.row(gap=0, pad="0 56px 12px 56px"), name="cap"), t.label())
        row = b.row(gap=gap, wrap=wrap, name="Fragments · " + t.label())
        for html in items(t):
            b.add(html, row)


def rb(t, rows, pad="0 8px"):
    return f'<div style="display:flex;flex-direction:column;gap:4px;padding:{pad}">{"".join(rows)}</div>'


if "S-1" in which:
    b = Board("S-1", width=2200, desc="A display control, not a text field: the domain only, 500 in ink, on a white veil over the rail with a 1 pt presence edge. A trailing accessory slot holds the zoom chip. Pressing it or ⌘L opens the one real URL input, the ⌘L overlay (C-6).",
              ids="bbr-address, bbr-address-domain, bbr-address-accessory, bbr-address-zoom, bbr-address-progress")
    def items(t):
        a = lambda **k: address(t, **k)
        return [frag(t, a(domain="x.com"), "Rest"), frag(t, a(domain="x.com", state="hover"), "Hover", "veil one step darker (light) / brighter (dark)"),
                frag(t, a(domain="x.com", state="pressed"), "Pressed"), frag(t, a(domain="x.com", state="focused"), "Keyboard focus", "button role; no caret"),
                frag(t, a(domain="github.com", zoom="140%"), "Zoom accessory", "shown while zoom ≠ 100 %; press resets"),
                frag(t, a(domain="dash.cloudflare.com", progress=55), "Loading (D5)"),
                frag(t, a(domain="router.local", secure=False), "Not secure", "http: attention glyph + domain (New · proposal)"),
                frag(t, a(domain="sub.very-long-domain…example.co.uk"), "Long domain", "mid-ellipsis; never the TLD"),
                frag(t, a(domain="münchen.de"), "IDN", "shown human-readable"),
                frag(t, a(domain="Library", glyph_name="bookmark"), "Internal page", "short name + glyph (§17.13 proposal)"),
                frag(t, a(domain="Settings", glyph_name="gear"), "Internal page"),
                frag(t, a(domain="report-q3.pdf", glyph_name="file"), "Local file", "file name, no path")]
    frag_rows(b, items)
    b.notes(["Height = density row (32 / 36 / 44), radius lg 12, 12 pt leading inset; the domain yields by mid-ellipsis, the accessory never pushes it out.",
             "Fill: light white 55 %, dark white 6 %; edge ink 4 % / white 4 % (IC 10 %); hover light white 40 %, dark white 10 %; no shadow.",
             "Not-secure, internal-page and file variants are proposals for §17.13 (delta D-2); the shipped control shows the host."])
    print(b.finish())

if "S-2" in which:
    b = Board("S-2", width=2200, desc="Favorites are destination tiles under the address: four columns at every width and density, the favicon centred on the favorite veil, no visible label. The region is omitted when a space has none; while a tab is dragged, the band or a drop cell offers itself.",
              ids="bbr-favorites, bbr-fav-<n>, bbr-fav-drop-cell, bbr-favorites-offer")
    def items(t):
        f = FAVS
        more = f + [("site", "cobalt", "G"), ("site", "pine", "W")]
        full = more + [("site", "violet", "N"), ("mono", "S", 3)]
        ctx = lambda favs, states=None: address(t, "x.com") + favorites(t, favs, states) + heading(t)
        offer = (f'<div layer-name="bbr-favorites-offer" style="margin:0;padding:0 8px 12px 8px"><div style="height:44px;display:flex;align-items:center;justify-content:center;gap:8px;border-radius:var(--radius-md);'
                 f'border:1.5px dashed {t.c("accent")}">{icon("star", t.c("accent-ink"), 14)}{text("Drop to add to Favorites", "meta", t.c("accent-ink"), clamp=False)}</div></div>')
        offer_live = offer.replace(f'border:1.5px dashed {t.c("accent")}', f'border:1.5px solid {t.c("accent")};background-color:{t.c("tint-accent")}')
        return [frag(t, ctx(f), "One row", "4 tiles"), frag(t, ctx(more), "Wrap", "6 tiles; row 2 left-aligned"),
                frag(t, ctx(full), "Two full rows"), frag(t, ctx(f, {3: "active"}), "Active", "the tile's page is selected"),
                frag(t, ctx(f, {1: "hover"}), "Hover"), frag(t, address(t, "x.com") + heading(t), "None", "region omitted; nothing fabricated"),
                frag(t, address(t, "x.com") + offer + heading(t), "Drag offer · empty", "band offered during a drag"),
                frag(t, address(t, "x.com") + offer_live + heading(t), "Drag offer · live target"),
                frag(t, ctx(more + [None], {6: "drop"}).replace('<div style="flex:1 1 0;min-width:0;height:44px"></div>', "", 0), "Drop cell · live", "next free cell takes the drop")]
    frag_rows(b, items)
    b.notes(["Tiles 64 × 44 at N (S 40 × 36, XL 88 × 52), gap 4/4/8, radius md; the empty slot and the drop cell share the tile radius.",
             "Adding a favorite pops the tile with the accent outline and speck burst (V-5); Reduce Motion flashes tint.accent.",
             "Favorite menu: M-2. Favorites belong to the space; dragging a favorite out removes it (tab stays)."])
    print(b.finish())

if "S-3" in which:
    b = Board("S-3", width=2200, desc="The space heading is the list's only heading: the space identity on the favicon lane and its name at 13 / 600 in ink.secondary. Pinned tabs follow as loose rows; at S density they become 32 × 32 icon tiles. A drag of an unpinned tab offers the ghost row 'Pin tab here'.",
              ids="bbr-active-space-title, bbr-active-space-identity, bbr-tab-pinned-<n>, bbr-pinned-drop-row")
    def items(t):
        pins = [tab_row(t, "Home / X", ("site", "ink", "X"), "selected"), tab_row(t, "Reddit", ("site", "terracotta", "R")), tab_row(t, "WhatsApp", ("site", "pine", "W"))]
        ghost = (f'<div layer-name="bbr-pinned-drop-row" style="height:36px;display:flex;align-items:center;gap:10px;padding:0 10px;border-radius:var(--radius-md);border:1.5px dashed {t.c("accent")}">'
                 f'{slot(icon("plus", t.c("accent-ink")))}{text("Pin tab here", "meta", t.c("accent-ink"), clamp=False)}</div>')
        ghost_live = ghost.replace(f'border:1.5px dashed {t.c("accent")}', f'border:1.5px solid {t.c("accent")};background-color:{t.c("tint-accent")}')
        tiles = "".join(f'<div style="width:32px;height:32px;display:flex;align-items:center;justify-content:center;border-radius:var(--radius-sm);{bg}">{favicon(t, fv, 16)}</div>'
                        for fv, bg in ((("site", "ink", "X"), f"background-color:{t.c('tint-selected')};box-shadow:{t.lift()}"), (("site", "terracotta", "R"), f"box-shadow:inset 0 0 0 1px {t.c('hairline')}"), (("site", "pine", "W"), f"box-shadow:inset 0 0 0 1px {t.c('hairline')}")))
        badge_head = (f'<div style="display:flex;align-items:center;gap:10px;padding:4px 18px 6px 18px"><div style="width:18px;height:18px;border-radius:5px;display:flex;align-items:center;justify-content:center;background-color:{t.c("wash-active")}">{text("WK", "badge", t.c("ink-secondary"), clamp=False, extra="font-size:8px;line-height:10px;letter-spacing:0;")}</div>{text("Work", "label", t.c("ink-secondary"), weight=600)}</div>')
        return [frag(t, heading(t) + rb(t, pins), "Heading + pinned", "N density"),
                frag(t, heading(t, "Research", "book") + rb(t, pins[1:]), "No selection in pinned"),
                frag(t, heading(t) + f'<div style="display:flex;gap:4px;padding:0 8px 8px 8px">{tiles}</div>', "S density", "icon-only tiles, hairline outline"),
                frag(t, badge_head + rb(t, pins[1:]), "Badge identity", "non-curated identity → two-letter badge"),
                frag(t, heading(t) + rb(t, pins + [ghost]), "Ghost row · offered", "during a drag of an unpinned tab"),
                frag(t, heading(t) + rb(t, pins + [ghost_live]), "Ghost row · live target"),
                frag(t, heading(t) + rb(t, [ghost]), "Empty organized area", "the only place pinning is discoverable")]
    frag_rows(b, items)
    b.notes(["The heading identity is the same value the footer chip shows — one mark per space — and starts on the favicon lane so its title aligns with every tab title.",
             "Pinned rows never reorder on selection; close is hover-only; dragging to the regular list unpins.",
             "A drop on the ghost row pins the tab at the end of the pinned block through the one pin authority (same tab, same session)."])
    print(b.finish())

if "S-4" in which:
    b = Board("S-4", width=2200, desc="A folder is the grouped form of pinned tabs: glyph and name at 600 in ink, members indented one step, a member's close revealed on hover or selection. Folders open and close in place, rename inline, and accept drops.",
              ids="bbr-folder-<n>, bbr-folder-<n>-member-<k>, bbr-folder-rename")
    def items(t):
        members = [tab_row(t, "CLIProxyAPI Limits", ("mono", "C", 2), indent=12), tab_row(t, "Grafana — tokens", ("site", "amber", "G"), indent=12)]
        rename = (f'<div style="height:36px;display:flex;align-items:center;gap:10px;padding:0 6px 0 10px;border-radius:var(--radius-md);background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("accent")}, 0 0 0 3px {t.c("tint-accent")}">'
                  f'{slot(icon("folder-open", t.c("ink-secondary")))}{text("Review Tools", "title", t.c("ink"), weight=600, clamp=False)}<div style="width:1.5px;height:16px;background-color:{t.c("accent")}"></div></div>')
        empty_hint = f'<div style="height:32px;display:flex;align-items:center;padding:0 0 0 38px">{text("Drop tabs here", "meta", t.c("ink-faint"), clamp=False)}</div>'
        target = folder_row(t, "Review Tools").replace("background-color:transparent", f"background-color:{t.c('tint-accent')};box-shadow:inset 0 0 0 1.5px {t.c('accent')}")
        return [frag(t, rb(t, [folder_row(t, "Home"), folder_row(t, "Review Tools"), folder_row(t, "Haus")]), "Closed"),
                frag(t, rb(t, [folder_row(t, "ai", True)] + members + [folder_row(t, "Haus")]), "Open"),
                frag(t, rb(t, [folder_row(t, "ai", True), tab_row(t, "CLIProxyAPI Limits", ("mono", "C", 2), "selected", indent=12), members[1]]), "Member selected", "lift inside the folder"),
                frag(t, rb(t, [folder_row(t, "Haus", count=9, state="hover"), folder_row(t, "Home")]), "Hover", "item count on hover"),
                frag(t, rb(t, [rename, folder_row(t, "Haus")]), "Rename inline", "Return commits, Escape restores"),
                frag(t, rb(t, [folder_row(t, "New Folder", True), empty_hint]), "Empty folder"),
                frag(t, rb(t, [folder_row(t, "Home"), target, folder_row(t, "Haus")]), "Drop target", "drag a tab onto a folder: it joins, pinned")]
    frag_rows(b, items)
    b.notes(["Folders only hold pinned tabs; moving a tab into a folder pins it, dragging it out to the tab list unpins it (folder-pinned invariant).",
             "Opening and closing snap (no animation); the chevron-free glyph change (folder ↔ folder-open) carries the state.",
             "New folders are created as 'New Folder' and open in rename immediately (proposal: today they keep the name until renamed via the menu)."])
    print(b.finish())

if "S-5" in which:
    b = Board("S-5", width=1560, desc="The tab list as it lives in the rail: titles ellipsize at the end, missing favicons become monogram slates, and every live state keeps its place in the trailing lane.", ids="bbr-tab-<n>, bbr-tab-<n>-title/-favicon/-close")
    def items(t):
        rows = [tab_row(t, "Knuddels-Bots.com | Knuddels Underground Community", ("mono", "K", 1)),
                tab_row(t, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), trailing="attention"),
                tab_row(t, "Vision not supported? · Issue #4127", ("mono", "P", 5), "hover"),
                tab_row(t, "Daybreak | OpenAI for Enterprise", ("mono", "O", 6), "selected"),
                tab_row(t, "Cloudflare Registrar | Domains", ("loading",), asleep=True, trailing="none"),
                tab_row(t, "YouTube Music — Discover Mix", ("site", "fuchsia", "Y"), trailing="audio"),
                tab_row(t, "Grafana dashboard", ("lifering",), trailing="none"),
                tab_row(t, "Design editor", ("lifering",), trailing="clock"),
                tab_row(t, "Suspended · Hacker News", ("site", "amber", "H"), asleep=True),
                tab_row(t, "untitled", ("mono", "U", 4)),
                tab_row(t, "localhost:5173", ("mono", "L", 2))]
        return [frag(t, new_tab_row(t) + rb(t, rows, "4px 0 8px 0").replace("padding:4px 0 8px 0", "padding:4px 0 8px 0"), "Tabs · " + t.label(), pad="8px 8px 8px 8px", w=320)]
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        for html in items(t):
            b.add(html, row)
    b.notes(["Empty titles fall back to the host, then the scheme; whitespace-only titles count as empty (§11.9).",
             "Full titles appear in a native tooltip on hover (§17.2); the ⌘T palette shows the full title and URL.",
             "Order: regular tabs append at the end; link-opened children insert after their opener (PR-04)."])
    print(b.finish())

if "S-6" in which:
    b = Board("S-6", width=2200, desc="New Tab is a row, not a tab: it opens ⌘T in New Tab mode and creates nothing until Enter commits a destination. Enter appends the tab, selects it, creates its session and moves focus into the page.", ids="bbr-new-tab")
    def items(t):
        after = rb(t, [tab_row(t, "YouTube", ("site", "fuchsia", "Y")), tab_row(t, "github.com/zed-industries/zed", ("loading",), "selected", asleep=True, trailing="none")])
        return [frag(t, rb(t, [new_tab_row(t)]), "Rest"), frag(t, rb(t, [new_tab_row(t, "hover")]), "Hover"),
                frag(t, rb(t, [new_tab_row(t).replace("background-color:transparent", f"background-color:{t.c('wash-active')}")]), "Pressed / ⌘T open", "the palette opens in New Tab mode (C-1)"),
                frag(t, rb(t, [new_tab_row(t).replace("background-color:transparent", f"box-shadow:inset 0 0 0 2px {t.c('accent')}")]), "Keyboard focus"),
                frag(t, rb(t, [new_tab_row(t)]) + after, "After Enter", "appended, selected, loading")]
    frag_rows(b, items)
    b.notes(["Escape from New Tab mode creates nothing and restores the previous focus and command text.",
             "Closing the last tab leaves the space empty and opens the empty New Tab palette; Escape returns to the empty-space pane (G-2)."])
    print(b.finish())

if "S-7" in which:
    b = Board("S-7", width=2200, desc="[Downloads] [space strip] [+] sit directly on the rail — no band, no rule. The strip scrolls horizontally when spaces overflow; the active chip is lifted. Plus opens the plus menu with New Space and New Folder.",
              ids="bbr-downloads, bbr-space-strip, bbr-space-chip-<n>, bbr-plus, bbr-plus-menu")
    def items(t):
        many = ("code", "bag", "moon", "compass", "leaf", "globe", "home", "book")
        sp = {"spaces": ("code", "bag", "moon")}
        focus = footer(t, **sp).replace(space_chip(t, "bag"), space_chip(t, "bag").replace("box-shadow:none", f"box-shadow:inset 0 0 0 2px {t.c('accent')}"))
        hover = footer(t, **sp).replace(space_chip(t, "moon"), space_chip(t, "moon", hover=True))
        target = footer(t, **sp).replace(space_chip(t, "moon"), f'<div style="position:relative">{space_chip(t, "moon").replace("background-color:transparent", "background-color:" + t.c("tint-accent")).replace("box-shadow:none", "box-shadow:inset 0 0 0 1.5px " + t.c("accent"))}'
                                   f'<div style="position:absolute;left:20px;top:-2px;height:16px;padding:0 5px;border-radius:5px;display:flex;align-items:center;background-color:{t.c("profile-violet")}">{text("Work", "badge", "var(--color-on-accent)", clamp=False, extra="font-size:9px;line-height:10px;letter-spacing:0.02em;")}</div></div>')
        from blocks import menu, menu_row
        plus = (f'<div style="height:110px;display:flex;justify-content:flex-end;align-items:flex-end">{menu(t, [menu_row(t, "New Space", "cursor", "⇧⌘N", "grid"), menu_row(t, "New Folder", icon_name="folder")], 200)}</div>')
        return [frag(t, footer(t, **sp), "Rest", "three spaces"), frag(t, footer(t, downloads_badge=True, **sp), "Downloads active", "dot while a download runs; count in the panel"),
                frag(t, hover, "Chip hover"), frag(t, focus, "Chip keyboard focus"),
                frag(t, footer(t, spaces=many, active=6), "Overflow", "one line, scrolls; the active chip is kept in view"),
                frag(t, target, "Drag target", "chip live + destination profile"),
                frag(t, f'<div style="padding:0 8px 6px 8px">{plus}</div>' + footer(t, **sp), "Plus menu", "exactly two items in v1")]
    frag_rows(b, items)
    b.notes(["Chips are icon-only; the accessible name carries space and profile; colour or glyph alone never carries identity.",
             "Pointer, keyboard (⌃1 … ⌃9) and a horizontal swipe over the rail use one switch authority (P-1).",
             "Target: chips start after Downloads as in the Paper reference; main and DESIGN §11.4 centre them while they fit (delta D01)."])
    print(b.finish())

if "S-8" in which:
    b = Board("S-8", width=2200, desc="When a tab plays media, one card appears above the footer: title, source, capability-gated transport and mute. It shows only what the page advertises — no fake duration, no assumed Previous/Next. Several playing tabs show the most recently controlled one plus a count.",
              ids="bbr-media-card, -title, -source, -progress, -elapsed, -duration, -previous, -play-pause, -next, -mute, -dismiss, -session-count")
    def card(t, title, source, playing=True, seek=True, prev_next=True, muted=False, count=None, fav=("site", "fuchsia", "Y"), elapsed="1:42", total="3:58", pct=43):
        ink, ink2 = t.c("ink"), t.c("ink-secondary")
        transport = []
        if prev_next:
            transport.append(icon_button(t, "prev", ink2, size=28))
        transport.append(icon_button(t, "pause" if playing else "play", ink, size=28))
        if prev_next:
            transport.append(icon_button(t, "next", ink2, size=28))
        cnt = (f'<div style="height:18px;padding:0 6px;border-radius:9999px;display:flex;align-items:center;background-color:{t.c("wash-active")}">{text(count, "key", ink2, clamp=False)}</div>' if count else "")
        scrub = ""
        if seek:
            scrub = (f'<div style="display:flex;align-items:center;gap:8px">{text(elapsed, "meta", t.c("ink-faint"), clamp=False, extra="font-variant-numeric:tabular-nums;")}'
                     f'<div style="flex:1 1 0;height:3px;border-radius:9999px;background-color:{t.c("wash-active")}"><div style="width:{pct}%;height:3px;border-radius:9999px;background-color:{t.c("accent")}"></div></div>'
                     f'{text(total, "meta", t.c("ink-faint"), clamp=False)}</div>')
        elif seek is None:
            scrub = f'<div style="display:flex;align-items:center;gap:6px"><div style="width:6px;height:6px;border-radius:9999px;background-color:{t.c("status-error")}"></div>{text("Live", "meta", ink2, clamp=False)}</div>'
        return (f'<div layer-name="bbr-media-card" style="margin:0 8px 8px 8px;display:flex;flex-direction:column;gap:8px;padding:10px 10px 8px 10px;border-radius:var(--radius-lg);background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("hairline")}">'
                f'<div style="display:flex;align-items:center;gap:10px">{favicon(t, fav, 22)}<div style="flex:1 1 0;min-width:0;display:flex;flex-direction:column">{text(title, "label", ink)}{text(source, "meta", ink2)}</div>{cnt}{icon_button(t, "close", t.c("ink-faint"), size=22)}</div>'
                f'{scrub}<div style="display:flex;align-items:center;justify-content:space-between"><div style="display:flex;gap:4px">{"".join(transport)}</div>{icon_button(t, "mute" if muted else "speaker", ink2, size=28)}</div></div>')
    def items(t):
        f = lambda c, label, note="": frag(t, c + footer(t), label, note)
        return [f(card(t, "Discover Mix", "YouTube Music · Default"), "Playing · seekable"),
                f(card(t, "Discover Mix", "YouTube Music · Default", playing=False), "Paused"),
                f(card(t, "Lo-fi radio", "Radio Garden · Work", seek=None, prev_next=False, fav=("site", "pine", "R")), "Live", "no scrubber, no total"),
                f(card(t, "Podcast #112", "Overcast · Default", seek=False, prev_next=False, fav=("site", "amber", "O")), "Unknown duration", "only play/pause advertised"),
                f(card(t, "Discover Mix", "YouTube Music · Default", muted=True), "Muted"),
                f(card(t, "Standup call", "Meet · Work", seek=False, prev_next=False, count="+2", fav=("site", "cobalt", "M")), "Several sessions", "most recent + count/switch")]
    frag_rows(b, items)
    b.notes(["material:field, elev:1, radius lg, accent progress; it consumes no space when nothing eligible plays.",
             "Clicking title or favicon selects the owning tab/space/profile; the other controls never select or focus the tab.",
             "Dismiss hides this media identity until it changes; playback continues.",
             "Focus order: media card controls come before Downloads (§9). Capability data comes only from the media-session contract."])
    print(b.finish())

if "S-9" in which:
    b = Board("S-9", width=2560, desc="After 4 pt of travel a row becomes a drag: the source row stays as a 92 % placeholder, a lifted proxy follows the pointer, and the rail offers every region the tab can go — Favorites, the pinned ghost row, folders, space chips. Only the region under the pointer takes the live-target look.",
              ids="bbr-tab-drag-proxy, bbr-drag-tick-<n>, bbr-pinned-drop-row, bbr-fav-drop-cell, bbr-space-chip-<n>")
    def rail_drag(t, target):
        dash = f"border:1.5px dashed {t.c('accent')}"
        live = f"border:1.5px solid {t.c('accent')};background-color:{t.c('tint-accent')}"
        fav_off = (f'<div style="padding:0 8px 12px 8px"><div style="display:flex;gap:4px">' + "".join(favorite_tile(t, fv) for fv in FAVS[:3])
                   + f'<div style="height:44px;flex:1 1 0;border-radius:var(--radius-md);{live if target == "fav" else dash}"></div></div></div>')
        ghost = (f'<div style="height:36px;display:flex;align-items:center;gap:10px;padding:0 10px;border-radius:var(--radius-md);{live if target == "pin" else dash}">'
                 f'{slot(icon("plus", t.c("accent-ink")))}{text("Pin tab here", "meta", t.c("accent-ink"), clamp=False)}</div>')
        folder = folder_row(t, "Review Tools")
        if target == "folder":
            folder = folder.replace("background-color:transparent", f"background-color:{t.c('tint-accent')};box-shadow:inset 0 0 0 1.5px {t.c('accent')}")
        tick = f'<div layer-name="bbr-drag-tick" style="height:2px;border-radius:2px;background-color:{t.c("accent")};margin:0 8px"></div>' if target == "list" else ""
        regular_rows = [tab_row(t, "Knuddels-Bots.com", ("mono", "K", 1)), tab_row(t, "Rust SIMD on the GPU", ("mono", "V", 3), "dragging", trailing="none")]
        regular_rows += ([tick] if tick else []) + [tab_row(t, "Vision not supported?", ("mono", "P", 5)), tab_row(t, "Daybreak | OpenAI", ("mono", "O", 6))]
        chip_footer = footer(t, spaces=("code", "bag", "moon"))
        if target == "space":
            chip_footer = chip_footer.replace(space_chip(t, "moon"), space_chip(t, "moon").replace("background-color:transparent", "background-color:" + t.c("tint-accent")).replace("box-shadow:none", "box-shadow:inset 0 0 0 1.5px " + t.c("accent")))
        body = (address(t, "x.com") + fav_off + heading(t) + rb(t, [tab_row(t, "Home / X", ("site", "ink", "X"), "selected"), folder_row(t, "Home"), folder, ghost])
                + divider(t) + rb(t, [new_tab_row(t)]) + rb(t, regular_rows) + '<div style="flex:1 1 0"></div>' + chip_footer)
        pos = {"list": (60, 470), "pin": (60, 262), "folder": (70, 224), "fav": (130, 64), "space": (40, 596)}[target]
        proxy = (f'<div layer-name="bbr-tab-drag-proxy" style="position:absolute;left:{pos[0]}px;top:{pos[1]}px;width:240px;height:36px;display:flex;align-items:center;gap:10px;padding:0 10px;border-radius:var(--radius-md);opacity:0.92;'
                 f'background-color:{t.c("tint-selected")};box-shadow:0 0 0 1px {t.c("border-strong")}, {t.elev2()}">{favicon(t, ("mono", "V", 3))}{text("Rust SIMD on the GPU", "title", t.c("ink"))}</div>')
        return body + proxy
    def items(t):
        return [frag(t, rail_drag(t, k), l, n, h=680) for k, l, n in (("list", "Reorder", "insertion tick at the resolved index"), ("pin", "Pin", "ghost row live"),
                                                                      ("folder", "Into folder", "folder row live"), ("fav", "Favorite", "drop cell live"), ("space", "Move to space", "chip live + profile companion (S-7)"))]
    frag_rows(b, items)
    b.notes(["Offers appear together at activation and grow the rail once (one pitch for the ghost row, one tile row when Favorites are full); nothing reflows while the pointer moves.",
             "Escape or a drop outside cancels; the proxy returns and every offer disappears in the same frame.",
             "Dropping on the card opens a split (G-10). Cross-profile chips show the destination profile before the drop (P-7)."])
    print(b.finish())

if "S-10" in which:
    b = Board("S-10", width=3040, desc="The rail is 300 at N and resizable from 232 to 360 by the 4 pt divider on its trailing edge; the divider is empty paint at rest, shows the resize cursor on hover and an accent line only under keyboard focus (←/→ resize, 0 resets).",
              ids="bbr-sidebar-divider")
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name="Widths · " + t.label())
        for w, label, divider_state in ((232, "Minimum 232", "rest"), (300, "Default 300", "hover"), (360, "Maximum 360", "focus")):
            col = b.column(row, name=label)
            b.caption(col, f"{label} · {t.label()}", {"rest": "divider at rest: no paint", "hover": "hover: resize cursor (native)", "focus": "keyboard focus: 2 pt accent"}[divider_state])
            win = window(b, col, t, std(t, rail_width=w, hover=None), 920, 620)
            if divider_state == "focus":
                b.add(f'<div layer-name="bbr-sidebar-divider · focus" style="position:absolute;left:{w - 1}px;top:10px;bottom:10px;width:2px;border-radius:2px;background-color:{t.c("accent")}"></div>', win)
            if divider_state == "hover":
                b.add(f'<div layer-name="Resize cursor (native)" style="position:absolute;left:{w - 12}px;top:300px;display:flex;align-items:center;gap:0">{icon("chevron-right", t.c("ink"), 12).replace("M7 4.5 11.5 9 7 13.5", "M7 4.5 2.5 9 7 13.5")}{icon("chevron-right", t.c("ink"), 12)}</div>', win)
    b.notes(["Resize commits geometry on the next frame; the drag shows no intermediate chrome states.",
             "Width memory is per window; ⌘B collapse keeps the last expanded width (W-3).",
             "Density changes the default width (232 / 300 / 340); the user's drag overrides it."])
    print(b.finish())
