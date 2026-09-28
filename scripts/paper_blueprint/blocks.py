"""Shared building-block HTML (page 11) reused by later boards."""
import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from lib import *


def button(t, label, kind="primary", state="rest", icon_name=None, width=None, size="n"):
    h = {"s": 24, "n": 28, "l": 32}[size]
    pad = {"s": 10, "n": 14, "l": 16}[size]
    bg, fg, ring, extra = t.c("field"), t.c("ink"), f"0 0 0 1px {t.c('border-strong')}", ""
    if kind == "primary":
        bg, fg, ring = t.c("accent"), t.c("on-accent"), "none"
        if state == "hover":
            bg = t.c("accent-ink")
    elif kind == "secondary":
        if state == "hover":
            bg = t.c("address-hover") if not t.dark else t.c("wash-active")
    elif kind == "quiet":
        bg, fg, ring = "transparent", t.c("ink-secondary"), "none"
        if state == "hover":
            bg = t.c("wash-hover")
    elif kind == "danger":
        fg = t.c("status-error")
        if state == "hover":
            bg = t.c("status-error-surface")
    if state == "pressed":
        extra = f"background-image:linear-gradient({t.c('wash-active')}, {t.c('wash-active')});"
    if state == "focused":
        ring = (ring + ", " if ring != "none" else "") + f"0 0 0 2px {t.c('canvas')}, 0 0 0 4px {t.c('accent')}"
    if state == "disabled":
        extra += "opacity:0.45;"
    w = f"width:{width}px;justify-content:center;" if width else ""
    ic = icon(icon_name, fg, 14) if icon_name else ""
    return (f'<div layer-name="Button · {kind} · {state}" style="height:{h}px;flex-shrink:0;display:flex;align-items:center;gap:6px;padding:0 {pad}px;{w}'
            f'border-radius:var(--radius-md);background-color:{bg};box-shadow:{ring};{extra}">{ic}{text(label, "label", fg, clamp=False)}</div>')


def keycap(t, label):
    return (f'<div style="height:20px;min-width:20px;flex-shrink:0;display:flex;align-items:center;justify-content:center;padding:0 6px;'
            f'border-radius:5px;background-color:{t.c("wash-hover")};box-shadow:inset 0 0 0 1px {t.c("hairline")}">{text(label, "key", t.c("ink-secondary"), clamp=False)}</div>')


def field(t, value="", placeholder="Search", state="rest", lead="search", width=260, trailing=""):
    ring = f"0 0 0 1px {t.c('border-strong')}"
    bg = t.c("field")
    if state == "hover":
        ring = f"0 0 0 1px {t.c('ink-faint')}"
    if state == "focused":
        ring = f"0 0 0 1px {t.c('accent')}, 0 0 0 3px {t.c('tint-accent')}"
    if state == "error":
        ring = f"0 0 0 1px {t.c('status-error')}"
    op = "opacity:0.45;" if state == "disabled" else ""
    content = text(value, "label", t.c("ink"), weight=400, grow=True) if value else text(placeholder, "label", t.c("ink-faint"), weight=400, grow=True)
    caret = f'<div style="width:1.5px;height:16px;background-color:{t.c("accent")};flex-shrink:0"></div>' if state == "focused" else ""
    lead_html = icon(lead, t.c("ink-faint"), 14) if lead else ""
    return (f'<div layer-name="Field · {state}" style="width:{width}px;height:32px;flex-shrink:0;display:flex;align-items:center;gap:8px;padding:0 10px;'
            f'border-radius:var(--radius-lg);background-color:{bg};box-shadow:{ring};{op}">{lead_html}{content if not caret else ""}'
            + (f'{text(value, "label", t.c("ink"), weight=400, clamp=False)}{caret}<div style="flex:1 1 0"></div>' if caret else "") + f'{trailing}</div>')


def checkbox(t, on=False, label=None, mixed=False):
    box = (f'<div style="width:16px;height:16px;flex-shrink:0;border-radius:4px;display:flex;align-items:center;justify-content:center;'
           + (f'background-color:{t.c("accent")};">{icon("minus" if mixed else "check", t.c("on-accent"), 12, 2)}' if on or mixed
              else f'background-color:{t.c("field")};box-shadow:inset 0 0 0 1px {t.c("border-strong")}">') + '</div>')
    lab = text(label, "label", t.c("ink"), weight=400, clamp=False) if label else ""
    return f'<div style="display:flex;align-items:center;gap:8px">{box}{lab}</div>'


