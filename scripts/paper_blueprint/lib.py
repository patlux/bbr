"""BBR blueprint component library: token-only HTML for Paper, parameterised
by theme (light/dark, increased contrast, space tint). Every colour is a
CSS variable of the file's tokens."""
import html as _html
import json
import sys

sys.path.insert(0, "/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint")
import paper  # noqa: E402
from inventory import BOARDS, PAGES  # noqa: E402

TOKENS = set(json.load(open("/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/tokennames.json")))


def esc(s):
    return _html.escape(str(s), quote=True)


class T:
    """A theme: mode light/dark, optional increased contrast, optional
    space tint token + intensity (the rail paint)."""

    def __init__(self, mode="light", ic=False, tint=None, step="standard"):
        self.mode, self.ic, self.tint, self.step = mode, ic, tint, step

    @property
    def dark(self):
        return self.mode == "dark"

    def c(self, name):
        if name == "sidebar" and self.tint:
            name = f"rail-{self.tint}-{self.step}"
        candidates = []
        if self.ic:
            candidates.append(("ic-dark-" if self.dark else "ic-") + name)
        candidates.append(("dark-" if self.dark else "") + name)
        candidates.append(name)
        for cand in candidates:
            if f"--color-{cand}" in TOKENS:
                return f"var(--color-{cand})"
        raise KeyError(name)

    def elev2(self):
        if self.dark:
            return f"0 0 0 1px {self.c('elev-ring')}, 0 16px 32px {self.c('elev-deep')}"
        return (f"0 0 0 1px {self.c('elev-ring')}, 0 2px 4px var(--color-elev-soft), "
                f"0 16px 36px -6px {self.c('elev-deep')}")

    def lift(self):
        if self.dark:
            return f"0 1px 2px {self.c('lift-shadow')}, 0 0 0 1px {self.c('lift-ring')}"
        return f"0 1px 3px {self.c('lift-shadow')}, 0 0 0 1px {self.c('lift-ring')}"

    def card_shadow(self):
        if self.dark:
            return ("0 0 0 1px var(--color-dark-card-ring), 0 1px 2px var(--color-dark-card-shadow-near), "
                    "0 10px 24px var(--color-dark-card-shadow-far)")
        return "0 1px 2px var(--color-card-shadow-near), 0 8px 20px var(--color-card-shadow-far)"

    def label(self):
        name = "Dark" if self.dark else "Light"
        if self.ic:
            name += " · increased contrast"
        if self.tint:
            name += f" · {self.tint.capitalize()} {self.step}"
        return name


LIGHT, DARK = T("light"), T("dark")
TYPE = {  # role: (size token, px line-height, weight, family)
    "display": ("var(--text-display)", 29, 600, "ui"),
    "title": ("var(--text-title)", 17, 500, "ui"),
    "title-xl": ("var(--text-title-xl)", 20, 500, "ui"),
    "label": ("var(--text-label)", 16, 500, "ui"),
    "meta": ("var(--text-meta)", 16, 400, "ui"),
    "mono": ("var(--text-mono)", 16, 400, "mono"),
    "key": ("var(--text-key)", 16, 500, "mono"),
    "badge": ("var(--text-badge)", 13, 600, "ui"),
}


def text(content, role="title", color="var(--color-ink)", weight=None, extra="", name=None,
         grow=False, clamp=True):
    size, lh, w, family = TYPE[role]
    font = "var(--font-mono)" if family == "mono" else "var(--font-ui)"
    style = (f"font-family:{font};font-size:{size};line-height:{lh}px;"
             f"font-weight:{weight or w};color:{color};")
    if role == "badge":
        style += "letter-spacing:0.06em;text-transform:uppercase;"
    if grow:
        style += "flex:1 1 0;min-width:0;"
    if clamp:
        style += "white-space:nowrap;overflow:hidden;text-overflow:ellipsis;"
    layer = f' layer-name="{esc(name)}"' if name else ""
    return f'<div{layer} style="{style}{extra}">{esc(content)}</div>'


