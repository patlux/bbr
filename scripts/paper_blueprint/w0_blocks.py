import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from blocks import *

which = sys.argv[1:]


def state_line(t, label, html, w=300):
    return (f'<div style="display:flex;align-items:center;gap:16px"><div style="width:96px;flex-shrink:0">{text(label, "key", t.c("ink-faint"), weight=400, clamp=False)}</div>'
            f'<div style="width:{w}px;flex-shrink:0;display:flex;flex-direction:column">{html}</div></div>')


def two_plates(b, title_fn, per_theme, width=700, note=None, row=None):
    row = row or b.row(gap=24)
    for t in (LIGHT, DARK):
        c = b.column(row, name=t.label(), width=width)
        b.caption(c, title_fn(t), note or "")
        p = plate(b, c, t, w=width, pad=24)
        for html in per_theme(t):
            b.add(html, p)
    return row


if "B-1" in which:
    b = Board("B-1", width=1560, desc="Three weights of action — primary accent fill for the one committing action, secondary on the field material, quiet text actions — plus danger with a text companion. Icon buttons carry the titlebar and footer. Focus is a 2 pt accent ring outside the control.")
    def rows(t):
        out = []
        for kind, label in (("primary", "Reload"), ("secondary", "Close tab"), ("quiet", "Not now"), ("danger", "Delete")):
            out.append(f'<div style="display:flex;align-items:center;gap:14px"><div style="width:84px">{text(kind, "key", t.c("ink-faint"), weight=400)}</div>'
                       + "".join(button(t, label, kind, s) for s in ("rest", "hover", "pressed", "focused", "disabled")) + '</div>')
        heads = "".join(f'<div style="width:{w}px">{text(s, "badge", t.c("ink-faint"), clamp=False)}</div>' for s, w in (("", 84), ("rest", 90), ("hover", 90), ("pressed", 90), ("focused", 90), ("disabled", 90)))
        icons = "".join(icon_button(t, "reload", active=a, extra=e) for a, e in ((False, ""), (False, f"background-color:{t.c('wash-hover')};"), (True, ""), (False, f"box-shadow:0 0 0 2px {t.c('accent')};"), (False, "opacity:0.45;")))
        out.append(f'<div style="display:flex;align-items:center;gap:14px"><div style="width:84px">{text("icon", "key", t.c("ink-faint"), weight=400)}</div>{icons}</div>')
        keys = "".join(keycap(t, k) for k in ("⌘", "T", "⌘L", "⇧⌘T", "⌃1", "esc", "↵"))
        out.append(f'<div style="display:flex;align-items:center;gap:14px"><div style="width:84px">{text("keycaps", "key", t.c("ink-faint"), weight=400)}</div><div style="display:flex;gap:6px">{keys}</div></div>')
        sizes = button(t, "Done", "primary", size="s") + button(t, "Done", "primary") + button(t, "Done", "primary", size="l")
        out.append(f'<div style="display:flex;align-items:center;gap:14px"><div style="width:84px">{text("S · N · XL", "key", t.c("ink-faint"), weight=400)}</div><div style="display:flex;gap:10px;align-items:center">{sizes}</div></div>')
        return out
    two_plates(b, lambda t: "Buttons · " + t.label(), rows)
    b.notes(["Primary uses on.accent text on the accent fill (the S1 contrast fix); hover steps to accent.ink; pressed adds wash.active.",
             "Danger is reserved for irreversible actions and always carries its word ('Delete', 'Remove'); colour never carries it alone.",
             "Heights follow density: 24 / 28 / 32; radius md; label 13 / 500. Icon buttons are 28 with an 18 glyph and a 32 hit minimum at N.",
             "Keycaps render shortcuts in SF Mono on a wash with a hairline edge; they appear in menus, hints and the onboarding."])
    print(b.finish())

