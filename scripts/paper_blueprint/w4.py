import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from cmdsurf import *
from blocks import menu, menu_row, menu_sep, menu_header, sheet, button, field, checkbox, switch, segmented, popup_button, badge, banner, progress, keycap
from w1_21 import frag, frag_rows
from w3 import VEIL, TOP_SHEET, at, p

which = sys.argv[1:]
TABS = [("General", "gear"), ("Profiles", "person"), ("Search", "search"), ("Privacy", "shield"), ("Downloads", "download"), ("Appearance", "eye"), ("Shortcuts", "keyboard")]


def app_icon(size=64):
    r = size * 0.225
    return (f'<div style="width:{size}px;height:{size}px;flex-shrink:0;border-radius:{r}px;display:flex;align-items:center;justify-content:center;background-color:var(--color-dark-canvas);box-shadow:0 1px 3px var(--color-icon-shadow)">'
            f'<div style="width:{size * 0.5}px;height:{size * 0.5}px;border-radius:{size * 0.12}px;box-shadow:inset 0 0 0 {max(2, size * 0.06):.0f}px var(--color-dark-accent)"></div></div>')


def settings_window(b, target, t, active, groups, w=780, h=620):
    win = b.add(f'<div layer-name="Settings window · {active} · {t.label()}" style="width:{w}px;height:{h}px;flex-shrink:0;display:flex;flex-direction:column;border-radius:12px;overflow:hidden;background-color:{t.c("canvas")};'
                f'box-shadow:0 0 0 1px {DOC.c("hairline")}, 0 24px 48px -12px var(--color-elev-deep)"></div>', target)
    tabs = "".join(
        f'<div style="width:78px;height:52px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:4px;border-radius:8px;'
        + (f'background-color:{t.c("wash-active")}' if n == active else "") + f'">{icon(ic, t.c("accent") if n == active else t.c("ink-secondary"), 20)}{text(n, "meta", t.c("ink") if n == active else t.c("ink-secondary"), clamp=False)}</div>'
        for n, ic in TABS)
    b.add(f'<div layer-name="Toolbar" style="flex-shrink:0;display:flex;flex-direction:column;align-items:center;gap:6px;padding:10px 12px 10px 12px;background-color:{t.c("sidebar")};border-bottom:1px solid {t.c("hairline")}">'
          f'<div style="align-self:stretch;display:flex;align-items:center;gap:12px">{traffic_lights(t)}<div style="flex:1 1 0;text-align:center">{text(active, "label", t.c("ink"), weight=600, clamp=False)}</div><div style="width:68px"></div></div>'
          f'<div style="display:flex;gap:2px">{tabs}</div></div>', win)
    body = b.add('<div layer-name="Content" style="flex:1 1 0;display:flex;flex-direction:column;gap:18px;padding:24px 40px;overflow:hidden"></div>', win)
    for title, rows in groups:
        rows_html = "".join(
            f'<div style="min-height:44px;display:flex;align-items:center;gap:16px;padding:8px 14px;{"border-top:1px solid " + t.c("hairline") if i else ""}">'
            f'<div style="flex:1 1 0;display:flex;flex-direction:column;gap:2px">{text(l, "label", t.c("ink"), weight=400, clamp=False)}{text(d, "meta", t.c("ink-faint"), clamp=False) if d else ""}</div>{ctrl}</div>'
            for i, (l, d, ctrl) in enumerate(rows))
        b.add(f'<div style="display:flex;flex-direction:column;gap:8px">{text(title, "badge", t.c("ink-faint"), clamp=False) if title else ""}'
              f'<div style="display:flex;flex-direction:column;border-radius:var(--radius-lg);background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("hairline")}">{rows_html}</div></div>', body)
    return win


def settings_board(code, desc, groups_fn, active, notes, ids="bbr-settings (proposal)", h=620):
    b = Board(code, width=1800, desc=desc, ids=ids)
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name=t.label())
        b.caption(col, f"Settings › {active} · {t.label()}")
        settings_window(b, col, t, active, groups_fn(t), h=h)
    b.notes(notes)
    print(b.finish())


