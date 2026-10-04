#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""PROTOTYPE (docs/design/seamless-wrapping.md): draws a `deeptime heightmap` .npy as a PNG.

  render.py <in.npy> <out.png> --box x0,z0,x1,z1 --step S [--sea Y] [--domain x0,x1,z0,z1]
            [--mark x,z]...

Hypsometric colours from channel 0 (the terrain height H) with a hill shade; the canonical domain
outlined in white, marks (singular points, poles) as yellow crosses. Plain Python (no numpy).
"""
import argparse, array, ast, struct, zlib

ap = argparse.ArgumentParser()
ap.add_argument("inp"); ap.add_argument("out")
ap.add_argument("--box", required=True); ap.add_argument("--step", type=float, required=True)
ap.add_argument("--sea", type=float, default=63.0)
ap.add_argument("--domain"); ap.add_argument("--mark", action="append", default=[])
a = ap.parse_args()
x0, z0, x1, z1 = [float(v) for v in a.box.split(",")]

raw = open(a.inp, "rb").read()
assert raw[:6] == b"\x93NUMPY", "not an .npy file"
hlen = struct.unpack("<H", raw[8:10])[0]
hdr = ast.literal_eval(raw[10:10 + hlen].decode())
assert hdr["descr"] == "<f4" and not hdr["fortran_order"], hdr
rows, cols, ch = hdr["shape"]
data = array.array("f"); data.frombytes(raw[10 + hlen:])
H = [[data[(j * cols + i) * ch] for i in range(cols)] for j in range(rows)]

img = bytearray(rows * cols * 3)
for j in range(rows):
    for i in range(cols):
        h = H[j][i]
        gx = H[j][min(i + 1, cols - 1)] - H[j][max(i - 1, 0)]
        gz = H[min(j + 1, rows - 1)][i] - H[max(j - 1, 0)][i]
        shade = min(1.3, max(0.6, 1.0 + 0.16 * (-gx - gz) / max(a.step / 8.0, 1.0)))
        if h <= a.sea:
            d = min(1.0, max(0.0, (a.sea - h) / 250.0))
            c = (0.10, 0.30 - 0.18 * d, 0.62 - 0.30 * d)
        else:
            t = min(1.0, max(0.0, (h - a.sea) / 160.0))
            c = (0.30 + 0.45 * t, 0.55 - 0.05 * t, 0.25 + 0.30 * t)
        k = (j * cols + i) * 3
        for q in range(3):
            img[k + q] = int(255 * min(1.0, max(0.0, c[q] * shade)))


def put(i, j, col):
    if 0 <= i < cols and 0 <= j < rows:
        k = (j * cols + i) * 3
        img[k:k + 3] = bytes(int(255 * v) for v in col)


def line(xa, za, xb, zb, col):
    n = int(max(abs(xb - xa), abs(zb - za)) / a.step) + 2
    for s in range(n + 1):
        i = int(round((xa + (xb - xa) * s / n - x0) / a.step))
        j = int(round((za + (zb - za) * s / n - z0) / a.step))
        for d in (-1, 0, 1):
            put(i + d, j, col); put(i, j + d, col)


if a.domain:
    dx0, dx1, dz0, dz1 = [float(v) for v in a.domain.split(",")]
    for xa, za, xb, zb in [(dx0, dz0, dx1, dz0), (dx1, dz0, dx1, dz1), (dx1, dz1, dx0, dz1), (dx0, dz1, dx0, dz0)]:
        line(xa, za, xb, zb, (1, 1, 1))
for m in a.mark:
    mx, mz = [float(v) for v in m.split(",")]
    r = 6 * a.step
    line(mx - r, mz - r, mx + r, mz + r, (1, 0.9, 0))
    line(mx - r, mz + r, mx + r, mz - r, (1, 0.9, 0))

scan = b"".join(b"\x00" + bytes(img[j * cols * 3:(j + 1) * cols * 3]) for j in range(rows))
def chunk(tag, body):
    return struct.pack(">I", len(body)) + tag + body + struct.pack(">I", zlib.crc32(tag + body) & 0xffffffff)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", cols, rows, 8, 2, 0, 0, 0)) \
    + chunk(b"IDAT", zlib.compress(scan, 9)) + chunk(b"IEND", b"")
open(a.out, "wb").write(png)
print(f"{a.out}: {cols}x{rows}")