if "B-2" in which:
    b = Board("B-2", width=1560, desc="Inputs for sheets, panels and Settings. Keyboard focus is always visible: a 1 pt accent edge plus a 2 pt tint.accent halo on fields, a 2 pt accent ring on everything else; web-content focus leaves the chrome at rest.")
    def rows(t):
        out = [state_line(t, s, field(t, v, state=s), 260) for s, v in (("rest", ""), ("hover", ""), ("focused", "github.com/zed"), ("error", "http//bad url"), ("disabled", ""))]
        out.append(state_line(t, "checkbox", f'<div style="display:flex;gap:18px">{checkbox(t, False, "Off")}{checkbox(t, True, "On")}{checkbox(t, mixed=True, label="Mixed")}</div>', 520))
        out.append(state_line(t, "switch", f'<div style="display:flex;gap:28px">{switch(t, False, "Off")}{switch(t, True, "On")}</div>', 520))
        out.append(state_line(t, "segmented", segmented(t, ["Subtle", "Standard", "Rich"], 1, 300), 300))
        out.append(state_line(t, "popup", popup_button(t, "Google"), 300))
        out.append(state_line(t, "focus ring", tab_row(t, "Focused row", ("mono", "F", 2), "focused"), 300))
        return out
    two_plates(b, lambda t: "Fields & controls · " + t.label(), rows)
    b.notes(["Fields: material:field, radius lg (12), 32 high at N; placeholder in ink.faint; the caret is the accent (1.5 pt).",
             "Switches, segmented controls and popup buttons are new for Settings and sheets (New · proposal); checkboxes follow the same accent fill.",
             "Focus order is always visible (§9); the ring never animates and never appears for pointer focus alone."])
    print(b.finish())

if "B-3" in which:
    b = Board("B-3", width=1560, desc="Every §10 state on the one row anatomy (§11.1): favicon slot, title, trailing slot. Selection is the lifted white surface, not a heavier title; hover and press are washes; attention, loading and error use small glyphs with a non-colour companion.", ids="bbr-tab-row-*, bbr-folder-*, bbr-new-tab")
    def rows(t):
        R = [
            ("default", tab_row(t, "Rust SIMD on the GPU — a field guide", ("mono", "R", 3))),
            ("hover", tab_row(t, "Rust SIMD on the GPU — a field guide", ("mono", "R", 3), "hover")),
            ("pressed", tab_row(t, "Rust SIMD on the GPU — a field guide", ("mono", "R", 3), "pressed", trailing="close")),
            ("selected", tab_row(t, "Home / X", ("mono", "X", 6), "selected")),
            ("focused", tab_row(t, "Keyboard focus", ("mono", "K", 1), "focused")),
            ("loading", tab_row(t, "Loading page…", ("loading",), trailing="none", asleep=True)),
            ("attention", tab_row(t, "Calendar — 2 new", ("site", "cobalt", "C"), trailing="attention")),
            ("audio", tab_row(t, "YouTube Music", ("site", "fuchsia", "Y"), trailing="audio")),
            ("error", tab_row(t, "This page crashed", ("lifering",), trailing="none")),
            ("unresponsive", tab_row(t, "Design editor — not responding", ("lifering",), trailing="clock")),
            ("asleep", tab_row(t, "Suspended — tap to wake", ("mono", "S", 4), asleep=True)),
            ("disabled", tab_row(t, "Unavailable in this profile", ("mono", "U", 5), "disabled")),
            ("dragging", tab_row(t, "Dragging row", ("mono", "D", 2), "dragging", trailing="none")),
            ("drop line", f'<div style="display:flex;flex-direction:column;gap:3px">{tab_row(t, "Above the gap", ("mono", "A", 1))}<div style="height:2px;margin:0;border-radius:2px;background-color:{t.c("accent")}"></div>{tab_row(t, "Below the gap", ("mono", "B", 2))}</div>'),
            ("folder", folder_row(t, "Review Tools")),
            ("folder open", f'<div style="display:flex;flex-direction:column;gap:4px">{folder_row(t, "ai", opened=True)}{tab_row(t, "CLIProxyAPI Limits", ("mono", "C", 2), indent=12)}</div>'),
            ("folder hover", folder_row(t, "Haus", count=9, state="hover")),
            ("new tab", new_tab_row(t)),
        ]
        return [state_line(t, k, v, 300) for k, v in R]
    two_plates(b, lambda t: "Rail rows · " + t.label(), rows, width=460)
    b.notes(["36 pt rows (N), radius md, 10 pt leading inset, 18 pt favicon slot, 18 pt trailing slot so every row shares one lane.",
             "The close glyph shows on hover and on the selected row; audio, attention and crash glyphs take the trailing slot when present.",
             "Loading replaces the favicon with the 640 ms accent arc (static under Reduce Motion) and drops the title to ink.secondary; suspended rows keep their place, title in ink.secondary. Error and unresponsive swap the favicon for the lifering; unresponsive adds a clock and never pulses.",
             "Folder members indent 12 pt; folder names are 600 in ink; the item count appears on hover.",
             "Dragging (after 4 pt travel) keeps the row at 92 % with elev:2 and a border.strong edge; the resolved index shows the 2 pt accent insertion tick (§10.9)."])
    print(b.finish())

