#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Median over runs of the sphere-grid benchmark's kernel tables (docs/research/notes/sphere-native-grids.md).

    python3 tools/research/sphere-grid-bench/summarize.py review/out/sphere-grid-bench [generic|native]

Reads every <flavour>-<cells>[-rK].md in the folder and prints, per size and thread count, each
layout's median ns per cell for diff, rcv, stack and acc, the ratio to the Lambert raster, the
run count, and the median CG iterations and times. Standard library only.
"""
import glob
import os
import re
import statistics
import sys
from collections import defaultdict

folder = sys.argv[1]
flav = sys.argv[2] if len(sys.argv) > 2 else "generic"
kern = defaultdict(list)  # (size, threads, layout) -> [(diff, rcv, stack, acc)]
cg = defaultdict(list)  # (size, layout, s) -> [(it, ms)]
for path in sorted(glob.glob(os.path.join(folder, f"{flav}-*.md"))):
    m = re.match(rf"{flav}-(\d+)(?:-r\d+)?\.md$", os.path.basename(path))
    if not m:
        continue
    size = int(m.group(1))
    section = None
    for line in open(path):
        if line.startswith("## "):
            section = line[3:].split(" ")[0]
            continue
        if not line.startswith("| ") or line.startswith("| layout") or line.startswith("|---"):
            continue
        f = [x.strip() for x in line.strip().strip("|").split("|")]
        if section == "Kernels":
            kern[(size, int(f[2]), f[0])].append(tuple(float(f[i]) for i in (4, 6, 8, 10)))
        elif section == "Implicit":
            cg[(size, f[0], float(f[1]))].append((int(f[2]), float(f[3])))

order = ["lambert", "cube", "healpix", "icos", "fib-plain", "fib-hilbert", "fib-random", "fib-hilbert-ell"]
for size in sorted({k[0] for k in kern}):
    for th in sorted({k[1] for k in kern if k[0] == size}):
        base = statistics.median(v[0] for v in kern[(size, th, "lambert")])
        base_r = statistics.median(v[1] for v in kern[(size, th, "lambert")])
        print(f"\n### {size} cells, {th} thread(s), {flav}: median ns/cell (x Lambert)\n")
        print("| layout | runs | diff | rcv | stack | acc |")
        print("|---|---|---|---|---|---|")
        for lay in order:
            rows = kern.get((size, th, lay))
            if not rows:
                continue
            med = [statistics.median(r[i] for r in rows) for i in range(4)]
            print(
                f"| {lay} | {len(rows)} | {med[0]:.2f} ({med[0] / base:.1f}x) | {med[1]:.2f} ({med[1] / base_r:.1f}x)"
                f" | {med[2]:.2f} | {med[3]:.2f} |"
            )
    print(f"\n### {size} cells: implicit solve, median over runs\n")
    print("| layout | s | iterations | ms (1 thread) |")
    print("|---|---|---|---|")
    for (sz, lay, s), rows in sorted(cg.items(), key=lambda kv: (kv[0][2], kv[0][1])):
        if sz != size:
            continue
        print(f"| {lay} | {s:g} | {rows[0][0]} | {statistics.median(r[1] for r in rows):.0f} |")
