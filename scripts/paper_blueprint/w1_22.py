import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from shell import *
from w1_21 import frag, frag_rows, rb

which = sys.argv[1:]


def config_panel(t, name="Research", theme="cobalt", step=1, grain=False, glyph_sel="book", title="Edit Space"):
    swatches = "".join(
        f'<div layer-name="bbr-space-config-theme-{k}" style="width:22px;height:22px;border-radius:9999px;display:flex;align-items:center;justify-content:center;'
        + (f'box-shadow:0 0 0 2px {t.c("overlay")}, 0 0 0 4px {t.c("accent")};' if k == theme else "")
        + f'background-color:{t.c(v)}"></div>'
        for k, v in (("cobalt", "profile-cobalt"), ("violet", "profile-violet"), ("fuchsia", "profile-fuchsia"), ("amber", "profile-amber"), ("terracotta", "profile-terracotta"),
                     ("slate", "profile-slate"), ("pine", "space-pine"), ("plum", "space-plum"), ("ink", "space-ink")))
    steps = "".join(
        f'<div style="flex:1 1 0;display:flex;flex-direction:column;gap:4px;align-items:center"><div layer-name="bbr-space-config-intensity-{s}" style="width:100%;height:36px;border-radius:8px;background-color:{t.c("rail-" + theme + "-" + s)};'
        + (f'box-shadow:0 0 0 2px {t.c("accent")}' if i == step else f'box-shadow:0 0 0 1px {t.c("hairline")}') + f'"></div>{text(s.capitalize(), "meta", t.c("ink") if i == step else t.c("ink-secondary"), clamp=False)}</div>'
        for i, s in enumerate(("subtle", "standard", "rich")))
    glyphs = "".join(
        f'<div style="width:28px;height:28px;border-radius:8px;display:flex;align-items:center;justify-content:center;'
        + (f'background-color:{t.c("tint-accent")};box-shadow:inset 0 0 0 1.5px {t.c("accent")}' if g == glyph_sel else "") + f'">{glyph(g, t.c("ink"), 16)}</div>' for g in GLYPHS)
    from blocks import button, switch
    field = (f'<div layer-name="bbr-space-config-name" style="height:32px;display:flex;align-items:center;gap:4px;padding:0 10px;border-radius:var(--radius-lg);background-color:{t.c("field")};box-shadow:0 0 0 1px {t.c("accent")}, 0 0 0 3px {t.c("tint-accent")}">'
             f'{text(name, "label", t.c("ink"), weight=400, clamp=False)}<div style="width:1.5px;height:16px;background-color:{t.c("accent")}"></div></div>')
    sec = lambda label: text(label, "badge", t.c("ink-faint"), clamp=False)
    return (f'<div layer-name="bbr-space-config" style="margin:0 8px 8px 8px;display:flex;flex-direction:column;gap:12px;padding:14px;border-radius:var(--radius-lg);'
            f'background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
            f'{text(title, "title", t.c("ink"), weight=600, clamp=False)}{field}'
            f'{sec("Colour")}<div style="display:flex;gap:5px">{swatches}</div>'
            f'{sec("Paper")}<div style="display:flex;gap:8px">{steps}</div>{switch(t, grain, "Grain")}'
            f'{sec("Symbol")}<div style="display:flex;gap:3px">{glyphs}</div>'
            f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(t, "Cancel", "secondary")}{button(t, "Done", "primary")}</div></div>')


