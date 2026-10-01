#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tile times and sizes of a workbench's "Final world" map, and what a full pyramid would take.

    python3 tools/workbench/tiles_bench.py PORT RUN [TILES_PER_ZOOM]

Asks a running `deeptime watch` on 127.0.0.1:PORT (run it on the Mac, against a private workbench,
never Ben's on 8642) for TILES_PER_ZOOM random tiles (default 16) of every layer at every zoom of
the finished run RUN, twice each: the first request draws and caches the tile (the *cold* time, as
the browser sees it: rendering, PNG encoding and the cache write), the second reads it back (the
*warm* time). Prints a table, the size a full pyramid of each layer would have (the mean tile
size of a zoom times its tiles, summed), and the server's own render counters and cache state.
Standard library only.
"""

import json
import random
import sys
import time
import urllib.request


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    port, run = sys.argv[1], sys.argv[2]
    n = int(sys.argv[3]) if len(sys.argv) > 3 else 16
    base = f"http://127.0.0.1:{port}/api/runs/{run}"
    info = json.load(urllib.request.urlopen(base + "/world.json"))
    if not info.get("available"):
        print("no world:", info.get("reason"))
        return 1
    random.seed(7)
    sizes: dict[str, dict[int, float]] = {}
    print("layer   zoom  tiles  cold_mean_ms  cold_max_ms  warm_mean_ms  mean_KB")
    for layer in [l["id"] for l in info["layers"]]:
        for z in range(info["max_zoom"] + 1):
            nx, ny = 2 << z, 1 << z
            want = list(dict.fromkeys((random.randrange(nx), random.randrange(ny)) for _ in range(n)))
            cold, warm, kb = [], [], []
            for x, y in want:
                t = time.perf_counter()
                b = urllib.request.urlopen(f"{base}/tiles/{layer}/{z}/{x}/{y}.png").read()
                cold.append((time.perf_counter() - t) * 1000)
                kb.append(len(b))
            for x, y in want:
                t = time.perf_counter()
                urllib.request.urlopen(f"{base}/tiles/{layer}/{z}/{x}/{y}.png").read()
                warm.append((time.perf_counter() - t) * 1000)
            mean = lambda a: sum(a) / len(a)
            print(f"{layer:7} {z:4} {len(want):6} {mean(cold):12.1f} {max(cold):11.1f} {mean(warm):13.1f} {mean(kb) / 1024:8.1f}")
            sizes.setdefault(layer, {})[z] = mean(kb)
    print("\nfull pyramid (mean tile size x tiles per zoom), MiB:")
    grand = 0.0
    for layer, per in sizes.items():
        total = sum(per[z] * (2 << z) * (1 << z) for z in per)
        grand += total
        print(f"  {layer:8} {total / 1048576:8.1f}   " + ", ".join(f"z{z}={per[z] * (2 << z) * (1 << z) / 1048576:.1f}" for z in sorted(per)))
    print(f"  all      {grand / 1048576:8.1f}")
    info = json.load(urllib.request.urlopen(base + "/world.json"))
    print("\nserver render times:", json.dumps(info["timing"]))
    print("cache:", info["cache"])
    return 0


if __name__ == "__main__":
    sys.exit(main())