if "B-4" in which:
    b = Board("B-4", width=1560, desc="Small identity carriers: Favorite tiles on the favorite veil, space chips in the footer strip, the address accessories and the kind badges used by command results and panels.", ids="bbr-favorite-*, bbr-space-chip-*, bbr-address-zoom, bbr-command-mode")
    def rows(t):
        tiles = "".join(f'<div style="width:64px;display:flex;flex-direction:column;gap:6px;align-items:stretch">{favorite_tile(t, ("site", "terracotta", "Y"), s)}{text(s, "key", t.c("ink-faint"), weight=400, clamp=False, extra="text-align:center;")}</div>'
                        for s in ("rest", "hover", "active", "drop", "empty"))
        out = [state_line(t, "favorite tile", f'<div style="display:flex;gap:10px">{tiles}</div>', 380)]
        chips = "".join(f'<div style="display:flex;flex-direction:column;gap:6px;align-items:center;width:48px">{space_chip(t, g, a, hover=(l == "hover"))}{text(l, "key", t.c("ink-faint"), weight=400, clamp=False)}</div>'
                        for g, a, l in (("code", False, "rest"), ("bag", True, "active"), ("moon", False, "hover")))
        out.append(state_line(t, "space chip", f'<div style="display:flex;gap:10px">{chips}</div>', 380))
        out.append(state_line(t, "zoom chip", f'<div style="display:flex"><div style="height:22px;display:flex;align-items:center;padding:0 8px;border-radius:var(--radius-sm);background-color:{t.c("address-accessory")}">{text("140%", "key", t.c("ink"), clamp=False)}</div></div>', 380))
        out.append(state_line(t, "mode pill", f'<div style="display:flex;gap:8px">{mode_pill(t, "NEW TAB")}{mode_pill(t, "OPEN")}{mode_pill(t, "ACTIONS")}</div>', 380))
        out.append(state_line(t, "kind badge", f'<div style="display:flex;gap:6px">{badge(t, "Tab")}{badge(t, "History")}{badge(t, "Saved", "accent")}{badge(t, "Paused", "attention")}{badge(t, "Failed", "error")}</div>', 380))
        tiles2 = "".join(favicon(t, f, s) for f, s in ((("mono", "K", 1), 16), (("mono", "K", 1), 18), (("mono", "K", 1), 22), (("site", "cobalt", "G"), 18), (("site", "pine", "W"), 18)))
        out.append(state_line(t, "favicon slot", f'<div style="display:flex;gap:12px;align-items:center">{tiles2}</div>', 380))
        dots = "".join(f'<div style="width:8px;height:8px;border-radius:9999px;background-color:{t.c(c)}"></div>' for c in ("status-attention", "status-error", "status-ok", "accent"))
        out.append(state_line(t, "status dots", f'<div style="display:flex;gap:12px;align-items:center">{dots}</div>', 380))
        return out
    two_plates(b, lambda t: "Tiles & chips · " + t.label(), rows, width=560)
    b.notes(["Favorite tile 64 × 44 (N) on material:favorite; active = the tile's page is the selected tab (1.5 pt accent edge); drop = tint.accent with an accent edge; the empty slot is dashed.",
             "The active space chip is the lifted selection surface with 8 % of its identity hue; inactive chips are glyph-only.",
             "Kind badges use type:badge; status tones always pair with a word."])
    print(b.finish())

