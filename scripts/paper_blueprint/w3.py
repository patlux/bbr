import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from cmdsurf import *
from blocks import menu, menu_row, menu_sep, menu_header, sheet, button, field, checkbox, switch, segmented, popup_button, badge, banner, progress
from w1_21 import frag, frag_rows

which = sys.argv[1:]
VEIL = lambda t: f'<div layer-name="Blocking veil" style="position:absolute;inset:0;background-color:{t.c("wash-active")}"></div>'
TOP_SHEET = lambda t, sh: f'<div style="position:absolute;left:0;right:0;top:0;display:flex;justify-content:center">{sh}</div>'


def at(x, y, html):
    return f'<div style="position:absolute;left:{x}px;top:{y}px">{html}</div>'


def p(t, s, colour="ink-secondary"):
    return f'<div style="font-family:var(--font-ui);font-size:var(--text-label);line-height:20px;color:{t.c(colour)}">{esc(s)}</div>'


def lib_surface(t, title, subtitle, side, toolbar, groups, name):
    side_html = "".join(
        f'<div style="height:32px;display:flex;align-items:center;gap:8px;padding:0 10px;border-radius:var(--radius-md);'
        + (f'background-color:{t.c("tint-selected")};box-shadow:{t.lift()}' if i == 0 else "") + f'">{icon(ic, t.c("ink-secondary"), 16)}{text(l, "label", t.c("ink"), weight=500 if i == 0 else 400, grow=True)}{text(n, "meta", t.c("ink-faint"), clamp=False)}</div>'
        for i, (ic, l, n) in enumerate(side))
    rows = ""
    for head, items in groups:
        rows += f'<div style="height:32px;display:flex;align-items:flex-end;padding:0 12px 6px 12px">{text(head, "badge", t.c("ink-faint"), clamp=False)}</div>'
        for it in items:
            rows += it
    return (f'<div layer-name="{esc(name)}" style="flex:1 1 0;display:flex;background-color:{t.c("field")}">'
            f'<div style="width:220px;flex-shrink:0;display:flex;flex-direction:column;gap:4px;padding:20px 12px;background-color:{t.c("canvas")};border-right:1px solid {t.c("hairline")}">'
            f'<div style="padding:0 10px 12px 10px;display:flex;flex-direction:column;gap:2px">{text(title, "title-xl", t.c("ink"), weight=600)}{text(subtitle, "meta", t.c("ink-faint"), clamp=False)}</div>{side_html}</div>'
            f'<div style="flex:1 1 0;min-width:0;display:flex;flex-direction:column"><div style="height:60px;flex-shrink:0;display:flex;align-items:center;gap:12px;padding:0 20px;border-bottom:1px solid {t.c("hairline")}">{toolbar}</div>'
            f'<div style="flex:1 1 0;display:flex;flex-direction:column;padding:8px 12px;overflow:hidden">{rows}</div></div></div>')


def lib_row(t, title, url, fav, meta="", selected=False, lead=None):
    bg = f"background-color:{t.c('tint-accent')};" if selected else ""
    lead_html = lead if lead is not None else ""
    return (f'<div style="height:40px;flex-shrink:0;display:flex;align-items:center;gap:12px;padding:0 12px;border-radius:var(--radius-md);{bg}">{lead_html}{favicon(t, fav)}'
            f'<div style="flex:0 1 auto;min-width:0;max-width:46%">{text(title, "title", t.c("ink"))}</div>{text(url, "mono", t.c("ink-faint"), extra="flex:1 1 0;min-width:0;")}'
            f'{text(meta, "meta", t.c("ink-faint"), clamp=False, extra="font-variant-numeric:tabular-nums;")}</div>')


