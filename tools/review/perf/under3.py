#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""For each frame key (argv[2:]): CPU seconds of samples under it, and the innermost deeptime frames below it."""
import sys, re, gzip, collections
HZ = 199.0
keys = sys.argv[2:]
cnt = {k: collections.Counter() for k in keys}; tot = collections.Counter(); cur = []
def short(f):
    f = re.sub(r"\{closure#\d+\}", "c", f)
    for _ in range(5): f = re.sub(r"<[^<>]*>", "", f)
    return f.strip()[:110]
def done(fr):
    for key in keys:
        if not any(key in f for f in fr): continue
        tot[key] += 1
        leaf = next((f for f in fr if "deeptime_" in f), "?")
        m = re.findall(r"(deeptime_core::[A-Za-z0-9_:]+?)::\{closure", leaf) if leaf.startswith("rayon") else None
        cnt[key][("[closure] " + m[-1]) if m else short(leaf)] += 1
for line in gzip.open(sys.argv[1], "rt", errors="replace"):
    if not line.strip():
        done(cur); cur = []; continue
    if not line[0].isspace(): continue
    m = re.match(r"^[0-9a-f]+ (.*)$", line.strip())
    if m: cur.append(m.group(1).rsplit(" (", 1)[0])
done(cur)
for key in keys:
    print(f"== {tot[key] / HZ:.1f} CPU-s under {key}")
    for k, v in cnt[key].most_common(12): print(f"{v / HZ:7.2f}  {k}")