if "B-5" in which:
    b = Board("B-5", width=1560, desc="One menu anatomy for every BBR-owned menu (§11.13): overlay material, 4 pt inset, inset cursor on tint.accent, hairline groups; plus the ⌘T result row that shares the cursor language.", ids="bbr-tab-menu, bbr-favorite-menu, bbr-folder-menu, bbr-page-menu")
    def rows(t):
        m = menu(t, [menu_row(t, "New Tab Below", icon_name="plus"), menu_row(t, "Duplicate", "cursor", icon_name="popup"),
                     menu_row(t, "Pin Tab", icon_name="pin", shortcut="⇧⌘P"), menu_row(t, "Move to Space", icon_name="grid", submenu=True),
                     menu_sep(t), menu_header(t, "Sound"), menu_row(t, "Mute Tab", icon_name="mute", checked=False),
                     menu_row(t, "Save Page", icon_name="bookmark", disabled=True), menu_sep(t),
                     menu_row(t, "Close Tab", icon_name="close", shortcut="⌘W"), menu_row(t, "Close Tabs Below…", icon_name="trash", danger=True)], width=280)
        res = "".join([result_row(t, "YouTube Music", "music.youtube.com", ("site", "fuchsia", "Y"), "Tab", True, "you"),
                       result_row(t, "Your Watch Later list", "youtube.com/playlist", ("site", "fuchsia", "Y"), "History", False, "you"),
                       result_row(t, "Search “you” on Google", None, ("glyph", "search"), "Search")])
        return [f'<div style="display:flex;gap:24px;align-items:flex-start">{m}<div style="display:flex;flex-direction:column;gap:6px;flex:1 1 0;min-width:0;padding:8px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}">{res}</div></div>']
    two_plates(b, lambda t: "Menu & result items · " + t.label(), rows, width=720)
    b.notes(["Menu rows 28 pt, radius sm cursor, label 13 / 400, shortcut in SF Mono ink.faint; checked rows show a check in accent.ink.",
             "The menu edge turns accent while it owns keyboard focus; pointer-only menus keep the hairline edge.",
             "Danger rows use status.error text plus a glyph; disabled rows stay only where the group needs them and expose a reason.",
             "Result rows 40 pt, radius md: favicon, title with the match in accent.ink, URL in mono ink.faint, kind hint (↵ on the cursor row)."])
    print(b.finish())

if "B-6" in which:
    b = Board("B-6", width=1560, desc="Every floating surface is the overlay material at elev:2 with a 1 pt edge. Sheets attach to the content card, menus open at the pointer, panels hang from the rail, banners sit in the card, the palette floats over the card. All paint above WebKit through the one native Portal.", ids="bbr-portal-*")
    def rows(t):
        mshell = menu(t, [menu_row(t, "Menu shell"), menu_row(t, "cursor row", "cursor"), menu_row(t, "row")], 200, focused=False)
        sh = sheet(t, text("Sheets carry a title, a short explanation and at most two actions.", "label", t.c("ink-secondary"), weight=400, clamp=False)
                   + '<div style="display:flex;gap:8px;justify-content:flex-end">' + button(t, "Cancel", "secondary") + button(t, "Allow", "primary") + '</div>', 300, "Sheet")
        pal = (f'<div layer-name="Palette shell" style="width:300px;display:flex;flex-direction:column;border-radius:var(--radius-xl);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
               f'<div style="height:48px;display:flex;align-items:center;gap:10px;padding:0 14px;border-bottom:1px solid {t.c("hairline")}">{icon("search", t.c("ink-faint"), 16)}{text("Search or enter address", "title", t.c("ink-faint"), weight=400, grow=True)}{mode_pill(t)}</div>'
               f'<div style="padding:8px;display:flex;flex-direction:column">{result_row(t, "Result row", "example.org", ("mono", "E", 2), "Tab", True)}</div></div>')
        ban = f'<div style="width:300px">{banner(t, "A pop-up was blocked on this page.", "info", button(t, "Open", "quiet", size="s"))}</div>'
        tip = (f'<div style="display:flex;flex-direction:column;gap:6px;align-items:flex-start"><div style="padding:4px 8px;border-radius:6px;background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{text("No back history", "meta", t.c("ink"), clamp=False)}</div>'
               f'{text("Tooltip · native NSToolTip (Native OS)", "key", t.c("ink-faint"), weight=400, clamp=False)}</div>')
        return [f'<div style="display:flex;gap:24px;align-items:flex-start;flex-wrap:wrap">{mshell}{sh}</div>',
                f'<div style="display:flex;gap:24px;align-items:flex-start;flex-wrap:wrap">{pal}<div style="display:flex;flex-direction:column;gap:20px">{ban}{tip}</div></div>']
    two_plates(b, lambda t: "Floating shells · " + t.label(), rows, width=720)
    b.notes(["Radii: menus md (10), sheets and panels lg (12), the ⌘T palette xl (16). Padding: menus 4, sheets 20, panels 12.",
             "Placement: sheets attach to the card's top edge centred on the card; menus open at the pointer clamped to the window; panels hang from their rail control; banners float over the card's top edge through the Portal, never in the chassis gap (§2.6).",
             "Floating surfaces never sample or blur web content (§13); there is no scrim over the page except a 12 % ink veil under blocking sheets (New · proposal).",
             "Tooltips stay native (§17.2 open decision)."])
    print(b.finish())