sw = lambda t, on: switch(t, on)

if "L-1" in which:
    b = Board("L-1", width=3040, desc="The first launch opens a normal window with one empty space and a short welcome in the card: what BBR is, three things to know, one button. No account, no tour overlay, nothing blocks the browser.", ids="bbr-welcome (proposal)")
    def spec(t):
        k = lambda icn, h, d: (f'<div style="display:flex;gap:14px;align-items:flex-start;width:420px"><div style="width:36px;height:36px;flex-shrink:0;border-radius:10px;display:flex;align-items:center;justify-content:center;background-color:{t.c("tint-accent")}">{icon(icn, t.c("accent-ink"), 18)}</div>'
                               f'<div style="display:flex;flex-direction:column;gap:2px">{text(h, "title", t.c("ink"), weight=600, clamp=False)}{text(d, "meta", t.c("ink-secondary"), clamp=False, extra="white-space:normal;")}</div></div>')
        body = (f'<div layer-name="Welcome (proposal)" style="flex:1 1 0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:20px;background-color:{t.c("canvas")}">{app_icon(72)}'
                f'<div style="font-family:var(--font-ui);font-size:32px;line-height:38px;font-weight:600;letter-spacing:-0.02em;color:{t.c("ink")}">Welcome to BBR</div>'
                f'{text("A fast browser for many contexts at once.", "title-xl", t.c("ink-secondary"), weight=400, clamp=False)}'
                f'<div style="display:flex;flex-direction:column;gap:14px;padding:12px 0">{k("cmd", "⌘T opens anything", "Tabs, history, saved pages, spaces and actions — one field.")}'
                f'{k("grid", "Spaces keep contexts apart", "Work, research, home: each with its own tabs and colour.")}{k("person", "Profiles keep logins apart", "Separate cookies and site data when you need them.")}</div>'
                f'<div style="display:flex;gap:10px">{button(t, "Get Started", "primary", size="l")}{button(t, "Import from Safari or Chrome…", "secondary", size="l")}</div></div>')
        return std(t, favorites=None, heading=("Default", "code"), groups=["new"], address={"domain": "Welcome", "glyph_name": "sparkle"}, footer={"spaces": ("code",), "active": 0}, card=body, card_bg="canvas")
    pair(b, spec)
    b.notes(["New · proposal (phase 9 'onboarding only after the real workflow exists'). Get Started opens the ⌘T palette in New Tab mode.",
             "The welcome is an internal page of the first tab; closing it leaves the empty space (G-2). It never returns on later launches."])
    print(b.finish())

