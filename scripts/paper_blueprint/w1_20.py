import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from shell import *

which = sys.argv[1:]

if "W-1" in which:
    b = Board("W-1", width=3040, desc="One tinted body: the full-height rail and the chassis are one paint; the web page sits in a seated content card, flush with the rail and inset 10 pt elsewhere. Top to bottom the rail holds the titlebar row, the address display, Favorites, the active space heading, the organized area (pinned rows and folders), New Tab, the tabs, and the footer strip.",
              ids="bbr-sidebar, bbr-titlebar, bbr-address, bbr-favorites, bbr-active-space-title, bbr-organized-hairline, bbr-new-tab, bbr-scroll, bbr-space-strip, bbr-content-card")
    pair(b, lambda t: std(t))
    b.notes(["The rail has no right hairline and no border; the card's edge and its seat shadow are the only separation (§2).",
             "Exactly one lifted surface per list: the selected row. Hover is a wash; the close glyph appears on hover and selection.",
             "No section labels: the space heading is the list heading; one hairline closes the organized area against New Tab.",
             "Traffic lights are native (Native OS) and their position comes from the safe titlebar area, never hardcoded."])
    print(b.finish())

if "W-2" in which:
    b = Board("W-2", width=3040, desc="The active space's curated paper tints the whole body — rail and chassis — while the card, text and web content keep their own tokens. Shown: the Work space in Cobalt at the standard step. The selection lift, the veils and the active chip follow the tint.", ids="bbr-space-strip, bbr-space-chip-<n>")
    def spec(t):
        return std(T(t.mode, tint="cobalt"), heading=("Work", "bag"), footer={"spaces": ("code", "bag", "moon"), "active": 1}, address={"domain": "linear.app"})
    row = b.row(gap=40)
    for t in (T("light", tint="cobalt"), T("dark", tint="cobalt")):
        col = b.column(row, name=t.label())
        b.caption(col, t.label())
        window(b, col, t, std(t, heading=("Work", "bag"), footer={"spaces": ("code", "bag", "moon"), "active": 1}, address={"domain": "linear.app"}))
    b.notes(["Each step blends the token's chrome wash into the rail and pins it to the untinted rail's luminance, so text bars never move (F-2, V-2).",
             "The active chip is tint.selected over the tinted rail with 8 % of the identity hue and elev:lift.",
             "A committed switch paints the destination tint on the commit frame and cross-fades only the chrome over dur:standard; a swipe interpolates with the gesture (P-1)."])
    print(b.finish())

if "W-3" in which:
    b = Board("W-3", width=3040, desc="⌘B hides the rail instantly. The card gains the leading inset and its top inset grows to the titlebar band, which stays empty, draggable chassis with the native traffic lights. Proposal: resting the pointer at the left edge for 300 ms reveals the rail as a floating panel over the card, without moving the card; it hides when the pointer leaves.",
              ids="bbr-chassis-band, bbr-titlebar-sidebar-toggle, bbr-sidebar-divider")
    b.caption(b.column(b.row(gap=0, pad="0 56px 16px 56px"), name="c"), "Hidden · shipped")
    pair(b, lambda t: std(t, sidebar=False))
    b.caption(b.column(b.row(gap=0, pad="0 56px 16px 56px"), name="c2"), "Hover reveal · New · proposal")
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name="Reveal · " + t.label())
        b.caption(col, "Reveal · " + t.label())
        win = window(b, col, t, std(t, sidebar=False))
        panel = b.add(f'<div layer-name="Revealed rail (proposal)" style="position:absolute;left:8px;top:8px;bottom:8px;width:300px;display:flex;border-radius:12px;overflow:hidden;'
                      f'background-color:{t.c("sidebar")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}"></div>', win)
        build_rail(b, panel, t, std(t), 884)
    b.notes(["Shipped: collapse removes the whole BBR titlebar row with the rail; no floating Back/Forward/Reload stays over the page (§11.14).",
             "Proposal (delta D-2): edge hover reveals the rail over the card at elev:2; it never reflows the card or moves the web view; Escape or leaving hides it; Reduce Motion shows it without the 120 ms slide.",
             "Collapse memory is global per window; expand restores the last width (§11.4)."])
    print(b.finish())

if "W-4" in which:
    b = Board("W-4", width=4000, desc="The rail keeps its width until the card would fall below 64 pt; then the rail yields (rail = min(requested, window − 64 − 10)). Wide windows give every extra point to the card. Floating surfaces clamp into the window.")
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name="Narrow · " + t.label())
        b.caption(col, "Narrow 720 × 640 · " + t.label())
        window(b, col, t, std(t, hover=None), 720, 640)
    for t in (LIGHT, DARK):
        col = b.column(row, name="Minimum · " + t.label())
        b.caption(col, "Minimum 360 × 560 · " + t.label(), "rail yields to 286 to keep a 64 pt card")
        window(b, col, t, std(t, hover=None, rail_width=286, card=None), 360, 560)
    row2 = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row2, name="Wide · " + t.label())
        b.caption(col, "Wide 1920 × 1080 · " + t.label())
        window(b, col, t, std(t, hover=None), 1920, 1080)
    b.notes(["Row heights, type and the rail width never scale with the window; density is the user's choice (V-3).",
             "Below the minimum the rail keeps 64 pt of card visible; the address and titles ellipsize (domain mid-ellipsis, titles at the end).",
             "Narrow windows centre the ⌘T palette over the card and clamp it to the window (C-1); menus and sheets clamp the same way."])
    print(b.finish())

