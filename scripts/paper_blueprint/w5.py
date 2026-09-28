import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from story import *
from blocks import result_row, button, banner, field, checkbox, switch, segmented, popup_button, menu, menu_row, menu_sep, menu_header, sheet, badge, mode_pill, progress
from w3 import VEIL, TOP_SHEET, at

L = LIGHT
which = sys.argv[1:]
W = lambda spec, w=1000, h=660: win_html(L, spec, w, h)
Rf = lambda spec, h=620, ov=(): rail_frame(L, spec, h, ov)
C = lambda content=None, ov=(), w=640, h=400: card_html(L, content, ov, w, h)
def ORG(sel=True):
    return [tab_row(L, "Home / X", ("site", "ink", "X"), "selected" if sel else "rest"), tab_row(L, "WhatsApp", ("site", "pine", "W")), folder_row(L, "Home"), folder_row(L, "Review Tools")]


def REG():
    return [tab_row(L, "Knuddels-Bots.com | Underground", ("mono", "K", 1)), tab_row(L, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), trailing="attention"),
            tab_row(L, "Daybreak | OpenAI for Enterprise", ("mono", "O", 6)), tab_row(L, "YouTube", ("site", "fuchsia", "Y"), trailing="audio")]


def S(**k):
    sel = k.pop("selected_pinned", "X") is not None
    spec = std(L, **({"hover": None} | k))
    if "groups" not in k:
        spec["groups"] = [ORG(sel), "divider", "new", REG()]
    return spec
toast = lambda msg, ic="check", tone="status-ok", action="": (f'<div style="position:absolute;left:0;right:0;bottom:20px;display:flex;justify-content:center"><div style="display:flex;align-items:center;gap:10px;height:38px;padding:0 8px 0 12px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:0 0 0 1px {L.c("hairline")}, {L.elev2()}">'
                                                             f'{icon(ic, L.c(tone), 16)}{text(msg, "label", L.c("ink"), clamp=False)}{action}</div></div>')
ban = lambda msg, tone="info", act="": f'<div style="position:absolute;left:16px;right:16px;top:12px">{banner(L, msg, tone, act)}</div>'
pm = mini_palette


def regs(**over):
    r = REG()
    for i, v in over.items():
        r[int(i[1:])] = v
    return r


def with_regs(spec, rows):
    spec["groups"][3] = rows
    if any("bbr-tab-row · selected" in r or "bbr-tab-row · focused" in r for r in rows if isinstance(r, str)):
        spec["groups"][0] = ORG(False)
    return spec


if "PR-01" in which:
    from w4 import app_icon
    welcome = (f'<div style="flex:1 1 0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:14px;background-color:{L.c("canvas")}">{app_icon(56)}{text("Welcome to BBR", "display", L.c("ink"), clamp=False)}'
               f'<div style="display:flex;gap:8px">{button(L, "Get Started", "primary")}{button(L, "Import…", "secondary")}</div></div>')
    first = dict(S(favorites=None, groups=["new"], footer={"spaces": ("code",), "active": 0}, address={"domain": "Welcome", "glyph_name": "sparkle"}), card=welcome, card_bg="canvas")
    step = lambda title, body: C(f'<div style="flex:1 1 0;display:flex;flex-direction:column;justify-content:center;gap:12px;padding:40px 60px;background-color:{L.c("canvas")}">{text(title, "title-xl", L.c("ink"), weight=600, clamp=False)}{body}</div>')
    frames = [("Launch", "The first launch opens a normal window: one empty space and the welcome page in its first tab.", "first frame ≤ budget", W(first)),
              ("Import (optional)", "Import from Safari or Chrome: bookmarks become Saved Pages, history goes to Default. Runs in the background.", "skippable", step("Bring your things over", checkbox(L, True, "Safari · 1 204 bookmarks") + checkbox(L, False, "Chrome · Profile 1"))),
              ("Default browser (optional)", "Make Default hands over to the macOS confirmation (Native OS).", "Native OS", step("Open links in BBR?", button(L, "Make Default", "primary"))),
              ("Name the first space", "Name and colour; defaults are fine.", "", step("Name your first space", field(L, "Personal", state="focused", lead=None, width=360))),
              ("Get started", "Get Started opens ⌘T in New Tab mode — the one habit to learn.", "dur:fine", W(dict(first, overlays=[pm(L)]))),
              ("Browsing", "The first page opens; the welcome tab is gone and never returns.", "", W(with_regs(S(favorites=None, footer={"spaces": ("code",), "active": 0}, heading=("Personal", "code"), address={"domain": "github.com", "progress": 40}), [tab_row(L, "GitHub", ("site", "ink", "G"), "selected")])))]
    print(storyboard("PR-01", "From first launch to the first page, with every setup step skippable and nothing blocking the browser.", frames, "L-1 L-2 G-2 C-1"))

