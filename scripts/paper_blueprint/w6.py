import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from story import *
from blocks import result_row, button, banner, menu, menu_row, menu_sep, badge, progress
from w1_21 import frag, frag_rows

which = sys.argv[1:]
TOK = ["cobalt", "violet", "fuchsia", "amber", "terracotta", "slate", "pine", "plum", "ink"]
IDH = {k: ("profile-" + k if k in ("cobalt", "violet", "fuchsia", "amber", "terracotta", "slate") else "space-" + k) for k in TOK}


def mini_rail(t, w=200):
    return (f'<div style="width:{w}px;flex-shrink:0;display:flex;flex-direction:column;gap:4px;padding:10px 6px 6px 6px;border-radius:10px;background-color:{t.c("sidebar")};box-shadow:0 0 0 1px {DOC.c("hairline")}">'
            f'<div style="display:flex;align-items:center;gap:8px;padding:0 6px 4px 6px">{glyph("bag", t.c("ink-secondary"), 14)}{text("Work", "meta", t.c("ink-secondary"), weight=600)}</div>'
            f'<div style="height:30px;display:flex;align-items:center;gap:8px;padding:0 8px;border-radius:8px;background-color:{t.c("tint-selected")};box-shadow:{t.lift()}">{favicon(t, ("mono", "L", 2), 16)}{text("Linear — Sprint 42", "label", t.c("ink"))}</div>'
            f'<div style="height:30px;display:flex;align-items:center;gap:8px;padding:0 8px">{favicon(t, ("mono", "G", 4), 16)}{text("Pull requests", "label", t.c("ink"), weight=400)}</div>'
            f'<div style="height:30px;display:flex;align-items:center;gap:8px;padding:0 8px">{favicon(t, ("mono", "C", 1), 16)}{text("Calendar · asleep", "label", t.c("ink-secondary"), weight=400)}</div></div>')


if "V-1" in which:
    b = Board("V-1", width=3040, desc="With the macOS 'Increase contrast' setting, ink goes to black / white, secondary text and lines deepen, washes and the address veil harden, accent and tint.accent step darker (light) or brighter (dark). Materials and layout do not change.", ids="snapshot contrast = increased")
    pair(b, lambda t: std(t), themes=(T("light", ic=True), T("dark", ic=True)))
    row = b.row(gap=32, name="Surfaces")
    for t in (T("light", ic=True), T("dark", ic=True)):
        col = b.column(row, name=t.label())
        b.caption(col, "Palette and menu · " + t.label())
        res = [result_row(t, "YouTube Music", "music.youtube.com", ("site", "fuchsia", "Y"), "Tab", True, "you"), result_row(t, "Your Watch Later list", "youtube.com/playlist", ("site", "fuchsia", "Y"), "History", False, "you")]
        b.add(f'<div style="display:flex;gap:24px;align-items:flex-start;padding:24px;border-radius:12px;background-color:{t.c("sidebar")}">'
              f'<div style="position:relative;width:560px;height:200px">{palette(t, "you", res, width=560, left=0, top=0)}</div>'
              + menu(t, [menu_row(t, "Pin Tab", "cursor", "⇧⌘P", "pin"), menu_row(t, "Copy Link", icon_name="external"), menu_sep(t), menu_row(t, "Close Tab", shortcut="⌘W", icon_name="close", danger=True)], 220) + '</div>', col)
    b.notes(["Bars at increased contrast: ink ≥ 14:1 and ink.secondary ≥ 9:1 on every rail and tint; hairlines become visible edges (#9FA29E / #454C55).",
             "The dark IC selection is opaque (#35383F) instead of a veil; the light selection stays white with its lift.", "Every tint step adds 4 % chroma at IC; luminance stays pinned."])
    print(b.finish())