if "W-5" in which:
    b = Board("W-5", width=3040, desc="Native full screen is full-bleed: the card loses its inset, radius and shadow; the rail stays; the traffic lights and the menu bar appear only when the pointer reaches the top edge (native). ⌘B hides the rail for a pure page.")
    pair(b, lambda t: std(t, titlebar={"lights": False}, card_pad="0", card_radius=0, win_radius=0))
    b.notes(["Inset 0, radius 0, no shadow (§2 rule 5); the titlebar row keeps the sidebar toggle and navigation without traffic lights.",
             "Mark New · proposal for the documented board only: the behaviour follows DESIGN §2; no separate packet owns a full-screen check yet (delta D-2)."])
    print(b.finish())

if "W-6" in which:
    b = Board("W-6", width=3040, desc="A navigation commits instantly: the row shows the loading arc, its title steps to ink.secondary, Reload becomes Stop, and the address field carries the 2 pt D5 progress line. Web content paints when WebKit presents it; nothing is drawn over the glass.",
              ids="bbr-address-progress, bbr-nav-reload, loading-arc moment")
    pair(b, lambda t: std(t, selected_pinned=None, selected="Cloudflare", loading="Cloudflare", hover=None, titlebar={"loading": True},
                          address={"domain": "dash.cloudflare.com", "progress": 45}, card=page_placeholder(t, lines=2)))
    row = b.row(gap=24, name="Progress frames")
    for t in (LIGHT, DARK):
        col = b.column(row, name="Frames · " + t.label(), width=1480)
        b.caption(col, "Address progress frames · " + t.label(), "indeterminate sweep → finish fills (dur:standard) → fades (dur:standard) → leaves the tree")
        frames = "".join(f'<div style="display:flex;flex-direction:column;gap:8px;width:300px">{address(t, "dash.cloudflare.com", progress=pc).replace("padding:0 4px 12px 4px", "padding:0")}{text(lbl, "key", t.c("ink-faint"), weight=400, clamp=False)}</div>'
                         for pc, lbl in ((20, "sweeping"), (60, "sweeping"), (100, "finish · filled"), (None, "rest")))
        plate(b, col, t, f'<div style="display:flex;gap:24px">{frames}</div>', w=1400, pad=24)
    b.notes(["Stop cancels the load (⌘.), the arc stops and the row returns to its favicon; Reduce Motion shows a static line and a static arc.",
             "Title changes and favicons arrive without layout shifts; the row keeps its place."])
    print(b.finish())

if "W-7" in which:
    b = Board("W-7", width=1560, desc="[traffic lights] [sidebar toggle] … [Back] [Forward] [Reload]. Enablement follows the real navigation state of the selected tab or focused split pane; glyphs are 18 pt on 32 pt hit areas with the hover wash and the disabled treatment.",
              ids="bbr-titlebar, bbr-nav-back, bbr-nav-forward, bbr-nav-reload")
    row = b.row(gap=24)
    for t in (LIGHT, DARK):
        col = b.column(row, name=t.label(), width=700)
        b.caption(col, "Titlebar · " + t.label())
        ink2, faint = t.c("ink-secondary"), t.c("ink-faint")
        def bar(label, back=True, fwd=False, loading=False, hover=None, pressed=None, focus=None):
            parts = [traffic_lights(t), icon_button(t, "sidebar", extra=f"box-shadow:0 0 0 2px {t.c('accent')};" if focus == "sidebar" else ""), '<div style="flex:1 1 0"></div>']
            for name, ic, enabled in (("back", "back", back), ("forward", "forward", fwd), ("reload", "stop" if loading else "reload", True)):
                extra = ""
                if hover == name:
                    extra = f"background-color:{t.c('wash-hover')};"
                if pressed == name:
                    extra = f"background-color:{t.c('wash-active')};"
                parts.append(icon_button(t, ic, ink2 if enabled else faint, extra=extra))
            return (f'<div style="display:flex;align-items:center;gap:16px"><div style="width:150px;flex-shrink:0">{text(label, "key", t.c("ink-faint"), weight=400, clamp=False)}</div>'
                    f'<div style="width:300px;height:52px;display:flex;align-items:center;gap:8px;padding:0 12px;border-radius:8px;background-color:{t.c("sidebar")}">{"".join(parts)}</div></div>')
        rows = [bar("back only"), bar("both enabled", True, True), bar("no history", False, False), bar("loading → Stop", True, False, True),
                bar("hover Back", hover="back"), bar("pressed Back", pressed="back"), bar("focus toggle", focus="sidebar")]
        tip = (f'<div style="display:flex;align-items:center;gap:16px"><div style="width:150px">{text("disabled reason", "key", t.c("ink-faint"), weight=400)}</div>'
               f'<div style="padding:4px 8px;border-radius:6px;background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{text("No back history", "meta", t.c("ink"), clamp=False)}</div>'
               f'{text("native tooltip", "key", t.c("ink-faint"), weight=400, clamp=False)}</div>')
        plate(b, col, t, rows + [tip], w=700, pad=24, bg="canvas")
    b.notes(["Long-press or secondary click on Back/Forward opens the session history menu (New · proposal; M-3 anatomy).",
             "Trackpad swipes over web content drive the same history authority (PR-03).",
             "Keyboard: ⌘[ ⌘] ⌘R ⌘. — the buttons reflect the committed state, never a guess."])
    print(b.finish())

