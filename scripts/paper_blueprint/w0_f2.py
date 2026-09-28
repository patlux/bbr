import json, random, sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from lib import *

TOKENS9 = ["cobalt", "violet", "fuchsia", "amber", "terracotta", "slate", "pine", "plum", "ink"]
IDENT = {k: ("profile-" + k if k in ("cobalt", "violet", "fuchsia", "amber", "terracotta", "slate") else "space-" + k) for k in TOKENS9}


def rail_swatch(t, token, step):
    return (f'<div style="width:178px;height:52px;flex-shrink:0;display:flex;flex-direction:column;justify-content:center;gap:2px;padding:0 12px;'
            f'border-radius:10px;background-color:{t.c("rail-" + token + "-" + step)};box-shadow:0 0 0 1px {t.c("hairline")}">'
            f'{text("Title", "title", t.c("ink"))}{text("secondary", "meta", t.c("ink-secondary"))}</div>')


def grain_svg(t, w, h, seed=7):
    rnd = random.Random(seed)
    dots = "".join(f'<circle cx="{rnd.uniform(0, w):.1f}" cy="{rnd.uniform(0, h):.1f}" r="0.6" fill="{t.c("grain")}"/>' for _ in range(int(w * h / 90)))
    return f'<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg" style="position:absolute;left:0;top:0">{dots}</svg>'


b = Board("F-2", width=1560, desc="The nine curated space tokens (§11.3). Each intensity step blends the token's chrome wash into the rail and pins the result to the untinted rail's luminance, so every text bar holds by construction; the Ink token reads as the untinted rail. Identity hues paint chips, favicon stand-ins and the configuration tiles.")
row = b.row(gap=24)
for t in (LIGHT, DARK):
    col = b.column(row, name=t.label(), width=720)
    b.caption(col, t.label(), "identity hue · subtle · standard · rich")
    p = plate(b, col, t, w=720, pad=20)
    b.add('<div style="display:flex;gap:12px"><div style="width:96px;flex-shrink:0"></div>'
          + "".join(f'<div style="width:178px">{text(s, "badge", t.c("ink-faint"))}</div>' for s in ("Subtle", "Standard", "Rich")) + '</div>', p)
    for token in TOKENS9:
        b.add(f'<div style="display:flex;align-items:center;gap:12px">'
              f'<div style="width:96px;flex-shrink:0;display:flex;align-items:center;gap:8px">'
              f'<div style="width:16px;height:16px;border-radius:5px;background-color:{t.c(IDENT[token])}"></div>'
              f'{text(token.capitalize(), "label", t.c("ink"))}</div>'
              + "".join(rail_swatch(t, token, s) for s in ("subtle", "standard", "rich")) + '</div>', p)
b.notes([
    "Shares of the chrome wash: light 18 / 36 / 54 %, dark 12 / 24 / 36 %, increased contrast +4 %; then pinned to the base rail luminance (space_tint.rs).",
    "The whole curated appearance (theme, identity, intensity, grain) persists per space (schema v4).",
    "The active chip is tint.selected over the tinted rail carrying 8 % of the identity hue, with elev:lift.",
])
row2 = b.row(gap=24, name="Grain & monograms")
for t in (LIGHT, DARK):
    col = b.column(row2, name="Grain · " + t.label(), width=720)
    b.caption(col, "Static grain · " + t.label(), "one generated tile; white ≤ 28 % light, black ≤ 36 % dark; about half the sheet clear")
    g = b.add(f'<div style="position:relative;width:720px;height:120px;border-radius:12px;overflow:hidden;background-color:{t.c("rail-cobalt-standard")};box-shadow:0 0 0 1px {DOC.c("hairline")}">{grain_svg(t, 720, 120)}'
              f'<div style="position:absolute;left:20px;top:20px;display:flex;flex-direction:column;gap:4px">{text("Grain on · Cobalt standard", "title", t.c("ink"))}{text("specks only raise text contrast", "meta", t.c("ink-secondary"))}</div></div>', col)
    b.caption(col, "Monogram slates · " + t.label(), "§7.2 missing-favicon tiles: six slates by host hash, white letter")
    tiles = "".join(favicon(t, ("mono", "ABCDEF"[i - 1], i), 28) for i in range(1, 7))
    plate(b, col, t, f'<div style="display:flex;gap:12px">{tiles}</div>', w=720, pad=20)
print(b.finish())