# ---------------------------------------------------------------- icons
# 18 × 18 viewBox strokes (the app's catalog, §7), coloured by the caller.
ICONS = {
    "sidebar": '<rect x="1.75" y="3.25" width="14.5" height="11.5" rx="2.5" {s}/><path d="M6.75 3.25V14.75" {s}/>',
    "back": '<path d="M15 9H3.5M3.5 9L8 4.5M3.5 9L8 13.5" {s}/>',
    "forward": '<path d="M3 9H14.5M14.5 9L10 4.5M14.5 9L10 13.5" {s}/>',
    "reload": '<path d="M14.75 9a5.75 5.75 0 1 1-1.9-4.27" {s}/><path d="M14.75 2.75V6.5H11" {s}/>',
    "stop": '<path d="M4.5 4.5L13.5 13.5M13.5 4.5L4.5 13.5" {s}/>',
    "close": '<path d="M5 5L13 13M13 5L5 13" {s}/>',
    "plus": '<path d="M9 3.75v10.5M3.75 9h10.5" {s}/>',
    "minus": '<path d="M3.75 9h10.5" {s}/>',
    "folder": '<path d="M2.25 5.25a1.5 1.5 0 0 1 1.5-1.5h2.7l1.5 1.8h6.3a1.5 1.5 0 0 1 1.5 1.5v6a1.5 1.5 0 0 1-1.5 1.5H3.75a1.5 1.5 0 0 1-1.5-1.5v-7.8Z" {s}/>',
    "folder-open": '<path d="M2.25 13.5v-8.25a1.5 1.5 0 0 1 1.5-1.5h2.7l1.5 1.8h5.55a1.5 1.5 0 0 1 1.5 1.5v1.2" {s}/><path d="M2.25 13.5 4.35 8.7a1.5 1.5 0 0 1 1.38-.9h9.36a.75.75 0 0 1 .69 1.05l-1.86 4.2a1.5 1.5 0 0 1-1.38.9H3.75a1.5 1.5 0 0 1-1.5-.45Z" {s}/>',
    "download": '<path d="M9 2.75v8.5M9 11.25 5.75 8M9 11.25 12.25 8" {s}/><path d="M3.25 13.75h11.5" {s}/>',
    "search": '<circle cx="8" cy="8" r="4.75" {s}/><path d="M11.5 11.5L15 15" {s}/>',
    "globe": '<circle cx="9" cy="9" r="6.25" {s}/><path d="M2.75 9h12.5M9 2.75c1.9 1.7 2.9 3.8 2.9 6.25S10.9 13.55 9 15.25C7.1 13.55 6.1 11.45 6.1 9S7.1 4.45 9 2.75Z" {s}/>',
    "lock": '<rect x="4" y="8" width="10" height="7.25" rx="1.5" {s}/><path d="M6.25 8V6a2.75 2.75 0 0 1 5.5 0v2" {s}/>',
    "lock-open": '<rect x="4" y="8" width="10" height="7.25" rx="1.5" {s}/><path d="M6.25 8V6a2.75 2.75 0 0 1 5.3-1" {s}/>',
    "warn": '<path d="M9 2.75 16 15H2L9 2.75Z" {s}/><path d="M9 7.5v3.5M9 13.1v.1" {s}/>',
    "info": '<circle cx="9" cy="9" r="6.25" {s}/><path d="M9 8.25v4.5M9 5.9v.1" {s}/>',
    "check": '<path d="M4 9.5 7.5 13 14 5.5" {s}/>',
    "chevron-right": '<path d="M7 4.5 11.5 9 7 13.5" {s}/>',
    "chevron-down": '<path d="M4.5 7 9 11.5 13.5 7" {s}/>',
    "more": '<circle cx="4.5" cy="9" r="0.6" {s}/><circle cx="9" cy="9" r="0.6" {s}/><circle cx="13.5" cy="9" r="0.6" {s}/>',
    "pin": '<path d="M7 2.75h4M9 2.75v4.5l3 2.5v1.25H6v-1.25l3-2.5M9 11v4.25" {s}/>',
    "star": '<path d="M9 2.75l1.85 3.95 4.3.5-3.2 2.95.85 4.25L9 12.25 5.2 14.4l.85-4.25-3.2-2.95 4.3-.5L9 2.75Z" {s}/>',
    "bookmark": '<path d="M5 2.75h8v12.5l-4-3-4 3V2.75Z" {s}/>',
    "clock": '<circle cx="9" cy="9" r="6.25" {s}/><path d="M9 5.25V9l2.75 1.75" {s}/>',
    "trash": '<path d="M3.5 5h11M7 5V3.25h4V5M5 5l.75 10h6.5L13 5" {s}/>',
    "split": '<rect x="1.75" y="3.25" width="14.5" height="11.5" rx="2.5" {s}/><path d="M9 3.25V14.75" {s}/>',
    "gear": '<circle cx="9" cy="9" r="2.25" {s}/><path d="M9 2.5v2M9 13.5v2M2.5 9h2M13.5 9h2M4.4 4.4l1.4 1.4M12.2 12.2l1.4 1.4M4.4 13.6l1.4-1.4M12.2 5.8l1.4-1.4" {s}/>',
    "person": '<circle cx="9" cy="6.25" r="3" {s}/><path d="M3.5 15.25c.6-2.9 2.8-4.5 5.5-4.5s4.9 1.6 5.5 4.5" {s}/>',
    "speaker": '<path d="M9 3.5 5.5 6.5H3v5h2.5L9 14.5v-11Z" {s}/><path d="M11.75 6.5a3.5 3.5 0 0 1 0 5M13.75 4.5a6.25 6.25 0 0 1 0 9" {s}/>',
    "mute": '<path d="M9 3.5 5.5 6.5H3v5h2.5L9 14.5v-11Z" {s}/><path d="M12 7l4 4M16 7l-4 4" {s}/>',
    "camera": '<rect x="2" y="5" width="10" height="8" rx="1.75" {s}/><path d="M12 8l4-2.25v6.5L12 10" {s}/>',
    "mic": '<rect x="6.5" y="2.5" width="5" height="8.5" rx="2.5" {s}/><path d="M4 9a5 5 0 0 0 10 0M9 14v1.75" {s}/>',
    "pinmap": '<path d="M9 15.5s-4.75-4.3-4.75-8a4.75 4.75 0 0 1 9.5 0c0 3.7-4.75 8-4.75 8Z" {s}/><circle cx="9" cy="7.5" r="1.75" {s}/>',
    "bell": '<path d="M4.5 12.5V8.25a4.5 4.5 0 0 1 9 0v4.25l1.25 1.5H3.25l1.25-1.5ZM7.5 15.5h3" {s}/>',
    "clipboard": '<rect x="4" y="3.5" width="10" height="12" rx="1.5" {s}/><path d="M7 3.5V2.5h4v1" {s}/>',
    "key": '<circle cx="6" cy="9" r="3" {s}/><path d="M9 9h6.5M13 9v2.5M15.5 9v2" {s}/>',
    "external": '<path d="M10 3h5v5M15 3l-6.5 6.5M13 10.5v3.25a1.25 1.25 0 0 1-1.25 1.25h-7.5A1.25 1.25 0 0 1 3 13.75v-7.5A1.25 1.25 0 0 1 4.25 5H7.5" {s}/>',
    "window": '<rect x="2" y="3.5" width="14" height="11" rx="2" {s}/><path d="M2 6.5h14" {s}/>',
    "play": '<path d="M5.5 3.5 14 9l-8.5 5.5v-11Z" {f}/>',
    "pause": '<path d="M6 4v10M12 4v10" {s}/>',
    "prev": '<path d="M13 4v10L6 9l7-5Z" {f}/><path d="M4.5 4v10" {s}/>',
    "next": '<path d="M5 4v10l7-5-7-5Z" {f}/><path d="M13.5 4v10" {s}/>',
    "sparkle": '<path d="M9 2.5v3M9 12.5v3M2.5 9h3M12.5 9h3M4.4 4.4l2 2M11.6 11.6l2 2M4.4 13.6l2-2M11.6 6.4l2-2" {s}/>',
    "history": '<path d="M3.25 9a5.75 5.75 0 1 0 1.7-4.1" {s}/><path d="M3 2.75v3h3" {s}/><path d="M9 5.75V9l2.25 1.5" {s}/>',
    "shield": '<path d="M9 2.5 14.5 4.5v4c0 3.5-2.4 5.9-5.5 7-3.1-1.1-5.5-3.5-5.5-7v-4L9 2.5Z" {s}/>',
    "eye": '<path d="M1.75 9S4.5 4 9 4s7.25 5 7.25 5-2.75 5-7.25 5S1.75 9 1.75 9Z" {s}/><circle cx="9" cy="9" r="2.25" {s}/>',
    "keyboard": '<rect x="1.75" y="4.5" width="14.5" height="9" rx="1.75" {s}/><path d="M4.5 7.5h.1M7 7.5h.1M9.5 7.5h.1M12 7.5h.1M5.5 10.5h7" {s}/>',
    "import": '<path d="M9 2.75v8M9 10.75 6 7.75M9 10.75l3-3" {s}/><path d="M3 11.5v2.25a1.25 1.25 0 0 0 1.25 1.25h9.5A1.25 1.25 0 0 0 15 13.75V11.5" {s}/>',
    "reveal": '<circle cx="8" cy="8" r="4.75" {s}/><path d="M11.5 11.5L15 15" {s}/>',
    "retry": '<path d="M14.75 9a5.75 5.75 0 1 1-1.9-4.27" {s}/><path d="M14.75 2.75V6.5H11" {s}/>',
    "cert": '<rect x="2.5" y="3" width="13" height="9" rx="1.5" {s}/><path d="M5.5 6.5h7M5.5 9h4" {s}/><circle cx="12" cy="13" r="2" {s}/>',
    "popup": '<rect x="2" y="2.75" width="10" height="8" rx="1.5" {s}/><rect x="6" y="7.25" width="10" height="8" rx="1.5" {s}/>',
    "file": '<path d="M4.5 2.5h6l3 3v10h-9v-13Z" {s}/><path d="M10.5 2.5v3h3" {s}/>',
    "cmd": '<path d="M6.5 6.5h5v5h-5z" {s}/><path d="M6.5 6.5V5a1.5 1.5 0 1 0-1.5 1.5h1.5M11.5 6.5V5A1.5 1.5 0 1 1 13 6.5h-1.5M6.5 11.5V13A1.5 1.5 0 1 1 5 11.5h1.5M11.5 11.5V13a1.5 1.5 0 1 0 1.5-1.5h-1.5" {s}/>',
    "bolt": '<path d="M10 2 4.5 10H9l-1 6 5.5-8H9l1-6Z" {s}/>',
    "moon": '<path d="M14.5 11A6 6 0 0 1 7 3.5a6 6 0 1 0 7.5 7.5Z" {s}/>',
    "lifering": '<circle cx="9" cy="9" r="6.25" {s}/><circle cx="9" cy="9" r="2.75" {s}/><path d="M4.6 4.6l2.45 2.45M13.4 4.6l-2.45 2.45M4.6 13.4l2.45-2.45M13.4 13.4l-2.45-2.45" {s}/>',
    "grid": '<rect x="3" y="3" width="5" height="5" rx="1" {s}/><rect x="10" y="3" width="5" height="5" rx="1" {s}/><rect x="3" y="10" width="5" height="5" rx="1" {s}/><rect x="10" y="10" width="5" height="5" rx="1" {s}/>',
}
# §11.3 identity glyphs (24 viewBox) from the shipped catalog.
GLYPHS = {
    "code": '<path d="M3.84 12 L10.78 7.51 M3.84 12 L10.78 16.49 M20.16 12 L13.22 7.51 M20.16 12 L13.22 16.49 M10.78 16.04 L13.22 7.96" {s}/>',
    "bag": '<rect x="5.2" y="6.57" width="13.6" height="13.6" rx="1.37" {s}/><path d="M9.28 6.57 A2.72 2.72 0 0 1 14.72 6.57 M12 6.83 L12 12.27" {s}/>',
    "moon": '<path d="M8.6 7.1 A6.8 6.8 0 1 1 8.6 16.9 A4.9 4.9 0 0 0 8.6 7.1" {s}/>',
    "compass": '<circle cx="12" cy="12" r="8.16" {s}/><path d="M12 5.48 L13.96 12 L12 18.52 L10.04 12 L12 5.48" {s}/>',
    "leaf": '<path d="M12 6.68 A6.51 6.51 0 0 0 5.49 17.32 M12 6.68 A6.51 6.51 0 0 1 18.51 17.32 M12 6.68 L12 17.32" {s}/>',
    "globe": '<circle cx="12" cy="12" r="8.16" {s}/><path d="M3.84 12 L20.16 12 M12 3.84 A8.16 8.16 0 0 1 12 20.16" {s}/>',
    "home": '<path d="M4.5 11 12 4.5l7.5 6.5M6.5 9.5V19h11V9.5" {s}/>',
    "book": '<path d="M5 5.5h5.5A1.5 1.5 0 0 1 12 7v12a1.5 1.5 0 0 0-1.5-1.5H5v-12ZM19 5.5h-5.5A1.5 1.5 0 0 0 12 7v12a1.5 1.5 0 0 1 1.5-1.5H19v-12Z" {s}/>',
}


