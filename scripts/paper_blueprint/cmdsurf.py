"""Command surfaces and glass panes (W2)."""
import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from shell import *

CARD_X, CARD_W = 300, 1130  # 1440 window: card from x=300 to 1430


def palette(t, query="", results=(), mode="NEW TAB", width=600, hint=None, message=None, left=None, top=120):
    left = CARD_X + (CARD_W - width) // 2 if left is None else left
    if query:
        q = (f'<div style="font-family:var(--font-ui);font-size:17px;line-height:22px;color:{t.c("ink")};white-space:pre">{esc(query)}</div>'
             f'<div layer-name="bbr-palette-cursor" style="width:2px;height:20px;background-color:{t.c("accent")};flex-shrink:0"></div><div style="flex:1 1 0"></div>')
    else:
        q = (f'<div layer-name="bbr-palette-cursor" style="width:2px;height:20px;background-color:{t.c("accent")};flex-shrink:0"></div>'
             f'<div style="flex:1 1 0;font-family:var(--font-ui);font-size:17px;line-height:22px;color:{t.c("ink-faint")};white-space:nowrap">Search or enter address</div>')
    hint_html = ""
    if hint:
        hint_html = (f'<div style="display:flex;align-items:center;gap:6px">{text(hint[0], "label", t.c("ink-secondary"), clamp=False)}{profile_chip(t, hint[1], hint[2], 20)}</div>')
    from blocks import mode_pill
    rows = "".join(results)
    msg = f'<div style="height:36px;display:flex;align-items:center;padding:0 12px">{text(message, "meta", t.c("ink-faint"), clamp=False)}</div>' if message else ""
    body = f'<div layer-name="Results" style="display:flex;flex-direction:column;padding:8px">{msg}{rows}</div>' if (rows or msg) else ""
    return (f'<div layer-name="bbr-palette" style="position:absolute;left:{left}px;top:{top}px;width:{width}px;display:flex;flex-direction:column;border-radius:var(--radius-xl);overflow:hidden;'
            f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
            f'<div layer-name="bbr-palette-field" style="height:56px;display:flex;align-items:center;gap:12px;padding:0 16px;{"border-bottom:1px solid " + t.c("hairline") if body else ""}">'
            f'{icon("search", t.c("ink-secondary"), 18)}{hint_html}{q}{mode_pill(t, mode) if mode else ""}</div>{body}</div>')


def overlay_l(t, url="https://x.com/home", results=(), width=440):
    sel = (f'<div style="display:flex;min-width:0;overflow:hidden"><div style="padding:0 1px;background-color:{t.c("tint-accent")};border-radius:3px">'
           f'{text(url, "mono", t.c("ink"), clamp=False, extra="white-space:nowrap;")}</div></div>')
    return (f'<div layer-name="bbr-command-overlay" style="position:absolute;left:4px;top:52px;width:{width}px;display:flex;flex-direction:column;border-radius:var(--radius-lg);overflow:hidden;'
            f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
            f'<div layer-name="bbr-command-overlay-field" style="height:40px;display:flex;align-items:center;gap:8px;padding:0 12px;border-bottom:1px solid {t.c("hairline")}">{sel}</div>'
            f'<div style="display:flex;flex-direction:column;padding:4px">{"".join(results)}</div></div>')


def lresult(t, title, url, fav, kind, selected=False):
    from blocks import result_row
    return result_row(t, title, url, fav, kind, selected).replace("height:40px", "height:36px")


# ---------------------------------------------------------------- glass
def empty_illustration(t, w=400, h=240):
    cards = [(28, 40, "profile-cobalt", 90), (236, 34, "profile-terracotta", 70), (258, 150, "space-pine", 80),
             (30, 168, "profile-amber", 64), (150, 194, "space-plum", 76), (282, 94, "ink", 60)]
    html = "".join(
        f'<div style="position:absolute;left:{x}px;top:{y}px;width:{bw + 48}px;height:34px;display:flex;align-items:center;gap:8px;padding-left:10px;border-radius:11px;background-color:{t.c("overlay")};box-shadow:{t.elev2()}">'
        f'<div style="width:18px;height:18px;border-radius:5px;background-color:{t.c(c)}"></div><div style="width:{bw}px;height:8px;border-radius:4px;background-color:{t.c("border-strong")}"></div></div>'
        for x, y, c, bw in cards)
    key = (f'<div style="position:absolute;left:160px;top:92px;width:80px;height:62px;border-radius:14px;background-color:{t.c("hairline")}"></div>'
           f'<div style="position:absolute;left:160px;top:88px;width:80px;height:60px;border-radius:14px;display:flex;align-items:center;justify-content:center;background-color:{t.c("canvas")};box-shadow:inset 0 0 0 1px {t.c("hairline")}">{icon("cmd", t.c("ink"), 26)}</div>')
    return f'<div layer-name="D10 illustration · empty space" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0">{key}{html}</div>'


