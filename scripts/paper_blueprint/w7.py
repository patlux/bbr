import sys
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
from blocks import *
from deltas import DELTAS, EXISTING, DECISIONS
from inventory import BOARDS

which = sys.argv[1:]

if "D-1" in which:
    b = Board("D-1", width=1800, desc="Questions the blueprint raises that only Patrick can answer. Each is settled by adopting, changing or rejecting the linked delta in DESIGN §19; until then main stays as shipped.")
    rows = [f'<div style="display:flex;gap:16px;padding-bottom:10px;border-bottom:1px solid var(--color-hairline)">' + "".join(f'<div style="width:{w}px;flex-shrink:0">{text(h, "badge", "var(--color-ink-faint)", clamp=False)}</div>' for h, w in (("#", 40), ("Topic", 280), ("Question", 1040), ("Delta", 100))) + '</div>']
    for i, (topic, q, ref) in enumerate(DECISIONS, start=1):
        rows.append(f'<div style="display:flex;gap:16px;align-items:center;min-height:48px;border-bottom:1px solid var(--color-hairline)"><div style="width:40px">{text(str(i), "key", "var(--color-ink-faint)", clamp=False)}</div>'
                    f'<div style="width:280px">{text(topic, "label", "var(--color-ink)", clamp=False)}</div><div style="width:1040px">{text(q, "label", "var(--color-ink-secondary)", weight=400, clamp=False)}</div>'
                    f'<div style="width:100px;display:flex">{badge(LIGHT, ref, "attention" if ref != "—" else "neutral")}</div></div>')
    b.add(f'<div layer-name="Decisions" style="display:flex;flex-direction:column;padding:0 56px 56px 56px">{"".join(rows)}</div>')
    b.notes(["Canonical owner of open decisions stays DESIGN §17 (item 18 points here) and §19.", "Answer in the delta ledger (D-2) or by comment on this board; the next writer updates §19 and the packet."])
    print(b.finish())

if "D-2" in which:
    b = Board("D-2", width=2400, desc="Every difference between the boards and main, with the boards that show it, what main does today, the target, and the packet that will close it. Status is 'proposed' for all until adopted in DESIGN §19.")
    rows = [f'<div style="display:flex;gap:16px;padding-bottom:10px;border-bottom:1px solid var(--color-hairline)">' + "".join(f'<div style="width:{w}px;flex-shrink:0">{text(h, "badge", "var(--color-ink-faint)", clamp=False)}</div>' for h, w in (("Delta", 60), ("Title", 300), ("Boards", 200), ("Main today", 620), ("Target", 620), ("Packet", 340))) + '</div>']
    for did, slug, title, boards, today, target, phase, refs, suite, perf in DELTAS:
        pk = slug or EXISTING[did]
        rows.append(f'<div style="display:flex;gap:16px;align-items:flex-start;padding:12px 0;border-bottom:1px solid var(--color-hairline)"><div style="width:60px">{text(did, "key", "var(--color-ink)", clamp=False)}</div>'
                    f'<div style="width:300px">{text(title, "label", "var(--color-ink)", clamp=False, extra="white-space:normal;")}</div><div style="width:200px">{text(boards, "key", "var(--color-accent-ink)", weight=400, clamp=False, extra="white-space:normal;")}</div>'
                    f'<div style="width:620px;font-family:var(--font-ui);font-size:var(--text-meta);line-height:18px;color:var(--color-ink-secondary)">{esc(today)}</div>'
                    f'<div style="width:620px;font-family:var(--font-ui);font-size:var(--text-meta);line-height:18px;color:var(--color-ink)">{esc(target)}</div>'
                    f'<div style="width:340px">{text(pk, "key", "var(--color-ink-faint)", weight=400, clamp=False, extra="white-space:normal;")}</div></div>')
    b.add(f'<div layer-name="Delta ledger" style="display:flex;flex-direction:column;padding:0 56px 56px 56px">{"".join(rows)}</div>')
    b.notes(["Packets live in docs/roadmap/tasks/blueprint-*.md; D23 and D24 target the existing split-app-composition and sidebar-media-control packets.",
             "Shipped boards that match main carry no delta. Token drift found in W0 was fixed in the Paper file (verify_tokens.py)."])
    print(b.finish())