def icon(name, color, size=18, width=1.5, filled=None):
    stroke = (f'fill="none" stroke="{color}" stroke-width="{width}" '
              f'stroke-linecap="round" stroke-linejoin="round"')
    fill = f'fill="{filled or color}"'
    inner = ICONS[name].replace("{s}", stroke).replace("{f}", fill)
    return (f'<svg width="{size}" height="{size}" viewBox="0 0 18 18" '
            f'xmlns="http://www.w3.org/2000/svg" style="flex-shrink:0">{inner}</svg>')


def glyph(name, color, size=18):
    stroke = f'fill="none" stroke="{color}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"'
    inner = GLYPHS[name].replace("{s}", stroke)
    return (f'<svg width="{size}" height="{size}" viewBox="0 0 24 24" '
            f'xmlns="http://www.w3.org/2000/svg" style="flex-shrink:0">{inner}</svg>')


def slot(inner, size=18, extra=""):
    return (f'<div style="width:{size}px;height:{size}px;flex-shrink:0;display:flex;'
            f'align-items:center;justify-content:center;{extra}">{inner}</div>')


def icon_button(t, name, color=None, size=28, active=False, extra="", label=None):
    bg = f"background-color:{t.c('wash-active')};" if active else ""
    layer = f' layer-name="{esc(label)}"' if label else ""
    return (f'<div{layer} style="width:{size}px;height:{size}px;flex-shrink:0;display:flex;'
            f'align-items:center;justify-content:center;border-radius:var(--radius-sm);{bg}{extra}">'
            f'{icon(name, color or t.c("ink-secondary"))}</div>')