if "L-2" in which:
    b = Board("L-2", width=3040, desc="Three optional steps, each skippable: bring bookmarks and history over, make BBR the default browser, name the first space. Nothing is imported silently; each step says where the data lands.", ids="bbr-onboarding (proposal)")
    def step(t, n, title, sub, content, primary):
        dots = "".join(f'<div style="width:{18 if i == n else 6}px;height:6px;border-radius:9999px;background-color:{t.c("accent") if i == n else t.c("border-strong")}"></div>' for i in range(3))
        return (f'<div style="flex:1 1 0;display:flex;align-items:center;justify-content:center;background-color:{t.c("canvas")}"><div style="width:460px;display:flex;flex-direction:column;gap:16px">'
                f'<div style="display:flex;gap:6px">{dots}</div>{text(title, "display", t.c("ink"), clamp=False)}{text(sub, "label", t.c("ink-secondary"), weight=400, clamp=False, extra="white-space:normal;")}{content}'
                f'<div style="display:flex;justify-content:space-between;padding-top:8px">{button(t, "Skip", "quiet")}{button(t, primary, "primary")}</div></div></div>')
    def src(t, name, sel, detail):
        return (f'<div style="display:flex;align-items:center;gap:12px;padding:12px;border-radius:var(--radius-md);background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("accent") if sel else t.c("hairline")}">'
                f'<div style="width:16px;height:16px;border-radius:9999px;{("background-color:" + t.c("accent") + ";box-shadow:inset 0 0 0 4px " + t.c("field")) if sel else ("box-shadow:inset 0 0 0 1px " + t.c("border-strong"))}"></div>'
                f'<div style="display:flex;flex-direction:column;gap:2px">{text(name, "label", t.c("ink"), clamp=False)}{text(detail, "meta", t.c("ink-faint"), clamp=False)}</div></div>')
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        steps = [("Import", step(t, 0, "Bring your things over", "Bookmarks become Saved Pages; history goes to the Default profile. Passwords stay in iCloud Keychain.",
                                  f'<div style="display:flex;flex-direction:column;gap:8px">{src(t, "Safari", True, "1 204 bookmarks · 18 months of history")}{src(t, "Google Chrome · Profile 1", False, "312 bookmarks · 3 months of history")}</div>', "Import")),
                 ("Default browser", step(t, 1, "Open links in BBR?", "Links from Mail, Slack and other apps will open in the space you used last. You can change this in Settings › General.",
                                          f'<div style="display:flex;align-items:center;gap:12px;padding:12px;border-radius:var(--radius-md);background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("hairline")}">{app_icon(32)}{text("BBR is not your default browser", "label", t.c("ink"), grow=True)}</div>'
                                          f'{text("macOS asks to confirm (Native OS).", "meta", t.c("ink-faint"), clamp=False)}', "Make Default")),
                 ("First space", step(t, 2, "Name your first space", "Spaces keep contexts apart. Add more any time from the footer.",
                                      field(t, "Personal", state="focused", lead=None, width=460) + f'<div style="display:flex;gap:6px">' + "".join(f'<div style="width:22px;height:22px;border-radius:9999px;background-color:{t.c(c)};{("box-shadow:0 0 0 2px " + t.c("canvas") + ", 0 0 0 4px " + t.c("accent")) if i == 6 else ""}"></div>' for i, c in enumerate(("profile-cobalt", "profile-violet", "profile-fuchsia", "profile-amber", "profile-terracotta", "profile-slate", "space-pine", "space-plum", "space-ink"))) + '</div>', "Done"))]
        for label, content in steps:
            col = b.column(row, name=label)
            b.caption(col, f"{label} · {t.label()}")
            card_plate(b, col, t, content, 940, 560)
    b.notes(["New · proposal (phase 7/9). Import runs in the background after Import; progress appears in the Downloads-style panel; nothing blocks browsing.",
             "Make Default triggers the macOS confirmation; the default-browser contract owns the registration."])
    print(b.finish())

if "L-3" in which:
    settings_board("L-3", "The Settings window (⌘,) is a normal macOS preferences window in BBR's materials: a toolbar of seven panes, grouped rows with the control on the right. General holds startup, links and tab lifecycle.",
                   lambda t: [("Browser", [("Default browser", "BBR is your default browser", badge(t, "Default", "accent")), ("On startup", "", popup_button(t, "Restore last session", 200)),
                                           ("Open links from other apps in", "", popup_button(t, "The last active space", 200))]),
                              ("Tabs", [("Open links that ask for a new window as Peek", "Preview over the current tab; promote with Open as Tab", sw(t, True)),
                                        ("Suspend background tabs after", "Frees memory; suspended tabs keep their place", popup_button(t, "1 hour", 140)),
                                        ("Close tabs not used for", "Pinned tabs and favorites are never closed", popup_button(t, "Never", 140))])],
                   "General", ["New · proposal: no settings window exists yet; the general-search-settings contract defines the durable values.",
                               "Changes apply immediately (macOS convention); there is no Save button."])

