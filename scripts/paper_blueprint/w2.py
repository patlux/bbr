import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from cmdsurf import *
from blocks import result_row, button, banner, field
from w1_21 import frag, frag_rows

which = sys.argv[1:]


def pal_board(code, desc, per_theme, ids="", notes=()):
    b = Board(code, width=3040, desc=desc, ids=ids)
    pair(b, per_theme)
    b.notes(list(notes))
    return b


if "C-1" in which:
    def spec(t):
        res = [result_row(t, "Rust SIMD on the GPU – a field guide", "simd.rs/guide", ("mono", "V", 3), "Tab", True),
               result_row(t, "Daybreak | OpenAI for Enterprise", "openai.com/…/daybreak", ("mono", "O", 6), "Tab"),
               result_row(t, "Linear — Sprint 42", "linear.app/…/sprint-42", ("site", "violet", "L"), "Tab", subtitle="Work"),
               result_row(t, "WKWebView | Apple Developer", "developer.apple.com/…/wkwebview", ("mono", "W", 2), "History"),
               result_row(t, "Reopen Closed Tab", None, ("glyph", "history"), "⇧⌘T")]
        return std(t, hover=None, overlays=[palette(t, results=res)])
    b = pal_board("C-1", "⌘T opens the palette in New Tab mode, centred over the card 110 pt below its top: the magnifier, the caret before the placeholder, the uppercase mode pill. Empty, it offers recent tabs and history first, so ⌘T ↵ returns to the last thing.",
                  spec, "bbr-palette, bbr-palette-field, bbr-palette-text, bbr-palette-cursor, bbr-palette-mode, bbr-palette-result-<k>",
                  ["Width follows density: S 508 · N 600 · XL 716; radius xl 16; field 56 with the 17 pt query; rows 40 in an 8 pt list.",
                   "Opens with a dur:fine opacity fade only; Escape restores the previous focus and creates nothing.",
                   "Recent rows on empty input are a proposal (the shipped palette waits for input); delta D-2."])
    print(b.finish())

if "C-2" in which:
    def spec(t):
        res = [result_row(t, "YouTube Music", "music.youtube.com", ("site", "fuchsia", "Y"), "Tab", True, "you"),
               result_row(t, "Your Watch Later list", "youtube.com/playlist", ("site", "fuchsia", "Y"), "History", False, "you"),
               result_row(t, "Your account security review", None, ("glyph", "bookmark"), "Saved", False, "you", subtitle="Saved Pages"),
               result_row(t, "Switch to space “Youth club”", None, ("glyph", "grid"), "Space", False, "you"),
               result_row(t, "youtube.com", "https://youtube.com", ("glyph", "globe"), "URL", False, "you"),
               result_row(t, "Search “you” with Google", None, ("glyph", "search"), "Search", False)]
        return std(t, hover=None, overlays=[palette(t, "you", res)])
    b = pal_board("C-2", "One result engine for tabs, history, Saved Pages, spaces, actions, addresses and search. Each row: badge, the title with the match in accent.ink, the short URL in mono, the kind — or ↵ on the selected row, which is the inset tint.accent surface.",
                  spec, "bbr-palette-result-<k>", ["Results fade in with a 20 ms stagger and a 4 pt rise (≤ 260 ms); ↑↓ commit the selection at once, the highlight glides in dur:micro.",
                                                   "Tab results switch to the tab (and its space); history and URL rows open a new tab; Search uses the default engine (L-5).",
                                                   "A cross-space identity or shortcut takes the URL column when present (C-5)."])
    print(b.finish())

if "C-3" in which:
    def spec(t):
        res = [result_row(t, "Search “qwzx plumbing berlin” with Google", None, ("glyph", "search"), "Search", True),
               result_row(t, "Open “qwzx plumbing berlin” as an address", None, ("glyph", "globe"), "URL")]
        return std(t, hover=None, overlays=[palette(t, "qwzx plumbing berlin", res, message="No tabs, pages or history match")])
    b = pal_board("C-3", "When nothing local matches, the palette says so in one quiet line and keeps the two useful answers: search the web, or treat the text as an address. ↵ always does something predictable.", spec,
                  "bbr-palette-result-<k>", ["The empty-match line is ink.faint type:meta and is not selectable.", "Whether a string is an address or a search follows the general search-settings contract."])
    print(b.finish())

