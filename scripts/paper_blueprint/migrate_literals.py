"""Replace literal colours on the blueprint pages with tokens."""
import sys, json
sys.path.insert(0, '/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint')
import paper
from inventory import PAGES

RULES = [
    ("backgroundColor", "#FF5F57", {"backgroundColor": "var(--color-native-close)"}),
    ("backgroundColor", "#FEBC2E", {"backgroundColor": "var(--color-native-minimize)"}),
    ("backgroundColor", "#28C840", {"backgroundColor": "var(--color-native-zoom)"}),
    ("boxShadow", "rgb(0 0 0 / 12%)", {"boxShadow": "inset 0 0 0 0.5px var(--color-native-edge)"}),
    ("boxShadow", "rgb(0 0 0 / 20%)", {"boxShadow": "0 1px 2px var(--color-knob-shadow)"}),
    ("boxShadow", "rgb(0 0 0 / 25%)", {"boxShadow": "0 1px 3px var(--color-icon-shadow)"}),
    ("color", "#FFFFFF", {"color": "var(--color-identity-letter)"}),
]
total = {}
for page, pid in PAGES.items():
    for style, value, repl in RULES:
        r = paper.call("find_nodes", {"pageId": pid, "filters": [{"styleName": style, "styleValue": value}]})
        ids = [n["id"] for n in r.get("nodes", []) if any(not str(m.get("styleValue", "")).startswith("var(") for m in n["matched"])]
        if ids:
            for i in range(0, len(ids), 200):
                paper.call("update_styles", {"updates": [{"nodeIds": ids[i:i + 200], "styles": repl}]})
            total[(style, value)] = total.get((style, value), 0) + len(ids)
print(total)
# verify nothing literal remains
left = 0
for page, pid in PAGES.items():
    for style, value, _ in RULES:
        r = paper.call("find_nodes", {"pageId": pid, "filters": [{"styleName": style, "styleValue": value}]})
        left += sum(1 for n in r.get("nodes", []) if any(not str(m.get("styleValue", "")).startswith("var(") for m in n["matched"]))
print("literal matches left:", left)