if "L-4" in which:
    settings_board("L-4", "Profiles & spaces lists each profile with its data boundary and its spaces. New Profile opens the sheet (P-6); deleting a profile removes its data store after naming what goes.",
                   lambda t: [("Profiles", [(f'Default', "4 spaces · 212 MB of site data", f'<div style="display:flex;gap:8px;align-items:center">{profile_chip(t, "D", "cobalt")}{button(t, "Edit…", "secondary", size="s")}</div>'),
                                            ("Work", "2 spaces · 88 MB of site data", f'<div style="display:flex;gap:8px;align-items:center">{profile_chip(t, "W", "violet")}{button(t, "Edit…", "secondary", size="s")}</div>'),
                                            ("Private", "1 space · clears on quit", f'<div style="display:flex;gap:8px;align-items:center">{profile_chip(t, "P", "slate")}{button(t, "Edit…", "secondary", size="s")}</div>')]),
                              ("", [("Add a profile", "Separate cookies, logins and site data", button(t, "New Profile…", "secondary", size="s")), ("Delete “Work”…", "Removes its cookies, logins, history and site data", button(t, "Delete…", "danger", size="s"))]),
                              ("Spaces in Work", [("Work", "Cobalt · standard", glyph("bag", t.c("ink"), 18)), ("Customers", "Terracotta · subtle", glyph("home", t.c("ink"), 18))])],
                   "Profiles", ["New · proposal (§12). The last profile cannot be deleted; deleting asks with the counts, like P-4.", "Moving a space to another profile is not offered: spaces belong to one profile for life."])

if "L-5" in which:
    settings_board("L-5", "Search sets the default engine, whether the palette asks the engine for suggestions, and keyword shortcuts (type the keyword and a space in ⌘T).",
                   lambda t: [("Default", [("Search engine", "", popup_button(t, "Google", 180)), ("Search suggestions", "Sends typed text to the engine while you type", sw(t, False)),
                                           ("Show tabs from all spaces in ⌘T", "", sw(t, True))]),
                              ("Keywords", [("Wikipedia", "w · en.wikipedia.org/w/index.php?search=%s", keycap(t, "w")), ("GitHub", "gh · github.com/search?q=%s", keycap(t, "gh")),
                                            ("Rust docs", "rs · doc.rust-lang.org/std/?search=%s", keycap(t, "rs")), ("", "", button(t, "Add Keyword…", "secondary", size="s"))])],
                   "Search", ["New · proposal over the general search-settings contract; suggestions are off by default (privacy).", "Keywords appear in ⌘T as a Search row with the engine name."])

if "L-6" in which:
    settings_board("L-6", "Privacy & site data is per profile: cookie policy, clearing on quit, and every site with its own settings. Clear Browsing Data opens the sheet (M-7).",
                   lambda t: [("Profile", [("Show settings for", "", popup_button(t, "Default", 160))]),
                              ("Privacy", [("Block third-party cookies", "", sw(t, True)), ("Clear cookies and site data when BBR quits", "", sw(t, False)), ("Browsing data", "History, cookies, caches", button(t, "Clear Browsing Data…", "secondary", size="s"))]),
                              ("Sites with their own settings", [("meet.google.com", "Camera · Microphone allowed", button(t, "Edit", "quiet", size="s")), ("news.example.com", "Notifications blocked", button(t, "Edit", "quiet", size="s")),
                                                                 ("github.com", "Zoom 110 %", button(t, "Edit", "quiet", size="s"))])],
                   "Privacy", ["New · proposal (site-settings contract). Edit opens the site settings popover content (M-8) in a sheet.", "Values never cross profiles."])

if "L-7" in which:
    settings_board("L-7", "Downloads sets where files go and how long the list remembers them.",
                   lambda t: [("Location", [("Save files to", "", popup_button(t, "Downloads", 180)), ("Ask where to save each file", "", sw(t, False))]),
                              ("List", [("Keep download history for", "Terminal records survive restarts", popup_button(t, "30 days", 140)), ("Download history", "", button(t, "Clear…", "secondary", size="s"))])],
                   "Downloads", ["New · proposal; the downloads panel (M-4) keeps its own Clear.", "Sandbox access to a custom folder is granted through the native panel (Native OS)."])