if "V-2" in which:
    b = Board("V-2", width=2960, desc="Every curated colour at every intensity, in light and dark, rendered as the rail it produces: the identity hue, three paper steps, the lifted selection and secondary text. The pinned luminance keeps every row identical in contrast; only hue changes.", ids="space_tint(token, step, theme)")
    for mode in ("light", "dark"):
        b.caption(b.column(b.row(gap=0, pad="0 56px 12px 56px"), name="c"), mode.capitalize())
        row = b.row(gap=20, wrap=True, name=mode)
        for tok in TOK:
            cells = "".join(f'<div style="display:flex;flex-direction:column;gap:6px">{mini_rail(T(mode, tint=tok, step=s), 200)}{text(s, "key", DOC.c("ink-faint"), weight=400, clamp=False)}</div>' for s in ("subtle", "standard", "rich"))
            t0 = T(mode)
            b.add(f'<div layer-name="{tok}" style="display:flex;flex-direction:column;gap:8px;padding:14px;border-radius:14px;background-color:{DOC.c("field")};box-shadow:0 0 0 1px {DOC.c("hairline")}">'
                  f'<div style="display:flex;align-items:center;gap:8px"><div style="width:14px;height:14px;border-radius:4px;background-color:{t0.c(IDH[tok])}"></div>{text(tok.capitalize(), "label", DOC.c("ink"), clamp=False)}</div>'
                  f'<div style="display:flex;gap:10px">{cells}</div></div>', row)
    b.notes(["Values are the 54 --color-rail-* tokens synced from space_tint.rs (verify_tokens.py checks them).", "Grain (F-2) is independent of the step and drawn in F-2."])
    print(b.finish())

if "V-3" in which:
    b = Board("V-3", width=2400, desc="One density choice scales the whole chrome together — rows, type, tiles, rail width, palette width — and nothing else. The rows of one session never change height.", ids="Density::{Small, Normal, ExtraLarge}")
    D = {"S": dict(rail=232, row=32, fs=12, tile=16, fav=(40, 36), gap=4, rad=6, pitch=4), "N": dict(rail=300, row=36, fs=14, tile=18, fav=(64, 44), gap=4, rad=10, pitch=4), "XL": dict(rail=340, row=44, fs=16, tile=22, fav=(88, 52), gap=8, rad=12, pitch=4)}
    def drail(t, d):
        row = lambda title, fav, sel=False, sec=False: (f'<div style="height:{d["row"]}px;flex-shrink:0;display:flex;align-items:center;gap:10px;padding:0 6px 0 10px;border-radius:{d["rad"]}px;'
                                                        + (f'background-color:{t.c("tint-selected")};box-shadow:{t.lift()};' if sel else "") + f'">{favicon(t, fav, d["tile"])}'
                                                        f'<div style="flex:1 1 0;min-width:0;font-family:var(--font-ui);font-size:{d["fs"]}px;line-height:{d["fs"] + 4}px;font-weight:500;color:{t.c("ink-secondary") if sec else t.c("ink")};white-space:nowrap;overflow:hidden;text-overflow:ellipsis">{esc(title)}</div></div>')
        favs = "".join(f'<div style="flex:1 1 0;min-width:0;max-width:{d["fav"][0]}px;height:{d["fav"][1]}px;display:flex;align-items:center;justify-content:center;border-radius:{d["rad"]}px;background-color:{t.c("favorite")}">{favicon(t, f, d["tile"])}</div>' for f in FAVS)
        rows = [row("Home / X", ("site", "ink", "X"), True), row("WhatsApp", ("site", "pine", "W")), row("Knuddels-Bots.com | Underground", ("mono", "K", 1)),
                row("Rust SIMD on the GPU – a field guide", ("mono", "V", 3)), row("Daybreak | OpenAI for Enterprise", ("mono", "O", 6)), row("Suspended · Hacker News", ("site", "amber", "H"), sec=True)]
        return (f'<div style="width:{d["rail"]}px;height:560px;flex-shrink:0;display:flex;flex-direction:column;gap:{d["pitch"]}px;padding:12px 8px;border-radius:12px;background-color:{t.c("sidebar")};box-shadow:0 0 0 1px {DOC.c("hairline")}">'
                f'<div style="height:{d["row"]}px;flex-shrink:0;display:flex;align-items:center;padding:0 12px;border-radius:12px;background-color:{t.c("address")};border:1px solid {t.c("address-border")}"><div style="font-family:var(--font-ui);font-size:{d["fs"]}px;font-weight:500;color:{t.c("ink")}">x.com</div></div>'
                f'<div style="display:flex;gap:{d["gap"]}px;padding:8px 0">{favs}</div>{"".join(rows)}</div>')
    for t in (LIGHT, DARK):
        row = b.row(gap=40, name=t.label())
        for k in ("S", "N", "XL"):
            b.add(f'<div style="display:flex;flex-direction:column;gap:8px">{text(k + " · " + t.label(), "badge", DOC.c("ink-secondary"), clamp=False)}{drail(t, D[k])}</div>', row)
    b.notes(["S 232 / 32 / 12 pt, N 300 / 36 / 14 pt, XL 340 / 44 / 16 pt (rail / row / title); favorite tile heights 36 · 44 · 52 with four columns that share the rail width (F-3).",
             "At S, loose pinned rows become 32 × 32 icon tiles (S-3). The ⌘T palette is 508 / 600 / 716 wide."])
    print(b.finish())

