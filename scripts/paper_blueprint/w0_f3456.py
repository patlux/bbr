import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from lib import *

which = sys.argv[1:]


def spec_table(t, headers, rows, widths):
    head = "".join(f'<div style="width:{w}px;flex-shrink:0">{text(h, "badge", t.c("ink-faint"), clamp=False)}</div>' for h, w in zip(headers, widths))
    out = [f'<div style="display:flex;gap:16px;padding:0 0 10px 0;border-bottom:1px solid {t.c("hairline")}">{head}</div>']
    for r in rows:
        cells = "".join(f'<div style="width:{w}px;flex-shrink:0;display:flex;align-items:center">{c}</div>' for c, w in zip(r, widths))
        out.append(f'<div style="display:flex;gap:16px;align-items:center;min-height:40px;border-bottom:1px solid {t.c("hairline")}">{cells}</div>')
    return f'<div style="display:flex;flex-direction:column">{"".join(out)}</div>'


def mono(s, t=LIGHT, color=None):
    return text(s, "key", color or t.c("ink-secondary"), weight=400, clamp=False)


if "F-3" in which:
    b = Board("F-3", width=1560, desc="System UI (SF Pro) for every role, SF Mono for URLs, keys and ids (JetBrains Mono stands in here). Row titles stay 500 in every state; the lift carries selection. Sizes step with the density the user picks; row heights never change within a session.")
    row = b.row(gap=40)
    col = b.column(row, name="Type ramp", width=860)
    b.caption(col, "Type ramp · normal density")
    ramp = [
        ("type:display", "No tabs in this space", "display", "24 / 600 · 1.2", "— / 24 / 26"),
        ("type:title", "Rust SIMD on the GPU — a field guide", "title", "14 / 500 · 1.2", "12 / 14 / 16"),
        ("type:label", "New Tab  ·  Reload  ·  Done", "label", "13 / 500 · 1.2", "12 / 13 / 14"),
        ("type:meta", "3 of 12 · 2.4 MB of 18 MB · 2 min left", "meta", "12 / 400 · tnum", "11 / 12 / 13"),
        ("type:mono", "github.com/zed-industries/zed", "mono", "13 / 400 · SF Mono", "12 / 13 / 14"),
        ("type:key", "⌘T  ⌘L  ⇧⌘T", "key", "12 / 500 · SF Mono", "11 / 12 / 13"),
        ("type:badge", "Pinned", "badge", "11 / 600 · +0.06 em caps", "11 / 11 / 12"),
    ]
    rows = [[mono(r[0]), text(r[1], r[2], LIGHT.c("ink"), clamp=False), mono(r[3]), mono(r[4])] for r in ramp]
    b.add(spec_table(LIGHT, ["Role", "Sample", "Size / weight", "S / N / XL"], rows, [120, 380, 190, 110]), col)
    col2 = b.column(row, name="Density", width=560)
    b.caption(col2, "Density · S / N / XL", "§8 tokens; the user picks; row heights stay stable")
    dens = [["Row height", "32", "36", "44"], ["Row gap", "4", "4", "4"], ["Row pitch", "36", "40", "48"],
            ["Row tile", "28", "36", "44"], ["Favorite tile", "40×36", "64×44", "88×52"],
            ["Tile gap", "4", "4", "8"], ["Rail width", "232", "300", "340"],
            ["⌘T palette", "508", "600", "716"], ["⌘L overlay min", "440", "440", "440"],
            ["Hit minimum", "28", "32", "40"], ["Title size", "12", "14", "16"]]
    b.add(spec_table(LIGHT, ["Token", "S", "N", "XL"], [[text(r[0], "label", LIGHT.c("ink"))] + [mono(x) for x in r[1:]] for r in dens], [200, 90, 90, 90]), col2)
    row2 = b.row(gap=24, name="Rows by density")
    for t in (LIGHT, DARK):
        c = b.column(row2, name="Rows · " + t.label(), width=700)
        b.caption(c, "Rows at S · N · XL · " + t.label())
        p = plate(b, c, t, w=700, pad=20)
        for label, h, size, tile in (("S", 32, 12, 16), ("N", 36, 14, 18), ("XL", 44, 16, 22)):
            b.add(f'<div style="display:flex;align-items:center;gap:16px"><div style="width:28px">{mono(label, t)}</div>'
                  f'<div style="flex:1 1 0;height:{h}px;display:flex;align-items:center;gap:10px;padding:0 6px 0 10px;border-radius:var(--radius-{"sm" if label == "S" else ("md" if label == "N" else "lg")});background-color:{t.c("tint-selected")};box-shadow:{t.lift()}">'
                  f'{favicon(t, ("mono", "R", 3), tile)}<div style="font-family:var(--font-ui);font-size:{size}px;line-height:{size + 4}px;font-weight:500;color:{t.c("ink")}">Rust SIMD on the GPU — a field guide</div></div></div>', p)
    b.notes(["GPUI text has no tracking control, so the documented +0.06 em (badge) and +0.01 em (space heading) are not rendered in the app.",
             "Selected titles drop from 600 to 500 in v2; the active-space heading is 13 / 600 in ink.secondary; the ⌘T query is 17 pt.",
             "Truncation: titles ellipsize at the end, URLs in the middle of the path, never the host (§4)."])
    print(b.finish())