if "L-8" in which:
    def g(t):
        prev = "".join(f'<div style="display:flex;flex-direction:column;gap:4px;align-items:center"><div style="width:84px;height:{h}px;border-radius:8px;display:flex;align-items:center;gap:6px;padding:0 8px;background-color:{t.c("tint-selected")};box-shadow:{t.lift()}">{favicon(t, ("mono", "R", 3), s)}<div style="flex:1 1 0;height:6px;border-radius:3px;background-color:{t.c("border-strong")}"></div></div>{text(l, "meta", t.c("ink-secondary"), clamp=False)}</div>'
                       for l, h, s in (("Small", 28, 14), ("Normal", 34, 16), ("Large", 42, 20)))
        return [("Theme", [("Appearance", "Follows macOS unless you choose", segmented(t, ["System", "Light", "Dark"], 0, 240)), ("Increase contrast", "Follows the macOS accessibility setting", popup_button(t, "Follow system", 160)),
                           ("Reduce motion", "Follows the macOS accessibility setting", popup_button(t, "Follow system", 160))]),
                ("Density", [("Sidebar and text size", "Rows, type and the rail width step together", f'<div style="display:flex;gap:10px">{prev}</div>')]),
                ("New spaces", [("Default paper", "", segmented(t, ["Subtle", "Standard", "Rich"], 1, 240))])]
    settings_board("L-8", "Appearance chooses light, dark or system, follows the macOS contrast and motion settings by default, and sets the density — the one size control for the whole chrome.", g, "Appearance",
                   ["System preferences stay authoritative unless overridden (§3.4, §6 v2 S1); the overrides are a proposal.", "Density maps to S / N / XL (F-3); a change re-lays the rail without changing row heights mid-session."])

if "L-9" in which:
    def g(t):
        keys = lambda *ks: '<div style="display:flex;gap:4px">' + "".join(keycap(t, k) for k in ks) + '</div>'
        return [("Tabs", [("New tab", "", keys("⌘", "T")), ("Close tab", "", keys("⌘", "W")), ("Reopen closed tab", "", keys("⇧", "⌘", "T")), ("Pin / unpin", "", keys("⇧", "⌘", "P"))]),
                ("Spaces", [("Space 1 … 9", "", keys("⌃", "1…9")), ("New space", "", keys("⇧", "⌘", "N")), ("Move tab to space", "", keys("⇧", "⌘", "M"))]),
                ("Navigation", [("Edit address", "", keys("⌘", "L")), ("Find in page", "", keys("⌘", "F")), ("Toggle sidebar", "", keys("⌘", "B"))])]
    settings_board("L-9", "Shortcuts lists every command with its keys, grouped like the menus; the same list powers the > actions in ⌘T. Keys follow macOS conventions first.", g, "Shortcuts",
                   ["New · proposal: read-only list in v1; editing is an open decision (D-1).", "Every action here is reachable by pointer too (phase 9 parity)."], h=760)

if "L-10" in which:
    b = Board("L-10", width=3040, desc="Relaunch restores chrome first: the rail, spaces and selection appear on the first frame; only the selected tab loads, every other tab waits asleep until visited. A quiet notice says what came back.", ids="bbr-restore-notice (proposal)")
    def spec(t):
        ban = f'<div style="position:absolute;left:20px;right:20px;top:14px">{banner(t, "Restored 3 spaces and 42 tabs from your last session.", "info", button(t, "Dismiss", "quiet", size="s"))}</div>'
        regs = regular(t, hover=None)
        regs = [r.replace(t.c("ink") + ";", t.c("ink-secondary") + ";") for r in regs]
        s = std(t, hover=None, selected_pinned=None, selected="Rust", loading="Rust", address={"domain": "simd.rs", "progress": 25}, card=page_placeholder(t, lines=1), card_overlays=[ban])
        return s
    pair(b, spec)
    b.notes(["Startup presents the complete rail and the selected tab's native view in the same first usable frame (phase 4 budget).",
             "Asleep tabs show ink.secondary titles and their cached favicons; the notice is a proposal and disappears on the first navigation."])
    print(b.finish())

