import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from blocks import *
from coverage import ROWS
from inventory import BOARDS, PAGES
import check_coverage

PAGE_TITLES = {"00": "Index & coverage", "10": "Foundations", "11": "Building blocks", "20": "Window & shell", "21": "Sidebar",
               "22": "Spaces & profiles", "23": "Command", "24": "Content & glass", "25": "Menus & panels",
               "26": "System & lifecycle", "30": "Processes", "40": "Variants & motion", "90": "Decisions & delta"}
WAVES = [("W0", "Setup: index, foundations, building blocks", "00 10 11"), ("W1", "Window, sidebar, spaces & profiles", "20 21 22"),
         ("W2", "Command, content & glass", "23 24"), ("W3", "Menus & panels", "25"), ("W4", "System, lifecycle, split view", "26 24"),
         ("W5", "Process storyboards", "30"), ("W6", "Variants & motion", "40"), ("W7", "Decisions, delta, DESIGN.md, packets", "90")]


def i1(wave_status):
    b = Board("I-1", width=1560, desc="The complete BBR browser as a target design: every surface, state and process, in light and dark. Drawn from the shipped code and DESIGN.md, extended where the product has no surface yet. DESIGN.md stays the contract; a board becomes authority only when its decision lands there with a task packet.",
              extra_meta="repo docs/roadmap/tasks/paper-browser-blueprint.md")
    row = b.row(gap=48)
    col = b.column(row, name="How to read", width=640)
    b.caption(col, "How to read a board")
    items = [("Name", "CODE · Title — the code maps to the coverage matrix and the task packet (e.g. S-2 · Favorites grid)."),
             ("Mark", "the chip under the code says how true the board is against main (below)."),
             ("Plates", "every surface is drawn in light and dark; increased contrast, tints, density and narrow windows appear in page 40 or where a surface changes shape."),
             ("Meta line", "DESIGN sections · owning packet · E2E assertion ids."),
             ("Stand-ins", "System Sans-Serif renders SF Pro; JetBrains Mono stands in for SF Mono. Site favicons are token hue tiles, web pages are neutral placeholders, so every colour is a token."),
             ("Native OS", "traffic lights, file panels, WebKit element menus, system prompts and the menu bar are drawn as labelled context only.")]
    b.add('<div style="display:flex;flex-direction:column;gap:14px">' + "".join(
        f'<div style="display:flex;gap:16px"><div style="width:96px;flex-shrink:0">{text(k, "label", "var(--color-ink)", weight=600, clamp=False)}</div>'
        f'<div style="flex:1 1 0;font-family:var(--font-ui);font-size:var(--text-label);line-height:20px;color:var(--color-ink-secondary)">{esc(v)}</div></div>' for k, v in items) + '</div>', col)
    b.caption(col, "Marks")
    marks = [("Shipped", "on main and E2E-proven; the board is the target and may refine it"), ("Change", "on main, but the board changes it; the delta ledger (page 90) owns the gap"),
             ("New · proposal", "no surface in the product yet; proposed here, decided in page 90"), ("Native OS", "owned by macOS or WebKit; shown for context, never repainted")]
    b.add('<div style="display:flex;flex-direction:column;gap:10px">' + "".join(
        f'<div style="display:flex;gap:16px;align-items:center"><div style="width:150px;flex-shrink:0;display:flex">{mark_chip(m)}</div>'
        f'<div style="flex:1 1 0;font-family:var(--font-ui);font-size:var(--text-label);line-height:20px;color:var(--color-ink-secondary)">{esc(d)}</div></div>' for m, d in marks) + '</div>', col)
    col2 = b.column(row, name="Waves", width=760)
    b.caption(col2, "Waves", "each wave is recorded in the packet ledger and committed")
    b.add('<div style="display:flex;flex-direction:column">' + "".join(
        f'<div style="display:flex;gap:16px;align-items:center;min-height:36px;border-bottom:1px solid var(--color-hairline)">'
        f'<div style="width:36px">{text(w, "key", "var(--color-ink-faint)")}</div><div style="flex:1 1 0">{text(d, "label", "var(--color-ink)", weight=400)}</div>'
        f'<div style="width:120px">{text("pages " + p, "key", "var(--color-ink-faint)", weight=400)}</div>'
        f'<div style="width:90px;display:flex">{badge(LIGHT, wave_status.get(w, "planned"), "accent" if wave_status.get(w) == "done" else ("attention" if wave_status.get(w) == "drawing" else "neutral"))}</div></div>'
        for w, d, p in WAVES) + '</div>', col2)
    b.caption(b.column(b.row(gap=0, name="Map header", pad="0 56px 16px 56px"), name="h"), "Page map")
    grid = b.row(gap=24, wrap=True, name="Page map")
    for page, title in PAGE_TITLES.items():
        codes = [c for c, v in BOARDS.items() if v[0] == page]
        lines = "".join(f'<div style="display:flex;gap:8px"><div style="width:44px;flex-shrink:0">{text(c, "key", "var(--color-ink-faint)", weight=400)}</div>'
                        f'{text(BOARDS[c][1], "meta", "var(--color-ink-secondary)", grow=True)}</div>' for c in codes)
        b.add(f'<div style="width:344px;display:flex;flex-direction:column;gap:6px;padding-top:12px;border-top:1px solid var(--color-hairline)">'
              f'{text(page + " — " + title, "label", "var(--color-ink)", weight=600)}{lines}</div>', grid)
    return b.finish()


