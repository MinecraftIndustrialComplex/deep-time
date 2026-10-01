#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Lava census of an area export (`deeptime-inspect export-area`, .dtva or .dtva.gz).

    python3 tools/workbench/lava_stats.py <area.dtva.gz> [--json out.json]

Prints lava, cave air and water per depth band, then the connected lava bodies (6-neighbour):
size, y range, extent, how many of their faces touch water or air, and how many lie open to the
sky. It is the measurement behind the lava rules of GeologicChunkGenerator (the build-log entry
"Lava floods" in STATUS.md). Standard library only; a 256 x 256 x 992 box takes about a minute.
"""
import array
import collections
import gzip
import json
import struct
import sys

BANDS = [(-352, -300), (-300, -250), (-250, -200), (-200, -150), (-150, -100), (-100, -54),
         (-54, -10), (-10, 30), (30, 55), (55, 64), (64, 80), (80, 640)]


def main():
    path = sys.argv[1]
    out_json = sys.argv[3] if len(sys.argv) > 3 and sys.argv[2] == "--json" else None
    d = gzip.open(path).read() if path.endswith(".gz") else open(path, "rb").read()
    hl = struct.unpack_from("<I", d, 8)[0]
    h = json.loads(d[12:12 + hl])
    blob = 12 + hl
    blob += (-blob) % 4

    def sec(name):
        s = h["sections"][name]
        a = array.array("H")
        a.frombytes(d[blob + s["offset"]: blob + s["offset"] + s["count"] * 2])
        return a

    rc, rv, rl = sec("run_counts"), sec("run_values"), sec("run_lengths")
    w, dep, y0, y1 = h["width"], h["depth"], h["y0"], h["y1"]
    ht = y1 - y0 + 1
    cat = [h["blocks"][e[0]]["cat"] for e in h["entries"]]
    ids = [h["blocks"][e[0]]["id"] for e in h["entries"]]
    print("box", h["box"], "y", y0, y1, "profile", h.get("meta", {}).get("profile"))

    # 0 solid, 1 air, 2 water, 3 lava
    grid = bytearray(w * dep * ht)
    k = 0
    by_y, air_y, water_y = collections.Counter(), collections.Counter(), collections.Counter()
    lava_ids = collections.Counter()
    lava_cols = set()
    top = {}  # column -> y of the highest block that is not air
    for c in range(w * dep):
        y = 0
        for _ in range(rc[c]):
            v, n = rv[k], rl[k]
            k += 1
            ct = cat[v]
            code = 1 if ct == "air" else 2 if ct == "water" else 3 if ct == "lava" else 0
            if code:
                base = c * ht + y
                grid[base:base + n] = bytes([code]) * n
                counter = (None, air_y, water_y, by_y)[code]
                for yy in range(y, y + n):
                    counter[yy + y0] += 1
                if code == 3:
                    lava_ids[ids[v]] += n
                    lava_cols.add(c)
            if code != 1:
                top[c] = y + n - 1 + y0
            y += n
    total = sum(by_y.values())
    print(f"lava blocks {total:,} {dict(lava_ids)}; columns with lava {len(lava_cols):,} of {w * dep:,}")
    print("band            lava   cave air     water   lava share of the open space")
    bands = []
    for a, b in BANDS:
        lava = sum(by_y[y] for y in range(a, b))
        # Air above the terrain is not cave air: count air only below each column's top.
        air = sum(air_y[y] for y in range(a, b)) if b <= 64 else 0
        water = sum(water_y[y] for y in range(a, b))
        bands.append({"lo": a, "hi": b, "lava": lava, "air": air, "water": water})
        print(f"{a:5d}..{b:<5d} {lava:9d} {air:10d} {water:9d}   {lava / max(1, lava + air + water):.3f}")

    seen = bytearray(len(grid))
    bodies = []
    bx, bz = h["box"]["x0"], h["box"]["z0"]
    for start in range(len(grid)):
        if grid[start] != 3 or seen[start]:
            continue
        stack = [start]
        seen[start] = 1
        n = wat = airc = sky = 0
        lo = [10 ** 9] * 3
        hi = [-10 ** 9] * 3
        while stack:
            q = stack.pop()
            n += 1
            c, y = divmod(q, ht)
            z, i = divmod(c, w)
            for a, val in enumerate((i, y, z)):
                lo[a] = min(lo[a], val)
                hi[a] = max(hi[a], val)
            for di, dz, dy in ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)):
                ii, zz, yy = i + di, z + dz, y + dy
                if not (0 <= ii < w and 0 <= zz < dep and 0 <= yy < ht):
                    continue
                r = (zz * w + ii) * ht + yy
                g = grid[r]
                if g == 3:
                    if not seen[r]:
                        seen[r] = 1
                        stack.append(r)
                elif g == 2:
                    wat += 1
                elif g == 1:
                    airc += 1
                    if dy == 1 and top.get(c, -999) <= y + y0:
                        sky += 1
        bodies.append({"blocks": n, "y": [lo[1] + y0, hi[1] + y0], "x": [lo[0] + bx, hi[0] + bx],
                       "z": [lo[2] + bz, hi[2] + bz], "water_faces": wat, "air_faces": airc, "open_sky": sky})
    bodies.sort(key=lambda b: -b["blocks"])
    touching = [b for b in bodies if b["water_faces"]]
    print(f"bodies {len(bodies)}; touching water {len(touching)} ({sum(b['water_faces'] for b in bodies)} faces); "
          f"open to the sky {sum(1 for b in bodies if b['open_sky'])} ({sum(b['open_sky'] for b in bodies)} faces)")
    print("  blocks    y lo..hi         x range          z range   water   air   sky")
    for b in bodies[:20]:
        print(f"{b['blocks']:8d} {b['y'][0]:5d}..{b['y'][1]:<5d} {b['x'][0]:7d}..{b['x'][1]:<7d} "
              f"{b['z'][0]:5d}..{b['z'][1]:<5d} {b['water_faces']:6d} {b['air_faces']:6d} {b['open_sky']:5d}")
    high = [b for b in bodies if b["y"][1] >= 40]
    print(f"bodies reaching y >= 40: {len(high)}, {sum(b['blocks'] for b in high):,} blocks")
    hist = collections.Counter((b["y"][1] + 352) // 50 * 50 - 352 for b in bodies)
    print("bodies by top y (50-block bins):", sorted(hist.items()))
    if out_json:
        with open(out_json, "w") as f:
            json.dump({"box": h["box"], "lava": total, "bands": bands, "bodies": bodies[:200],
                       "bodies_total": len(bodies)}, f, indent=1)


if __name__ == "__main__":
    main()