if "L-11" in which:
    b = Board("L-11", width=3040, desc="If part of the saved session cannot be read, BBR starts anyway with what it could restore, keeps the unreadable file aside and says so once. Profile data (cookies, logins) is never deleted to recover.", ids="bbr-recovery (proposal)")
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        col = b.column(row, name="Partial")
        b.caption(col, "Partial restore · " + t.label())
        card_plate(b, col, t, page_placeholder(t, lines=3), 900, 480, overlays=[f'<div style="position:absolute;left:20px;right:20px;top:14px">{banner(t, "2 tabs from the Work space could not be restored.", "attention", button(t, "Details", "quiet", size="s"))}</div>'])
        col = b.column(row, name="Failed")
        b.caption(col, "Session unreadable · " + t.label())
        lif = f'<div style="width:56px;height:56px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:{t.c("status-attention-surface")}">{icon("warn", t.c("status-attention"), 28)}</div>'
        card_plate(b, col, t, pane(t, lif, "Your last session couldn't be opened", "BBR started with an empty space. The old session file was kept in case it can be recovered; your profiles, logins and Saved Pages are untouched.",
                                   button(t, "Try Again", "primary") + button(t, "Show File in Finder", "secondary"), bg="field", title_role="title-xl"), 900, 480)
        col = b.column(row, name="Migration")
        b.caption(col, "Newer data version · " + t.label())
        card_plate(b, col, t, pane(t, lif, "This session was saved by a newer BBR", "Update BBR to open it. Until then BBR runs with a fresh session and leaves the newer data unchanged.",
                                   button(t, "Check for Updates", "primary") + button(t, "Continue", "secondary"), bg="field", title_role="title-xl"), 900, 480)
    b.notes(["New · proposal for phase 7 'first-run, empty, migration-error and recovery UI'; the repository already returns typed warnings without mutating state.",
             "The closed schema-version policy discards older stores; the copy never promises recovery it cannot do."])
    print(b.finish())

if "L-12" in which:
    b = Board("L-12", width=1800, desc="About BBR shows the icon, version and update state. Updates download in the background and apply on the next relaunch; nothing interrupts browsing.", ids="bbr-about (proposal)")
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        for state in ("up", "ready"):
            col = b.column(row, name=state)
            b.caption(col, ("Up to date" if state == "up" else "Update ready") + " · " + t.label())
            status = (f'<div style="display:flex;align-items:center;gap:6px">{icon("check", t.c("status-ok"), 14)}{text("BBR is up to date", "meta", t.c("ink-secondary"), clamp=False)}</div>' if state == "up"
                      else f'<div style="display:flex;flex-direction:column;gap:8px;align-items:center">{text("Version 0.10.0 is ready", "label", t.c("ink"), clamp=False)}{button(t, "Relaunch to Update", "primary")}{text("Release notes", "meta", t.c("accent-ink"), clamp=False)}</div>')
            b.add(f'<div style="width:340px;height:380px;display:flex;flex-direction:column;border-radius:12px;overflow:hidden;background-color:{t.c("canvas")};box-shadow:0 0 0 1px {DOC.c("hairline")}, {t.elev2()}">'
                  f'<div style="height:40px;display:flex;align-items:center;padding:0 12px">{traffic_lights(t)}</div><div style="flex:1 1 0;display:flex;flex-direction:column;align-items:center;gap:10px;padding:10px 24px">{app_icon(88)}'
                  f'{text("BBR", "display", t.c("ink"), clamp=False)}{text("Version 0.9.3 (2026.09.28)", "meta", t.c("ink-faint"), clamp=False)}<div style="height:6px"></div>{status}</div></div>', col)
    b.notes(["New · proposal; signing, notarisation and the update channel are owned by macos-app-packaging and phase 9.", "The app icon is the shipped Phosphor-b mark (v2-brand-icon), drawn here as a token stand-in."])
    print(b.finish())