def heartbeat(t, head=210, w=400, h=90):
    import math
    def beat(x):
        d = x - head
        if not (-26 < d < 0):
            return 0
        p = (d + 26) / 26
        return 0 if p >= 0.9 else -38 * math.sin(p * 2 * math.pi) * math.exp(-3 * (p - 0.5) ** 2)
    segs = []
    x = 0
    while x < w:
        d = head - x
        key = int(d // 40) if 0 <= d < 260 else None
        pts = []
        while x < w and ((int((head - x) // 40) if 0 <= head - x < 260 else None) == key):
            pts.append(f"{x},{h / 2 + beat(x):.1f}")
            x += 2
        pts.append(f"{x},{h / 2 + beat(x):.1f}")
        colour, op = (t.c("accent"), max(0.12, 1 - (key * 40 + 20) / 260)) if key is not None else (t.c("ink-faint"), 0.25)
        segs.append(f'<polyline points="{" ".join(pts)}" fill="none" stroke="{colour}" opacity="{op:.2f}" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>')
    dot = f'<circle cx="{head}" cy="{h / 2}" r="10" fill="{t.c("accent")}" opacity="0.15"/><circle cx="{head}" cy="{h / 2}" r="4" fill="{t.c("accent")}"/>'
    return f'<svg layer-name="D10 illustration · heartbeat" width="{w}" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg" style="flex-shrink:0">{"".join(segs)}{dot}</svg>'


def offline_link(t, w=310, h=58):
    ink, dots, acc, warn = t.c("ink"), t.c("border-strong"), t.c("accent"), t.c("status-attention")
    X = lambda x: x - 40
    Y = lambda y: y - 92
    parts = [f'<rect x="{X(58.5)}" y="{Y(96.5)}" width="63" height="43" rx="7.5" fill="none" stroke="{ink}" stroke-width="3"/>',
             f'<path d="M{X(48.5)} {Y(146)} H{X(131.5)}" stroke="{ink}" stroke-width="3" stroke-linecap="round"/>',
             f'<circle cx="{X(310)}" cy="{Y(120)}" r="28" fill="none" stroke="{ink}" stroke-width="3"/>',
             f'<ellipse cx="{X(310)}" cy="{Y(120)}" rx="12" ry="28" fill="none" stroke="{ink}" stroke-width="2.4"/>',
             f'<path d="M{X(282)} {Y(120)} H{X(338)}" stroke="{ink}" stroke-width="2.4"/>']
    for i in range(12):
        x = 138 + i * 12
        if x > 270:
            break
        if 186 < x < 214:
            continue
        parts.append(f'<circle cx="{X(x)}" cy="{Y(120)}" r="2" fill="{dots}"/>')
    for k in range(3):
        x = 138 + (186 - 138) * (k / 3)
        parts.append(f'<circle cx="{X(x):.1f}" cy="{Y(120)}" r="3.6" fill="{acc}"/>')
    parts.append(f'<circle cx="{X(200)}" cy="{Y(120)}" r="14" fill="{warn}" opacity="0.10"/>')
    parts.append(f'<path d="M{X(188)} {Y(111)} l6 6 M{X(212)} {Y(129)} l-6 -6" stroke="{warn}" stroke-width="2.6" stroke-linecap="round"/>')
    return f'<svg layer-name="D10 illustration · offline" width="{w}" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg" style="flex-shrink:0">{"".join(parts)}</svg>'


def pane(t, illustration, title, copy, actions, bg="canvas", title_role="display", url=None, name="Glass pane"):
    u = text(url, "mono", t.c("ink-faint"), clamp=False) if url else ""
    return (f'<div layer-name="{esc(name)}" style="flex:1 1 0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;padding:40px;background-color:{t.c(bg)}">'
            f'{illustration}<div style="height:8px"></div>{text(title, title_role, t.c("ink"), clamp=False)}'
            f'<div style="max-width:420px;text-align:center;font-family:var(--font-ui);font-size:var(--text-meta);line-height:18px;color:{t.c("ink-secondary")}">{esc(copy)}</div>{u}'
            f'<div style="display:flex;gap:12px;padding-top:12px">{actions}</div></div>')


def card_plate(b, target, t, content, w=900, h=560, name=None, overlays=()):
    frame = b.add(f'<div layer-name="{esc(name or ("Card · " + t.label()))}" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0;display:flex;padding:10px;border-radius:12px;'
                  f'background-color:{t.c("sidebar")};box-shadow:0 0 0 1px {DOC.c("hairline")}"></div>', target)
    card = b.add(f'<div layer-name="bbr-content-card" style="position:relative;flex:1 1 0;min-width:0;display:flex;flex-direction:column;border-radius:10px;overflow:hidden;background-color:{t.c("glass")};box-shadow:{t.card_shadow()}"></div>', frame)
    for part in (content if isinstance(content, list) else [content]):
        b.add(part, card)
    for o in overlays:
        b.add(o, card)
    return frame
