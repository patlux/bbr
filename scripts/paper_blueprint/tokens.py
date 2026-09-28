"""Sync the Paper token set with the shipped code values (palette.rs,
space_tint.rs, theme.rs, elevation.rs)."""
import json
import sys

sys.path.insert(0, "/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint")
import paper  # noqa: E402

tint = json.load(open("/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/tint.json"))

updates = [
    {"name": "--font-ui", "value": "System Sans-Serif",
     "description": "SF Pro on macOS (DESIGN §4 system UI)"},
    {"name": "--color-dark-footer", "value": "rgb(255 255 255 / 5%)"},
    {"name": "--color-address-border", "value": "rgb(17 19 21 / 4%)"},
    {"name": "--color-address-accessory", "value": "rgb(17 19 21 / 8%)"},
    {"name": "--color-dark-address-accessory", "value": "rgb(255 255 255 / 8%)"},
    {"name": "--color-lift-shadow", "value": "rgb(17 19 21 / 10%)"},
    {"name": "--container-palette-n", "value": "600px"},
]

creates = []


def color(name, value, description=None):
    entry = {"type": "color", "name": name, "value": value}
    if description:
        entry["description"] = description
    creates.append(entry)


color("--color-address-hover", "rgb(255 255 255 / 40%)")
color("--color-dark-address-hover", "rgb(255 255 255 / 10%)")
color("--color-status-error-surface", "rgb(194 46 36 / 10%)")
color("--color-status-attention-surface", "rgb(143 92 0 / 10%)")
color("--color-dark-status-error-surface", "rgb(255 106 99 / 10%)")
color("--color-dark-status-attention-surface", "rgb(255 198 63 / 10%)")
color("--color-ic-tint-accent", "#D9EAE5")
color("--color-ic-dark-tint-accent", "#2B4A48")
color("--color-grain", "rgb(255 255 255 / 28%)", "static grain speck, light chrome")
color("--color-dark-grain", "rgb(0 0 0 / 36%)", "static grain speck, dark chrome")
# Elevation colours (§18.2): elev:2 and elev:lift.
color("--color-elev-ring", "rgb(17 19 21 / 7%)")
color("--color-elev-soft", "rgb(17 19 21 / 5%)")
color("--color-elev-deep", "rgb(17 19 21 / 18%)")
color("--color-lift-ring", "rgb(17 19 21 / 4%)")
color("--color-dark-elev-ring", "rgb(255 255 255 / 8%)")
color("--color-dark-elev-deep", "rgb(0 0 0 / 45%)")
color("--color-dark-lift-shadow", "rgb(0 0 0 / 35%)")
color("--color-dark-lift-ring", "rgb(255 255 255 / 4%)")
# Monogram slates (§7.2), light/dark.
for index, (light, dark) in enumerate([
    ("#616A75", "#54606E"), ("#6B7582", "#5B6775"), ("#5A636E", "#4E5A68"),
    ("#707A87", "#606B77"), ("#666D77", "#59606A"), ("#535C67", "#4A5662"),
], start=1):
    color(f"--color-monogram-{index}", light)
    color(f"--color-dark-monogram-{index}", dark)
# Curated space identity hues not covered by the profile hues.
for token in ("pine", "plum", "ink"):
    color(f"--color-space-{token}", "#" + tint["ident"]["Light"][token])
    color(f"--color-dark-space-{token}", "#" + tint["ident"]["Dark"][token])
# Luminance-pinned rail tints (§11.3, S12): token × step × mode.
for key, value in tint["rail"].items():
    mode, token, step = key.split("-")
    prefix = "--color-rail" if mode == "Light" else "--color-dark-rail"
    color(f"{prefix}-{token}-{step}", value)
creates += [
    {"type": "container", "name": "--container-palette-s", "value": "508px"},
    {"type": "container", "name": "--container-palette-xl", "value": "716px"},
    {"type": "container", "name": "--container-overlay-min", "value": "440px"},
]

if __name__ == "__main__":
    print(json.dumps(paper.call("set_tokens", {"tokens": updates}))[:1500])
    result = paper.call("create_tokens", {"tokens": creates})
    errors = [r for r in result if isinstance(r, dict) and r.get("result") != "created"] \
        if isinstance(result, list) else result
    print(len(creates), "created; errors:", json.dumps(errors)[:1500])
