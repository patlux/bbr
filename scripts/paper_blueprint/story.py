"""Storyboard helpers (W5): whole frames as single HTML strings."""
import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from cmdsurf import *


def rail_html(t, spec, h=620, w=300):
    parts = []
    if spec.get("titlebar", {}) is not None:
        parts.append(titlebar(t, **spec.get("titlebar", {})))
    if spec.get("address", {}) is not None:
        parts.append(address(t, **spec.get("address", {})))
    if spec.get("favorites"):
        parts.append(favorites(t, spec["favorites"], spec.get("fav_states")))
    parts += spec.get("above_heading", [])
    if spec.get("heading", ("Default", "code")):
        parts.append(heading(t, *spec.get("heading", ("Default", "code"))))
    body = []
    for group in spec.get("groups", []):
        if group == "divider":
            body.append(divider(t))
        elif group == "new":
            body.append(f'<div style="flex-shrink:0;padding:0 8px;display:flex;flex-direction:column">{new_tab_row(t, spec.get("new_state", "rest"))}</div>')
        elif isinstance(group, str):
            body.append(group)
        else:
            body.append(rows_block(t, group))
    parts.append(f'<div style="position:relative;flex:1 1 0;min-height:0;display:flex;flex-direction:column;gap:4px;padding-bottom:8px;overflow:hidden">{"".join(body)}</div>')
    parts += spec.get("above_footer", [])
    if spec.get("media"):
        parts.append(spec["media"])
    if spec.get("footer", {}) is not None:
        parts.append(footer(t, **spec.get("footer", {})))
    bg = spec.get("bg", t.c("sidebar"))
    layer = ""
    if spec.get("bg_layer"):
        c, o = spec["bg_layer"]
        layer = f'<div style="position:absolute;inset:0;background-color:{c};opacity:{o}"></div>'
    return (f'<div layer-name="bbr-sidebar" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0;display:flex;flex-direction:column;background-color:{bg}">{layer}'
            + f'<div style="position:relative;flex:1 1 0;min-height:0;display:flex;flex-direction:column">{"".join(parts)}</div></div>')


def win_html(t, spec, w=1000, h=620):
    rail = rail_html(t, spec, h, spec.get("rail_width", 300)) if spec.get("sidebar", True) else ""
    pad = "10px 10px 10px 0" if spec.get("sidebar", True) else "52px 10px 10px 10px"
    content = spec.get("card", "placeholder")
    if content == "placeholder":
        content = page_placeholder(t, lines=3, width=420)
    content = "".join(content) if isinstance(content, list) else (content or "")
    ov = "".join(spec.get("card_overlays", []))
    wov = "".join(spec.get("overlays", []))
    band = ""
    if not spec.get("sidebar", True):
        band = f'<div style="position:absolute;left:0;top:0;right:0;height:52px;display:flex;align-items:center;padding:0 12px">{traffic_lights(t)}</div>'
    return (f'<div layer-name="Window" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0;display:flex;border-radius:12px;overflow:hidden;background-color:{t.c("sidebar")};'
            f'box-shadow:0 0 0 1px {DOC.c("hairline")}, 0 16px 32px -12px var(--color-elev-deep)">{band}{rail}'
            f'<div style="flex:1 1 0;min-width:0;display:flex;padding:{pad}"><div layer-name="bbr-content-card" style="position:relative;flex:1 1 0;min-width:0;display:flex;flex-direction:column;border-radius:10px;overflow:hidden;'
            f'background-color:{t.c(spec.get("card_bg", "glass"))};box-shadow:{t.card_shadow()}">{content}{ov}</div></div>{wov}</div>')


def card_html(t, content, overlays=(), w=640, h=400):
    content = content if content is not None else page_placeholder(t, lines=3, width=420)
    return (f'<div layer-name="Card" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0;display:flex;padding:10px;border-radius:12px;background-color:{t.c("sidebar")};box-shadow:0 0 0 1px {DOC.c("hairline")}">'
            f'<div style="position:relative;flex:1 1 0;min-width:0;display:flex;flex-direction:column;border-radius:10px;overflow:hidden;background-color:{t.c("glass")};box-shadow:{t.card_shadow()}">{content}{"".join(overlays)}</div></div>')


def rail_frame(t, spec, h=620, overlays=()):
    return (f'<div style="position:relative;width:300px;height:{h}px;flex-shrink:0;border-radius:12px;overflow:hidden;box-shadow:0 0 0 1px {DOC.c("hairline")}">{rail_html(t, spec, h)}{"".join(overlays)}</div>')


def mini_palette(t, query="", results=(), mode="NEW TAB", hint=None, message=None, top=90):
    return palette(t, query, results, mode, width=560, hint=hint, message=message, left=300 + (690 - 560) // 2, top=top)


def frame(n, title, caption, timing, visual):
    chip = (f'<div style="height:22px;display:flex;align-items:center;padding:0 8px;border-radius:9999px;background-color:var(--color-tint-accent)">'
            f'{text(timing, "key", "var(--color-accent-ink)", weight=500, clamp=False)}</div>') if timing else ""
    return (f'<div layer-name="Frame {n}" style="display:flex;flex-direction:column;gap:12px;flex-shrink:0">'
            f'<div style="display:flex;align-items:center;gap:10px"><div style="width:26px;height:26px;border-radius:9999px;display:flex;align-items:center;justify-content:center;background-color:var(--color-ink)">'
            f'{text(str(n), "key", "var(--color-canvas)", weight=600, clamp=False)}</div>{text(title, "title", "var(--color-ink)", weight=600, clamp=False)}{chip}</div>'
            f'{visual}<div style="max-width:560px;font-family:var(--font-ui);font-size:var(--text-label);line-height:20px;color:var(--color-ink-secondary)">{esc(caption)}</div></div>')


def arrow():
    return f'<div style="align-self:center;flex-shrink:0;padding-top:30px">{icon("forward", "var(--color-ink-faint)", 22)}</div>'


def storyboard(code, desc, frames, refs="", width=3240, notes=()):
    b = Board(code, width=width, desc=desc, extra_meta=("boards " + refs) if refs else "")
    row = b.row(gap=28, wrap=True, name="Storyboard · light")
    for i, f in enumerate(frames, start=1):
        b.add(frame(i, *f), row)
    if notes:
        b.notes(list(notes))
    return b.finish()
