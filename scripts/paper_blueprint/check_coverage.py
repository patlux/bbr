"""Verify the blueprint: every planned artboard exists in Paper (by its
code prefix), every coverage row maps to existing artboards. With
--render, rewrite the packet's coverage section."""
import sys

sys.path.insert(0, "/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint")
import paper  # noqa: E402
from coverage import ROWS  # noqa: E402
from inventory import BOARDS, PAGES, board_name  # noqa: E402

PACKET = "/Users/patwoz/dev/Personal/macos/bbr/docs/roadmap/tasks/paper-browser-blueprint.md"


def paper_boards():
    found = {}
    for page, page_id in PAGES.items():
        info = paper.call("get_basic_info", {"pageId": page_id})
        for board in info.get("artboards", []):
            code = board["name"].split(" · ")[0]
            found.setdefault(code, []).append((page, board["name"]))
    return found


def main():
    found = paper_boards()
    missing = [code for code in BOARDS if code not in found]
    wrong_page = [code for code in BOARDS if code in found
                  and found[code][0][0] != BOARDS[code][0]]
    rows_missing = [(src, item, [c for c in codes if c not in found])
                    for src, item, codes in ROWS if any(c not in found for c in codes)]
    print(f"boards planned {len(BOARDS)}, drawn {len(BOARDS) - len(missing)}")
    print(f"coverage rows {len(ROWS)}, fully drawn {len(ROWS) - len(rows_missing)}")
    if wrong_page:
        print("wrong page:", wrong_page)
    if "--verbose" in sys.argv:
        print("missing boards:", missing)
    if "--render" in sys.argv:
        render(found)
    return 0 if not missing and not rows_missing and not wrong_page else 1


def render(found):
    lines = ["| Source | Item | Artboards | Drawn |", "|---|---|---|---|"]
    for src, item, codes in ROWS:
        drawn = all(c in found for c in codes)
        names = "; ".join(board_name(c) for c in codes)
        lines.append(f"| {src} | {item} | {names} | {'yes' if drawn else 'planned'} |")
    lines += ["", "Board inventory (page · code · title · mark · DESIGN · packet):", "",
              "| Page | Board | Mark | DESIGN | Packet | Drawn |", "|---|---|---|---|---|---|"]
    for code, (page, title, mark, refs, packet) in BOARDS.items():
        link = packet if packet == "paper-browser-blueprint" else f"[{packet}]({packet}.md)"
        lines.append(f"| {page} | {code} · {title} | {mark} | {refs} | {link} | "
                     f"{'yes' if code in found else 'planned'} |")
    text = open(PACKET).read()
    start, end = "<!-- coverage:start -->", "<!-- coverage:end -->"
    head, rest = text.split(start)
    _, tail = rest.split(end)
    open(PACKET, "w").write(head + start + "\n" + "\n".join(lines) + "\n" + end + tail)
    print("rendered coverage into the packet")


if __name__ == "__main__":
    sys.exit(main())