if "M-1" in which:
    b = Board("M-1", width=3040, desc="Secondary click on a tab row opens the tab menu at the pointer; it acts on exactly that tab and never selects, activates or reloads it by opening. Submenus open to the right; the menu edge turns accent while it owns the keyboard.", ids="bbr-tab-menu, bbr-tab-menu-<action>")
    def spec(t):
        m = menu(t, [menu_row(t, "Reload Tab", icon_name="reload"), menu_row(t, "Suspend Tab", icon_name="moon"), menu_sep(t),
                     menu_row(t, "Pin Tab", shortcut="⇧⌘P", icon_name="pin"), menu_row(t, "Add to Favorites", icon_name="star"),
                     menu_row(t, "Move to Folder", icon_name="folder", submenu=True), menu_row(t, "Move to Space", "cursor", icon_name="grid", submenu=True),
                     menu_row(t, "Move to Start"), menu_row(t, "Move to End"), menu_sep(t),
                     menu_row(t, "Bookmark Page", shortcut="⌘D", icon_name="bookmark"), menu_row(t, "Copy Link", icon_name="external"), menu_sep(t),
                     menu_row(t, "Close Tab", shortcut="⌘W", icon_name="close", danger=True)], 250)
        sub = menu(t, [menu_header(t, "Default"), menu_row(t, "Research", icon_name="bookmark"), menu_row(t, "Work · Profile Work", "cursor", icon_name="grid"), menu_row(t, "Private · Profile Private", icon_name="shield")], 240, focused=False)
        return std(t, overlays=[at(186, 480, m), at(432, 690, sub)])
    pair(b, spec)
    b.notes(["Close Tab is the danger row: text colour plus glyph, never colour alone. Destinations in another profile carry the profile name.",
             "Disabled rows appear only where the group needs them and name the reason (e.g. Copy Link without a URL).",
             "Keyboard: ↑↓ traverse, → / ← enter and leave submenus, ↵ commits, Escape or one outside press dismisses."])
    print(b.finish())

if "M-2" in which:
    b = Board("M-2", width=2200, desc="Favorites and folders reuse the menu anatomy. A favorite's menu manages the favorite, not the space; a folder's menu offers a safe delete that keeps the tabs.", ids="bbr-favorite-menu, bbr-folder-menu")
    def items(t):
        fav = menu(t, [menu_row(t, "Reload Tab", icon_name="reload"), menu_row(t, "Copy Link", icon_name="external"), menu_row(t, "Bookmark Page", icon_name="bookmark"), menu_sep(t),
                       menu_row(t, "Remove from Favorites", "cursor", icon_name="star")], 240)
        fol = menu(t, [menu_row(t, "Rename…", "cursor", icon_name="folder"), menu_row(t, "New Tab in Folder", icon_name="plus"), menu_row(t, "Open All Tabs", icon_name="popup"), menu_sep(t),
                       menu_row(t, "Delete Folder — keep its tabs", icon_name="trash"), menu_row(t, "Close Folder and Its Tabs…", icon_name="trash", danger=True)], 270)
        return [frag(t, address(t, "x.com") + favorites(t, FAVS, {1: "hover"}) + f'<div style="padding:0 8px 8px 60px">{fav}</div>', "Favorite menu", "opened on a tile", w=320),
                frag(t, f'<div style="display:flex;flex-direction:column;gap:4px;padding:0 8px">{folder_row(t, "Home")}{folder_row(t, "Review Tools", state="hover", count=4)}</div><div style="padding:4px 8px 8px 40px">{fol}</div>', "Folder menu", "safe delete ungroups", w=330)]
    frag_rows(b, items, gap=40)
    b.notes(["'Delete Folder — keep its tabs' moves the members back to loose pinned rows; the danger row closes them and names the count in its confirmation.",
             "Favorite menu rows never switch space or select the tab."])
    print(b.finish())

if "M-3" in which:
    b = Board("M-3", width=3040, desc="Secondary click on page background opens BBR's page menu above WebKit through the Portal. Links, images, selections, editable text and media keep WebKit's native element menus (Native OS). Long-press on Back shows the session history.", ids="bbr-page-menu, bbr-nav-history-menu")
    def spec(t):
        pm = menu(t, [menu_row(t, "Back", "cursor", "⌘[", "back"), menu_row(t, "Forward", icon_name="forward", disabled=True), menu_row(t, "Reload", shortcut="⌘R", icon_name="reload"), menu_sep(t),
                      menu_row(t, "Bookmark Page", shortcut="⌘D", icon_name="bookmark"), menu_row(t, "Copy Page Link", icon_name="external"), menu_row(t, "Find in Page…", shortcut="⌘F", icon_name="search"),
                      menu_sep(t), menu_row(t, "Open in Split View", icon_name="split"), menu_row(t, "Zoom", icon_name="plus", submenu=True)], 240)
        native = (f'<div layer-name="WebKit element menu · Native OS" style="width:230px;display:flex;flex-direction:column;gap:2px;padding:6px;border-radius:8px;background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("border-strong")}, {t.elev2()}">'
                  + "".join(f'<div style="height:22px;display:flex;align-items:center;padding:0 8px">{text(l, "label", t.c("ink"), weight=400)}</div>' for l in ("Open Link in New Tab", "Open Link in Background", "Copy Link", "Share…"))
                  + f'<div style="padding:4px 8px 0 8px">{text("WebKit element menu · Native OS", "key", t.c("ink-faint"), weight=400, clamp=False)}</div></div>')
        hist = menu(t, [menu_header(t, "Back"), menu_row(t, "Pull requests · bbr", "cursor", icon_name="history"), menu_row(t, "Issue #377 · wgpu", icon_name="history"), menu_row(t, "crates.io: tokio", icon_name="history"), menu_sep(t), menu_row(t, "Show Full History", shortcut="⌘Y")], 240)
        return std(t, hover=None, overlays=[at(700, 240, pm), at(1060, 520, native), at(206, 88, hist)])
    pair(b, spec)
    b.notes(["The page menu opens at the pointer, clamped to the window, and binds the exact page/pane that opened it.",
             "BBR never repaints or extends WebKit element menus; extension rows, QR codes or 'send to device' do not exist without a contract.",
             "The history menu (long-press or secondary click on Back/Forward) is a proposal (delta D-2)."])
    print(b.finish())