# ---------------------------------------------------------------- favicons
HUE = {  # site-favicon stand-ins, painted in token hues (no brand colours)
    "cobalt": "profile-cobalt", "violet": "profile-violet", "fuchsia": "profile-fuchsia",
    "amber": "profile-amber", "terracotta": "profile-terracotta", "slate": "profile-slate",
    "pine": "space-pine", "plum": "space-plum", "ink": "space-ink",
}


def favicon(t, fav, size=18):
    """fav: ('mono', 'K', n) monogram slate n; ('site', hue, 'H') a site
    favicon stand-in; ('glyph', name) a chrome glyph; None: empty slot."""
    if fav is None:
        return slot("", size)
    kind = fav[0]
    radius = "var(--radius-sm)" if size >= 16 else "4px"
    font = max(9, round(size * 0.58))
    if kind == "glyph":
        return slot(icon(fav[1], t.c("ink-secondary"), size), size)
    if kind == "mono":
        bg = t.c(f"monogram-{fav[2]}")
        letter = fav[1]
    else:
        bg = t.c(HUE[fav[1]])
        letter = fav[2]
    return (f'<div style="width:{size}px;height:{size}px;flex-shrink:0;display:flex;align-items:center;'
            f'justify-content:center;border-radius:{radius};background-color:{bg};">'
            f'<div style="font-family:var(--font-ui);font-size:{font}px;line-height:{font + 2}px;'
            f'font-weight:600;color:var(--color-on-accent);">{esc(letter)}</div></div>')


# ---------------------------------------------------------------- rail parts
def traffic_lights(t, dim=False):
    """Native window controls (context only, never BBR paint)."""
    colours = ["var(--color-native-close)", "var(--color-native-minimize)", "var(--color-native-zoom)"] if not dim else [t.c("border-strong")] * 3
    dots = "".join(
        f'<div style="width:12px;height:12px;border-radius:9999px;flex-shrink:0;background-color:{c};'
        f'box-shadow:inset 0 0 0 0.5px var(--color-native-edge);"></div>' for c in colours)
    return f'<div layer-name="Traffic lights · native" style="display:flex;gap:8px;align-items:center;padding-right:8px;flex-shrink:0">{dots}</div>'


def titlebar(t, back=True, forward=False, loading=False, lights=True, sidebar_toggle=True, dim=False):
    ink2, faint = t.c("ink-secondary"), t.c("ink-faint")
    parts = [traffic_lights(t, dim) if lights else ""]
    if sidebar_toggle:
        parts.append(icon_button(t, "sidebar", label="bbr-sidebar-toggle"))
    parts.append('<div style="flex:1 1 0"></div>')
    parts.append(icon_button(t, "back", ink2 if back else faint, label="bbr-nav-back"))
    parts.append(icon_button(t, "forward", ink2 if forward else faint, label="bbr-nav-forward"))
    parts.append(icon_button(t, "stop" if loading else "reload", ink2, label="bbr-nav-reload"))
    return (f'<div layer-name="Titlebar" style="height:52px;flex-shrink:0;display:flex;align-items:center;'
            f'padding:0 12px;gap:8px">{"".join(parts)}</div>')