if "F-4" in which:
    b = Board("F-4", width=1560, desc="The 4 pt grid, the four radii, and the elevation ladder from the chassis up to floating surfaces. The rail has no border; the content card's own edge and seat shadow are the only separation.")
    row = b.row(gap=40)
    col = b.column(row, name="Spacing", width=440)
    b.caption(col, "Spacing · sp(N) = N × 4 pt")
    sp = "".join(f'<div style="display:flex;align-items:center;gap:12px;height:28px"><div style="width:110px">{mono(f"--spacing-{i}")}</div>'
                 f'<div style="width:{v}px;height:12px;border-radius:3px;background-color:var(--color-accent)"></div>{mono(f"{v}")}</div>'
                 for i, v in enumerate([4, 8, 12, 16, 20, 24, 32, 40], start=1))
    b.add(f'<div style="display:flex;flex-direction:column;gap:4px">{sp}<div style="display:flex;align-items:center;gap:12px;height:28px"><div style="width:110px">{mono("content-inset")}</div><div style="width:10px;height:12px;border-radius:3px;background-color:var(--color-status-info)"></div>{mono("10 · card inset = card radius")}</div></div>', col)
    col = b.column(row, name="Radius", width=920)
    b.caption(col, "Radius · v2", "chrome stays within 6–12; 16 only for the ⌘T sheet")
    rad = "".join(f'<div style="display:flex;flex-direction:column;gap:8px;align-items:flex-start"><div style="width:120px;height:72px;border-radius:var(--radius-{k});background-color:var(--color-field);box-shadow:0 0 0 1px var(--color-border-strong)"></div>{mono(f"radius:{k} · {v}")}{text(u, "meta", "var(--color-ink-faint)", clamp=False)}</div>'
                  for k, v, u in (("sm", 6, "S rows, small tiles, cursors"), ("md", 10, "N rows, tiles, buttons, menus"), ("lg", 12, "address, fields, panels"), ("xl", 16, "⌘T palette sheet")))
    b.add(f'<div style="display:flex;gap:28px">{rad}</div>', col)
    row2 = b.row(gap=24, name="Elevation")
    for t in (LIGHT, DARK):
        c = b.column(row2, name="Elevation · " + t.label(), width=700)
        b.caption(c, "Elevation · " + t.label())
        p = plate(b, c, t, w=700, pad=28)
        def box(label, style, bg):
            return (f'<div style="display:flex;flex-direction:column;gap:10px;align-items:flex-start"><div style="width:108px;height:80px;border-radius:10px;background-color:{bg};{style}"></div>'
                    f'{text(label, "key", t.c("ink-secondary"), weight=400, clamp=False)}</div>')
        b.add('<div style="display:flex;gap:18px">'
              + box("elev:0 · rail", f"box-shadow:inset 0 0 0 1px {t.c('border-strong')}", t.c("sidebar"))
              + box("elev:1 · field", f"box-shadow:0 0 0 1px {t.c('hairline')}", t.c("field"))
              + box("elev:lift · row", f"box-shadow:{t.lift()}", t.c("tint-selected"))
              + box("elev:1 · card", f"box-shadow:{t.card_shadow()}", t.c("glass"))
              + box("elev:2 · floating", f"box-shadow:{t.elev2()}", t.c("overlay")) + '</div>', p)
    b.add(spec_table(LIGHT, ["Layer", "Surface", "Rule"], [
        [mono("L0"), text("chassis", "label", "var(--color-ink)"), text("the rail's own (tinted) paint; nothing sits in the inset gap", "meta", "var(--color-ink-secondary)", clamp=False)],
        [mono("L0.5"), text("content card", "label", "var(--color-ink)"), text("inset 10 top/trailing/bottom, flush with the rail; canvas only before attach", "meta", "var(--color-ink-secondary)", clamp=False)],
        [mono("L1"), text("rail, panels", "label", "var(--color-ink)"), text("matte paint, no vibrancy, no border", "meta", "var(--color-ink-secondary)", clamp=False)],
        [mono("L2"), text("rows, fields, buttons", "label", "var(--color-ink)"), text("one lift per list: the selected row and the active chip", "meta", "var(--color-ink-secondary)", clamp=False)],
        [mono("L3"), text("palette, menus, sheets, Peek", "label", "var(--color-ink)"), text("elev:2, 1 pt edge; above web content through the native Portal", "meta", "var(--color-ink-secondary)", clamp=False)],
    ], [80, 260, 1000]), b.add('<div layer-name="Layers" style="display:flex;flex-direction:column;padding:0 56px 56px 56px"></div>'))
    print(b.finish())