if "V-4" in which:
    b = Board("V-4", width=2400, desc="With 'Reduce motion' every moment keeps its information and drops its movement: loops become static marks, pops become a flash, fades become instant, illustrations show their final frame.", ids="MotionMode::Reduced")
    moments = [("Loading arc", "the arc turns 640 ms per turn", "a static arc", lambda t: spinner(t)),
               ("Address progress (D5)", "sweeps, then fills and fades", "a static line while loading, removed on finish", lambda t: address(t, "github.com", progress=100).replace("padding:0 4px 12px 4px", "padding:0")),
               ("Attention", "two beats with a ring", "opacity 1 → 0.45 → 1, no scale", lambda t: f'<div style="width:8px;height:8px;border-radius:9999px;background-color:{t.c("status-attention")};opacity:0.45"></div>'),
               ("Favorite added", "pop 0.86 → 1, outline, specks", "a 120 ms tint.accent flash on the tile", lambda t: favorite_tile(t, ("site", "terracotta", "Y"), "drop").replace("flex:1 1 0;min-width:0;", "width:64px;")),
               ("Download finished", "check draws + one ring", "the check appears", lambda t: slot(icon("check", t.c("status-ok"), 16))),
               ("⌘T results", "20 ms stagger, 4 pt rise; cursor glides", "fade only; the cursor snaps", lambda t: f'<div style="width:280px">{result_row(t, "YouTube Music", None, ("site", "fuchsia", "Y"), "Tab", True)}</div>'),
               ("Space switch", "chrome tint cross-fades (dur:standard)", "instant", lambda t: space_chip(t, "bag", True)),
               ("Empty states (D10)", "two cycles, then rest", "the rest frame only", lambda t: heartbeat(t, w=240, h=40, head=130))]
    head = "".join(f'<div style="width:{w}px;flex-shrink:0">{text(h, "badge", DOC.c("ink-faint"), clamp=False)}</div>' for h, w in (("Moment", 200), ("Standard", 280), ("Reduced", 300), ("Reduced · light", 320), ("Reduced · dark", 320)))
    rows = [f'<div style="display:flex;gap:16px;padding-bottom:10px;border-bottom:1px solid {DOC.c("hairline")}">{head}</div>']
    for name, std_txt, red_txt, vis in moments:
        cell = lambda t: f'<div style="width:320px;flex-shrink:0;height:64px;display:flex;align-items:center;padding:0 16px;border-radius:10px;background-color:{t.c("sidebar")}">{vis(t)}</div>'
        rows.append(f'<div style="display:flex;gap:16px;align-items:center;min-height:80px;border-bottom:1px solid {DOC.c("hairline")}">'
                    f'<div style="width:200px;flex-shrink:0">{text(name, "label", DOC.c("ink"), clamp=False)}</div><div style="width:280px;flex-shrink:0">{text(std_txt, "meta", DOC.c("ink-secondary"), clamp=False)}</div>'
                    f'<div style="width:300px;flex-shrink:0">{text(red_txt, "meta", DOC.c("ink"), clamp=False)}</div>{cell(LIGHT)}{cell(DARK)}</div>')
    b.add(f'<div layer-name="Reduce Motion table" style="display:flex;flex-direction:column;padding:0 56px 56px 56px">{"".join(rows)}</div>')
    b.notes(["Reduce Motion follows the macOS setting unless overridden in Settings › Appearance (L-8).", "Nothing that carries state is removed: only movement and timing change."])
    print(b.finish())