if "PR-02" in which:
    res = [result_row(L, "GitHub · Pull requests", "github.com/pulls", ("site", "ink", "G"), "History", True, "gith"), result_row(L, "github.com", "https://github.com", ("glyph", "globe"), "URL", False, "gith"),
           result_row(L, "Search “gith” with Google", None, ("glyph", "search"), "Search")]
    frames = [("⌘T", "The palette opens in New Tab mode over the card; nothing is created yet.", "dur:fine fade", W(S(overlays=[pm(L)]))),
              ("Type", "Results arrive per keystroke; the first result is selected.", "20 ms stagger", W(S(overlays=[pm(L, "gith", res)]))),
              ("↵", "Enter appends the tab at the end of the list, selects it, creates its session and navigates.", "commit ≤ 50 ms", W(with_regs(S(address={"domain": "github.com", "progress": 30}), REG() + [tab_row(L, "github.com/pulls", ("loading",), "selected", asleep=True, trailing="none")]))),
              ("Page", "The page presents natively; focus is in the page.", "", W(with_regs(S(address={"domain": "github.com"}), REG() + [tab_row(L, "Pull requests", ("site", "ink", "G"), "selected")]))),
              ("Escape instead", "Escape at step 1 or 2 creates nothing and restores the previous focus.", "", W(S()))]
    print(storyboard("PR-02", "Opening a destination from the keyboard: stage, type, commit, land.", frames, "C-1 C-2 S-6 W-6"))