if "F-5" in which:
    b = Board("F-5", width=1560, desc="One stroke family: 1.5 pt on an 18 pt box, round caps and joins, ink.secondary at rest. The identity glyphs (2 pt on 24) mark spaces. Favicons are site content; the blueprint paints them as hue tiles so every colour stays a token.")
    row = b.row(gap=24)
    for t in (LIGHT, DARK):
        c = b.column(row, name="Icons · " + t.label(), width=700)
        b.caption(c, "Chrome icons · " + t.label())
        p = plate(b, c, t, w=700, pad=20)
        cells = "".join(f'<div style="width:100px;display:flex;flex-direction:column;align-items:center;gap:8px;padding:10px 0">{icon(n, t.c("ink-secondary"), 18)}{text(n, "meta", t.c("ink-faint"), clamp=False)}</div>' for n in ICONS)
        b.add(f'<div style="display:flex;flex-wrap:wrap;gap:4px">{cells}</div>', p)
        b.caption(c, "Identity glyphs · " + t.label())
        cells = "".join(f'<div style="width:100px;display:flex;flex-direction:column;align-items:center;gap:8px;padding:10px 0">{glyph(n, t.c("ink"), 22)}{text(n, "meta", t.c("ink-faint"), clamp=False)}</div>' for n in GLYPHS)
        plate(b, c, t, f'<div style="display:flex;flex-wrap:wrap;gap:4px">{cells}</div>', w=700, pad=20)
    b.notes(["Stroke glyphs never use Unicode stand-ins (§7); the bookmark mark is a drawn glyph, not ★.",
             "Glyph colour follows the state: ink.secondary at rest, ink on hover/selection, ink.faint disabled, accent only for focus rings and drag ticks.",
             "The app icon (Phosphor-b) and the favicon pipeline are owned by v2-brand-icon and real-favicon-pipeline."])
    print(b.finish())

if "F-6" in which:
    b = Board("F-6", width=1560, desc="Commit first, delight after: motion decorates a state that is already presented, never delays it, and has a Reduce Motion variant. One-shots stay within 260 ms; only loading and progress loop; an idle shell requests no frames.")
    row = b.row(gap=40)
    col = b.column(row, name="Durations", width=640)
    b.caption(col, "Durations")
    durs = [("dur:instant", 0), ("dur:micro", 70), ("dur:fine", 120), ("dur:standard", 180), ("dur:emphasis", 260), ("load:indeterminate", 580), ("loading arc turn", 640), ("illustration cycle", 1400)]
    bars = "".join(f'<div style="display:flex;align-items:center;gap:12px;height:30px"><div style="width:170px">{mono(n)}</div>'
                   f'<div style="width:{max(2, v * 0.25):.0f}px;height:10px;border-radius:5px;background-color:var(--color-accent)"></div>{text(f"{v} ms", "key", LIGHT.c("ink-secondary"), weight=400)}</div>' for n, v in durs)
    b.add(f'<div style="display:flex;flex-direction:column;gap:4px">{bars}</div>', col)
    col = b.column(row, name="Easing", width=720)
    b.caption(col, "Easing curves")

    def curve(name, p1, p2):
        x1, y1 = p1; x2, y2 = p2
        path = f"M0 100 C{x1 * 100:.0f} {100 - y1 * 100:.0f} {x2 * 100:.0f} {100 - y2 * 100:.0f} 100 0"
        return (f'<div style="display:flex;flex-direction:column;gap:8px;align-items:flex-start">'
                f'<div style="width:140px;height:140px;padding:20px;border-radius:10px;background-color:var(--color-field);box-shadow:0 0 0 1px var(--color-hairline)">'
                f'<svg width="100" height="100" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="M0 100 L100 0" stroke="var(--color-hairline)" fill="none"/>'
                f'<path d="{path}" stroke="var(--color-accent)" stroke-width="2" fill="none"/></svg></div>'
                f'{mono(name)}{text(f"cubic({x1}, {y1}, {x2}, {y2})", "meta", "var(--color-ink-faint)", clamp=False)}</div>')
    b.add('<div style="display:flex;gap:24px">' + curve("ease:standard", (0.22, 1), (0.36, 1)) + curve("ease:accelerate", (0.36, 0), (1, 1))
          + curve("ease:decelerate", (0, 0), (0.3, 1)) + curve("linear", (0, 0), (1, 1)) + '</div>', col)
    b.notes(["Principles (§6): commit first; motion never blocks input; one moment per kind at a time (a newer commit supersedes); Reduce Motion always has a static or opacity-only variant.",
             "Moments and their exact frames are specified on page 40 (V-5, V-6); the Reduce Motion variants on V-4.",
             "The motion snapshot (`motion.*` in the testkit) proves idle: no animation element and no frame request when nothing is transient."])
    print(b.finish())