if "M-4" in which:
    b = Board("M-4", width=2200, desc="The footer's Downloads control opens an anchored popover above the footer, inside the rail: Active above, History below, one contextual action per row, Clear for history. It never covers the web view.",
              ids="bbr-downloads-panel, bbr-downloads-active, bbr-downloads-history, bbr-download-<n>, bbr-download-delete-<n>, bbr-downloads-clear")
    def row(t, name, lead, line2, action, hover=False):
        bg = f"background-color:{t.c('wash-hover')};" if hover else ""
        return (f'<div style="min-height:48px;display:flex;align-items:center;gap:10px;padding:6px 8px 6px 10px;border-radius:var(--radius-md);{bg}">{lead}'
                f'<div style="flex:1 1 0;min-width:0;display:flex;flex-direction:column;gap:4px">{text(name, "label", t.c("ink"))}{line2}</div>{action}</div>')
    def panel(t, empty=False):
        if empty:
            body = f'<div style="height:120px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px">{icon("download", t.c("ink-faint"), 22)}{text("No downloads yet", "label", t.c("ink-secondary"), clamp=False)}</div>'
        else:
            meta = lambda s, c="ink-faint": text(s, "meta", t.c(c), clamp=False, extra="font-variant-numeric:tabular-nums;")
            body = (f'<div style="padding:4px 8px 2px 8px">{text("Active", "badge", t.c("ink-faint"), clamp=False)}</div>'
                    + row(t, "gpui-ce-0.3.2.tar.gz", spinner(t), f'{progress(t, 62, 150)}{meta("11.2 of 18 MB · 32 s left")}', button(t, "Cancel", "quiet", size="s"))
                    + row(t, "wwdc-session-10213.mov", spinner(t), f'<div style="height:4px;width:150px;border-radius:9999px;background-color:{t.c("wash-active")}"><div style="margin-left:40%;width:30%;height:4px;border-radius:9999px;background-color:{t.c("accent")}"></div></div>{meta("48 MB")}', button(t, "Cancel", "quiet", size="s"))
                    + f'<div style="height:9px;display:flex;align-items:center;padding:0 8px"><div style="flex:1 1 0;height:1px;background-color:{t.c("hairline")}"></div></div>'
                    + f'<div style="padding:4px 8px 2px 8px">{text("History", "badge", t.c("ink-faint"), clamp=False)}</div>'
                    + row(t, "invoice-2026-09.pdf", slot(icon("check", t.c("status-ok"), 16)), meta("184 KB · Completed"), button(t, "Show in Finder", "quiet", size="s"), hover=True)
                    + row(t, "dataset-full.zip", slot(icon("warn", t.c("status-error"), 16)), meta("Network failed", "status-error"), button(t, "Retry", "secondary", size="s"))
                    + row(t, "installer.dmg", slot(icon("close", t.c("ink-secondary"), 14)), meta("Cancelled"), button(t, "Restart", "quiet", size="s") + icon_button(t, "trash", size=24))
                    + f'<div style="display:flex;justify-content:flex-end;padding:6px 4px 2px 4px">{button(t, "Clear", "quiet", size="s")}</div>')
        return (f'<div layer-name="bbr-downloads-panel" style="margin:0 4px 4px 4px;display:flex;flex-direction:column;padding:6px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{body}</div>')
    def items(t):
        return [frag(t, '<div style="flex:1 1 0"></div>' + panel(t) + footer(t, spaces=("code", "bag", "moon"), downloads_badge=True), "Active and history", "badge counts in-flight only", w=340, h=560),
                frag(t, '<div style="flex:1 1 0"></div>' + panel(t, True) + footer(t, spaces=("code", "bag", "moon")), "Empty", w=340, h=560)]
    frag_rows(b, items, gap=40)
    b.notes(["Width = rail − 8 (N); grows upward; opens with a dur:fine fade; Escape or an outside press dismisses.",
             "Succeeded rows draw the check with one status.ok ring (V-5). Retry and Restart start a fresh attempt linked to the old one.",
             "Terminal history persists through restarts; live attempts never resurrect. Save location lives in Settings › Downloads (L-7)."])
    print(b.finish())