if "P-1" in which:
    b = Board("P-1", width=3040, desc="One switch authority for chips, ⌃1–⌃9, the command bar and a horizontal swipe over the rail. A committed switch paints the destination list and tint on the commit frame and cross-fades only the chrome tint (dur:standard). A swipe interpolates the tint with the finger and commits or cancels; the web view never cross-fades.",
              ids="bbr-space-strip, bbr-space-chip-<n>, space-tint-fade moment")
    for t in (LIGHT, DARK):
        tw = T(t.mode, tint="cobalt")
        src = lambda **k: std(t, hover=None, footer={"spaces": ("code", "bag", "moon"), "active": 0}, **k)
        dst = lambda **k: dict(std(tw, hover=None, footer={"spaces": ("code", "bag", "moon"), "active": 1}, heading=("Work", "bag"), address={"domain": "linear.app"}), groups=work_rows(tw), **k)
        mix = lambda p: (tw.c("sidebar"), p / 100)
        b.caption(b.column(b.row(gap=0, pad="0 56px 12px 56px"), name="c"), t.label())
        row = b.row(gap=28)
        frames = [("Source · Default", src(), ""), ("Swipe 40 %", src(bg_layer=mix(40)), "tint follows the finger; list stays"),
                  ("Swipe released → commit", dst(), "already at the destination; no fade"),
                  ("Click · commit frame", dst(bg=t.c("sidebar")), "destination list, source tint"),
                  ("Click · 90 ms", dst(bg=t.c("sidebar"), bg_layer=mix(50)), "chrome tint cross-fades"), ("Click · 180 ms", dst(), "rest")]
        for label, spec, note in frames:
            col = b.column(row, name=label)
            b.caption(col, label, note or " ")
            rail_plate(b, col, t if ("Source" in label or "Swipe 40" in label) else tw, spec, h=720)
    b.notes(["D4: the list snaps on commit; only the chrome tint fades. Reduce Motion switches instantly.",
             "A swipe below the commit threshold returns to the source tint; the list never moved.",
             "The selected tab of each space stays warm; the switch keeps the 50 / 100 ms budgets."])
    print(b.finish())

if "P-2" in which:
    b = Board("P-2", width=2200, desc="An in-rail panel above the footer edits the space as a draft: name, curated colour, paper (three intensity previews at the draft colour, grain) and symbol. Nothing applies until Done; Cancel or Escape leaves the committed space untouched.",
              ids="bbr-space-config, -title, -name, -theme-<name>, -intensity-<step>, -grain, -glyph-<name>, -done, -cancel")
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name=t.label())
        b.caption(col, "Editing Research · " + t.label())
        rail_plate(b, col, t, std(t, hover=None, heading=("Research", "book"), address={"domain": "doc.rust-lang.org"}, above_footer=[config_panel(t)]), h=900)
        col = b.column(row, name="Draft · " + t.label())
        b.caption(col, "Draft Pine · rich · grain · " + t.label(), "the rail keeps the committed paper while editing")
        rail_plate(b, col, t, std(t, hover=None, heading=("Research", "book"), address={"domain": "doc.rust-lang.org"}, above_footer=[config_panel(t, theme="pine", step=2, grain=True, glyph_sel="leaf")]), h=900)
    b.notes(["Only curated values exist: nine colours, three steps, grain on/off, the curated glyphs and emoji — no free colour or strength.",
             "Keyboard: typing edits the name; Tab walks the pickers; Enter commits; Escape cancels.",
             "The panel binds the exact space it opened for; a switch while it is open cannot edit another space."])
    print(b.finish())

if "P-3" in which:
    b = Board("P-3", width=3040, desc="New Space (plus menu or ⇧⌘N) creates and activates the space first — usable at once, empty — then opens the configuration panel over it. Cancel keeps the new space with its derived defaults.", ids="bbr-plus-new-space, bbr-space-config")
    def spec(t):
        return std(t, heading=("Space 4", "compass"), favorites=None, address={"domain": "New Tab", "glyph_name": "plus"},
                   groups=["new"], footer={"spaces": ("code", "bag", "moon", "compass"), "active": 3},
                   above_footer=[config_panel(t, name="Space 4", theme="slate", step=1, glyph_sel="compass", title="New Space")], card=empty_pane(t, "Default", "Space 4"))
    pair(b, spec)
    b.notes(["The new space inherits the active space's profile; choosing another profile is part of the New Space sheet in Settings (L-4, proposal).",
             "Colour and symbol default deterministically from the space order, so an unconfigured space is still distinguishable."])
    print(b.finish())

