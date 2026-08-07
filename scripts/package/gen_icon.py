"""Deterministic BBR placeholder app icon (.icns), Python stdlib only.

Renders a rounded-square tile in the DESIGN token accent (Phosphor teal
`#0E8F79`, DESIGN §3 `color:accent` light) with a white geometric lowercase
"b" monogram, then packs the canonical icns sizes (128/256/512/1024) as PNG
chunks. Every step is pure integer/float math: no randomness, no timestamps,
fixed zlib level, so identical input always yields byte-identical output.

Geometry (1024-unit design space, y grows downward):

  background   rounded square, full canvas, corner radius 192
  stem         pill: x 340..428, y 300..760, corner radius 44
  bowl         ring centred (620, 545), outer radius 205, inner radius 118

The inner disc of the bowl shows the accent tile through the ring hole; the
stem overlaps the ring's left edge so both read as one "b" glyph.

The largest size is rendered one sample per pixel; the smaller icns sizes are
produced by exact box downsampling, which also anti-aliases them.
"""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

# DESIGN §3 `color:accent` (light) #0E8F79 — the single Phosphor signal hue.
ACCENT = (14, 143, 121)
MONOGRAM = (255, 255, 255)

# icns chunk type -> square pixel size. Modern macOS scales the largest PNG
# chunk down for every display size, so the four canonical chunks suffice.
ICNS_SIZES = ((b"ic07", 128), (b"ic08", 256), (b"ic09", 512), (b"ic10", 1024))
BASE = ICNS_SIZES[-1][1]  # 1024

# Design-space geometry (unit scale 0..BASE).
TILE_RADIUS = 192
STEM = (340, 300, 428, 760, 44)  # x0, y0, x1, y1, corner radius
RING = (620, 545, 205, 118)  # centre x, centre y, outer radius, inner radius


def _inside_rounded_rect(x: float, y: float, x0: float, y0: float, x1: float,
                         y1: float, r: float) -> bool:
    """Point-in-rounded-rectangle test (radius r may be 0 for a plain rect)."""
    cx = min(max(x, x0 + r), x1 - r)
    cy = min(max(y, y0 + r), y1 - r)
    dx = x - cx
    dy = y - cy
    return dx * dx + dy * dy <= r * r


def _inside_ring(x: float, y: float, cx: float, cy: float, outer: float,
                 inner: float) -> bool:
    """Point-in-annulus test; the inner disc shows the tile through the hole."""
    d2 = (x - cx) ** 2 + (y - cy) ** 2
    return d2 <= outer * outer and d2 >= inner * inner


def render_rgba(size: int) -> bytes:
    """Render the icon at `size` x `size` as raw RGBA bytes (8-bit)."""
    scale = BASE / size
    out = bytearray(size * size * 4)
    i = 0
    for py in range(size):
        y = (py + 0.5) * scale
        for px in range(size):
            x = (px + 0.5) * scale
            if _inside_rounded_rect(x, y, 0, 0, BASE, BASE, TILE_RADIUS):
                if (_inside_rounded_rect(x, y, *STEM)
                        or _inside_ring(x, y, *RING)):
                    out[i] = MONOGRAM[0]
                    out[i + 1] = MONOGRAM[1]
                    out[i + 2] = MONOGRAM[2]
                else:
                    out[i] = ACCENT[0]
                    out[i + 1] = ACCENT[1]
                    out[i + 2] = ACCENT[2]
                out[i + 3] = 255
            i += 4
    return bytes(out)


def _box_downsample(rgba: bytes, src: int, dst: int) -> bytes:
    """Exact box filter (k x k mean) from `src` to `dst`; dst divides src."""
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
                    r += rgba[j]
                    g += rgba[j + 1]
                    b += rgba[j + 2]
                    a += rgba[j + 3]
            i = (y * dst + x) * 4
            out[i] = r // n
            out[i + 1] = g // n
            out[i + 2] = b // n
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
        rgba = rgba1024 if size == BASE else _box_downsample(rgba1024, BASE, size)
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
