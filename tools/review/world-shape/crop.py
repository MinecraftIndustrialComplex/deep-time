#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Cut a tall page screenshot into viewable pieces: crop.py <in.png> <out prefix> [piece height]."""
import sys

from PIL import Image

im = Image.open(sys.argv[1])
h = int(sys.argv[3]) if len(sys.argv) > 3 else 1800
w, H = im.size
n = 0
for y in range(0, H, h):
    box = im.crop((0, y, w, min(H, y + h)))
    if box.getextrema()[0][0] == box.getextrema()[0][1] and y > 0:
        break
    box.save(f"{sys.argv[2]}-{n:02d}.png")
    n += 1
print(n, "pieces of", im.size)