if "V-5" in which:
    b = Board("V-5", width=3000, desc="The after-commit moments as frames: each starts only after its state is presented, stays within 260 ms (loops excepted) and leaves nothing animating when it ends.", ids="motion.moments[*]")
    def strip(title, frames, t):
        cells = "".join(f'<div style="display:flex;flex-direction:column;gap:6px;align-items:center"><div style="width:110px;height:84px;display:flex;align-items:center;justify-content:center;border-radius:10px;background-color:{t.c("sidebar")}">{html}</div>{text(ts, "key", DOC.c("ink-faint"), weight=400, clamp=False)}</div>' for ts, html in frames)
        return f'<div style="display:flex;flex-direction:column;gap:8px">{text(title + " · " + t.label(), "badge", DOC.c("ink-secondary"), clamp=False)}<div style="display:flex;gap:10px">{cells}</div></div>'
    def pop(t, s, ring, speck):
        sp = "".join(f'<div style="position:absolute;left:{55 + speck * __import__("math").cos(a / 10 * 6.283 + 0.3) - 2:.0f}px;top:{42 + 0.8 * speck * __import__("math").sin(a / 10 * 6.283 + 0.3) - 2:.0f}px;width:4px;height:4px;border-radius:9999px;background-color:{t.c(["accent", "profile-cobalt", "profile-amber", "space-plum", "space-pine"][a % 5])}"></div>' for a in range(10)) if speck else ""
        return (f'<div style="position:relative;width:110px;height:84px;display:flex;align-items:center;justify-content:center">{sp}<div style="width:{int(64 * s)}px;height:{int(44 * s)}px;border-radius:10px;display:flex;align-items:center;justify-content:center;background-color:{t.c("favorite")};'
                + (f'box-shadow:0 0 0 {1.5 + ring:.1f}px {t.c("accent")}' if ring is not None else "") + f'">{favicon(t, ("site", "terracotta", "Y"))}</div></div>')
    for t in (LIGHT, DARK):
        row = b.row(gap=40, wrap=True, name=t.label())
        b.add(strip("Favorite pop", [("0 ms", pop(t, 0.86, 0, 18)), ("60", pop(t, 1.06, 1, 28)), ("120", pop(t, 1.0, 2, 36)), ("260 · rest", pop(t, 1.0, None, 0))], t), row)
        b.add(strip("Download check", [("0 ms", slot(icon("check", t.c("status-ok"), 8))), ("120", f'<div style="width:24px;height:24px;border-radius:9999px;display:flex;align-items:center;justify-content:center;box-shadow:0 0 0 2px {t.c("status-ok")}">{icon("check", t.c("status-ok"), 16)}</div>'), ("260 · rest", slot(icon("check", t.c("status-ok"), 16)))], t), row)
        b.add(strip("Attention", [("0", f'<div style="width:8px;height:8px;border-radius:9999px;background-color:{t.c("status-attention")}"></div>'), ("60", f'<div style="width:12px;height:12px;border-radius:9999px;background-color:{t.c("status-attention")};box-shadow:0 0 0 4px {t.c("status-attention-surface")}"></div>'),
                                  ("240 · rest", f'<div style="width:8px;height:8px;border-radius:9999px;background-color:{t.c("status-attention")}"></div>')], t), row)
        b.add(strip("Tint fade", [("0", f'<div style="width:90px;height:60px;border-radius:8px;background-color:{t.c("sidebar")}"></div>'), ("90", f'<div style="position:relative;width:90px;height:60px;border-radius:8px;background-color:{t.c("sidebar")}"><div style="position:absolute;inset:0;border-radius:8px;background-color:{T(t.mode, tint="cobalt").c("sidebar")};opacity:0.5"></div></div>'),
                                  ("180 · rest", f'<div style="width:90px;height:60px;border-radius:8px;background-color:{T(t.mode, tint="cobalt").c("sidebar")}"></div>')], t), row)
        b.add(strip("Heartbeat (D10)", [("0", heartbeat(t, w=100, h=40, head=-10)), ("700", heartbeat(t, w=100, h=40, head=60)), ("2800 · rest", heartbeat(t, w=100, h=40, head=52))], t), row)
    b.notes(["Favorite pop: 0.86 → 1 back-ease over 60 %, outline 0 → 3 pt, ten specks 18 → 44 pt (dur:emphasis). Download: check stroke then one ring 0 → 8 pt.",
             "Attention: two dur:fine beats. Tint fade: dur:standard, ease:standard. D10 illustrations: two 1400 ms cycles, then the poster frame.",
             "Evidence: each start is recorded in the typed motion snapshot; an idle shell has no moment and requests no frames."])
    print(b.finish())