if "M-5" in which:
    b = Board("M-5", width=3040, desc="⇧⌘L opens the Library over the card: every Saved Page, searchable and sortable, in flat collections. Opening a page creates one fresh tab in the active space. Removing never touches open tabs.", ids="bbr-library, bbr-library-row-<n>, bbr-library-search, bbr-library-sort")
    def spec(t):
        side = [("bookmark", "All Saved Pages", "128"), ("folder", "Reading", "34"), ("folder", "Work", "21"), ("folder", "Fun", "9"), ("plus", "New collection", "")]
        bm = slot(icon("bookmark", t.c("ink-faint"), 14), 16)
        tb = field(t, "", "Search Saved Pages…", width=320) + '<div style="flex:1 1 0"></div>' + popup_button(t, "Recently updated", 180) + button(t, "Done", "secondary")
        rows = [("Today", [lib_row(t, "Understanding lifetimes — Rust by Example", "doc.rust-lang.org/rust-by-example/…", ("mono", "R", 3), "Reading", True, bm),
                           lib_row(t, "Your account security review", "accounts.example.com/security", ("site", "cobalt", "A"), "", False, bm)]),
                ("Earlier", [lib_row(t, "GPUI internals — rendering", "zed.dev/blog/gpui", ("mono", "Z", 1), "Work", False, bm),
                             lib_row(t, "WKWebView | Apple Developer Documentation", "developer.apple.com/documentation/…", ("mono", "W", 2), "Work", False, bm),
                             lib_row(t, "Sourdough, step by step", "theperfectloaf.com/…", ("site", "amber", "T"), "Fun", False, bm),
                             lib_row(t, "untitled", "192.168.1.14:8080", ("mono", "U", 4), "", False, bm)])]
        return std(t, hover=None, card=lib_surface(t, "Saved Pages", "global · all profiles", side, tb, rows, "bbr-library"), address={"domain": "Library", "glyph_name": "bookmark"})
    pair(b, spec)
    b.notes(["Selection is tint.accent with ink text (the §18.4 fix); rename uses the pencil; multi-select removes with one confirmation.",
             "States also drawn by the shipped Gallery: empty, loading, no results, destructive confirmation, thousands of records (windowed).",
             "Escape closes the Library and restores the page underneath."])
    print(b.finish())

if "M-6" in which:
    b = Board("M-6", width=3040, desc="⌘Y opens History in the same Library anatomy: visits per profile grouped by day, searchable, with the recently closed tabs on top. Opening a visit creates a tab in the active space; Clear Browsing Data is one step away.", ids="bbr-history (proposal)")
    def spec(t):
        side = [("history", "All history", ""), ("close", "Recently closed", "6"), ("person", "Profile · Default", ""), ("person", "Profile · Work", "")]
        tb = field(t, "", "Search history…", width=320) + '<div style="flex:1 1 0"></div>' + segmented(t, ["Today", "Week", "All"], 0, 200) + button(t, "Clear Browsing Data…", "secondary")
        tm = lambda s: text(s, "meta", t.c("ink-faint"), clamp=False, extra="width:44px;font-variant-numeric:tabular-nums;")
        rows = [("Today — Monday 28 September", [lib_row(t, "Pull requests · bbr", "github.com/patwoz/bbr/pulls", ("site", "ink", "G"), "", True, tm("18:42")),
                                                lib_row(t, "Rust SIMD on the GPU – a field guide", "simd.rs/guide", ("mono", "V", 3), "", False, tm("18:10")),
                                                lib_row(t, "Linear — Sprint 42", "linear.app/…/sprint-42", ("site", "violet", "L"), "Work", False, tm("17:55")),
                                                lib_row(t, "Grafana — latency", "grafana.internal/d/latency", ("site", "amber", "G"), "Work", False, tm("16:02"))]),
                ("Yesterday", [lib_row(t, "YouTube Music — Discover Mix", "music.youtube.com", ("site", "fuchsia", "Y"), "", False, tm("22:31")),
                               lib_row(t, "Sourdough, step by step", "theperfectloaf.com/…", ("site", "amber", "T"), "", False, tm("11:08"))])]
        return std(t, hover=None, card=lib_surface(t, "History", "per profile · stays on this Mac", side, tb, rows, "History (proposal)"), address={"domain": "History", "glyph_name": "history"})
    pair(b, spec)
    b.notes(["New · proposal (phase 8 'History UI and clear-data controls'): reuses the Library surface, rows and selection.",
             "Profile rows filter; visits of other profiles are marked with the profile name, never merged silently.",
             "Delete on a row removes that visit; ⌘⇧⌫ opens Clear Browsing Data (M-7)."])
    print(b.finish())