def switch(t, on=False, label=None):
    track = t.c("accent") if on else t.c("border-strong")
    knob_pos = "left:14px" if on else "left:2px"
    sw = (f'<div style="position:relative;width:30px;height:18px;flex-shrink:0;border-radius:9999px;background-color:{track}">'
          f'<div style="position:absolute;top:2px;{knob_pos};width:14px;height:14px;border-radius:9999px;background-color:var(--color-field);box-shadow:0 1px 2px rgb(0 0 0 / 20%)"></div></div>')
    lab = text(label, "label", t.c("ink"), weight=400, clamp=False, grow=True) if label else ""
    return f'<div style="display:flex;align-items:center;gap:10px">{lab}{sw}</div>'


def segmented(t, options, selected=0, width=None):
    segs = "".join(
        f'<div style="flex:1 1 0;height:24px;display:flex;align-items:center;justify-content:center;padding:0 10px;border-radius:6px;'
        + (f'background-color:{t.c("tint-selected")};box-shadow:{t.lift()}' if i == selected else "")
        + f'">{text(o, "label", t.c("ink") if i == selected else t.c("ink-secondary"), clamp=False)}</div>'
        for i, o in enumerate(options))
    w = f"width:{width}px;" if width else ""
    return (f'<div layer-name="Segmented" style="{w}display:flex;gap:2px;padding:2px;border-radius:var(--radius-md);'
            f'background-color:{t.c("wash-hover")}">{segs}</div>')


def popup_button(t, label, width=180):
    return (f'<div style="width:{width}px;height:28px;flex-shrink:0;display:flex;align-items:center;gap:6px;padding:0 8px 0 10px;border-radius:var(--radius-md);'
            f'background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("border-strong")}">{text(label, "label", t.c("ink"), weight=400, grow=True)}{icon("chevron-down", t.c("ink-secondary"), 14)}</div>')


def menu_row(t, label, state="rest", shortcut=None, icon_name=None, submenu=False, danger=False, checked=False, disabled=False):
    bg = t.c("tint-accent") if state == "cursor" else "transparent"
    fg = t.c("status-error") if danger else t.c("ink")
    lead = slot(icon(icon_name, t.c("status-error") if danger else t.c("ink-secondary"), 16), 16) if icon_name else ""
    if checked:
        lead = slot(icon("check", t.c("accent-ink"), 14), 16)
    tail = ""
    if shortcut:
        tail = text(shortcut, "key", t.c("ink-faint"), weight=400, clamp=False)
    if submenu:
        tail = icon("chevron-right", t.c("ink-secondary"), 14)
    op = "opacity:0.45;" if disabled else ""
    return (f'<div layer-name="Menu row · {state}" style="height:28px;flex-shrink:0;display:flex;align-items:center;gap:8px;padding:0 8px;'
            f'border-radius:var(--radius-sm);background-color:{bg};{op}">{lead}{text(label, "label", fg, weight=400, grow=True)}{tail}</div>')


def menu_sep(t):
    return f'<div style="height:9px;flex-shrink:0;display:flex;align-items:center;padding:0 8px"><div style="flex:1 1 0;height:1px;background-color:{t.c("hairline")}"></div></div>'


def menu_header(t, label):
    return f'<div style="height:24px;flex-shrink:0;display:flex;align-items:flex-end;padding:0 8px 4px 8px">{text(label, "badge", t.c("ink-faint"), clamp=False)}</div>'