if "V-6" in which:
    b = Board("V-6", width=2400, desc="Everything that is not a moment: how surfaces appear, change and leave. The rule stays: the committed state is on screen first; appearance is an opacity fade at most dur:fine, never a scale or slide that delays reading.")
    rows = [("⌘T palette, ⌘L overlay", "open / close", "dur:fine fade", "linear", "instant", "Shipped"), ("Menus, plus menu", "open / close", "dur:fine fade", "linear", "instant", "Shipped"),
            ("Downloads popover", "toggle", "dur:fine fade", "linear", "instant", "Shipped"), ("Sheets (auth, delete, clear data)", "open", "dur:fine fade", "linear", "instant", "Change"),
            ("Row hover / press wash", "pointer", "dur:fine", "standard", "instant", "Shipped"), ("Selection, focus ring", "commit", "snap", "—", "snap", "Shipped"),
            ("Drag lift", "4 pt travel", "dur:micro opacity", "linear", "instant", "Shipped"), ("Sidebar ⌘B", "toggle", "instant (next frame)", "—", "instant", "Shipped"),
            ("Rail hover reveal (proposal)", "edge dwell 300 ms", "120 ms slide + fade", "decelerate", "fade only", "New · proposal"),
            ("Toast (proposal)", "reversible action", "dur:fine fade + 4 pt rise; 4 s", "standard", "fade only", "New · proposal"),
            ("Zoom readout (proposal)", "⌘+ / ⌘−", "dur:fine in, fade after 1.2 s", "linear", "no fade", "New · proposal"),
            ("Peek", "present / dismiss", "bound to the portal frame", "—", "same", "Shipped"), ("Space switch list", "commit", "snap (D4)", "—", "snap", "Shipped")]
    head = "".join(f'<div style="width:{w}px;flex-shrink:0">{text(h, "badge", DOC.c("ink-faint"), clamp=False)}</div>' for h, w in (("Element", 300), ("Trigger", 200), ("Duration", 300), ("Easing", 140), ("Reduce Motion", 160), ("Status", 180)))
    body = [f'<div style="display:flex;gap:16px;padding-bottom:10px;border-bottom:1px solid {DOC.c("hairline")}">{head}</div>']
    for r in rows:
        body.append(f'<div style="display:flex;gap:16px;align-items:center;min-height:44px;border-bottom:1px solid {DOC.c("hairline")}">'
                    + "".join(f'<div style="width:{w}px;flex-shrink:0">{text(v, "label" if i == 0 else "meta", DOC.c("ink") if i == 0 else DOC.c("ink-secondary"), weight=500 if i == 0 else 400, clamp=False)}</div>' for i, (v, w) in enumerate(zip(r[:5], (300, 200, 300, 140, 160))))
                    + f'<div style="width:180px">{mark_chip(r[5])}</div></div>')
    b.add(f'<div layer-name="Transition table" style="display:flex;flex-direction:column;padding:0 56px 40px 56px">{"".join(body)}</div>')
    row = b.row(gap=32, name="Frames")
    for t in (LIGHT, DARK):
        frames = "".join(f'<div style="display:flex;flex-direction:column;gap:6px;align-items:center"><div style="width:220px;height:120px;display:flex;align-items:center;justify-content:center;border-radius:10px;background-color:{t.c("sidebar")}"><div style="opacity:{o}">{menu(t, [menu_row(t, "New Space", "cursor"), menu_row(t, "New Folder")], 160)}</div></div>{text(ts, "key", DOC.c("ink-faint"), weight=400, clamp=False)}</div>'
                         for ts, o in (("0 ms", 0.0), ("60 ms", 0.5), ("120 ms · rest", 1.0)))
        b.add(f'<div style="display:flex;flex-direction:column;gap:8px">{text("Menu open (dur:fine fade) · " + t.label(), "badge", DOC.c("ink-secondary"), clamp=False)}<div style="display:flex;gap:10px">{frames}</div></div>', row)
    b.notes(["Mark Change: sheets fade today only where they are Portal surfaces; the table makes the rule uniform.", "No transition may translate web content or scale the card."])
    print(b.finish())
