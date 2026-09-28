import json, sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from lib import *

VALUES = json.load(open('/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/tokenvalues.json'))


def resolve(t, name):
    var = t.c(name)[4:-1]
    value = VALUES.get(var, '')
    while str(value).startswith('var('):
        value = VALUES.get(value[4:-1], value)
    return var, value


def swatch(t, name, over=None):
    var, value = resolve(t, name)
    chip_bg = f"background-color:{t.c(name)};"
    inner = ""
    if over:  # a veil: show it over the rail paint
        chip_bg = f"background-color:{t.c(over)};"
        inner = f'<div style="position:absolute;inset:6px;border-radius:6px;background-color:{t.c(name)}"></div>'
    return (f'<div style="display:flex;gap:12px;align-items:center;width:300px">'
            f'<div style="position:relative;width:44px;height:44px;flex-shrink:0;border-radius:10px;{chip_bg}box-shadow:0 0 0 1px {t.c("hairline")}">{inner}</div>'
            f'<div style="display:flex;flex-direction:column;gap:2px;min-width:0;flex:1 1 0">'
            f'{text(var, "key", t.c("ink"), weight=500)}{text(value, "meta", t.c("ink-secondary"))}</div></div>')

GROUPS = [
    ("Materials", ["canvas", "sidebar", "field", "overlay", "glass"], None),
    ("Veils over the rail", ["favorite", "footer", "address", "address-hover", "address-accessory", "wash-hover", "wash-active", "tint-selected"], "sidebar"),
    ("Ink & lines", ["ink", "ink-secondary", "ink-faint", "hairline", "border-strong"], None),
    ("Accent — Phosphor", ["accent", "accent-ink", "on-accent", "tint-accent"], None),
    ("Status", ["status-error", "status-attention", "status-ok", "status-info", "status-error-surface", "status-attention-surface"], None),
]

b = Board("F-1", width=1560, desc="Every colour role the chrome paints, in the four supported theme columns. Veils are translucent and are shown over the rail paint they sit on. Values are synced from palette.rs; verify_tokens.py checks them against the code.")
row = b.row(gap=24, wrap=True)
for t in (LIGHT, DARK, T("light", ic=True), T("dark", ic=True)):
    col = b.column(row, name=t.label(), width=340)
    b.caption(col, t.label())
    p = plate(b, col, t, w=340, pad=20)
    for title, names, over in GROUPS:
        b.add(f'<div style="display:flex;flex-direction:column;gap:10px">{text(title, "badge", t.c("ink-faint"), clamp=False)}'
              + "".join(swatch(t, n, over) for n in names) + '</div>', p)
b.notes([
    "Text contrast bars (D3): ink ≥ 12:1 and ink.secondary ≥ 5.9:1 on every rail and tint (IC ≥ 14:1 / ≥ 9:1); gated in bbr-ui palette tests.",
    "accent is for rails, rings, drag ticks and primary fills; accent.ink is accent-coloured text on neutral surfaces; on.accent is text on an accent fill.",
    "Status hues appear only on small glyphs or dots with a non-colour companion; their 10 % surfaces back error and attention panes.",
    "tint.accent is the one accent moment on white and overlay surfaces: command, menu and Library cursors, drop targets.",
    "Increased contrast replaces ink, lines, accent, washes and tint.accent; materials stay.",
])
print(b.finish())
