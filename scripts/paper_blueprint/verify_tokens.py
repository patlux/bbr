"""Compare the Paper token set against the code (palette.rs hex table,
the veil alphas, space_tint.rs rails). Prints mismatches; exit 1 on any."""
import json
import re
import sys

sys.path.insert(0, "/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint")
import paper  # noqa: E402

REPO = "/Users/patwoz/dev/Personal/macos/bbr"
src = open(f"{REPO}/crates/bbr-ui/src/palette.rs").read()
hexes = dict(re.findall(r'([A-Z_]+) = "([0-9A-F]{6})"', src))

tokens = paper.call("get_tokens", {})
items = tokens.get("items", tokens.get("tokens", [])) if isinstance(tokens, dict) else tokens
paper_values = {t["name"]: str(t["value"]).upper() for t in items}

expect = {
    "--color-ink": "INK_LIGHT", "--color-dark-ink": "INK_DARK",
    "--color-ink-secondary": "SECONDARY_LIGHT", "--color-dark-ink-secondary": "SECONDARY_DARK",
    "--color-ink-faint": "FAINT_LIGHT", "--color-dark-ink-faint": "FAINT_DARK",
    "--color-hairline": "HAIRLINE_LIGHT", "--color-dark-hairline": "HAIRLINE_DARK",
    "--color-border-strong": "BORDER_STRONG_LIGHT", "--color-dark-border-strong": "BORDER_STRONG_DARK",
    "--color-ic-ink": "INK_IC_LIGHT", "--color-ic-dark-ink": "INK_IC_DARK",
    "--color-ic-ink-secondary": "SECONDARY_IC_LIGHT", "--color-ic-dark-ink-secondary": "SECONDARY_IC_DARK",
    "--color-ic-ink-faint": "FAINT_IC_LIGHT", "--color-ic-dark-ink-faint": "FAINT_IC_DARK",
    "--color-ic-hairline": "HAIRLINE_IC_LIGHT", "--color-ic-dark-hairline": "HAIRLINE_IC_DARK",
    "--color-accent": "ACCENT_LIGHT", "--color-dark-accent": "ACCENT_DARK",
    "--color-accent-ink": "ACCENT_INK_LIGHT", "--color-dark-accent-ink": "ACCENT_INK_DARK",
    "--color-ic-accent": "ACCENT_IC_LIGHT", "--color-ic-dark-accent": "ACCENT_IC_DARK",
    "--color-status-error": "ERROR_LIGHT", "--color-dark-status-error": "ERROR_DARK",
    "--color-status-attention": "ATTENTION_LIGHT", "--color-dark-status-attention": "ATTENTION_DARK",
    "--color-status-ok": "OK_LIGHT", "--color-dark-status-ok": "OK_DARK",
    "--color-status-info": "INFO_LIGHT", "--color-dark-status-info": "INFO_DARK",
    "--color-tint-accent": "TINT_ACCENT_LIGHT", "--color-dark-tint-accent": "TINT_ACCENT_DARK",
    "--color-ic-tint-accent": "TINT_ACCENT_IC_LIGHT", "--color-ic-dark-tint-accent": "TINT_ACCENT_IC_DARK",
    "--color-canvas": "SURFACE_CANVAS_LIGHT", "--color-dark-canvas": "SURFACE_CANVAS_DARK",
    "--color-sidebar": "SURFACE_SIDEBAR_LIGHT", "--color-dark-sidebar": "SURFACE_SIDEBAR_DARK",
    "--color-field": "SURFACE_FIELD_LIGHT", "--color-dark-field": "SURFACE_FIELD_DARK",
    "--color-overlay": "SURFACE_OVERLAY_LIGHT", "--color-dark-overlay": "SURFACE_OVERLAY_DARK",
    "--color-glass": "SURFACE_GLASS_LIGHT", "--color-dark-glass": "SURFACE_GLASS_DARK",
    "--color-ic-dark-tint-selected": "TINT_SELECTED_IC_DARK",
}
veils = {  # palette.rs alpha / 255 as the Paper percentage
    "--color-favorite": ("255 255 255", 128), "--color-dark-favorite": ("255 255 255", 15),
    "--color-footer": ("255 255 255", 158), "--color-dark-footer": ("255 255 255", 13),
    "--color-wash-hover": ("17 19 21", 13), "--color-wash-active": ("17 19 21", 23),
    "--color-dark-wash-hover": ("255 255 255", 13), "--color-dark-wash-active": ("255 255 255", 20),
    "--color-dark-tint-selected": ("255 255 255", 23),
    "--color-address": ("255 255 255", 140), "--color-dark-address": ("255 255 255", 15),
    "--color-address-hover": ("255 255 255", 102), "--color-dark-address-hover": ("255 255 255", 26),
    "--color-address-border": ("17 19 21", 10), "--color-dark-address-border": ("255 255 255", 10),
    "--color-address-accessory": ("17 19 21", 20), "--color-dark-address-accessory": ("255 255 255", 20),
    # increased contrast (light_increased / dark_increased)
    "--color-ic-wash-hover": ("17 19 21", 20), "--color-ic-wash-active": ("17 19 21", 31),
    "--color-ic-dark-wash-hover": ("255 255 255", 20), "--color-ic-dark-wash-active": ("255 255 255", 31),
    "--color-ic-address-border": ("17 19 21", 26), "--color-ic-dark-address": ("255 255 255", 20),
    "--color-ic-dark-address-hover": ("255 255 255", 31), "--color-ic-dark-address-border": ("255 255 255", 26),
}
bad = []
for name, const in expect.items():
    want = "#" + hexes[const]
    if paper_values.get(name) != want:
        bad.append((name, paper_values.get(name), want))
for name, (rgb, alpha) in veils.items():
    got = paper_values.get(name, "")
    m = re.match(r"RGB\((\d+ \d+ \d+) / ([\d.]+)%\)", got)
    if not m or m.group(1) != rgb or abs(float(m.group(2)) - alpha / 2.55) > 1.0:
        bad.append((name, got, f"rgb({rgb} / {alpha / 2.55:.1f}%)"))
tint = json.load(open("/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/tint.json"))
for key, value in tint["rail"].items():
    mode, token, step = key.split("-")
    name = ("--color-rail" if mode == "Light" else "--color-dark-rail") + f"-{token}-{step}"
    if paper_values.get(name) != value:
        bad.append((name, paper_values.get(name), value))
print(f"checked {len(expect) + len(veils) + len(tint['rail'])} tokens; mismatches: {len(bad)}")
for row in bad:
    print("  ", row)
sys.exit(1 if bad else 0)
