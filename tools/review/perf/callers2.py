#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Samples with frame key (argv[2]) on the stack: the nearest deeptime caller frames above it, CPU seconds."""
import sys, re, gzip, collections
HZ = 199.0
key = sys.argv[2]
cnt = collections.Counter(); cur = []; tot = 0
def short(f):
    f = re.sub(r"\{closure#\d+\}", "c", f)
    for _ in range(5): f = re.sub(r"<[^<>]*>", "", f)
    return f.strip()[:90]
def done(fr):
    global tot
    idx = next((i for i, f in enumerate(fr) if key in f), None)
    if idx is None: return
    tot += 1
    above = [short(f) for f in fr[idx + 1:] if "deeptime_" in f][:3]
    cnt[" <- ".join(above)] += 1
for line in gzip.open(sys.argv[1], "rt", errors="replace"):
    if not line.strip():
        done(cur); cur = []; continue
    if not line.startswith("\t"): continue
    m = re.match(r"^[0-9a-f]+ (.*)$", line.strip())
    if m: cur.append(m.group(1).rsplit(" (", 1)[0])
done(cur)
print(f"{tot / HZ:.1f} CPU-s with {key}")
for k, v in cnt.most_common(12): print(f"{v / HZ:7.2f}  {k}")