if "P-4" in which:
    b = Board("P-4", width=3040, desc="Secondary click on a space chip opens the space menu; deleting asks once, names what goes, and offers no undo beyond Reopen Closed Tab. The last space of a profile cannot be deleted.", ids="bbr-space-menu, bbr-space-delete-sheet")
    from blocks import menu, menu_row, menu_sep, sheet, button
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row, name="Menu · " + t.label())
        b.caption(col, "Space menu · " + t.label())
        m = menu(t, [menu_row(t, "Edit Space…", icon_name="gear"), menu_row(t, "New Tab in Space", icon_name="plus"), menu_row(t, "Move Left", icon_name="chevron-right"),
                     menu_row(t, "Move Right", "cursor", icon_name="chevron-right"), menu_sep(t), menu_row(t, "Delete Space…", icon_name="trash", danger=True)], 220)
        window(b, col, t, std(t, hover=None, footer={"spaces": ("code", "bag", "moon"), "active": 0}),
               overlays=None or [], w=1440, h=900) if False else None
        spec = std(t, hover=None, footer={"spaces": ("code", "bag", "moon"), "active": 0}, overlays=[f'<div style="position:absolute;left:92px;top:640px">{m}</div>'])
        window(b, col, t, spec)
    row2 = b.row(gap=40)
    for t in (LIGHT, DARK):
        col = b.column(row2, name="Delete · " + t.label())
        b.caption(col, "Delete confirmation · " + t.label())
        body = (f'<div style="font-family:var(--font-ui);font-size:var(--text-label);line-height:20px;color:{t.c("ink-secondary")}">Its 14 tabs and 2 folders close. Favorites, history and site data stay in the profile.</div>'
                f'<div style="display:flex;justify-content:flex-end;gap:8px">{button(t, "Cancel", "secondary")}{button(t, "Delete Space", "danger")}</div>')
        sh = sheet(t, body, 400, "Delete “Research”?")
        veil = f'<div layer-name="Veil" style="position:absolute;inset:0;background-color:{t.c("wash-active")}"></div>'
        spec = std(t, hover=None, heading=("Research", "book"), footer={"spaces": ("code", "bag", "book"), "active": 2},
                   card_overlays=[veil, f'<div style="position:absolute;left:0;right:0;top:0;display:flex;justify-content:center">{sh}</div>'])
        window(b, col, t, spec)
    b.notes(["Mark Change: the chip menu and the delete sheet are proposals over the shipped space model (delta D-2).",
             "The sheet attaches to the card's top edge and dims the card with a 9 % wash (blocking sheets only).",
             "Delete is the danger button (text + colour); Return triggers Cancel, not Delete."])
    print(b.finish())

