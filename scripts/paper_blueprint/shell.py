"""Standard shell content used across W1–W5 boards."""
import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from blocks import *

FAVS = [("site", "amber", "H"), ("site", "fuchsia", "Y"), ("site", "terracotta", "M"), ("site", "ink", "X")]


def organized(t, selected="X", folder_open=True):
    rows = [tab_row(t, "Home / X", ("site", "ink", "X"), "selected" if selected == "X" else "rest"),
            tab_row(t, "Reddit – Das Herz des Internets", ("site", "terracotta", "R"), asleep=True),
            tab_row(t, "WhatsApp", ("site", "pine", "W"))]
    rows += [folder_row(t, "Home"), folder_row(t, "ai", opened=folder_open)]
    if folder_open:
        rows.append(tab_row(t, "CLIProxyAPI Limits", ("mono", "C", 2), indent=12))
    rows += [folder_row(t, "Review Tools"), folder_row(t, "Haus")]
    return rows


def regular(t, hover="Daybreak", selected=None, loading=None):
    items = [("Knuddels-Bots.com | Underground", ("mono", "K", 1), None),
             ("Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "attention"),
             ("Vision not supported? · Issue #4127", ("mono", "P", 5), None),
             ("Daybreak | OpenAI for Enterprise", ("mono", "O", 6), None),
             ("Cloudflare Registrar | Domains", ("site", "amber", "C"), None),
             ("YouTube", ("site", "fuchsia", "Y"), "audio")]
    out = []
    for title, fav, trailing in items:
        state = "rest"
        key = title.split(" ")[0]
        if hover and key.startswith(hover):
            state = "hover"
        if selected and key.startswith(selected):
            state = "selected"
        if loading and key.startswith(loading):
            fav = ("loading",)
        out.append(tab_row(t, title, fav, state, trailing=trailing if state == "rest" else None, asleep=(loading and key.startswith(loading))))
    return out


def std(t, **kw):
    spec = {
        "address": {"domain": "x.com"},
        "favorites": FAVS,
        "heading": ("Default", "code"),
        "groups": [organized(t, kw.pop("selected_pinned", "X"), kw.pop("folder_open", True)), "divider", "new",
                   regular(t, kw.pop("hover", "Daybreak"), kw.pop("selected", None), kw.pop("loading", None))],
        "footer": {"spaces": ("code", "bag", "moon"), "active": 0},
    }
    spec.update(kw)
    return spec


def pair(b, spec_fn, w=1440, h=900, row=None, label=None, themes=(LIGHT, DARK), gap=40):
    row = row or b.row(gap=gap)
    for t in themes:
        col = b.column(row, name=t.label())
        b.caption(col, (label + " · " if label else "") + t.label())
        window(b, col, t, spec_fn(t), w, h)
    return row


def work_rows(t, selected="Linear"):
    pins = [tab_row(t, "Linear — Sprint 42", ("site", "violet", "L"), "selected" if selected == "Linear" else "rest"),
            tab_row(t, "Design system · Components", ("site", "fuchsia", "D")), folder_row(t, "Customers")]
    tabs = [tab_row(t, "Pull requests · bbr", ("site", "ink", "G")), tab_row(t, "Incident review 09/24", ("mono", "I", 2)),
            tab_row(t, "Grafana — latency", ("site", "amber", "G")), tab_row(t, "Calendar · Week 40", ("site", "cobalt", "C"), trailing="attention")]
    return [pins, "divider", "new", tabs]


def profile_chip(t, letter, hue, size=24):
    fg = "var(--color-profile-amber-letter)" if hue == "amber" and not t.dark else "var(--color-on-accent)"
    if t.dark:
        fg = "var(--color-identity-letter)"
    return (f'<div layer-name="bbr-profile-chip · {hue}" style="width:{size}px;height:{size}px;flex-shrink:0;display:flex;align-items:center;justify-content:center;border-radius:var(--radius-sm);'
            f'background-color:{t.c("profile-" + hue)}">{text(letter, "badge", fg, clamp=False, extra="letter-spacing:0;")}</div>')


def empty_pane(t, profile="Default", space="Space 3"):
    from blocks import button
    return (f'<div layer-name="bbr-empty-new-tab" style="flex:1 1 0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;background-color:{t.c("canvas")}">'
            f'<div style="width:80px;height:60px;border-radius:14px;display:flex;align-items:center;justify-content:center;background-color:{t.c("canvas")};box-shadow:0 0 0 1px {t.c("hairline")}, 0 4px 0 {t.c("hairline")}">{icon("cmd", t.c("ink"), 22)}</div>'
            f'{text("No tabs in this space", "display", t.c("ink"), clamp=False)}{text("in Profile · " + profile, "meta", t.c("ink-secondary"), clamp=False)}'
            f'<div style="padding-top:12px">{button(t, "New Tab   ⌘T", "primary", size="l")}</div></div>')