def menu(t, rows, width=240, focused=True, name="Menu"):
    edge = t.c("accent") if focused else t.c("hairline")
    return (f'<div layer-name="{esc(name)}" style="width:{width}px;flex-shrink:0;display:flex;flex-direction:column;padding:4px;border-radius:var(--radius-md);'
            f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {edge}, {t.elev2()}">{"".join(rows)}</div>')


def result_row(t, title, url=None, fav=("mono", "A", 1), kind="Tab", selected=False, match=None, subtitle=None):
    bg = t.c("tint-accent") if selected else "transparent"
    if match and match.lower() in title.lower():
        i = title.lower().index(match.lower())
        parts = [(title[:i], t.c("ink")), (title[i:i + len(match)], t.c("accent-ink")), (title[i + len(match):], t.c("ink"))]
        title_html = '<div style="display:flex;min-width:0;flex:0 0 auto;max-width:72%;overflow:hidden">' + "".join(
            f'<div style="font-family:var(--font-ui);font-size:var(--text-title);line-height:17px;font-weight:{600 if c == t.c("accent-ink") else 500};color:{c};white-space:pre">{esc(p)}</div>' for p, c in parts if p) + '</div>'
    else:
        title_html = f'<div style="min-width:0;flex:0 0 auto;max-width:72%">{text(title, "title", t.c("ink"))}</div>'
    url_html = text(url, "mono", t.c("ink-faint"), extra="flex:0 1 auto;min-width:0;") if url else ""
    sub = text(subtitle, "meta", t.c("ink-secondary"), extra="flex:0 1 auto;min-width:0;") if subtitle else ""
    hint = text("↵" if selected else kind, "key", t.c("accent-ink") if selected else t.c("ink-faint"), clamp=False)
    return (f'<div layer-name="Result · {esc(kind)}{" · selected" if selected else ""}" style="height:40px;flex-shrink:0;display:flex;align-items:center;gap:10px;padding:0 12px;'
            f'border-radius:var(--radius-md);background-color:{bg}">{favicon(t, fav)}'
            f'<div style="flex:1 1 0;min-width:0;display:flex;align-items:center;gap:10px;overflow:hidden">{title_html}{url_html}{sub}</div>{hint}</div>')


def badge(t, label, tone="neutral"):
    colours = {"neutral": (t.c("wash-hover"), t.c("ink-secondary")), "accent": (t.c("tint-accent"), t.c("accent-ink")),
               "attention": (t.c("status-attention-surface"), t.c("status-attention")),
               "error": (t.c("status-error-surface"), t.c("status-error"))}
    bg, fg = colours[tone]
    return (f'<div style="height:20px;flex-shrink:0;display:flex;align-items:center;padding:0 8px;border-radius:9999px;background-color:{bg}">'
            f'{text(label, "badge", fg, clamp=False)}</div>')


def mode_pill(t, label="NEW TAB"):
    return (f'<div layer-name="bbr-command-mode" style="height:22px;flex-shrink:0;display:flex;align-items:center;padding:0 8px;border-radius:6px;'
            f'background-color:{t.c("tint-accent")}">{text(label, "badge", t.c("accent-ink"), clamp=False)}</div>')


def sheet(t, body, width=420, title=None, name="Sheet", pad=20, radius="var(--radius-lg)"):
    head = text(title, "title-xl", t.c("ink"), weight=600, clamp=False) if title else ""
    return (f'<div layer-name="{esc(name)}" style="width:{width}px;flex-shrink:0;display:flex;flex-direction:column;gap:14px;padding:{pad}px;border-radius:{radius};'
            f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">{head}{body}</div>')


def banner(t, message, tone="info", actions=""):
    ic = {"info": ("info", "status-info"), "attention": ("warn", "status-attention"), "error": ("warn", "status-error"), "ok": ("check", "status-ok")}[tone]
    surf = {"info": "tint-accent", "attention": "status-attention-surface", "error": "status-error-surface", "ok": "tint-accent"}[tone]
    return (f'<div layer-name="Banner · {tone}" style="display:flex;align-items:center;gap:10px;min-height:40px;padding:8px 8px 8px 12px;border-radius:var(--radius-md);'
            f'background-color:{t.c(surf)}">{icon(ic[0], t.c(ic[1]), 16)}'
            f'<div style="flex:1 1 0;min-width:0;font-family:var(--font-ui);font-size:var(--text-label);line-height:18px;color:{t.c("ink")}">{esc(message)}</div>{actions}</div>')


def progress(t, pct, width=200, tone="accent"):
    return (f'<div style="width:{width}px;height:4px;flex-shrink:0;border-radius:9999px;background-color:{t.c("wash-active")}">'
            f'<div style="width:{pct}%;height:4px;border-radius:9999px;background-color:{t.c(tone)}"></div></div>')