if "C-4" in which:
    def spec(t):
        acts = [("Pin Tab", "⇧⌘P", "pin", True), ("Move Tab to Space…", "⇧⌘M", "grid", False), ("Reopen Closed Tab", "⇧⌘T", "history", False), ("Save Page", "⌘D", "bookmark", False),
                ("Find in Page", "⌘F", "search", False), ("New Space", "⇧⌘N", "plus", False), ("Toggle Sidebar", "⌘B", "sidebar", False), ("Settings…", "⌘,", "gear", False)]
        res = [result_row(t, a, None, ("glyph", g), k, s, None) for a, k, g, s in acts]
        return std(t, hover=None, overlays=[palette(t, "> ", res, mode="ACTIONS")])
    b = pal_board("C-4", "Typing > switches the palette to actions: every browser command with its shortcut, filtered as you type. The mode pill says ACTIONS; the shortcut takes the kind column so the palette teaches the keyboard.", spec,
                  "bbr-palette-mode", ["Mark Change: action rows exist in the shipped engine (kind Action); the > prefix mode and the shortcut column are proposals (delta D-2).",
                                       "Actions act on the selected tab or the focused split pane; disabled actions are omitted, not greyed."])
    print(b.finish())

if "C-5" in which:
    def spec(t):
        res = [result_row(t, "Linear — Sprint 42", None, ("site", "violet", "L"), "Tab", True, "lin", subtitle="Work · Profile Work"),
               result_row(t, "Linear — Roadmap", None, ("site", "violet", "L"), "Tab", False, "lin", subtitle="Research"),
               result_row(t, "linear.app/settings", "linear.app/settings", ("site", "violet", "L"), "History", False, "lin")]
        return std(t, hover=None, overlays=[palette(t, "lin", res)])
    b = pal_board("C-5", "A tab in another space names the space; one in another profile names both. Choosing it switches space (and profile context) through the normal switch; the next command field shows 'in Profile · Work' once.", spec,
                  "bbr-command-profile-hint, bbr-profile-chip-<n>", ["The hint clears on the next command-surface interaction (⌘L, ⌘T, ⇧⌘M, new-space staging).", "Identity text, never colour alone: the subtitle carries the names."])
    row = b.row(gap=40, name="Hint")
    for t in (LIGHT, DARK):
        tw = T(t.mode, tint="cobalt")
        col = b.column(row, name="Hint · " + t.label())
        b.caption(col, "After the switch · ⌘T · " + t.label(), "one-interaction profile hint")
        spec = dict(std(tw, hover=None, heading=("Work", "bag"), address={"domain": "linear.app"}, footer={"spaces": ("code", "bag", "moon"), "active": 1}), groups=work_rows(tw),
                    overlays=[palette(tw, hint=("in Profile · Work", "W", "violet"))])
        window(b, col, tw, spec)
    print(b.finish())

if "C-6" in which:
    def spec(t):
        res = [lresult(t, "x.com/home", "https://x.com/home", ("site", "ink", "X"), "URL", True),
               lresult(t, "Home / X", "x.com/home", ("site", "ink", "X"), "Tab"),
               lresult(t, "x.com/notifications", "x.com/notifications", ("site", "ink", "X"), "History"),
               lresult(t, "Search “https://x.com/home”", None, ("glyph", "search"), "Search")]
        return std(t, hover=None, overlays=[overlay_l(t, results=res)])
    b = pal_board("C-6", "⌘L (or a click on the address) opens the palette over the address control: flush with its top-left, 440 pt to the palette width, the full URL selected so one keystroke replaces it. The address never appears twice.", spec,
                  "bbr-command-overlay, bbr-command-overlay-field, -text, -cursor, bbr-command-result-<k>",
                  ["Field 40 in type:mono, rows 36 in a 4 pt list, radius lg 12; the same result engine as ⌘T.", "Escape restores focus where it was (page or rail); ↵ navigates the current tab."])
    print(b.finish())