if "PR-03" in which:
    ov = overlay_l(L, "https://github.com/zed-industries/zed", [lresult(L, "github.com/zed-industries/zed", "https://github.com/…/zed", ("site", "ink", "G"), "URL", True)])
    frames = [("Click the address", "The ⌘L overlay covers the address with the full URL selected.", "dur:fine", W(S(address={"domain": "github.com"}, overlays=[ov]))),
              ("Type ↵", "The current tab navigates; the D5 line runs in the address; Reload becomes Stop.", "", W(S(address={"domain": "zed.dev", "progress": 50}, titlebar={"loading": True}))),
              ("Swipe back", "Two-finger swipe over the page goes back through the same history authority as ⌘[.", "gesture", W(S(address={"domain": "github.com"}, titlebar={"forward": True}))),
              ("Forward", "Forward is enabled after going back; the titlebar reflects committed state.", "", C(f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("sidebar")}">{titlebar(L, back=True, forward=True).replace("height:52px", "height:52px;width:300px")}</div>', w=360, h=120)),
              ("Back at a child's root", "Back on a link-opened child tab at its first page closes it and selects the opener.", "atomic", W(with_regs(S(), regs(r1=tab_row(L, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "selected")))))]
    print(storyboard("PR-03", "Editing the address and moving through history by pointer, keyboard and gesture.", frames, "C-6 W-6 W-7 M-3"))

if "PR-04" in which:
    kid = tab_row(L, "Lifetimes — Rust by Example", ("mono", "R", 3), trailing="none")
    from w2 import *  # noqa
    peek_ov = [f'<div style="position:absolute;inset:0;background-color:{L.c("wash-active")}"></div>',
               f'<div style="position:absolute;left:16px;right:16px;top:12px;bottom:12px;display:flex;flex-direction:column;gap:6px"><div style="height:34px;display:flex;align-items:center;gap:8px;padding:0 6px 0 12px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:{L.elev2()}">{text("Lifetimes — Rust by Example", "label", L.c("ink"), grow=True)}{badge(L, "Peek", "accent")}{button(L, "Open as Tab", "secondary", size="s")}</div><div style="flex:1 1 0;border-radius:10px;overflow:hidden;display:flex;box-shadow:{L.elev2()}">{page_placeholder(L, lines=2, width=360)}</div></div>']
    frames = [("Click a link", "A plain click navigates the current tab.", "", W(S())),
              ("⌘-click", "Opens a background child right after its opener, in click order; selection and focus stay.", "no focus change", W(with_regs(S(), regs(r1=tab_row(L, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "selected"), r2=kid)))),
              ("New-window link", "target=_blank with a user gesture opens a Peek over the origin.", "", W(S(card_overlays=peek_ov))),
              ("Open as Tab", "Promotes the same session to a tab after the opener.", "same session", W(with_regs(S(), regs(r1=tab_row(L, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3)), r2=tab_row(L, "Lifetimes — Rust by Example", ("mono", "R", 3), "selected"))))),
              ("zoom: link", "Another app's scheme asks once before leaving (M-12).", "", W(S(card_overlays=[VEIL(L), TOP_SHEET(L, sheet(L, text("zoom.us wants to open the Zoom app.", "label", L.c("ink-secondary"), clamp=False) + f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(L, "Cancel", "secondary")}{button(L, "Open Zoom", "primary")}</div>', 360, "Open in Zoom?"))]))),
              ("Unrequested pop-up", "Blocked with a quiet notice; Open or Always Allow (M-13).", "", W(S(card_overlays=[ban("A pop-up from example-shop.com was blocked.", "info", button(L, "Open", "quiet", size="s"))])))]
    print(storyboard("PR-04", "Where a link goes: same tab, background child, Peek, another app or nowhere — and what the user sees each time.", frames, "G-8 M-12 M-13 S-5"))

if "PR-05" in which:
    fav_pop = f'<div style="position:absolute;left:236px;top:98px;width:58px;height:48px;border-radius:12px;box-shadow:0 0 0 1.5px {L.c("accent")}"></div>'
    frames = [("Hover", "Hover shows the close glyph; the row keeps its place.", "", Rf(with_regs(S(), regs(r2=tab_row(L, "Daybreak | OpenAI for Enterprise", ("mono", "O", 6), "hover"))), 640)),
              ("Pin", "⇧⌘P or the menu moves it to the organized area, same session.", "commit next frame", Rf(dict(S(), groups=[ORG(False)[:3] + [tab_row(L, "Daybreak | OpenAI for Enterprise", ("mono", "O", 6), "selected")] + ORG()[3:], "divider", "new", [r for r in REG() if "Daybreak" not in r]]), 640)),
              ("Favorite", "Dragging to Favorites adds a tile; the pop plays after the commit.", "≤ 260 ms", Rf(S(favorites=FAVS[:3] + [("mono", "O", 6)]), 640, [fav_pop])),
              ("Close", "⌘W closes; the deterministic neighbour is selected.", "", Rf(with_regs(S(selected_pinned=None), regs(r2=tab_row(L, "YouTube", ("site", "fuchsia", "Y"), "selected"), r3="")), 640)),
              ("Reopen", "⇧⌘T restores it at its old index with its history.", "", Rf(with_regs(S(selected_pinned=None), regs(r2=tab_row(L, "Daybreak | OpenAI for Enterprise", ("loading",), "selected", asleep=True, trailing="none"))), 640)),
              ("Suspend and wake", "Unused for an hour, a tab sleeps (title in ink.secondary); selecting it wakes and reloads it.", "setting", Rf(with_regs(S(), regs(r0=tab_row(L, "Knuddels-Bots.com | Underground", ("mono", "K", 1), asleep=True))), 640))]
    print(storyboard("PR-05", "The life of one tab: hover, pin, favorite, close, reopen, sleep and wake.", frames, "B-3 S-2 S-3 M-1"))

if "PR-06" in which:
    ghost_live = f'<div style="height:36px;display:flex;align-items:center;gap:10px;padding:0 10px;border-radius:var(--radius-md);border:1.5px solid {L.c("accent")};background-color:{L.c("tint-accent")}">{slot(icon("plus", L.c("accent-ink")))}{text("Pin tab here", "meta", L.c("accent-ink"), clamp=False)}</div>'
    rename = f'<div style="height:36px;display:flex;align-items:center;gap:10px;padding:0 10px;border-radius:var(--radius-md);background-color:{L.c("field")};box-shadow:0 0 0 1px {L.c("accent")}, 0 0 0 3px {L.c("tint-accent")}">{slot(icon("folder-open", L.c("ink-secondary")))}{text("Reading", "title", L.c("ink"), weight=600, clamp=False)}<div style="width:1.5px;height:16px;background-color:{L.c("accent")}"></div></div>'
    target = folder_row(L, "Reading", True).replace("background-color:transparent", f"background-color:{L.c('tint-accent')};box-shadow:inset 0 0 0 1.5px {L.c('accent')}")
    frames = [("New Folder", "Plus › New Folder creates it at the end of the organized area, open, in rename.", "", Rf(dict(S(), groups=[ORG() + [rename], "divider", "new", REG()]), 640)),
              ("Drop into it", "A pinned tab dropped on the folder joins it.", "", Rf(dict(S(), groups=[ORG()[:2] + ORG()[3:] + [target], "divider", "new", REG()]), 640)),
              ("Drag out", "A member dragged to the tab list leaves the folder and unpins.", "", Rf(dict(S(), groups=[ORG(), "divider", "new", [f'<div style="height:2px;margin:0 8px;background-color:{L.c("accent")}"></div>'] + REG()]), 640)),
              ("Pin by drag", "An unpinned tab over the ghost row pins at the end of the pinned block.", "", Rf(dict(S(), groups=[ORG() + [ghost_live], "divider", "new", REG()]), 640)),
              ("Escape", "Escape cancels: the proxy returns, every offer disappears in the same frame.", "", Rf(S(), 640))]
    print(storyboard("PR-06", "Folders are grouped pins; every drag names its destination before the drop.", frames, "S-3 S-4 S-9 M-2"))

if "PR-07" in which:
    tw = T("light", tint="pine")
    from w1_22 import config_panel
    frames = [("New Space", "Plus › New Space (⇧⌘N) creates and activates it; the panel opens over the usable space.", "commit first", W(dict(S(favorites=None, groups=["new"], heading=("Space 4", "compass"), footer={"spaces": ("code", "bag", "moon", "compass"), "active": 3}, above_footer=[config_panel(L, name="Garden", theme="slate", step=1, glyph_sel="leaf", title="New Space")]), card=empty_pane(L), card_bg="canvas"))),
              ("Draft", "Colour, paper and symbol are a draft; the rail keeps its paint until Done.", "", Rf(dict(S(favorites=None, groups=["new"], heading=("Garden", "leaf"), footer={"spaces": ("code", "bag", "moon", "leaf"), "active": 3}, above_footer=[config_panel(L, name="Garden", theme="pine", step=2, glyph_sel="leaf", title="New Space")])), 640)),
              ("Done", "The paper applies on the commit frame.", "", rail_frame(tw, dict(std(tw, hover=None, favorites=None, groups=["new"], heading=("Garden", "leaf"), footer={"spaces": ("code", "bag", "moon", "leaf"), "active": 3})), 640)),
              ("Switch", "A chip click switches; only the chrome tint cross-fades.", "dur:standard", Rf(dict(S(footer={"spaces": ("code", "bag", "moon", "leaf"), "active": 0}), bg=tw.c("sidebar"), bg_layer=(L.c("sidebar"), 0.5)), 640)),
              ("Swipe", "A horizontal swipe interpolates the tint with the finger; release commits or cancels.", "gesture", Rf(dict(S(footer={"spaces": ("code", "bag", "moon", "leaf"), "active": 0}), bg_layer=(tw.c("sidebar"), 0.4)), 640)),
              ("Delete", "Space menu › Delete asks once with the counts.", "", C(page_placeholder(L, lines=2, width=360), [VEIL(L), TOP_SHEET(L, sheet(L, text("Its 3 tabs close. Favorites and history stay.", "label", L.c("ink-secondary"), clamp=False) + f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(L, "Cancel", "secondary")}{button(L, "Delete Space", "danger")}</div>', 360, "Delete “Garden”?"))]))]
    print(storyboard("PR-07", "Create, style, switch and delete a space.", frames, "P-1 P-2 P-3 P-4"))

if "PR-08" in which:
    tw = T("light", tint="cobalt")
    frames = [("New Profile", "Settings › Profiles › New Profile… (or ⌘T › New Profile).", "", C(f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("canvas")}">{button(L, "New Profile…", "secondary")}</div>', w=420, h=260)),
              ("Name and hue", "The sheet names what a profile separates.", "", C(page_placeholder(L, lines=2, width=360), [VEIL(L), TOP_SHEET(L, sheet(L, field(L, "Client work", state="focused", lead=None, width=320) + f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(L, "Cancel", "secondary")}{button(L, "Create Profile", "primary")}</div>', 360, "New Profile"))])),
              ("First space", "Its first space is created, empty, and activated.", "", W(dict(S(favorites=None, groups=["new"], heading=("Client work", "home"), footer={"spaces": ("code", "bag", "home"), "active": 2}), card=empty_pane(L, "Client work"), card_bg="canvas"))),
              ("Move a tab", "Dragging a tab to a chip of another profile names the destination.", "", Rf(S(footer={"spaces": ("code", "bag", "home")}), 640, [f'<div style="position:absolute;left:40px;top:520px;height:28px;padding:0 10px 0 4px;display:flex;align-items:center;gap:6px;border-radius:8px;background-color:{L.c("overlay")};box-shadow:{L.elev2()}">{profile_chip(L, "C", "terracotta", 20)}{text("Move to Client work", "meta", L.c("ink"), clamp=False)}</div>'])),
              ("Recreated", "The page reloads in the destination data store; a toast says so, with Undo.", "", W(dict(S(favorites=None, heading=("Client work", "home"), footer={"spaces": ("code", "bag", "home"), "active": 2}, groups=[[tab_row(L, "Rust SIMD on the GPU", ("loading",), "selected", asleep=True, trailing="none")]]), card_overlays=[toast("Reloaded in Profile · Client work", "person", "ink-secondary", button(L, "Undo", "quiet", size="s"))])))]
    print(storyboard("PR-08", "Create a profile and move a tab into it — the only move that recreates a page.", frames, "P-5 P-6 P-7 L-4"))

if "PR-09" in which:
    zone = f'<div style="position:absolute;right:8px;top:8px;bottom:8px;width:24%;border-radius:12px;box-shadow:inset 0 0 0 2px {L.c("accent")};background-color:{L.c("tint-accent")};opacity:0.9"></div>'
    def split(focus=1, ratio=(1, 1), accent_div=False):
        pane = lambda i, fl: f'<div style="position:relative;flex:{fl} 1 0;min-width:0;display:flex;background-color:{L.c("glass")}">{page_placeholder(L, lines=3, width=260)}' + (f'<div style="position:absolute;inset:0;box-shadow:inset 0 0 0 2px {L.c("accent")}"></div>' if i == focus else "") + '</div>'
        return f'<div style="flex:1 1 0;display:flex">{pane(0, ratio[0])}<div style="width:2px;background-color:{L.c("accent") if accent_div else L.c("hairline")}"></div>{pane(1, ratio[1])}</div>'
    frames = [("Drag over the card", "The zone under the pointer previews the split.", "snap", W(S(card_overlays=[zone]))),
              ("Drop", "The split opens; the original tab keeps focus, the dropped tab takes the second pane.", "", W(S(card=split(0)))),
              ("Focus a pane", "Clicking a pane moves command focus; the outline follows.", "", W(S(card=split(1)))),
              ("Resize", "Drag the divider; both native views stay live.", "", W(S(card=split(1, (6, 4), True)))),
              ("Close a pane", "⌘W in a pane closes that tab; the split dissolves to the other.", "", W(S()))]
    print(storyboard("PR-09", "Two pages side by side (phase 6 target).", frames, "G-9 G-10"))

if "PR-10" in which:
    from w3 import *  # noqa
    frames = [("Start", "Clicking a download link starts it; the footer control shows the dot.", "", Rf(S(footer={"spaces": ("code", "bag", "moon"), "downloads_badge": True}), 640)),
              ("Progress", "The panel shows the active attempt with bytes and time left.", "", Rf(dict(S(footer={"spaces": ("code", "bag", "moon"), "downloads_badge": True}), above_footer=[f'<div style="margin:0 4px 4px 4px;padding:10px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:0 0 0 1px {L.c("hairline")}, {L.elev2()};display:flex;gap:10px;align-items:center">{spinner(L)}<div style="flex:1 1 0;display:flex;flex-direction:column;gap:4px">{text("gpui-ce-0.3.2.tar.gz", "label", L.c("ink"))}{progress(L, 62, 170)}</div>{button(L, "Cancel", "quiet", size="s")}</div>']), 640)),
              ("Done", "The check draws with one status.ok ring; the row moves to History.", "≤ 260 ms", Rf(dict(S(footer={"spaces": ("code", "bag", "moon")}), above_footer=[f'<div style="margin:0 4px 4px 4px;padding:10px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:0 0 0 1px {L.c("hairline")}, {L.elev2()};display:flex;gap:10px;align-items:center"><div style="width:22px;height:22px;border-radius:9999px;display:flex;align-items:center;justify-content:center;box-shadow:0 0 0 2px {L.c("status-ok")}">{icon("check", L.c("status-ok"), 14)}</div>{text("gpui-ce-0.3.2.tar.gz", "label", L.c("ink"), grow=True)}{button(L, "Show in Finder", "quiet", size="s")}</div>']), 640)),
              ("Fail and retry", "A failed row names the reason; Retry starts a linked attempt.", "", Rf(dict(S(footer={"spaces": ("code", "bag", "moon")}), above_footer=[f'<div style="margin:0 4px 4px 4px;padding:10px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:0 0 0 1px {L.c("hairline")}, {L.elev2()};display:flex;gap:10px;align-items:center">{icon("warn", L.c("status-error"), 16)}<div style="flex:1 1 0;display:flex;flex-direction:column">{text("dataset-full.zip", "label", L.c("ink"))}{text("Network failed", "meta", L.c("status-error"), clamp=False)}</div>{button(L, "Retry", "secondary", size="s")}</div>']), 640))]
    print(storyboard("PR-10", "A download from start to Finder, including failure.", frames, "M-4 S-7 L-7"))

if "PR-11" in which:
    from w2 import *  # noqa
    def fb(q, c, tone="ink-faint"):
        return (f'<div style="position:absolute;right:16px;top:14px;width:300px;height:34px;display:flex;align-items:center;gap:6px;padding:0 6px 0 10px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:0 0 0 1px {L.c("hairline")}, {L.elev2()}">'
                f'{icon("search", L.c("ink-faint"), 14)}<div style="flex:1 1 0;display:flex;align-items:center;gap:2px">{text(q, "label", L.c("ink"), weight=400, clamp=False)}<div style="width:1.5px;height:15px;background-color:{L.c("accent")}"></div></div>{text(c, "meta", L.c(tone), clamp=False)}{icon("close", L.c("ink-secondary"), 14)}</div>')
    frames = [("⌘F", "The bar opens top-right over the card with the last query of this tab selected.", "dur:fine", C(None, [fb("", "")])),
              ("Type", "WebKit highlights every match; the count updates.", "", C(None, [fb("lifetime", "1 of 12")])),
              ("↵ / ⇧↵", "Steps through matches; the page scrolls natively.", "", C(None, [fb("lifetime", "3 of 12")])),
              ("No match", "The count says so in the attention tone.", "", C(None, [fb("lifetimes'", "No matches", "status-attention")])),
              ("Escape", "Closes the bar; focus returns to the page.", "", C(None))]
    print(storyboard("PR-11", "Find in page.", frames, "G-6"))

if "PR-12" in which:
    frames = [("⌘D", "Saves the page's canonical URL; a toast confirms, Undo removes it.", "idempotent", W(S(card_overlays=[toast("Saved to Library", "bookmark", "accent-ink", button(L, "Undo", "quiet", size="s"))]))),
              ("Find it in ⌘T", "Saved Pages appear as results with the bookmark mark.", "", W(S(overlays=[pm(L, "lifet", [result_row(L, "Lifetimes — Rust by Example", "doc.rust-lang.org/…", ("glyph", "bookmark"), "Saved", True, "lifet")])]))),
              ("⇧⌘L", "The Library covers the card: search, sort, collections.", "", W(S(address={"domain": "Library", "glyph_name": "bookmark"}, card=f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("field")}">{text("Library (M-5)", "title", L.c("ink-secondary"), clamp=False)}</div>'))),
              ("Open", "Opening creates one fresh tab in the active space.", "", W(with_regs(S(address={"domain": "doc.rust-lang.org", "progress": 40}), REG() + [tab_row(L, "Lifetimes — Rust by Example", ("loading",), "selected", asleep=True, trailing="none")])))]
    print(storyboard("PR-12", "Save a page, find it again, open it.", frames, "M-5 C-2"))

if "PR-13" in which:
    lif = f'<div style="width:48px;height:48px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:{L.c("status-error-surface")}">{icon("lifering", L.c("status-error"), 26)}</div>'
    clock = f'<div style="width:48px;height:48px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:{L.c("wash-hover")}">{icon("clock", L.c("ink-secondary"), 24)}</div>'
    frames = [("Offline", "The load fails for the network; the offline pane explains and offers Try again.", "", C(pane(L, offline_link(L), "You're offline", "Check your connection and try again.", button(L, "Try again", "primary"), bg="field", title_role="title"))),
              ("Back online", "Try again reloads the same tab.", "", C(None)),
              ("Certificate", "A certificate error keeps the lifering and offers Go back.", "", C(pane(L, lif, "This page can't be loaded securely", "The site's certificate has expired.", button(L, "Go back", "primary"), bg="field", title_role="title"))),
              ("Crash", "The row shows the lifering; the pane offers Reload.", "", C(pane(L, heartbeat(L, w=360), "This page crashed", "The tab stays open; reload to restore it.", button(L, "Reload", "primary"), bg="field", title_role="title"))),
              ("Hang", "A hung page never blocks the chrome; Wait, Reload or Close.", "chrome stays live", C(pane(L, clock, "This page is not responding", "The page is busy; the tab stays open.", button(L, "Wait", "primary") + button(L, "Reload", "secondary"), bg="field", title_role="title")))]
    print(storyboard("PR-13", "When a page fails, the tab stays and the pane says what to do.", frames, "G-3 G-4 G-5"))

if "PR-14" in which:
    from w3 import *  # noqa
    fr = lambda html: C(page_placeholder(L, lines=2, width=360), [f'<div style="position:absolute;left:12px;top:12px">{html}</div>'], w=640, h=420)
    frames = [("Request", "A site asks for camera and microphone; one prompt names site and profile.", "", fr(f'<div style="width:320px;padding:14px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:{L.elev2()};display:flex;flex-direction:column;gap:10px">{text("meet.google.com wants to use your camera and microphone", "label", L.c("ink"), weight=600, clamp=False, extra="white-space:normal;")}<div style="display:flex;justify-content:flex-end;gap:8px">{button(L, "Don’t Allow", "secondary")}{button(L, "Allow", "primary")}</div></div>')),
              ("macOS asks once", "The first time BBR itself needs the device, macOS asks (Native OS).", "Native OS", fr(f'<div style="width:300px;padding:14px;border-radius:12px;background-color:{L.c("overlay")};box-shadow:0 0 0 1px {L.c("border-strong")}, {L.elev2()}">{text("“BBR” would like to access the camera.", "label", L.c("ink"), clamp=False, extra="white-space:normal;")}{text("Native OS", "key", L.c("ink-faint"), weight=400, clamp=False)}</div>')),
              ("In use", "The address shows the live devices.", "", Rf(S(address={"domain": "meet.google.com", "glyph_name": "lock"}), 260, [f'<div style="position:absolute;left:210px;top:61px;height:22px;display:flex;align-items:center;gap:4px;padding:0 6px;border-radius:var(--radius-sm);background-color:{L.c("status-error-surface")}">{icon("camera", L.c("status-error"), 13)}{icon("mic", L.c("status-error"), 13)}</div>'])),
              ("Revoke", "Site settings change Allow to Block; the stream stops at once.", "", fr(f'<div style="width:300px;padding:10px;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:{L.elev2()};display:flex;flex-direction:column;gap:6px"><div style="display:flex;align-items:center;gap:10px">{icon("camera", L.c("ink-secondary"), 16)}{text("Camera", "label", L.c("ink"), grow=True)}{popup_button(L, "Block", 100)}</div></div>'))]
    print(storyboard("PR-14", "A permission from request to revocation.", frames, "M-8 M-9"))

if "PR-15" in which:
    body = lambda err=False: (banner(L, "Wrong username or password.", "error") if err else "") + field(L, "p.wozniak", lead=None, width=320) + field(L, "••••••••", state="focused", lead=None, width=320) + f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(L, "Cancel", "secondary")}{button(L, "Log In", "primary")}</div>'
    frames = [("Challenge", "A 401 asks for credentials; the sheet names host and realm.", "", C(page_placeholder(L, lines=2, width=360), [VEIL(L), TOP_SHEET(L, sheet(L, body(), 360, "Authentication required"))], h=420)),
              ("Wrong", "A failed attempt keeps the sheet and says why.", "", C(page_placeholder(L, lines=2, width=360), [VEIL(L), TOP_SHEET(L, sheet(L, body(True), 360, "Authentication required"))], h=420)),
              ("Signed in", "The page loads; nothing is stored by BBR.", "", C(None)),
              ("Cancel", "Cancel shows the site's own 401 page.", "", C(f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center">{text("401 Authorization Required (site page)", "mono", L.c("ink-faint"), clamp=False)}</div>'))]
    print(storyboard("PR-15", "HTTP sign-in, wrong password, success and cancel.", frames, "M-10 M-11"))

if "PR-16" in which:
    ro = lambda z: f'<div style="position:absolute;left:50%;top:14px;margin-left:-55px;width:110px;height:30px;display:flex;align-items:center;justify-content:center;border-radius:var(--radius-lg);background-color:{L.c("overlay")};box-shadow:{L.elev2()}">{text(z, "key", L.c("ink"), clamp=False)}</div>'
    frames = [("⌘+", "Zoom steps up; the readout confirms; the address shows the chip.", "", W(S(address={"domain": "github.com", "zoom": "110%"}, card_overlays=[ro("110 %")]))),
              ("Again", "125 %; the site remembers it for this profile.", "", W(S(address={"domain": "github.com", "zoom": "125%"}, card_overlays=[ro("125 %")]))),
              ("Same site elsewhere", "Another github.com tab uses the same level.", "", W(with_regs(S(address={"domain": "github.com", "zoom": "125%"}), regs(r0=tab_row(L, "github.com/zed-industries", ("site", "ink", "G"), "selected"))))),
              ("Reset", "Pressing the chip resets to 100 %; the chip disappears.", "", W(S(address={"domain": "github.com"})))]
    print(storyboard("PR-16", "Per-site zoom.", frames, "G-7 S-1 M-8"))

if "PR-17" in which:
    from w1_21 import *  # noqa
    import w1_21
    card = lambda title, playing=True, count=None: (f'<div style="margin:0 8px 8px 8px;padding:10px;border-radius:var(--radius-lg);background-color:{L.c("field")};box-shadow:0 0 0 1px {L.c("hairline")};display:flex;align-items:center;gap:10px">{favicon(L, ("site", "fuchsia", "Y"), 22)}'
                                                    f'<div style="flex:1 1 0;display:flex;flex-direction:column">{text(title, "label", L.c("ink"))}{text("YouTube Music · Default", "meta", L.c("ink-secondary"))}</div>' + (badge(L, count) if count else "") + f'{icon_button(L, "pause" if playing else "play", L.c("ink"), size=28)}</div>')
    frames = [("Play", "A tab plays; its row shows the audio glyph.", "", Rf(S(), 640)),
              ("Leave the tab", "Selecting another tab shows the media card above the footer.", "", Rf(S(media=card("Discover Mix")), 640)),
              ("Pause", "Pause from the card; the tab is not selected.", "", Rf(S(media=card("Discover Mix", False)), 640)),
              ("Second source", "Another tab plays: the most recent one shows, with a count.", "", Rf(S(media=card("Standup call", True, "+1")), 640)),
              ("Go to source", "Clicking the title selects the owning tab, space and profile.", "", Rf(with_regs(S(), regs(r3=tab_row(L, "YouTube", ("site", "fuchsia", "Y"), "selected"))), 640))]
    print(storyboard("PR-17", "Media playing in a background tab (capability-gated).", frames, "S-8"))

if "PR-18" in which:
    lif = f'<div style="width:48px;height:48px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:{L.c("status-attention-surface")}">{icon("warn", L.c("status-attention"), 24)}</div>'
    frames = [("⌘Q", "Quits at once; there is nothing to lose.", "no dialog", W(S())),
              ("Relaunch", "Chrome first: rail, spaces and selection on the first frame.", "first usable frame", W(with_regs(S(selected_pinned=None, address={"domain": "simd.rs", "progress": 20}, card=page_placeholder(L, lines=1, width=420)), regs(r1=tab_row(L, "Rust SIMD on the GPU – a field guide", ("loading",), "selected", asleep=True, trailing="none"))))),
              ("Others asleep", "Other tabs keep their place asleep until visited.", "", Rf(S(), 640)),
              ("After a crash", "The last flushed state comes back with a notice.", "", C(None, [ban("Restored 3 spaces and 42 tabs after an unexpected quit.", "info")])),
              ("Unreadable", "If the session cannot be read, BBR starts empty and keeps the file.", "", C(pane(L, lif, "Your last session couldn't be opened", "Profiles, logins and Saved Pages are untouched.", button(L, "Try Again", "primary"), bg="field", title_role="title")))]
    print(storyboard("PR-18", "Quit, relaunch and recover.", frames, "L-10 L-11"))

if "PR-19" in which:
    focus_row = tab_row(L, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "focused")
    frames = [("Page focused", "Typing goes to the page; the chrome is at ease — no rings, no caret.", "", W(S())),
              ("⌘L", "The overlay owns the keyboard; Escape returns to the page.", "", W(S(overlays=[overlay_l(L, results=[lresult(L, "x.com/home", "https://x.com/home", ("site", "ink", "X"), "URL", True)])]))),
              ("Into the rail", "⌥⌘S (or Tab from the address) moves focus to the rail; the ring shows.", "", Rf(with_regs(S(), regs(r1=focus_row)), 640)),
              ("Move and open", "↑↓ move the ring; ↵ selects and focus returns to the page.", "", Rf(with_regs(S(), regs(r1=tab_row(L, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "selected"))), 640))]
    print(storyboard("PR-19", "Keyboard focus moves between the page and the chrome predictably.", frames, "B-2 L-9", notes=["⌥⌘S for rail focus is a proposal (delta D-2); the shipped chrome focus order is address → rail → footer."]))

if "PR-20" in which:
    frames = [("Click in another app", "A link in Mail (Native OS) opens in BBR, the default browser.", "Native OS", C(f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("canvas")}">{text("Mail · Native OS", "title", L.c("ink-faint"), clamp=False)}</div>', w=420, h=300)),
              ("New tab in the last space", "BBR activates and opens it at the end of the last active space.", "", W(with_regs(S(address={"domain": "example.com", "progress": 40}), REG() + [tab_row(L, "example.com/invoice", ("loading",), "selected", asleep=True, trailing="none")]))),
              ("Ask (setting)", "With 'Ask', a small palette chooses the space.", "", W(S(overlays=[pm(L, "", [result_row(L, "Open in Default", None, ("glyph", "grid"), "Space", True), result_row(L, "Open in Work · Profile Work", None, ("glyph", "grid"), "Space")], mode="OPEN")]))),
              ("Not running", "If BBR was closed, it launches, restores chrome first, then opens the link.", "", W(S()))]
    print(storyboard("PR-20", "Links from other apps.", frames, "L-2 L-3 M-12"))

if "PR-21" in which:
    frames = [("⌘N", "A new window opens on the current profile with New Tab ready.", "", W(dict(S(favorites=None, groups=["new"]), card=empty_pane(L), card_bg="canvas"))),
              ("Move a tab", "Drag a row to the other window's rail (proposal).", "", Rf(S(), 640)),
              ("⇧⌘W", "The window closes; a toast offers reopen.", "", W(S(card_overlays=[toast("Window with 12 tabs closed", "window", "ink-secondary", button(L, "Reopen", "quiet", size="s"))]))),
              ("⇧⌘T", "The window returns with its spaces and tabs.", "", W(S()))]
    print(storyboard("PR-21", "Windows: new, move, close, reopen.", frames, "W-9 L-14"))

if "PR-22" in which:
    frames = [("⌘Y", "History opens over the card, grouped by day.", "", W(S(address={"domain": "History", "glyph_name": "history"}, card=f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("field")}">{text("History (M-6)", "title", L.c("ink-secondary"), clamp=False)}</div>'))),
              ("Search", "Typing filters visits of the chosen profile.", "", W(S(address={"domain": "History", "glyph_name": "history"}, card=f'<div style="flex:1 1 0;display:flex;flex-direction:column;gap:6px;padding:24px;background-color:{L.c("field")}">{field(L, "rust", state="focused", width=300)}{result_row(L, "Rust SIMD on the GPU", "simd.rs/guide", ("mono", "V", 3), "18:10", True, "rust")}</div>'))),
              ("Clear data", "Clear Browsing Data asks for range and kinds, per profile.", "", C(page_placeholder(L, lines=2, width=360), [VEIL(L), TOP_SHEET(L, sheet(L, checkbox(L, True, "Browsing history") + checkbox(L, True, "Cookies and site data") + f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(L, "Cancel", "secondary")}{button(L, "Clear Data", "danger")}</div>', 340, "Clear Browsing Data"))], h=420)),
              ("Cleared", "A toast confirms; open tabs stay.", "", C(None, [toast("Cleared 24 hours of data from Default")]))]
    print(storyboard("PR-22", "History and clearing data (proposal).", frames, "M-6 M-7 L-6"))

if "PR-23" in which:
    frames = [("⌘,", "Settings open on General.", "", C(f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("canvas")}">{text("Settings › General (L-3)", "title", L.c("ink-secondary"), clamp=False)}</div>', w=520, h=340)),
              ("Density", "Appearance › Large re-lays the rail at XL; rows never change height mid-session otherwise.", "", C(f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{L.c("canvas")}">{segmented(L, ["Small", "Normal", "Large"], 2, 300)}</div>', w=520, h=340)),
              ("Search engine", "Search › DuckDuckGo; the next ⌘T search row uses it.", "", W(S(overlays=[pm(L, "weather", [result_row(L, "Search “weather” with DuckDuckGo", None, ("glyph", "search"), "Search", True)])]))),
              ("Close", "Changes apply as you make them; closing needs no Save.", "", W(S()))]
    print(storyboard("PR-23", "Changing settings (proposal).", frames, "L-3 L-5 L-8"))
