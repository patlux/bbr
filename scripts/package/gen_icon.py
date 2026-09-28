"""Deterministic BBR app icon (.icns) — "Phosphor b", Python stdlib only.

DESIGN §18.1 D8 concept C: a lowercase b on a dark continuous-corner tile
whose bowl is the accent loading arc (the same 270°-class spinner the tab rows
turn while loading), with a soft Phosphor halo. The vector master is
`packaging/icon/bbr-icon.svg`; this module renders the same geometry with
pure float math — analytic signed distances give anti-aliased edges, no
randomness, no timestamps, fixed zlib level — so identical input always
yields byte-identical output.

Geometry (1024-unit design space, y grows downward, the macOS icon grid):

  tile     superellipse (n = 5) filling 100..924 — the continuous corner
  fill     vertical gradient #1C2127 → #0B0D10, Phosphor halo at the bowl
  stem     rounded bar x 353..411, y 259..751, radius 29, #ECEEF1
  bowl     ring centred (555, 591), radius 159, stroke 58: a faint white
           track plus the accent arc (62 % of the turn, starting 40° past
           3 o'clock, round caps, #3DF0CE → #0A7967) with a soft glow

The largest size is rendered at one analytic sample per pixel; the smaller
icns sizes are exact box downsamples of it.
"""

from __future__ import annotations

import math
import struct
import sys
import zlib
from pathlib import Path

# Colours (sRGB 8-bit).
BG_TOP = (0x1C, 0x21, 0x27)
BG_BOTTOM = (0x0B, 0x0D, 0x10)
STEM_COLOR = (0xEC, 0xEE, 0xF1)
ARC_FROM = (0x3D, 0xF0, 0xCE)  # color:accent.ink (dark)
ARC_TO = (0x0A, 0x79, 0x67)  # color:accent (light)
HALO = (0x2A, 0xD4, 0xB6)  # color:accent (dark)
WHITE = (0xFF, 0xFF, 0xFF)

# icns chunk type -> square pixel size. Modern macOS scales the largest PNG
# chunk down for every display size, so the four canonical chunks suffice.
ICNS_SIZES = ((b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024))
BASE = ICNS_SIZES[-1][1]  # 1024

# Design-space geometry (unit scale 0..BASE).
TILE = (100.0, 100.0, 924.0, 924.0)  # x0, y0, x1, y1 — macOS 824 pt grid
TILE_EXPONENT = 5.0  # superellipse n: the continuous corner
STEM = (353.0, 259.0, 411.0, 751.0, 29.0)  # x0, y0, x1, y1, corner radius
BOWL = (555.0, 591.0, 159.0, 58.0)  # centre x, centre y, radius, stroke
ARC_START_DEG = 40.0  # clockwise from 3 o'clock (screen coordinates)
ARC_SWEEP = 0.62  # share of the full turn
TRACK_OPACITY = 0.12
GLOW_SIGMA = 22.0
GLOW_OPACITY = 0.45
HALO_RADIUS = 343.0
HALO_OPACITY = 0.35


def _coverage(signed_distance: float) -> float:
    """Pixel coverage of a shape from its signed distance (px, < 0 inside)."""
    return min(1.0, max(0.0, 0.5 - signed_distance))


def _tile_distance(x: float, y: float) -> float:
    """Approximate signed distance to the superellipse tile boundary."""
    x0, y0, x1, y1 = TILE
    half = (x1 - x0) / 2.0
    u = abs(x - (x0 + x1) / 2.0) / half
    v = abs(y - (y0 + y1) / 2.0) / half
    if u == 0.0 and v == 0.0:
        return -half
    f = (u ** TILE_EXPONENT + v ** TILE_EXPONENT) ** (1.0 / TILE_EXPONENT)
    return (f - 1.0) * half


def _rounded_rect_distance(x: float, y: float, x0: float, y0: float,
                           x1: float, y1: float, r: float) -> float:
    """Signed distance to a rounded rectangle."""
    cx, cy = (x0 + x1) / 2.0, (y0 + y1) / 2.0
    hx, hy = (x1 - x0) / 2.0 - r, (y1 - y0) / 2.0 - r
    qx, qy = abs(x - cx) - hx, abs(y - cy) - hy
    outside = math.hypot(max(qx, 0.0), max(qy, 0.0))
    return outside + min(max(qx, qy), 0.0) - r


def _arc_centreline_distance(x: float, y: float) -> float:
    """Distance from a point to the accent arc's centreline (round caps)."""
    cx, cy, radius, _ = BOWL
    dx, dy = x - cx, y - cy
    start = math.radians(ARC_START_DEG)
    sweep = ARC_SWEEP * 2.0 * math.pi
    angle = (math.atan2(dy, dx) - start) % (2.0 * math.pi)
    if angle <= sweep:
        return abs(math.hypot(dx, dy) - radius)
    ends = (start, start + sweep)
    return min(
        math.hypot(x - (cx + radius * math.cos(a)), y - (cy + radius * math.sin(a)))
        for a in ends
    )