# ---------------------------------------------------------------- page 24
if "G-1" in which:
    b = Board("G-1", width=2080, desc="Before WebKit presents a page the card shows material:canvas — nothing else, no spinner over the glass. A blank page is the glass colour. The card's clipped edge is the only boundary; the chrome never samples, blurs or re-renders web content.", ids="bbr-content-card")
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        for label, content, bg in (("Before attach", '<div style="flex:1 1 0"></div>', "canvas"), ("about:blank", '<div style="flex:1 1 0"></div>', "glass"), ("Page", page_placeholder(t, lines=4), "glass")):
            col = b.column(row, name=label)
            b.caption(col, f"{label} · {t.label()}")
            card_plate(b, col, t, content.replace("flex:1 1 0", f"flex:1 1 0;background-color:{t.c(bg)}", 1) if bg == "canvas" else content, 600, 380)
    b.notes(["Loading feedback lives in the chrome (row arc, address line), never on the glass (§18.5).", "Web content is always a neutral placeholder on these boards (never a site screenshot)."])
    print(b.finish())

if "G-2" in which:
    b = Board("G-2", width=3040, desc="A space without tabs shows the D10 illustration — six tab cards floating around a ⌘ key — the headline, the owning profile and one primary action. The cards drift for two cycles when the pane appears, then rest on this frame.", ids="bbr-empty-new-tab, bbr-empty-new-tab-button, motion.illustration")
    def spec(t):
        p = pane(t, empty_illustration(t), "No tabs in this space", "in Profile · Default", button(t, "New Tab   ⌘T", "primary", size="l"), name="bbr-empty-new-tab")
        return std(t, heading=("Research", "book"), favorites=None, groups=["new"], footer={"spaces": ("code", "bag", "book"), "active": 2}, address={"domain": "New Tab", "glyph_name": "plus"}, card=p, card_bg="canvas")
    pair(b, spec)
    b.notes(["Closing the last tab opens the empty New Tab palette first; this pane is what Escape returns to.", "Reduce Motion shows only this rest frame (V-4)."])
    print(b.finish())

if "G-3" in which:
    b = Board("G-3", width=3040, desc="A crashed page keeps its tab: the row shows the lifering, the card shows the heartbeat illustration, the title and Reload / Close tab. No automatic reload.", ids="bbr-crash-panel, bbr-crash-reload, bbr-crash-close")
    def spec(t):
        p = pane(t, heartbeat(t), "This page crashed", "The tab stays open; reload to restore it.", button(t, "Reload", "primary") + button(t, "Close tab", "secondary"), bg="field", title_role="title-xl", name="bbr-crash-panel")
        regs = regular(t, hover=None)
        regs[3] = tab_row(t, "Daybreak | OpenAI for Enterprise", ("lifering",), "selected")
        s = std(t, hover=None, selected_pinned=None, card=p)
        s["groups"][3] = regs
        return s
    pair(b, spec)
    b.notes(["Focus order: Reload → Close tab → rail. Reload recreates the session (fresh web view, same tab and profile).", "The beat sweeps once, decelerates into this frame, then rests."])
    print(b.finish())

if "G-4" in which:
    b = Board("G-4", width=3040, desc="A renderer that misses its liveness deadline keeps its tab and session; the chrome stays responsive. The pane offers Wait, Reload or Close tab; nothing reloads by itself and nothing pulses.", ids="bbr-unresponsive-panel, bbr-unresponsive-wait, -reload, -close")
    def spec(t):
        clock = f'<div style="width:56px;height:56px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:{t.c("wash-hover")}">{icon("clock", t.c("ink-secondary"), 28)}</div>'
        p = pane(t, clock, "This page is not responding", "The page is busy; the tab stays open.", button(t, "Wait", "primary") + button(t, "Reload", "secondary") + button(t, "Close tab", "secondary"), bg="field", title_role="title-xl", name="bbr-unresponsive-panel")
        regs = regular(t, hover=None)
        regs[2] = tab_row(t, "Design editor — board", ("lifering",), "selected", trailing="clock")
        s = std(t, hover=None, selected_pinned=None, card=p)
        s["groups"][3] = regs
        return s
    pair(b, spec)
    b.notes(["Wait keeps the same session and resumes bounded monitoring; a completing probe clears the state immediately; a crash takes precedence (G-3).",
             "Never shown over Peek, a non-focused split pane or an error surface."])
    print(b.finish())