if "W-8" in which:
    b = Board("W-8", width=3040, desc="500 tabs stay responsive: rows keep one height, only the visible range plus 8 rows of overscan render, pinned rows and Favorites stay fixed above the scroll view, and the selected row scrolls into view instantly.", ids="bbr-scroll, bbr-tab-row-<n>")
    names = ["Pull request #4812 · gpui", "Issue #377 · wgpu", "crates.io: tokio", "docs.rs · serde_json", "Stack Overflow — lifetimes", "RFC 3498 · lifetime capture",
             "Rust Blog — 2026 roadmap", "The Rustonomicon", "cargo-nextest · docs", "Zed — GPUI internals", "Tracking issue #91611", "rustc dev guide",
             "Compiler Explorer", "Pull request #4855 · gpui", "wgpu shaders · WGSL", "Discussions · WebKit", "WKWebView | Apple Docs", "NSView layer clipping"]
    def spec(t):
        rows = [tab_row(t, n, ("mono", n[0].upper(), i % 6 + 1), "selected" if i == 7 else "rest") for i, n in enumerate(names)]
        thumb = f'<div layer-name="Scroll thumb" style="position:absolute;right:3px;top:210px;width:6px;height:44px;border-radius:9999px;background-color:{t.c("ink-faint")};opacity:0.4"></div>'
        return std(t, address={"domain": "doc.rust-lang.org"}, heading=("Research", "book"), favorites=FAVS[:2],
                   groups=[[tab_row(t, "Rust Playground", ("mono", "P", 3)), tab_row(t, "This Week in Rust", ("mono", "T", 4))], "divider", "new", rows, thumb])
    pair(b, spec)
    b.notes(["Scrollbar: an 8 pt track-less thumb in ink.faint at 40 % (60 % on hover); keyboard ↑ ↓ PgUp PgDn move selection and viewport together.",
             "The ⌘T palette is the fast path through 500 tabs; the rail stays a stable map.",
             "Performance: open/close, strip switching, swipe and palette keep their budgets at 500 tabs (performance contract)."])
    print(b.finish())

if "W-9" in which:
    b = Board("W-9", width=3040, desc="Every window is one full shell with its own active space, rail width and native Portal. Spaces, profiles and tabs are shared state; a tab lives in one window at a time. The inactive window keeps its paint, its traffic lights turn grey (native) and its selection stays lifted.")
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name="Desktop · " + t.label())
        b.caption(col, "Two windows · " + t.label())
        desk = b.add(f'<div layer-name="Desktop" style="position:relative;width:1440px;height:900px;flex-shrink:0;border-radius:12px;background-color:{DOC.c("border-strong") if not t.dark else "var(--color-dark-canvas)"}"></div>', col)
        tint_t = T(t.mode, tint="cobalt")
        window(b, desk, tint_t, std(tint_t, heading=("Work", "bag"), footer={"spaces": ("code", "bag", "moon"), "active": 1}, address={"domain": "linear.app"}, hover=None, titlebar={"dim": True}, ring=tint_t.c("elev-ring") if t.dark else DOC.c("hairline")),
               1100, 700, name="Window 2 · inactive", )
        w2 = window(b, desk, t, std(t, hover=None, ring=t.c("elev-ring") if t.dark else DOC.c("hairline")), 1100, 700, name="Window 1 · key")
        paper.call("update_styles", {"updates": [{"nodeIds": [w2], "styles": {"position": "absolute", "left": "300px", "top": "170px"}}]})
    b.notes(["⌘N opens a new window on the current space (the new-window seam); ⌘W on the last tab keeps the window (empty space); ⌘⇧W closes the window.",
             "Multi-window Portal ownership, display moves and teardown are hardened by native-portal-hardening.",
             "Moving a tab to another window is drag between rails (New · proposal)."])
    print(b.finish())