def _arc_colour(x: float, y: float) -> tuple[float, float, float]:
    """The arc's diagonal gradient across the bowl's bounding box."""
    cx, cy, radius, _ = BOWL
    t = ((x - (cx - radius)) + (y - (cy - radius))) / (4.0 * radius)
    t = min(1.0, max(0.0, t))
    return tuple(a + (b - a) * t for a, b in zip(ARC_FROM, ARC_TO))


def _over(dst: list[float], colour: tuple[float, ...], alpha: float) -> None:
    """Source-over an opaque colour at `alpha` onto the opaque `dst` RGB."""
    if alpha <= 0.0:
        return
    for c in range(3):
        dst[c] = colour[c] * alpha + dst[c] * (1.0 - alpha)


def _pixel(x: float, y: float) -> tuple[int, int, int, int]:
    """Shade one design-space sample; returns straight (non-premultiplied)
    8-bit RGBA."""
    tile = _coverage(_tile_distance(x, y))
    if tile <= 0.0:
        return (0, 0, 0, 0)
    t = (y - TILE[1]) / (TILE[3] - TILE[1])
    rgb = [a + (b - a) * min(1.0, max(0.0, t)) for a, b in zip(BG_TOP, BG_BOTTOM)]
    cx, cy, radius, stroke = BOWL
    halo = max(0.0, 1.0 - math.hypot(x - cx, y - cy) / HALO_RADIUS)
    _over(rgb, HALO, HALO_OPACITY * halo)
    _over(rgb, STEM_COLOR, _coverage(_rounded_rect_distance(x, y, *STEM)))
    ring = abs(math.hypot(x - cx, y - cy) - radius) - stroke / 2.0
    _over(rgb, WHITE, TRACK_OPACITY * _coverage(ring))
    arc = _arc_centreline_distance(x, y) - stroke / 2.0
    colour = _arc_colour(x, y)
    glow = math.exp(-(max(0.0, arc) ** 2) / (2.0 * GLOW_SIGMA ** 2))
    _over(rgb, colour, GLOW_OPACITY * glow)
    _over(rgb, colour, _coverage(arc))
    return (
        int(round(rgb[0])),
        int(round(rgb[1])),
        int(round(rgb[2])),
        int(round(255.0 * tile)),
    )


def render_rgba(size: int) -> bytes:
    """Render the icon at `size` x `size` as raw RGBA bytes (8-bit), one
    analytic sample per pixel centre."""
    scale = BASE / size
    out = bytearray(size * size * 4)
    i = 0
    for py in range(size):
        y = (py + 0.5) * scale
        for px in range(size):
            out[i:i + 4] = bytes(_pixel((px + 0.5) * scale, y))
            i += 4
    return bytes(out)


def downsample(rgba: bytes, src: int, dst: int) -> bytes:
    """Exact box filter (k x k mean) from `src` to `dst`; dst divides src.
    Colour is averaged alpha-weighted so transparent margins never darken
    the tile edge."""
    k = src // dst
    if src % dst or k <= 0:
        raise ValueError(f"cannot downsample {src} to {dst}")
    n = k * k
    out = bytearray(dst * dst * 4)
    for y in range(dst):
        for x in range(dst):
            r = g = b = a = 0
            for sy in range(k):
                row = (y * k + sy) * src * 4 + x * k * 4
                for sx in range(k):
                    j = row + sx * 4
                    alpha = rgba[j + 3]
                    r += rgba[j] * alpha
                    g += rgba[j + 1] * alpha
                    b += rgba[j + 2] * alpha
                    a += alpha
            i = (y * dst + x) * 4
            if a:
                out[i] = r // a
                out[i + 1] = g // a
                out[i + 2] = b // a
            out[i + 3] = a // n
    return bytes(out)


def png_rgba(rgba: bytes, width: int, height: int) -> bytes:
    """Encode raw RGBA as a PNG (bit depth 8, colour type 6, no tIME chunk)."""
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (struct.pack(">I", len(data)) + tag + data
                + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    stride = width * 4
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter: none
        raw += rgba[y * stride:(y + 1) * stride]
    idat = zlib.compress(bytes(raw), 9)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr)
            + chunk(b"IDAT", idat) + chunk(b"IEND", b""))


def build_icns() -> bytes:
    """Build the complete .icns file (ic07..ic10 PNG chunks)."""
    rgba1024 = render_rgba(BASE)
    chunks = b""
    for ctype, size in ICNS_SIZES:
        rgba = rgba1024 if size == BASE else downsample(rgba1024, BASE, size)
        data = png_rgba(rgba, size, size)
        chunks += struct.pack(">4sI", ctype, 8 + len(data)) + data
    return struct.pack(">4sI", b"icns", 8 + len(chunks)) + chunks


def write_icns(path: Path) -> int:
    """Write the .icns file; returns the byte count written."""
    data = build_icns()
    path.write_bytes(data)
    return len(data)


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(f"usage: {argv[0]} <out.icns>", file=sys.stderr)
        return 2
    written = write_icns(Path(argv[1]))
    print(f"wrote {written} bytes to {argv[1]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