if "G-5" in which:
    b = Board("G-5", width=3040, desc="Classified load errors replace the glass with a calm pane: what happened, what to do, one action. Offline shows the D10 link illustration; certificate and navigation errors keep the lifering, because their cause is not the network.", ids="bbr-error-panel, bbr-error-title, bbr-error-copy, bbr-error-url, bbr-error-action, bbr-error-dismiss")
    lif = lambda t: f'<div style="width:56px;height:56px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:{t.c("status-error-surface")}">{icon("lifering", t.c("status-error"), 30)}</div>'
    states = [("Offline", lambda t: pane(t, offline_link(t), "You're offline", "The network can't reach this site right now. Check your connection and try again.", button(t, "Try again", "primary") + button(t, "Dismiss", "secondary"), bg="field", title_role="title-xl")),
              ("Server not found", lambda t: pane(t, lif(t), "This page can't be loaded", "The site's server couldn't be found. Check the address and try again.", button(t, "Try again", "primary") + button(t, "Dismiss", "secondary"), bg="field", title_role="title-xl", url="https://exmaple.com")),
              ("Certificate", lambda t: pane(t, lif(t), "This page can't be loaded securely", "The site's security certificate has expired, so the connection wasn't made. Contact the site owner or try again later.", button(t, "Go back", "primary") + button(t, "Dismiss", "secondary"), bg="field", title_role="title-xl", url="https://expired.badssl.com"))]
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        for label, fn in states:
            col = b.column(row, name=label)
            b.caption(col, f"{label} · {t.label()}")
            card_plate(b, col, t, fn(t), 940, 560)
    b.notes(["Copy per reason comes from the error-surface contract (DNS, refused, timeout, redirects, scheme, certificate expired / not yet valid / untrusted / revoked / name mismatch).",
             "Certificate errors offer no 'proceed anyway' in v1; Go back returns to the previous page.",
             "The offline packets run twice when the pane appears, then rest (V-5)."])
    print(b.finish())

if "G-6" in which:
    b = Board("G-6", width=3040, desc="⌘F opens the find bar top-right over the card through the native Portal: field, match count, previous/next, close. The page highlights matches natively; the chrome never paints over them.", ids="bbr-find-bar, bbr-find-field, bbr-find-text, bbr-find-count, bbr-find-previous, bbr-find-next, bbr-find-close")
    def bar(t, q, count, tone="ink-faint"):
        return (f'<div layer-name="bbr-find-bar" style="position:absolute;right:24px;top:24px;width:360px;height:36px;display:flex;align-items:center;gap:6px;padding:0 4px 0 10px;border-radius:var(--radius-lg);'
                f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{icon("search", t.c("ink-faint"), 14)}'
                f'<div style="flex:1 1 0;display:flex;align-items:center;gap:2px">{text(q, "label", t.c("ink"), weight=400, clamp=False)}<div style="width:1.5px;height:16px;background-color:{t.c("accent")}"></div></div>'
                f'{text(count, "meta", t.c(tone), clamp=False, extra="font-variant-numeric:tabular-nums;")}{icon_button(t, "chevron-down", size=26).replace("M4.5 7 9 11.5 13.5 7", "M4.5 11 9 6.5 13.5 11")}{icon_button(t, "chevron-down", size=26)}{icon_button(t, "close", size=26)}</div>')
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name=t.label())
        b.caption(col, "3 of 12 · " + t.label())
        window(b, col, t, std(t, hover=None, card_overlays=[bar(t, "lifetime", "3 of 12")]))
    row = b.row(gap=32, name="States")
    for t in (LIGHT, DARK):
        for label, q, c, tone in (("No match", "lifetimes'", "No matches", "status-attention"), ("Empty", "", "", "ink-faint")):
            col = b.column(row, name=label)
            b.caption(col, f"{label} · {t.label()}")
            card_plate(b, col, t, page_placeholder(t, lines=4), 680, 260, overlays=[bar(t, q, c, tone)])
    b.notes(["↵ / ⇧↵ step through matches; Escape closes and returns focus to the page; the query is kept per tab for the session.",
             "The bar is 36 pt (density row), radius lg, 1 pt edge, elev:2; the count uses tabular numerals."])
    print(b.finish())