if "P-5" in which:
    b = Board("P-5", width=2200, desc="A profile is an isolated data store; its identity is a hue tile with a monogram — never colour alone, never animated. It appears wherever a tab or space crosses contexts: the space chip's name, the command field hint, drag targets and cross-profile results.",
              ids="bbr-profile-chip-<n>, bbr-command-profile-hint")
    from blocks import result_row, mode_pill
    def items(t):
        chips = "".join(f'<div style="display:flex;flex-direction:column;gap:6px;align-items:center;width:64px">{profile_chip(t, l, h)}{text(h, "meta", t.c("ink-faint"), clamp=False)}</div>'
                        for l, h in (("D", "cobalt"), ("W", "violet"), ("P", "fuchsia"), ("C", "amber"), ("T", "terracotta"), ("S", "slate")))
        hint = (f'<div style="margin:0 8px;height:48px;display:flex;align-items:center;gap:10px;padding:0 12px;border-radius:var(--radius-lg);background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}">'
                f'{icon("search", t.c("ink-faint"), 16)}{text("in Profile · Work", "label", t.c("ink-secondary"), clamp=False)}{profile_chip(t, "W", "violet", 20)}<div style="flex:1 1 0"></div>{mode_pill(t, "OPEN")}</div>')
        res = f'<div style="margin:0 8px;padding:6px;border-radius:var(--radius-lg);background-color:{t.c("overlay")}">{result_row(t, "Linear — Sprint 42", None, ("site", "violet", "L"), "Tab", True, subtitle="Work · Profile Work")}</div>'
        tip = (f'<div style="display:flex;flex-direction:column;gap:8px;padding:0 12px">{footer(t, spaces=("code", "bag", "moon"))}'
               f'<div style="align-self:center;padding:4px 8px;border-radius:6px;background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}">{text("Work — Profile Work", "meta", t.c("ink"), clamp=False)}</div></div>')
        return [frag(t, f'<div style="display:flex;gap:2px;padding:8px 6px">{chips}</div>', "Profile chips", "six hues, round-robin, stable", w=420),
                frag(t, hint, "Command hint", "after a cross-profile switch, one interaction", w=420),
                frag(t, res, "Cross-profile result", "space and profile named", w=420),
                frag(t, tip, "Chip accessible name", "space + profile", w=300)]
    frag_rows(b, items)
    b.notes(["At most one profile identity is loud at a time: the active space's.", "Profile hues are never used for status, accent or favicons."])
    print(b.finish())

if "P-6" in which:
    b = Board("P-6", width=3040, desc="Profiles are created from Settings or the command bar: a name, a hue, a monogram and the first space. The sheet says plainly what a profile separates — cookies, logins, history and site data.")
    from blocks import sheet, button, field
    def body(t):
        hues = "".join(f'<div style="width:28px;height:28px;border-radius:8px;display:flex;align-items:center;justify-content:center;background-color:{t.c("profile-" + h)};'
                       + (f'box-shadow:0 0 0 2px {t.c("overlay")}, 0 0 0 4px {t.c("accent")}' if h == "terracotta" else "") + f'">{text("C", "badge", "var(--color-identity-letter)", clamp=False)}</div>'
                       for h in ("cobalt", "violet", "fuchsia", "amber", "terracotta", "slate"))
        return (f'<div style="display:flex;align-items:center;gap:14px">{profile_chip(t, "C", "terracotta", 44)}<div style="display:flex;flex-direction:column;gap:2px">{text("Client work", "title-xl", t.c("ink"), weight=600)}{text("Separate cookies, logins, history and site data", "meta", t.c("ink-secondary"), clamp=False)}</div></div>'
                f'{text("Name", "label", t.c("ink-secondary"), clamp=False)}{field(t, "Client work", state="focused", lead=None, width=380)}'
                f'{text("Colour", "label", t.c("ink-secondary"), clamp=False)}<div style="display:flex;gap:10px">{hues}</div>'
                f'{text("First space", "label", t.c("ink-secondary"), clamp=False)}{field(t, "Client work", lead=None, width=380)}'
                f'<div style="display:flex;justify-content:flex-end;gap:8px;padding-top:4px">{button(t, "Cancel", "secondary")}{button(t, "Create Profile", "primary")}</div>')
    def spec(t):
        veil = f'<div style="position:absolute;inset:0;background-color:{t.c("wash-active")}"></div>'
        return std(t, hover=None, card_overlays=[veil, f'<div style="position:absolute;left:0;right:0;top:0;display:flex;justify-content:center">{sheet(t, body(t), 420, "New Profile")}</div>'])
    pair(b, spec)
    b.notes(["A new profile gets the next hue round-robin; the user may pick another; the monogram is the name's first letter(s).",
             "Creating a profile also creates and activates its first space, empty, with the profile's hue as colour.",
             "Editing and deleting profiles live in Settings › Profiles (L-4); deleting removes the data store after confirmation."])
    print(b.finish())