if "M-7" in which:
    b = Board("M-7", width=3040, desc="Clearing data is per profile and says exactly what goes. The time range comes first; the destructive button names the action; open tabs stay open.", ids="bbr-clear-data (proposal)")
    def spec(t):
        body = (f'<div style="display:flex;align-items:center;gap:10px">{profile_chip(t, "D", "cobalt", 22)}{p(t, "Profile · Default")}</div>'
                f'<div style="display:flex;align-items:center;justify-content:space-between">{p(t, "Time range", "ink")}{popup_button(t, "Last 24 hours", 170)}</div>'
                f'<div style="display:flex;flex-direction:column;gap:10px;padding:4px 0">{checkbox(t, True, "Browsing history · 1 214 visits")}{checkbox(t, True, "Cookies and site data · signs you out of most sites")}'
                f'{checkbox(t, True, "Cached images and files · 412 MB")}{checkbox(t, False, "Download history")}{checkbox(t, False, "Saved Pages (all profiles)")}</div>'
                f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(t, "Cancel", "secondary")}{button(t, "Clear Data", "danger")}</div>')
        return std(t, hover=None, card_overlays=[VEIL(t), TOP_SHEET(t, sheet(t, body, 440, "Clear Browsing Data"))])
    pair(b, spec)
    b.notes(["New · proposal. The default selection never includes Saved Pages; Saved Pages are global, the rest is per profile.",
             "After clearing, a toast confirms ('Cleared 24 hours of data from Default') with no undo."])
    print(b.finish())

if "M-8" in which:
    b = Board("M-8", width=3040, desc="Pressing the site identity in the address opens site settings for this site and profile: connection, zoom, permissions and site data, each with the current value. It hangs from the address and floats over the card through the Portal.", ids="bbr-site-settings (proposal)")
    def spec(t):
        def line(ic, label, value):
            return (f'<div style="height:34px;display:flex;align-items:center;gap:10px;padding:0 6px">{icon(ic, t.c("ink-secondary"), 16)}{text(label, "label", t.c("ink"), weight=400, grow=True)}{popup_button(t, value, 120).replace("height:28px", "height:26px")}</div>')
        body = (f'<div style="display:flex;align-items:center;gap:10px;padding:2px 6px 8px 6px">{icon("lock", t.c("status-ok"), 16)}<div style="display:flex;flex-direction:column">{text("meet.google.com", "title", t.c("ink"))}{text("Connection is secure · Profile Work", "meta", t.c("ink-secondary"), clamp=False)}</div></div>'
                f'<div style="height:1px;background-color:{t.c("hairline")}"></div>'
                + line("camera", "Camera", "Allow") + line("mic", "Microphone", "Allow") + line("pinmap", "Location", "Ask") + line("bell", "Notifications", "Block") + line("clipboard", "Clipboard", "Ask") + line("popup", "Pop-ups", "Block")
                + f'<div style="height:34px;display:flex;align-items:center;gap:10px;padding:0 6px">{icon("search", t.c("ink-secondary"), 16)}{text("Zoom", "label", t.c("ink"), weight=400, grow=True)}{text("110 %", "key", t.c("ink"), clamp=False)}{button(t, "Reset", "quiet", size="s")}</div>'
                + f'<div style="height:1px;background-color:{t.c("hairline")}"></div><div style="display:flex;justify-content:space-between;padding-top:6px">{button(t, "Clear Site Data…", "quiet", size="s")}{button(t, "All Site Settings", "quiet", size="s")}</div>')
        pop = f'<div layer-name="Site settings (proposal)" style="width:340px;display:flex;flex-direction:column;gap:4px;padding:10px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{body}</div>'
        return std(t, hover=None, address={"domain": "meet.google.com", "zoom": "110%", "glyph_name": "lock", "state": "pressed"}, overlays=[at(8, 94, pop)])
    pair(b, spec)
    b.notes(["New · proposal. Values: Allow / Ask / Block per permission; changes apply at once and are stored per site and profile (site-settings contract).",
             "The trigger is the address control's leading edge (a lock or attention glyph); the address itself stays a display control.",
             "All Site Settings opens Settings › Privacy & site data (L-6)."])
    print(b.finish())

