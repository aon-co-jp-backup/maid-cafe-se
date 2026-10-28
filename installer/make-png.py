#!/usr/bin/env python3
"""ピンクの丸のアイコン(トレイと同じ絵)をPNGで書き出す。標準ライブラリだけ。使い方: make-png.py 出力.png [大きさ=512]"""
import struct
import sys
import zlib


def png(n):
    c = (n - 1) / 2.0
    rows = []
    for y in range(n):
        row = bytearray([0])
        for x in range(n):
            d = ((x - c) ** 2 + (y - c) ** 2) ** 0.5
            a = min(1.0, max(0.0, (c - d) * (n / 21.0) + 0.5))
            t = min(1.0, max(0.0, d / c))
            row += bytes([int(255 - 30 * t), int(150 - 60 * t), int(190 - 60 * t), int(a * 255)])
        rows.append(bytes(row))

    def chunk(tag, data):
        body = tag + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", n, n, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(b"".join(rows), 9)) + chunk(b"IEND", b"")


if __name__ == "__main__":
    size = int(sys.argv[2]) if len(sys.argv) > 2 else 512
    with open(sys.argv[1], "wb") as f:
        f.write(png(size))