if "P-7" in which:
    b = Board("P-7", width=3040, desc="A tab moved to a space of another profile cannot keep its session: the drop recreates the page in the destination data store. The drag names the destination profile before the drop; afterwards a toast says the page reloaded in that profile.",
              ids="bbr-space-chip-<n>, bbr-profile-chip-<n>, bbr-toast")
    from blocks import button
    row = b.row(gap=40)
    for t in (LIGHT, DARK):
        tw = T(t.mode, tint="cobalt")
        col = b.column(row, name="Drag · " + t.label())
        b.caption(col, "Over the Work chip · " + t.label(), "destination profile named")
        target = footer(t, spaces=("code", "bag", "moon")).replace(space_chip(t, "bag"), space_chip(t, "bag").replace("background-color:transparent", "background-color:" + t.c("tint-accent")).replace("box-shadow:none", "box-shadow:inset 0 0 0 1.5px " + t.c("accent")))
        label = (f'<div layer-name="Destination profile" style="position:absolute;left:52px;top:808px;height:28px;padding:0 10px 0 4px;display:flex;align-items:center;gap:6px;border-radius:8px;background-color:{t.c("overlay")};box-shadow:0 0 0 1px {t.c("hairline")}, {t.elev2()}">'
                 f'{profile_chip(t, "W", "violet", 20)}{text("Move to Work · Profile Work", "meta", t.c("ink"), clamp=False)}</div>')
        proxy = (f'<div layer-name="bbr-tab-drag-proxy" style="position:absolute;left:20px;top:760px;width:220px;height:36px;display:flex;align-items:center;gap:10px;padding:0 10px;border-radius:var(--radius-md);opacity:0.92;'
                 f'background-color:{t.c("tint-selected")};box-shadow:0 0 0 1px {t.c("border-strong")}, {t.elev2()}">{favicon(t, ("mono", "V", 3))}{text("Rust SIMD on the GPU", "title", t.c("ink"))}</div>')
        regs = regular(t, hover=None)
        regs[1] = tab_row(t, "Rust SIMD on the GPU – a field guide", ("mono", "V", 3), "dragging", trailing="none")
        spec = dict(std(t, hover=None, footer=None, above_footer=[target]), overlays=[proxy, label])
        spec["groups"][3] = regs
        rail_plate(b, col, t, spec, h=900)
        col = b.column(row, name="After · " + t.label())
        b.caption(col, "After the drop · " + t.label(), "Work space, page recreated in the Work profile")
        toast = (f'<div style="position:absolute;left:0;right:0;bottom:24px;display:flex;justify-content:center"><div style="display:flex;align-items:center;gap:10px;height:40px;padding:0 8px 0 12px;border-radius:var(--radius-lg);background-color:{tw.c("overlay")};box-shadow:0 0 0 1px {tw.c("hairline")}, {tw.elev2()}">'
                 f'{profile_chip(tw, "W", "violet", 20)}{text("Reloaded in Profile · Work — sign-ins may differ", "label", tw.c("ink"), clamp=False)}{button(tw, "Undo", "quiet", size="s")}</div></div>')
        rows = work_rows(tw)
        rows[3] = rows[3] + [tab_row(tw, "Rust SIMD on the GPU", ("loading",), "selected", asleep=True, trailing="none")]
        rows[0][0] = tab_row(tw, "Linear — Sprint 42", ("site", "violet", "L"))
        spec = dict(std(tw, hover=None, heading=("Work", "bag"), address={"domain": "simd.rs", "progress": 30}, footer={"spaces": ("code", "bag", "moon"), "active": 1}), groups=rows,
                    card=page_placeholder(tw, lines=2), card_overlays=[toast])
        window(b, col, tw, spec, 1100, 900)
    b.notes(["Same-profile moves keep the live session; only cross-profile moves recreate it (PRODUCT: profile isolation).",
             "Undo moves the tab back and recreates it in the source profile. Keyboard: ⇧⌘M opens 'Move to Space' in the command bar.",
             "Mark Change: the destination-profile label and the toast are proposals; the shipped move recreates silently (delta D-2)."])
    print(b.finish())