def address(t, domain="x.com", zoom=None, state="rest", progress=None, secure=True, label=None, glyph_name=None):
    """§11.5 address display control. state: rest | hover | pressed | focused."""
    fill = {"rest": t.c("address"), "hover": t.c("address-hover"),
            "pressed": t.c("wash-active"), "focused": t.c("address")}[state]
    lead = "" if secure else slot(icon("warn", t.c("status-attention"), 14), 16)
    if glyph_name:
        lead = slot(icon(glyph_name, t.c("ink-secondary"), 14), 16)
    chip = ""
    if zoom:
        chip = (f'<div layer-name="bbr-address-zoom" style="height:22px;flex-shrink:0;display:flex;align-items:center;'
                f'padding:0 8px;border-radius:var(--radius-sm);background-color:{t.c("address-accessory")}">'
                f'{text(zoom, "key", t.c("ink"))}</div>')
    bar = ""
    if progress is not None:
        bar = (f'<div layer-name="Address progress (D5)" style="position:absolute;left:10px;right:10px;bottom:0;height:2px;">'
               f'<div style="width:{progress}%;height:2px;border-radius:9999px;background-color:{t.c("accent")}"></div></div>')
    return (f'<div layer-name="{esc(label or "bbr-address")}" style="flex-shrink:0;display:flex;padding:0 4px 12px 4px">'
            f'<div style="position:relative;flex:1 1 0;height:36px;display:flex;align-items:center;gap:8px;'
            f'padding:0 6px 0 12px;border-radius:var(--radius-lg);background-color:{fill};'
            f'border:1px solid {t.c("address-border")};{"box-shadow:0 0 0 2px " + t.c("accent") if state == "focused" else ""}">{lead}'
            f'{text(domain, "title", t.c("ink"), grow=True)}{chip}{bar}</div></div>')


def favorite_tile(t, fav, state="rest", w=None):
    extra = ""
    bg = t.c("favorite")
    if state == "active":
        extra = f"box-shadow:0 0 0 1.5px {t.c('accent')};"
    if state == "hover":
        bg = t.c("address-hover")
    if state == "drop":
        bg = t.c("tint-accent")
        extra = f"box-shadow:inset 0 0 0 1.5px {t.c('accent')};"
    if state == "empty":
        return (f'<div style="height:44px;flex:1 1 0;min-width:0;border-radius:var(--radius-md);'
                f'border:1px dashed {t.c("border-strong")};display:flex;align-items:center;justify-content:center">'
                f'{icon("star", t.c("ink-faint"), 14)}</div>')
    return (f'<div style="height:44px;flex:1 1 0;min-width:0;display:flex;align-items:center;justify-content:center;'
            f'border-radius:var(--radius-md);background-color:{bg};{extra}">{favicon(t, fav)}</div>')


def favorites(t, favs, states=None):
    states = states or {}
    rows, row = [], []
    for index, fav in enumerate(favs):
        row.append(favorite_tile(t, fav, states.get(index, "rest")))
        if len(row) == 4:
            rows.append(row)
            row = []
    if row:
        row += ['<div style="flex:1 1 0;min-width:0;height:44px"></div>'] * (4 - len(row))
        rows.append(row)
    body = "".join(f'<div style="display:flex;gap:4px">{"".join(r)}</div>' for r in rows)
    return (f'<div layer-name="bbr-favorites" style="flex-shrink:0;display:flex;flex-direction:column;'
            f'gap:4px;padding:0 8px 12px 8px">{body}</div>')


def heading(t, name="Default", glyph_name="code"):
    return (f'<div layer-name="bbr-space-heading" style="flex-shrink:0;display:flex;align-items:center;gap:10px;'
            f'padding:4px 18px 6px 18px">{glyph(glyph_name, t.c("ink-secondary"))}'
            f'{text(name, "label", t.c("ink-secondary"), weight=600, extra="letter-spacing:0.01em;")}</div>')


def tab_row(t, title, fav=("mono", "A", 1), state="rest", indent=0, trailing=None,
            asleep=False, name=None):
    """§11.1 row. state: rest | hover | pressed | selected | focused |
    dragging | drop-before | disabled."""
    bg, shadow, extra = "transparent", "none", ""
    colour = t.c("ink-secondary") if asleep else t.c("ink")
    if state in ("hover",):
        bg = t.c("wash-hover")
    if state == "pressed":
        bg = t.c("wash-active")
    if state in ("selected", "dragging"):
        bg, shadow = t.c("tint-selected"), t.lift()
        colour = t.c("ink-secondary") if asleep else t.c("ink")
    if state == "dragging":
        shadow = f"0 0 0 1px {t.c('border-strong')}, {t.elev2()}"
        extra = "opacity:0.92;"
    if state == "focused":
        extra = f"box-shadow:inset 0 0 0 2px {t.c('accent')};"
    if state == "disabled":
        extra = "opacity:0.45;"
    close = ""
    if trailing == "close" or (trailing is None and state in ("hover", "selected")):
        close = slot(icon("close", t.c("ink-secondary"), 14), 18)
    elif trailing == "audio":
        close = slot(icon("speaker", t.c("ink-secondary"), 14), 18)
    elif trailing == "attention":
        close = slot(f'<div style="width:8px;height:8px;border-radius:9999px;background-color:{t.c("status-attention")}"></div>', 18)
    elif trailing == "error":
        close = slot(icon("warn", t.c("status-error"), 14), 18)
    elif trailing == "clock":
        close = slot(icon("clock", t.c("ink-secondary"), 14), 18)
    elif trailing and trailing.startswith("key:"):
        close = text(trailing[4:], "key", t.c("ink-faint"), clamp=False)
    else:
        close = slot("", 18)
    if fav and fav[0] == "loading":
        lead = spinner(t)
    elif fav and fav[0] == "lifering":
        lead = slot(icon("lifering", t.c("status-error"), 16), 18)
    else:
        lead = favicon(t, fav)
    style = (f"height:36px;flex-shrink:0;display:flex;align-items:center;gap:10px;"
             f"padding:0 6px 0 {10 + indent}px;border-radius:var(--radius-md);background-color:{bg};")
    if state not in ("focused",):
        style += f"box-shadow:{shadow};"
    return (f'<div layer-name="{esc(name or ("bbr-tab-row · " + state))}" style="{style}{extra}">{lead}'
            f'{text(title, "title", colour, grow=True)}{close}</div>')