if "G-7" in which:
    b = Board("G-7", width=3040, desc="Zoom is per site and per profile. ⌘+ / ⌘− / ⌘0 change it; the address shows a chip while it is not 100 %, pressing the chip resets. A transient level readout confirms each step.", ids="bbr-address-zoom")
    def spec(t, z="140%"):
        readout = (f'<div layer-name="Zoom readout (proposal)" style="position:absolute;left:50%;top:24px;width:120px;margin-left:-60px;height:32px;display:flex;align-items:center;justify-content:center;gap:10px;border-radius:var(--radius-lg);'
                   f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{icon("minus", t.c("ink-secondary"), 14)}{text(z, "key", t.c("ink"), clamp=False)}{icon("plus", t.c("ink-secondary"), 14)}</div>')
        return std(t, hover=None, address={"domain": "github.com", "zoom": z}, card_overlays=[readout])
    pair(b, spec)
    b.notes(["Steps: 50 · 67 · 80 · 90 · 100 · 110 · 125 · 140 · 160 · 200 %. The readout fades after 1.2 s (proposal, delta D-2); the chip stays.",
             "Site settings (M-8) list the saved zoom levels per profile."])
    print(b.finish())

if "G-8" in which:
    b = Board("G-8", width=3040, desc="A link that asks for a new window opens as a Peek over the card: the preview loads in the source profile while the origin stays selected and warm underneath. Open as Tab promotes the same session; Escape dismisses.", ids="bbr-peek-bar, bbr-peek-url, bbr-peek-state, bbr-peek-back, -forward, -reload, bbr-peek-open-as-tab, bbr-peek-close")
    def peek(t, state="Peek"):
        lead = spinner(t, 16) if state == "Loading" else (slot(icon("lifering", t.c("status-error"), 14), 16) if state == "Crashed" else "")
        from blocks import badge
        bar = (f'<div layer-name="bbr-peek-bar" style="height:36px;flex-shrink:0;display:flex;align-items:center;gap:8px;padding:0 6px 0 12px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
               f'{lead}{text("Understanding lifetimes — Rust by Example", "label", t.c("ink"), grow=True)}{text("doc.rust-lang.org", "mono", t.c("ink-faint"), clamp=False)}{badge(t, state, "accent" if state == "Peek" else ("error" if state == "Crashed" else "neutral"))}'
               f'{icon_button(t, "back", size=26)}{icon_button(t, "forward", t.c("ink-faint"), size=26)}{icon_button(t, "reload", size=26)}'
               f'{button(t, "Open as Tab", "secondary", size="s", icon_name="external")}{icon_button(t, "close", size=26)}</div>')
        body = page_placeholder(t, lines=3) if state != "Crashed" else pane(t, heartbeat(t), "This preview crashed", "Close it and open the link again.", "", bg="field", title_role="title")
        return (f'<div layer-name="Peek" style="position:absolute;left:24px;right:24px;top:16px;bottom:16px;display:flex;flex-direction:column;gap:8px">{bar}'
                f'<div style="flex:1 1 0;display:flex;border-radius:10px;overflow:hidden;box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{body}</div></div>')
    pair(b, lambda t: std(t, hover=None, card_overlays=[f'<div style="position:absolute;inset:0;background-color:{t.c("wash-active")}"></div>', peek(t)]))
    row = b.row(gap=32, name="States")
    for t in (LIGHT, DARK):
        for s in ("Loading", "Crashed"):
            col = b.column(row, name=s)
            b.caption(col, f"{s} · {t.label()}")
            card_plate(b, col, t, page_placeholder(t, lines=3), 680, 420, overlays=[f'<div style="position:absolute;inset:0;background-color:{t.c("wash-active")}"></div>', peek(t, s)])
    b.notes(["One preview per pane; a new eligible link from inside the preview navigates it, any other source replaces it.",
             "While open, Back/Forward/Reload/⌘L route to the preview; dismissal returns focus to the exact origin.",
             "The dimmed origin and the inset preview frame are proposals for the later entrance polish; the shipped bar spans the card top (delta D-2)."])
    print(b.finish())