if "L-13" in which:
    b = Board("L-13", width=3040, desc="The macOS menu bar carries every command with its shortcut (Native OS); BBR supplies the items, macOS draws them. The Dock menu offers New Window and the spaces.", ids="NSMenu (Native OS)")
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        col = b.column(row, name="Menu bar")
        b.caption(col, "Menu bar · Tab menu open · " + t.label(), "Native OS — context only")
        bar = "".join(f'<div style="padding:0 10px">{text(m, "label", t.c("ink"), weight=600 if m == "BBR" else 400, clamp=False)}</div>' for m in ("BBR", "File", "Edit", "View", "History", "Spaces", "Tab", "Window", "Help"))
        m = menu(t, [menu_row(t, "New Tab", shortcut="⌘T"), menu_row(t, "Close Tab", shortcut="⌘W"), menu_row(t, "Reopen Closed Tab", shortcut="⇧⌘T"), menu_sep(t), menu_row(t, "Pin Tab", "cursor", "⇧⌘P"),
                     menu_row(t, "Move Tab to Space…", shortcut="⇧⌘M"), menu_row(t, "Duplicate Tab"), menu_sep(t), menu_row(t, "Select Next Tab", shortcut="⌃⇥"), menu_row(t, "Select Previous Tab", shortcut="⌃⇧⇥")], 260, focused=False)
        b.add(f'<div layer-name="Menu bar · Native OS" style="position:relative;width:1200px;height:420px;border-radius:12px;overflow:hidden;background-color:{t.c("canvas")};box-shadow:0 0 0 1px {DOC.c("hairline")}">'
              f'<div style="height:28px;display:flex;align-items:center;padding:0 12px;background-color:{t.c("overlay")};border-bottom:1px solid {t.c("hairline")}">{icon("cmd", t.c("ink"), 14)}{bar}</div>'
              f'<div style="position:absolute;left:470px;top:30px">{m}</div></div>', col)
        col = b.column(row, name="Dock")
        b.caption(col, "Dock menu · " + t.label(), "Native OS")
        dm = menu(t, [menu_header(t, "Spaces"), menu_row(t, "Default", icon_name="grid"), menu_row(t, "Work", icon_name="grid"), menu_row(t, "Research", icon_name="bookmark"), menu_sep(t), menu_row(t, "New Window", "cursor"), menu_row(t, "New Tab")], 220, focused=False)
        b.add(f'<div style="display:flex;flex-direction:column;align-items:center;gap:10px;width:260px">{dm}{app_icon(64)}</div>', col)
    b.notes(["Native OS: BBR defines the menu structure and titles; appearance is macOS's. Menu titles match the ⌘T action names (C-4) and Settings › Shortcuts (L-9).",
             "Change: the shipped menu bar covers the basic items; the Spaces and Tab menus as drawn are proposals (delta D-2)."])
    print(b.finish())

if "L-14" in which:
    b = Board("L-14", width=3040, desc="Windows open with ⌘N, close with ⇧⌘W and come back with ⇧⌘T like tabs. Quitting never asks: everything restores (L-10). Holding ⌘Q is not needed because nothing is lost.", ids="bbr-window-* (proposal)")
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        col = b.column(row, name="New window")
        b.caption(col, "⌘N · new window on the current profile · " + t.label())
        window(b, col, t, std(t, favorites=FAVS, heading=("Default", "code"), groups=["new"], footer={"spaces": ("code", "bag", "moon"), "active": 0}, card=empty_pane(t), card_bg="canvas"), 1100, 720)
        col = b.column(row, name="Closed window")
        b.caption(col, "After ⇧⌘W · " + t.label(), "toast offers reopen")
        toast = (f'<div style="position:absolute;left:0;right:0;bottom:24px;display:flex;justify-content:center"><div style="display:flex;align-items:center;gap:10px;height:40px;padding:0 8px 0 14px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
                 f'{icon("window", t.c("ink-secondary"), 16)}{text("Window with 12 tabs closed", "label", t.c("ink"), clamp=False)}{button(t, "Reopen  ⇧⌘T", "quiet", size="s")}</div></div>')
        window(b, col, t, std(t, hover=None, card_overlays=[toast]), 1100, 720)
    b.notes(["Change: the process-lifecycle packet owns quit/restore semantics; the reopen-window toast is a proposal (delta D-2).",
             "A new window takes the active space's profile and opens an empty space view with New Tab ready."])
    print(b.finish())