def i2():
    found = check_coverage.paper_boards()
    b = Board("I-2", width=1560, desc="Every DESIGN component and state, every UI-bearing roadmap item, every daily-driver capability and every E2E suite maps to at least one board. Generated from the blueprint inventory; 'drawn' is read from this file. check_coverage.py fails while any row is still planned.")
    groups = {}
    for src, item, codes in ROWS:
        key = src.split(" ")[0] if not src.startswith("E2E") else "E2E"
        groups.setdefault(key, []).append((src, item, codes))
    total = len(ROWS)
    drawn = sum(all(c in found for c in codes) for _, _, codes in ROWS)
    b.caption(b.column(b.row(gap=0, name="Summary", pad="0 56px 24px 56px"), name="s"), f"{drawn} of {total} rows drawn")
    for key, rows in groups.items():
        col = b.column(b.row(gap=0, name=key, pad="0 56px 32px 56px"), name=key, width=1448)
        b.caption(col, {"DESIGN": "DESIGN.md", "Phase": "Roadmap phases", "Inventory": "Daily-driver inventory", "E2E": "E2E suites"}.get(key, key))
        html = ""
        for src, item, codes in rows:
            ok = all(c in found for c in codes)
            chips = "".join(f'<div style="height:20px;display:flex;align-items:center;padding:0 6px;border-radius:5px;background-color:{"var(--color-tint-accent)" if c in found else "var(--color-wash-hover)"}">{text(c, "key", "var(--color-accent-ink)" if c in found else "var(--color-ink-faint)", weight=500)}</div>' for c in codes)
            html += (f'<div style="display:flex;gap:16px;align-items:center;min-height:32px;border-bottom:1px solid var(--color-hairline)">'
                     f'<div style="width:200px;flex-shrink:0">{text(src, "key", "var(--color-ink-faint)", weight=400)}</div>'
                     f'<div style="width:560px;flex-shrink:0">{text(item, "label", "var(--color-ink)", weight=400)}</div>'
                     f'<div style="flex:1 1 0;display:flex;gap:4px;flex-wrap:wrap">{chips}</div>'
                     f'<div style="width:80px;display:flex;justify-content:flex-end">{badge(LIGHT, "drawn" if ok else "planned", "accent" if ok else "neutral")}</div></div>')
        b.add(f'<div style="display:flex;flex-direction:column">{html}</div>', col)
    return b.finish()


if __name__ == "__main__":
    status = dict(arg.split("=") for arg in sys.argv[1:] if "=" in arg)
    if "i1" in sys.argv:
        print(i1(status))
    if "i2" in sys.argv:
        print(i2())
