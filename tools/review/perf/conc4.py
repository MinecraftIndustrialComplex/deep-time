#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""perf script -F tid,time,ip,sym (gzip) on stdin: where the run's idle cores go. Each sample's concurrency c is the
number of samples (all threads) within +-10 ms over what one busy thread gives at 199 Hz, so c is the instantaneous
count of busy threads in expectation even when threads alternate faster than any window; the sample's share of the
idle cores is (8 - c) / (8 c 199) wall-equivalent seconds. Spinning samples (a rayon idle frame before any deeptime
frame from the leaf) count as idle. Totals by stage and by deeptime frame (inclusive) (the coordinator, 2026-10-28)."""
import sys, re, collections, gzip, bisect
HZ, HALF, T = 199.0, 0.010, 8
SPIN = re.compile(r"wait_until_cold|__sched_yield|rayon_core::sleep|Sleep>::|crossbeam_deque|crossbeam_epoch")
DT = re.compile(r"^<?(deeptime_core|deeptime_format)::")
STAGES = [("amplify", "deeptime_core::surface::amplify"), ("plates", "deeptime_core::plates::"), ("climate", "deeptime_core::climate::"),
          ("sea_level", "deeptime_core::sea_level"), ("surface", "deeptime_core::surface::"), ("bake", "deeptime_core::bake::"),
          ("structure", "deeptime_core::structure::"), ("deposits", "deeptime_core::deposits::"), ("hydro", "deeptime_core::hydro::")]
def short(f):
    f = re.sub(r"\{closure#\d+\}", "c", f)
    for _ in range(5):
        f = re.sub(r"<[^<>]*>", "", f)
    return f.replace("&", "").strip()[:100]
def inner(f):
    m = re.findall(r"(deeptime_core::[A-Za-z0-9_:]+?)::\{closure", f)
    return "[closure] " + m[-1] if m else None
recs = []
cur = None
def done(c):
    if c is None: return
    t, tid, fr = c
    seen = False
    for f in fr:
        if not seen and SPIN.search(f):
            recs.append((t, None)); return
        if "deeptime_" in f:
            seen = True
    stage = "other"
    for f in reversed(fr):
        for name, key in STAGES:
            if key in f:
                stage = name; break
        if stage != "other": break
    names = []
    for f in fr:
        n = short(f) if DT.match(f) else (inner(f) if "deeptime_" in f and f.startswith("rayon") else None)
        if n and n not in names: names.append(n)
    recs.append((t, (stage, names)))
for line in gzip.open(sys.argv[1], "rt", errors="replace"):
    if not line.strip():
        done(cur); cur = None; continue
    if not line[0].isspace():
        m = re.match(r"\s*\S*?\s*(\d+)\s+([\d.]+):", line)
        if m: cur = (float(m.group(2)), int(m.group(1)), [])
        continue
    m = re.match(r"^[0-9a-f]+ (.*)$", line.strip())
    if m and cur is not None: cur[2].append(m.group(1).rsplit(" (", 1)[0])
done(cur)
recs.sort(key=lambda r: r[0])
work_t = [t for t, w in recs if w is not None]
span = recs[-1][0] - recs[0][0]
cpu = len(work_t) / HZ
print(f"samples {len(recs)}, working {len(work_t)} ({cpu:.0f} CPU-s), spinning {len(recs) - len(work_t)}; run {span:.0f} s; "
      f"mean busy threads {cpu / span:.2f}; idle-core wall-equivalent {(T * span - cpu) / T:.0f} s")
lost_stage = collections.Counter(); cpu_stage = collections.Counter(); lost = collections.Counter(); cpu_fr = collections.Counter()
for t, w in recs:
    if w is None: continue
    lo = bisect.bisect_left(work_t, t - HALF); hi = bisect.bisect_right(work_t, t + HALF)
    c = max(1.0, (hi - lo) / (2 * HALF * HZ))
    wl = max(0.0, T - c) / (T * c * HZ)
    stage, names = w
    lost_stage[stage] += wl; cpu_stage[stage] += 1 / HZ
    for n in names:
        lost[n] += wl; cpu_fr[n] += 1 / HZ
print(f"-- attributed idle wall-equivalent {sum(lost_stage.values()):.0f} s (the rest is time with no working sample)")
print("-- by stage: lost s / CPU s")
for k, v in lost_stage.most_common(): print(f"   {k:10s} {v:7.1f} / {cpu_stage[k]:7.1f}")
print("-- lost s (CPU s), inclusive by deeptime frame")
for k, v in lost.most_common(90): print(f"{v:8.1f} ({cpu_fr[k]:7.1f})  {k}")