if "M-9" in which:
    b = Board("M-9", width=3040, desc="A site asking for a device or capability gets one prompt, anchored to the address, naming the site and the profile. Nothing is granted by default; the decision can be remembered; an active camera or microphone shows in the address while in use.", ids="bbr-permission-prompt (proposal)")
    def prompt(t, ic, title, sub, remember=True, extra=""):
        rem = checkbox(t, True, "Remember for this site") if remember else ""
        return (f'<div layer-name="Permission prompt (proposal)" style="width:360px;display:flex;flex-direction:column;gap:12px;padding:16px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
                f'<div style="display:flex;gap:12px;align-items:flex-start"><div style="width:32px;height:32px;flex-shrink:0;border-radius:9px;display:flex;align-items:center;justify-content:center;background-color:{t.c("tint-accent")}">{icon(ic, t.c("accent-ink"), 18)}</div>'
                f'<div style="display:flex;flex-direction:column;gap:2px">{text(title, "title", t.c("ink"), weight=600, clamp=False, extra="white-space:normal;")}{text(sub, "meta", t.c("ink-secondary"), clamp=False)}</div></div>{extra}{rem}'
                f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(t, "Don’t Allow", "secondary")}{button(t, "Allow", "primary")}</div></div>')
    def spec(t):
        dev = f'<div style="display:flex;flex-direction:column;gap:8px">{popup_button(t, "FaceTime HD Camera", 328)}{popup_button(t, "MacBook Pro Microphone", 328)}</div>'
        return std(t, hover=None, address={"domain": "meet.google.com", "glyph_name": "lock"}, overlays=[at(8, 94, prompt(t, "camera", "meet.google.com wants to use your camera and microphone", "Profile · Work", extra=dev))])
    pair(b, spec)
    def items(t):
        inuse = (address(t, "meet.google.com").replace('<div style="position:relative;flex:1 1 0;height:36px', '<div style="position:relative;flex:1 1 0;height:36px')
                 .replace(text("meet.google.com", "title", t.c("ink"), grow=True), text("meet.google.com", "title", t.c("ink"), grow=True) + f'<div style="height:22px;display:flex;align-items:center;gap:4px;padding:0 6px;border-radius:var(--radius-sm);background-color:{t.c("status-error-surface")}">{icon("camera", t.c("status-error"), 13)}{icon("mic", t.c("status-error"), 13)}</div>'))
        blocked = address(t, "news.example.com").replace(text("news.example.com", "title", t.c("ink"), grow=True), text("news.example.com", "title", t.c("ink"), grow=True) + f'<div style="height:22px;display:flex;align-items:center;padding:0 6px;border-radius:var(--radius-sm);background-color:{t.c("address-accessory")}">{icon("bell", t.c("ink-secondary"), 13)}</div>')
        return [frag(t, f'<div style="padding:0 8px 8px 8px">{prompt(t, "pinmap", "maps.example.com wants to know your location", "Profile · Default")}</div>', "Location", w=380),
                frag(t, f'<div style="padding:0 8px 8px 8px">{prompt(t, "clipboard", "docs.example.com wants to read your clipboard", "Profile · Default", False)}</div>', "Clipboard", "asked each time", w=380),
                frag(t, inuse, "In use", "camera and microphone live", w=300),
                frag(t, blocked, "Quietly blocked", "notifications: a chip, never a prompt storm", w=300)]
    frag_rows(b, items)
    b.notes(["New · proposal over the permissions contract: one prompt per site at a time; a second request queues behind it.",
             "The macOS privacy prompt (Native OS) follows only after the user allowed the site, the first time BBR itself needs the device.",
             "Revoking lives in site settings (M-8); the in-use chip opens it."])
    print(b.finish())