def spinner(t, size=18, colour=None):
    colour = colour or t.c("accent")
    return slot(
        f'<svg width="14" height="14" viewBox="0 0 14 14" xmlns="http://www.w3.org/2000/svg">'
        f'<circle cx="7" cy="7" r="5.5" fill="none" stroke="{colour}" opacity="0.22" stroke-width="1.6"/>'
        f'<path d="M7 1.5a5.5 5.5 0 0 1 5.5 5.5" fill="none" stroke="{colour}" stroke-width="1.6" stroke-linecap="round"/></svg>',
        size)


def folder_row(t, name, opened=False, count=None, state="rest"):
    bg = t.c("wash-hover") if state == "hover" else "transparent"
    tail = text(str(count), "key", t.c("ink-faint")) if count is not None and state == "hover" else slot("", 18)
    return (f'<div layer-name="bbr-folder · {esc(name)}" style="height:36px;flex-shrink:0;display:flex;align-items:center;gap:10px;'
            f'padding:0 6px 0 10px;border-radius:var(--radius-md);background-color:{bg}">'
            f'{slot(icon("folder-open" if opened else "folder", t.c("ink-secondary")))}'
            f'{text(name, "title", t.c("ink"), weight=600, grow=True)}{tail}</div>')


def divider(t):
    return (f'<div layer-name="Organized hairline" style="height:17px;flex-shrink:0;display:flex;align-items:center;padding:0 16px">'
            f'<div style="flex:1 1 0;height:1px;background-color:{t.c("hairline")}"></div></div>')


def new_tab_row(t, state="rest"):
    bg = t.c("wash-hover") if state == "hover" else "transparent"
    return (f'<div layer-name="bbr-new-tab" style="height:36px;flex-shrink:0;display:flex;align-items:center;gap:10px;'
            f'padding:0 6px 0 10px;border-radius:var(--radius-md);background-color:{bg}">'
            f'{slot(icon("plus", t.c("ink-secondary")))}{text("New Tab", "title", t.c("ink-secondary"), weight=400, grow=True)}</div>')


def rows_block(t, rows_html, grow=False):
    g = "flex:1 1 0;min-height:0;overflow:hidden;" if grow else "flex-shrink:0;"
    return f'<div style="{g}display:flex;flex-direction:column;gap:4px;padding:0 8px">{"".join(rows_html)}</div>'


def space_chip(t, glyph_name, active=False, tint=None, hover=False):
    bg = t.c("wash-hover") if hover else "transparent"
    shadow = "none"
    if active:
        bg, shadow = t.c("tint-selected"), t.lift()
    colour = t.c("ink") if active else t.c("ink-secondary")
    return (f'<div layer-name="bbr-space-chip" style="width:32px;height:32px;flex-shrink:0;display:flex;align-items:center;justify-content:center;'
            f'border-radius:var(--radius-md);background-color:{bg};box-shadow:{shadow}">{glyph(glyph_name, colour, 16)}</div>')


def footer(t, spaces=("code", "bag"), active=0, downloads_badge=None):
    chips = "".join(space_chip(t, g, i == active) for i, g in enumerate(spaces))
    dl = icon_button(t, "download", label="bbr-footer-downloads")
    if downloads_badge:
        dl = (f'<div style="position:relative;flex-shrink:0">{dl}<div style="position:absolute;right:3px;top:3px;'
              f'width:7px;height:7px;border-radius:9999px;background-color:{t.c("accent")}"></div></div>')
    return (f'<div layer-name="bbr-footer" style="height:44px;flex-shrink:0;display:flex;align-items:center;gap:12px;'
            f'padding:0 8px 8px 8px">{dl}<div layer-name="bbr-space-strip" style="display:flex;gap:4px;align-items:center;flex:1 1 0;min-width:0;overflow:hidden">{chips}</div>'
            f'{icon_button(t, "plus", label="bbr-footer-new-space")}</div>')


# ---------------------------------------------------------------- page placeholder
def page_placeholder(t, title="x.com", lines=6, width=640):
    """Neutral stand-in for native web content (never a site screenshot)."""
    ink = t.c("hairline")
    bars = "".join(
        f'<div style="height:10px;width:{w}%;border-radius:5px;background-color:{ink}"></div>'
        for w in [92, 84, 96, 70, 88, 60][:lines])
    return (f'<div layer-name="Web content · placeholder" style="flex:1 1 0;display:flex;flex-direction:column;gap:24px;'
            f'padding:56px 72px;min-width:0;overflow:hidden;background-color:{t.c("glass")}">'
            f'<div style="display:flex;flex-direction:column;gap:14px;width:100%;max-width:{width}px">'
            f'<div style="height:22px;width:55%;border-radius:6px;background-color:{t.c("border-strong")}"></div>{bars}</div>'
            f'<div style="display:flex;gap:16px;width:100%;max-width:{width}px">'
            + "".join(f'<div style="flex:1 1 0;height:120px;border-radius:10px;background-color:{ink}"></div>' for _ in range(3))
            + '</div></div>')


