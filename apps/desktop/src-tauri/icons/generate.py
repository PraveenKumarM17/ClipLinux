#!/usr/bin/env python3
"""Generate ClipLinux window icons (stdlib only).

Usage (from repo root):
    python3 apps/desktop/src-tauri/icons/generate.py
"""
from __future__ import annotations

import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent

BG = (24, 24, 27, 255)  # zinc-900
BOARD = (39, 39, 42, 255)  # zinc-800
CLIP = (161, 161, 170, 255)  # zinc-400
PAPER = (250, 250, 250, 255)
ACCENT = (245, 158, 11, 255)  # amber-500
CLEAR = (0, 0, 0, 0)


def png_rgba(pixels: list[list[tuple[int, int, int, int]]]) -> bytes:
    height = len(pixels)
    width = len(pixels[0])
    raw = bytearray()
    for row in pixels:
        raw.append(0)
        for r, g, b, a in row:
            raw.extend((r, g, b, a))
    compressed = zlib.compress(bytes(raw), 9)

    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    return b"".join(
        [
            b"\x89PNG\r\n\x1a\n",
            chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)),
            chunk(b"IDAT", compressed),
            chunk(b"IEND", b""),
        ]
    )


def fill_round_rect(
    px: list[list[tuple[int, int, int, int]]],
    x0: float,
    y0: float,
    x1: float,
    y1: float,
    radius: float,
    color: tuple[int, int, int, int],
) -> None:
    size = len(px)
    x0, x1 = min(x0, x1), max(x0, x1)
    y0, y1 = min(y0, y1), max(y0, y1)
    r = max(0.0, min(radius, (x1 - x0) / 2, (y1 - y0) / 2))
    ix0 = max(0, int(x0))
    iy0 = max(0, int(y0))
    ix1 = min(size - 1, int(x1))
    iy1 = min(size - 1, int(y1))
    r2 = r * r
    for y in range(iy0, iy1 + 1):
        for x in range(ix0, ix1 + 1):
            cx = min(max(x + 0.5, x0 + r), x1 - r)
            cy = min(max(y + 0.5, y0 + r), y1 - r)
            dx = x + 0.5 - cx
            dy = y + 0.5 - cy
            if dx * dx + dy * dy <= r2:
                px[y][x] = color


def draw(size: int) -> bytes:
    s = float(size)
    px = [[CLEAR for _ in range(size)] for _ in range(size)]
    # App-tile background
    fill_round_rect(px, 0.06 * s, 0.06 * s, 0.94 * s, 0.94 * s, 0.22 * s, BG)
    # Clipboard body
    fill_round_rect(px, 0.24 * s, 0.28 * s, 0.76 * s, 0.86 * s, 0.07 * s, BOARD)
    # Paper
    fill_round_rect(px, 0.30 * s, 0.36 * s, 0.70 * s, 0.80 * s, 0.04 * s, PAPER)
    # Clip
    fill_round_rect(px, 0.38 * s, 0.16 * s, 0.62 * s, 0.34 * s, 0.08 * s, CLIP)
    fill_round_rect(px, 0.44 * s, 0.20 * s, 0.56 * s, 0.30 * s, 0.05 * s, BG)
    # History lines
    for i, y in enumerate((0.46, 0.54, 0.62)):
        color = ACCENT if i == 0 else BOARD
        fill_round_rect(px, 0.36 * s, y * s, 0.64 * s, (y + 0.035) * s, 0.015 * s, color)
    return png_rgba(px)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "icon.png").write_bytes(draw(512))
    (OUT / "128x128.png").write_bytes(draw(128))
    (OUT / "32x32.png").write_bytes(draw(32))


if __name__ == "__main__":
    main()