# ---------------------------------------------------------------- split view (page 24)
if "G-9" in which:
    b = Board("G-9", width=3040, desc="Two tabs side by side in one card, split by a 2 pt hairline-etched divider. The focused pane receives commands and the address; it shows a quiet inset outline only when command focus matters. The rail marks the pair.", ids="bbr-pane-0, bbr-pane-1, bbr-split-divider")
    def spec(t, focus=1):
        pane_html = lambda i: (f'<div layer-name="bbr-pane-{i}" style="position:relative;flex:1 1 0;min-width:0;display:flex;background-color:{t.c("glass")}">{page_placeholder(t, lines=4, width=440)}'
                               + (f'<div style="position:absolute;inset:0;box-shadow:inset 0 0 0 2px {t.c("accent")}"></div>' if i == focus else "") + '</div>')
        card = (f'<div style="flex:1 1 0;display:flex">{pane_html(0)}<div layer-name="bbr-split-divider" style="width:2px;flex-shrink:0;background-color:{t.c("hairline")}"></div>{pane_html(1)}</div>')
        regs = regular(t, hover=None)
        pair_mark = lambda r: r.replace(slot("", 18), slot(icon("split", t.c("ink-secondary"), 14), 18))
        regs[0] = pair_mark(tab_row(t, "Knuddels-Bots.com | Underground", ("mono", "K", 1)))
        regs[1] = pair_mark(tab_row(t, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "selected"))
        s = std(t, hover=None, selected_pinned=None, address={"domain": "simd.rs"}, card=card)
        s["groups"][3] = regs
        return s
    pair(b, spec)
    b.notes(["Change: the Split composition is scheduling-deferred; this board is the target for phase 6 (split-app-composition).",
             "The address, Back/Forward/Reload and ⌘T commands target the focused pane; clicking a pane focuses it.",
             "In the rail both tabs carry the split mark; selecting either shows the pair; closing one pane keeps its tab and dissolves the split."])
    print(b.finish())

if "G-10" in which:
    b = Board("G-10", width=3040, desc="Drag a tab over the card: the edge zone under the pointer (24 % of the pane) shows a 2 pt accent frame with a 12 % accent fill and the split glyph; dropping opens the split. The divider drags with live native resize; below 320 pt per pane the split rests on the focused pane.", ids="bbr-split-zone-<edge>, bbr-split-divider")
    for t in (LIGHT, DARK):
        row = b.row(gap=32, name=t.label())
        col = b.column(row, name="Zone")
        b.caption(col, "Drop zone · right · " + t.label())
        zone = (f'<div layer-name="bbr-split-zone-right" style="position:absolute;right:8px;top:8px;bottom:8px;width:24%;border-radius:12px;box-shadow:inset 0 0 0 2px {t.c("accent")};background-color:{t.c("tint-accent")};opacity:0.9;display:flex;align-items:flex-start;justify-content:flex-end;padding:10px">{icon("split", t.c("accent-ink"), 18)}</div>')
        card_plate(b, col, t, page_placeholder(t, lines=4), 900, 520, overlays=[zone])
        col = b.column(row, name="Resize")
        b.caption(col, "Divider drag · " + t.label(), "resize cursor, accent while dragging")
        card = (f'<div style="flex:1 1 0;display:flex"><div style="flex:6 1 0;min-width:0;display:flex">{page_placeholder(t, lines=4, width=400)}</div>'
                f'<div style="width:2px;flex-shrink:0;background-color:{t.c("accent")}"></div><div style="flex:4 1 0;min-width:0;display:flex">{page_placeholder(t, lines=4, width=300)}</div></div>')
        card_plate(b, col, t, card, 900, 520)
        col = b.column(row, name="Compact")
        b.caption(col, "Compact · < 640 pt card · " + t.label(), "the split rests on the focused pane")
        chip = (f'<div style="position:absolute;right:12px;top:12px;height:26px;display:flex;align-items:center;gap:6px;padding:0 10px;border-radius:9999px;background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{icon("split", t.c("ink-secondary"), 14)}{text("1 of 2", "meta", t.c("ink"), clamp=False)}</div>')
        card_plate(b, col, t, page_placeholder(t, lines=4, width=360), 560, 520, overlays=[chip])
    b.notes(["Change (phase 6): drop zones, divider and the compact rule are targets; no split UI ships today (Split is scheduling-deferred).",
             "The divider never invalidates while idle; both native views stay interactive during resize.", "Keyboard split creation is an open decision (D-1)."])
    print(b.finish())