if "B-7" in which:
    b = Board("B-7", width=1560, desc="Feedback that lives in the chrome: the loading arc, the D5 address progress line, the attention beat, download progress, inline banners and toasts. Motion frames are specified on page 40.")
    def rows(t):
        arcs = "".join(f'<div style="display:flex;flex-direction:column;gap:6px;align-items:center;width:64px">{spinner(t)}{text(l, "key", t.c("ink-faint"), weight=400, clamp=False)}</div>' for l in ("0°", "turning", "static"))
        out = [state_line(t, "loading arc", f'<div style="display:flex;gap:8px">{arcs}</div>', 460)]
        prog = (f'<div style="display:flex;flex-direction:column;gap:10px;width:300px">'
                + address(t, "github.com", progress=35).replace("padding:0 4px 12px 4px", "padding:0") + address(t, "github.com", progress=100).replace("padding:0 4px 12px 4px", "padding:0") + '</div>')
        out.append(state_line(t, "address progress", prog, 460))
        beats = "".join(f'<div style="display:flex;flex-direction:column;gap:6px;align-items:center;width:64px"><div style="position:relative;width:32px;height:32px;display:flex;align-items:center;justify-content:center">'
                        + (f'<div style="position:absolute;width:{r}px;height:{r}px;border-radius:9999px;box-shadow:0 0 0 1.5px {t.c("status-attention")};opacity:{o}"></div>' if r else "")
                        + f'<div style="width:{d}px;height:{d}px;border-radius:9999px;background-color:{t.c("status-attention")}"></div></div>{text(l, "key", t.c("ink-faint"), weight=400, clamp=False)}</div>'
                        for d, r, o, l in ((8, 0, 0, "0 ms"), (11.6, 12, 0.7, "60"), (8, 20, 0.35, "120"), (11.6, 26, 0.15, "180"), (8, 0, 0, "rest")))
        out.append(state_line(t, "attention beat", f'<div style="display:flex;gap:4px">{beats}</div>', 460))
        dl = (f'<div style="display:flex;flex-direction:column;gap:8px;width:300px">{progress(t, 62, 300)}'
              f'<div style="display:flex;justify-content:space-between">{text("11.2 MB of 18 MB", "meta", t.c("ink-secondary"), clamp=False)}{text("32 s left", "meta", t.c("ink-faint"), clamp=False)}</div></div>')
        out.append(state_line(t, "download", dl, 460))
        out.append(state_line(t, "banners", '<div style="display:flex;flex-direction:column;gap:8px;width:420px">' + banner(t, "BBR restored 14 tabs from your last session.", "info") + banner(t, "Camera access is blocked for this site.", "attention", button(t, "Settings", "quiet", size="s")) + banner(t, "The download failed: disk full.", "error", button(t, "Retry", "secondary", size="s")) + '</div>', 460))
        toast = (f'<div style="display:flex;align-items:center;gap:10px;height:40px;padding:0 8px 0 14px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
                 f'{icon("check", t.c("status-ok"), 16)}{text("Page saved to Library", "label", t.c("ink"), clamp=False)}{button(t, "Undo", "quiet", size="s")}</div>')
        out.append(state_line(t, "toast", f'<div style="display:flex">{toast}</div>', 460))
        return out
    two_plates(b, lambda t: "Feedback · " + t.label(), rows, width=640)
    b.notes(["The address line is 2 pt accent at the field's inner bottom edge; content-pane progress bars are withdrawn (GPUI cannot paint above native glass).",
             "Attention: two dur:fine beats (dot 1 → 1.45 → 1 plus one ring 4 → 16 pt fading), then the static dot. Reduce Motion: opacity 1 → 0.45 → 1.",
             "Banners float over the card's top edge through the Portal (like the find bar), never in the chassis gap; toasts (New · proposal) confirm reversible actions for 4 s with Undo, bottom-centre of the card."])
    print(b.finish())