# ---------------------------------------------------------------- boards
MARK_COLOUR = {"Shipped": "status-ok", "Change": "status-attention",
               "New · proposal": "status-info", "Native OS": "ink-faint"}
DOC = LIGHT  # board chrome (annotations) always uses the light tokens


def mark_chip(mark):
    colour = f"var(--color-{MARK_COLOUR[mark]})"
    return (f'<div layer-name="Mark · {esc(mark)}" style="display:flex;align-items:center;gap:6px;height:24px;padding:0 10px;'
            f'border-radius:9999px;background-color:var(--color-field);box-shadow:0 0 0 1px var(--color-hairline)">'
            f'<div style="width:8px;height:8px;border-radius:9999px;background-color:{colour}"></div>'
            f'{text(mark, "key", "var(--color-ink-secondary)", clamp=False)}</div>')


class Board:
    def __init__(self, code, width=3040, desc="", ids="", extra_meta=""):
        page, title, mark, refs, packet = BOARDS[code]
        self.code, self.page = code, page
        self.name = f"{code} · {title}"
        existing = self.find()
        if existing:
            paper.call("delete_nodes", {"nodeIds": [existing]})
        res = paper.call("create_artboard", {
            "pageId": PAGES[page], "name": self.name,
            "styles": {"width": f"{width}px", "height": "1200px", "display": "flex",
                       "flexDirection": "column", "backgroundColor": "var(--color-canvas)"}})
        self.id = res["id"]
        meta = f"DESIGN {refs} · packet {packet}"
        if ids:
            meta += f" · ids {ids}"
        if extra_meta:
            meta += f" · {extra_meta}"
        self.add(
            f'<div layer-name="Board header" style="display:flex;flex-direction:column;gap:12px;padding:48px 56px 36px 56px">'
            f'<div style="display:flex;align-items:center;gap:12px">{text(code, "key", "var(--color-ink-faint)", clamp=False)}{"" if code.startswith("I-") else mark_chip(mark)}</div>'
            f'<div style="font-family:var(--font-ui);font-size:var(--text-doc-title);line-height:var(--leading-doc-title);font-weight:600;letter-spacing:-0.02em;color:var(--color-ink)">{esc(title)}</div>'
            + (f'<div style="font-family:var(--font-ui);font-size:var(--text-title-xl);line-height:24px;color:var(--color-ink-secondary);max-width:920px">{esc(desc)}</div>' if desc else "")
            + f'{text(meta, "key", "var(--color-ink-faint)", weight=400, clamp=False, extra="max-width:1600px;white-space:normal;")}</div>')

    def find(self):
        info = paper.call("get_basic_info", {"pageId": PAGES[self.page]})
        for board in info.get("artboards", []):
            if board["name"] == self.name:
                return board["id"]
        return None

    def add(self, html, target=None):
        res = paper.call("write_html", {"html": html, "targetNodeId": target or self.id,
                                        "mode": "insert-children"})
        nodes = res.get("createdNodes", []) if isinstance(res, dict) else []
        if not nodes:
            raise RuntimeError(f"write failed: {str(res)[:300]}")
        return nodes[0]["id"]

    def row(self, gap=40, name="Plates", wrap=False, pad="0 56px 56px 56px"):
        w = "flex-wrap:wrap;" if wrap else ""
        return self.add(f'<div layer-name="{esc(name)}" style="display:flex;gap:{gap}px;align-items:flex-start;{w}padding:{pad}"></div>')

    def column(self, target, gap=16, name="Column", width=None, pad="0"):
        w = f"width:{width}px;" if width else ""
        return self.add(f'<div layer-name="{esc(name)}" style="display:flex;flex-direction:column;gap:{gap}px;{w}padding:{pad};flex-shrink:0"></div>', target)

    def caption(self, target, label, note=""):
        n = text(note, "meta", "var(--color-ink-faint)", clamp=False) if note else ""
        return self.add(f'<div style="display:flex;flex-direction:column;gap:2px">{text(label, "badge", "var(--color-ink-secondary)", clamp=False)}{n}</div>', target)

    def notes(self, items, title="Notes", target=None, width=1400):
        rows = "".join(
            f'<div style="display:flex;gap:12px;align-items:flex-start">'
            f'<div style="width:22px;flex-shrink:0;font-family:var(--font-mono);font-size:var(--text-key);line-height:20px;color:var(--color-ink-faint)">{i}</div>'
            f'<div style="font-family:var(--font-ui);font-size:var(--text-label);line-height:20px;color:var(--color-ink-secondary);flex:1 1 0">{esc(item)}</div></div>'
            for i, item in enumerate(items, start=1))
        return self.add(
            f'<div layer-name="{esc(title)}" style="display:flex;flex-direction:column;gap:10px;padding:0 56px 56px 56px;width:{width}px">'
            f'{text(title, "badge", "var(--color-ink-faint)", clamp=False)}{rows}</div>', target)

    def finish(self, shot=True, scale=0.5):
        paper.call("update_styles", {"updates": [{"nodeIds": [self.id], "styles": {"height": "fit-content"}}]})
        if shot:
            path = f"/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/shots/{self.code}.jpg"
            paper.shot(self.id, path, scale)
            return path