if "M-10" in which:
    b = Board("M-10", width=3040, desc="HTTP and proxy sign-in challenges get a sheet attached to the card: who asks (host and realm), the two fields, Log In. The password never appears in snapshots or logs.", ids="bbr-auth-sheet, bbr-auth-username, bbr-auth-password, bbr-auth-login, bbr-auth-cancel")
    def body(t, error=False, proxy=False):
        who = "The proxy proxy.corp.example:3128 requires sign-in" if proxy else "vpn.corp.example — realm “Finance VPN”"
        err = banner(t, "Wrong username or password. Try again.", "error") if error else ""
        return (f'{p(t, who)}{err}{text("Username", "label", t.c("ink-secondary"), clamp=False)}{field(t, "p.wozniak", lead=None, width=380)}'
                f'{text("Password", "label", t.c("ink-secondary"), clamp=False)}{field(t, "••••••••••", state="focused", lead=None, width=380)}'
                f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(t, "Cancel", "secondary")}{button(t, "Log In", "primary")}</div>')
    pair(b, lambda t: std(t, hover=None, address={"domain": "vpn.corp.example"}, card_overlays=[VEIL(t), TOP_SHEET(t, sheet(t, body(t), 420, "Authentication required"))]))
    row = b.row(gap=32, name="Variants")
    for t in (LIGHT, DARK):
        for label, kw in (("Wrong password", {"error": True}), ("Proxy", {"proxy": True})):
            col = b.column(row, name=label)
            b.caption(col, f"{label} · {t.label()}")
            card_plate(b, col, t, page_placeholder(t, lines=2), 680, 520, overlays=[VEIL(t), TOP_SHEET(t, sheet(t, body(t, **kw), 420, "Authentication required"))])
    b.notes(["Cancel shows the site's own 401 page; Escape equals Cancel. Credentials are never saved by BBR in v1 (Keychain autofill is WebKit's).",
             "Proxy challenges name the proxy, not the page; one sheet per window at a time."])
    print(b.finish())

if "M-11" in which:
    b = Board("M-11", width=3040, desc="When a site asks for a client certificate, BBR lists the matching identities from the Keychain with issuer and expiry; the user picks one or sends none. The choice is remembered per site and profile.", ids="bbr-client-cert (proposal)")
    def body(t):
        def cert(name, issuer, exp, sel=False, bad=False):
            radio = f'<div style="width:16px;height:16px;border-radius:9999px;flex-shrink:0;{("background-color:" + t.c("accent") + ";box-shadow:inset 0 0 0 4px " + t.c("overlay")) if sel else ("box-shadow:inset 0 0 0 1px " + t.c("border-strong"))}"></div>'
            bg = f"background-color:{t.c('tint-accent')};" if sel else ""
            return (f'<div style="display:flex;align-items:center;gap:12px;padding:10px 12px;border-radius:var(--radius-md);{bg}">{radio}{icon("cert", t.c("ink-secondary"), 18)}'
                    f'<div style="flex:1 1 0;display:flex;flex-direction:column;gap:2px">{text(name, "label", t.c("ink"))}{text(issuer + " · " + exp, "meta", t.c("status-error") if bad else t.c("ink-faint"), clamp=False)}</div></div>')
        return (f'{p(t, "vpn.corp.example asks for a certificate to identify you. Profile · Work")}'
                f'<div style="display:flex;flex-direction:column;gap:2px">{cert("Patrick Wozniak", "Corp Issuing CA 2", "expires 3 Mar 2027", True)}{cert("p.wozniak@corp.example", "Corp Device CA", "expires 12 Jan 2027")}{cert("Patrick W (old)", "Corp Issuing CA 1", "expired 2 Feb 2026", bad=True)}</div>'
                f'{checkbox(t, True, "Remember for this site")}<div style="display:flex;justify-content:space-between">{button(t, "Don’t Send", "quiet")}<div style="display:flex;gap:8px">{button(t, "Cancel", "secondary")}{button(t, "Continue", "primary")}</div></div>')
    pair(b, lambda t: std(t, hover=None, address={"domain": "vpn.corp.example"}, card_overlays=[VEIL(t), TOP_SHEET(t, sheet(t, body(t), 460, "Choose a certificate"))]))
    b.notes(["New · proposal for phase 8 'native client-certificate enumeration'. Expired identities stay visible but cannot be chosen.", "No private-key material ever reaches BBR's UI or logs; the list shows Keychain metadata only."])
    print(b.finish())