def build_rail(b, parent, t, spec, h, rail_w=300, extra_style=""):
    rail = b.add(f'<div layer-name="bbr-sidebar" style="width:{rail_w}px;flex-shrink:0;display:flex;flex-direction:column;height:{h}px;{extra_style}">'
                 f'{titlebar(t, **spec.get("titlebar", {})) if spec.get("titlebar", {}) is not None else ""}</div>', parent)
    if spec.get("address", {}) is not None:
        b.add(address(t, **spec.get("address", {})), rail)
    if spec.get("favorites"):
        b.add(favorites(t, spec["favorites"], spec.get("fav_states")), rail)
    for extra in spec.get("above_heading", []):
        b.add(extra, rail)
    if spec.get("heading", ("Default", "code")):
        b.add(heading(t, *spec.get("heading", ("Default", "code"))), rail)
    body = b.add('<div layer-name="bbr-scroll" style="position:relative;flex:1 1 0;min-height:0;display:flex;flex-direction:column;gap:4px;padding-bottom:8px;overflow:hidden"></div>', rail)
    for group in spec.get("groups", []):
        if group == "divider":
            b.add(divider(t), body)
        elif group == "new":
            b.add(f'<div style="flex-shrink:0;padding:0 8px;display:flex;flex-direction:column">{new_tab_row(t, spec.get("new_state", "rest"))}</div>', body)
        elif isinstance(group, str):
            b.add(group, body)
        else:
            b.add(rows_block(t, group), body)
    for extra in spec.get("above_footer", []):
        b.add(extra, rail)
    if spec.get("media"):
        b.add(spec["media"], rail)
    if spec.get("footer", {}) is not None:
        b.add(footer(t, **spec.get("footer", {})), rail)
    return rail


def window(b, target, t, spec, w=1440, h=900, name=None):
    """Write a whole browser window into `target`, group by group."""
    chassis = t.c("sidebar")
    win = b.add(
        f'<div layer-name="{esc(name or ("Window · " + t.label()))}" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0;display:flex;'
        f'border-radius:{spec.get("win_radius", 12)}px;overflow:hidden;background-color:{chassis};box-shadow:0 0 0 1px {spec.get("ring", DOC.c("hairline"))}, 0 24px 48px -12px var(--color-elev-deep);{spec.get("win_style", "")}"></div>',
        target)
    rail_w = spec.get("rail_width", 300)
    if spec.get("sidebar", True):
        build_rail(b, win, t, spec, h, rail_w)
    area_pad = spec.get("card_pad", "10px 10px 10px 0" if spec.get("sidebar", True) else "52px 10px 10px 10px")
    if not spec.get("sidebar", True) and spec.get("band", True):
        b.add(f'<div layer-name="bbr-chassis-band" style="position:absolute;left:0;top:0;right:0;height:52px;display:flex;align-items:center;padding:0 12px;gap:8px">'
              f'{traffic_lights(t)}</div>', win)
    area = b.add(f'<div layer-name="Card area" style="flex:1 1 0;min-width:0;display:flex;padding:{area_pad}"></div>', win)
    radius = spec.get("card_radius", 10)
    card = b.add(f'<div layer-name="bbr-content-card" style="position:relative;flex:1 1 0;min-width:0;display:flex;flex-direction:column;border-radius:{radius}px;'
                 f'overflow:hidden;background-color:{t.c(spec.get("card_bg", "glass"))};box-shadow:{t.card_shadow() if radius else "none"}"></div>', area)
    content = spec.get("card", "placeholder")
    if content == "placeholder":
        b.add(page_placeholder(t), card)
    elif content:
        for part in (content if isinstance(content, list) else [content]):
            b.add(part, card)
    for overlay in spec.get("card_overlays", []):
        b.add(overlay, card)
    for overlay in spec.get("overlays", []):
        b.add(overlay, win)
    return win


def rail_plate(b, target, t, spec, h=900, name=None, w=300):
    """A rail alone (no card) on its chassis, for sidebar boards."""
    frame = b.add(f'<div layer-name="{esc(name or ("Rail · " + t.label()))}" style="position:relative;width:{w}px;height:{h}px;flex-shrink:0;display:flex;'
                  f'border-radius:12px;overflow:hidden;background-color:{spec.get("bg", t.c("sidebar"))};box-shadow:0 0 0 1px {DOC.c("hairline")}"></div>', target)
    if spec.get("bg_layer"):
        colour, opacity = spec["bg_layer"]
        b.add(f'<div layer-name="Tint cross-fade layer" style="position:absolute;inset:0;background-color:{colour};opacity:{opacity}"></div>', frame)
    rail = build_rail(b, frame, t, spec, h, w)
    for overlay in spec.get("overlays", []):
        b.add(overlay, frame)
    return frame


def plate(b, target, t, inner_html=None, w=None, h=None, pad=24, name=None, bg="sidebar"):
    """A context plate: a slice of chassis/rail paint in theme `t`."""
    size = (f"width:{w}px;" if w else "") + (f"height:{h}px;" if h else "")
    pid = b.add(f'<div layer-name="{esc(name or ("Plate · " + t.label()))}" style="position:relative;{size}flex-shrink:0;display:flex;flex-direction:column;gap:16px;'
                f'padding:{pad}px;border-radius:12px;background-color:{t.c(bg)};box-shadow:0 0 0 1px {DOC.c("hairline")}"></div>', target)
    if inner_html:
        for part in (inner_html if isinstance(inner_html, list) else [inner_html]):
            b.add(part, pid)
    return pid