if "M-12" in which:
    b = Board("M-12", width=3040, desc="A link to another app's scheme (zoom:, slack:, mailto:) asks once before leaving the browser, naming the app and the site that asked. Blocked or unknown schemes say so instead of failing silently.", ids="bbr-external-open (proposal)")
    def body(t):
        return (f'<div style="display:flex;gap:12px;align-items:center"><div style="width:40px;height:40px;border-radius:10px;display:flex;align-items:center;justify-content:center;background-color:{t.c("wash-hover")}">{icon("external", t.c("ink-secondary"), 20)}</div>{p(t, "zoom.us wants to open the Zoom app.")}</div>'
                f'{checkbox(t, False, "Always allow zoom.us to open Zoom links")}<div style="display:flex;justify-content:flex-end;gap:8px">{button(t, "Cancel", "secondary")}{button(t, "Open Zoom", "primary")}</div>')
    pair(b, lambda t: std(t, hover=None, address={"domain": "zoom.us"}, card_overlays=[VEIL(t), TOP_SHEET(t, sheet(t, body(t), 400, "Open in Zoom?"))]))
    row = b.row(gap=32, name="Blocked")
    for t in (LIGHT, DARK):
        col = b.column(row, name="Blocked · " + t.label())
        b.caption(col, "No app for the link · " + t.label())
        card_plate(b, col, t, page_placeholder(t, lines=2), 900, 300, overlays=[f'<div style="position:absolute;left:20px;right:20px;top:14px">{banner(t, "No app on this Mac opens “figma:” links.", "attention", button(t, "Dismiss", "quiet", size="s"))}</div>'])
    b.notes(["Mark Change: the external-URL contract opens known apps; the confirmation sheet and 'always allow' are proposals (delta D-2).",
             "Links arriving from other apps (BBR as default browser) are PR-20."])
    print(b.finish())

if "M-13" in which:
    b = Board("M-13", width=3040, desc="Pop-ups without a user gesture are blocked with a quiet notice; the user can open this one or always allow the site. Pop-ups with explicit size (sign-in windows) open as a small BBR window with a read-only address strip.", ids="bbr-popup-blocked (proposal), bbr-popup-window")
    def spec(t):
        ban = f'<div style="position:absolute;left:20px;right:20px;top:14px">{banner(t, "A pop-up from example-shop.com was blocked.", "info", button(t, "Open", "quiet", size="s") + button(t, "Always Allow", "quiet", size="s"))}</div>'
        return std(t, hover=None, address={"domain": "example-shop.com"}, card_overlays=[ban])
    pair(b, spec)
    row = b.row(gap=40, name="Popup window")
    for t in (LIGHT, DARK):
        col = b.column(row, name="Popup · " + t.label())
        b.caption(col, "Sign-in pop-up window · " + t.label(), "explicit size → small window, not a tab")
        strip = (f'<div style="height:40px;flex-shrink:0;display:flex;align-items:center;gap:8px;padding:0 12px">{traffic_lights(t)}<div style="flex:1 1 0;height:26px;display:flex;align-items:center;justify-content:center;gap:6px;border-radius:8px;background-color:{t.c("address")}">'
                 f'{icon("lock", t.c("ink-secondary"), 12)}{text("accounts.google.com", "label", t.c("ink"), clamp=False)}</div></div>')
        b.add(f'<div layer-name="bbr-popup-window" style="width:480px;height:600px;display:flex;flex-direction:column;border-radius:12px;overflow:hidden;background-color:{t.c("sidebar")};box-shadow:0 0 0 1px {DOC.c("hairline")}, {t.elev2()}">{strip}'
              f'<div style="flex:1 1 0;display:flex;margin:0 8px 8px 8px;border-radius:10px;overflow:hidden;box-shadow:{t.card_shadow()}">{page_placeholder(t, lines=3, width=340)}</div></div>', col)
    b.notes(["Mark Change: the new-window seam opens direct activations as tabs or Peek; the notice and the popup-window chrome are proposals (delta D-2).",
             "The popup window has no rail, tabs or command bar; closing it returns focus to the opener tab."])
    print(b.finish())

if "M-14" in which:
    b = Board("M-14", width=3040, desc="Upload and Save panels are macOS's own (Native OS): BBR only attaches them to the right window and routes the result. Saves then appear in the Downloads panel like any download.", ids="file-panels contract")
    def spec(t):
        native = (f'<div layer-name="NSOpenPanel · Native OS" style="position:absolute;left:50%;top:0;margin-left:-330px;width:660px;height:420px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:10px;border-radius:0 0 12px 12px;'
                  f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("border-strong")}, {t.elev2()}">{icon("file", t.c("ink-faint"), 32)}{text("macOS open panel", "title", t.c("ink-secondary"), clamp=False)}{text("Native OS · attached to the window, owned by AppKit", "key", t.c("ink-faint"), weight=400, clamp=False)}</div>')
        return std(t, hover=None, address={"domain": "drive.example.com"}, card_overlays=[native])
    pair(b, spec)
    b.notes(["BBR never draws its own file picker; cancellation returns an empty selection to the page.", "Sandbox access is scoped to the chosen files; nothing else in the panel is BBR paint."])
    print(b.finish())
