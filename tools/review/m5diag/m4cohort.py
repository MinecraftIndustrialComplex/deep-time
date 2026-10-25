#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How fast old belts lose crust and height, by cohort: m4cohort.py <run folder> ...

The front trace's last-touch fields cannot give the erosion of a column (m4erode.py: the snapshots fall on
resampling steps, so the last change of half the thick samples is the blend). A belt's own clock can: the orogeny
tag never clears and its age (Myr since the last orogenic uplift) grows by the snapshot interval unless the belt
is uplifted again. So the samples of tag c and age [a0, a1) at a snapshot t are, 50 Myr later, the samples of tag c
and age [a0 + 50, a1 + 50) (less those subducted or hidden, plus the resampling's new samples, which take their
major corner's tag and age). The difference of the cohort's mean thickness is its net crust change over the
interval by every process (erosion and deposition, collapse, flow, foundering, the blend), and of its mean height
over the sea its height change.

Reads the front trace's attribution (`trace/att_t<Myr>.bin`, 27 f32: lat, lon, T m, z m, tag, age, L km, K km, ...)
when the run has it, with the keel split (K >= 100 km at each end), else the crust-budget maps (`maps/t<Myr>.bin`,
11 f32: lat, lon, z m, T m, kind, tag, age, ...), all continental samples. Per tag and starting age, pooled over
the runs and the snapshot pairs (each pair weighted by the smaller cohort): pairs, samples at the start, the share
left at the end, the mean T at the start (km), the crust loss rate E_c (m/Myr, thinning positive), the mean height
over the sea at the start (m) and its loss rate (m/Myr), and E_c against Pinet & Souriau 1988's law for old
orogens, Ds = 61e-6 H m/kyr (0.061 H m/Myr, H the mean height, read by a Sonnet sub-agent), at the cohort's mean
height over the interval.

With the trace, also the part that took thick columns (they crossed 45 km since the trace began) below 45 km during
each interval (`touch_part` freezes when the column drops under 45 km; a gap_budget::Part id), by tag and keel.
Needs numpy only."""
import json
import sys
from pathlib import Path

import numpy as np

AGES = [0, 10, 20, 30, 50, 100, 150, 250, 1e9]
PS = 0.061  # m/Myr per m of mean height, Pinet & Souriau 1988 (old orogens)
PART_NAMES = ["birth", "erosion", "rebound", "water", "flexure", "roots", "released", "flexrest", "spread", "later",
              "carry", "subduction", "shortening", "belts", "arcs", "forearc", "founder", "collapse", "flow",
              "transition", "collision", "rifting", "hotspots", "settle", "rate", "plate_surface", "finish", "bury",
              "resample_blend", "resample_accrete", "other"]


def load(run):
    """Per snapshot: t, sea, and the continental samples' T, z, tag, age, K (NaN without the trace), cross_t,
    touch_part, touch_t (NaN without the trace)."""
    rec = json.loads((run / "rec.json").read_text())
    ts = np.array([s["t"] for s in rec["steps"]])
    seas = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
    sea_at = lambda t: float(seas[int(np.argmin(np.abs(ts - t)))])  # noqa: E731
    out = []
    idx = run / "trace" / "att_index.txt"
    if idx.exists():
        for line in idx.read_text().splitlines():
            kv = dict(p.split("=", 1) for p in line.split())
            t = float(kv["t"])
            a = np.fromfile(run / "trace" / kv["file"], "<f4").reshape(-1, 27).astype(np.float64)
            out.append({"t": t, "sea": sea_at(t), "T": a[:, 2], "z": a[:, 3], "tag": np.rint(a[:, 4]),
                        "age": a[:, 5], "K": a[:, 7], "cross_t": a[:, 10], "touch": a[:, 14], "touch_t": a[:, 16]})
        return out, True
    for path in sorted((run / "maps").glob("t*.bin"), key=lambda p: float(p.stem[1:])):
        t = float(path.stem[1:])
        a = np.fromfile(path, "<f4").reshape(-1, 11).astype(np.float64)
        a = a[a[:, 4] > 0.5]
        nan = np.full(len(a), np.nan)
        out.append({"t": t, "sea": sea_at(t), "T": a[:, 3], "z": a[:, 2], "tag": np.rint(a[:, 5]), "age": a[:, 6],
                    "K": nan, "cross_t": nan, "touch": nan, "touch_t": nan})
    return out, False


def main():
    runs = [Path(r) for r in sys.argv[1:]]
    acc = {}  # (tag, keel, bin) -> sums
    drops = {}  # (tag, keel) -> {part: count}
    traced = True
    for run in runs:
        snaps, tr = load(run)
        traced &= tr
        for s0, s1 in zip(snaps[:-1], snaps[1:]):
            dt = s1["t"] - s0["t"]
            for tag in (2, 1):
                keels = (("keel", lambda s: s["K"] >= 100), ("no keel", lambda s: s["K"] < 100)) if tr else \
                    (("all", lambda s: np.ones(len(s["T"]), bool)),)
                for kname, kf in keels:
                    for a0, a1 in zip(AGES[:-1], AGES[1:]):
                        m0 = (s0["tag"] == tag) & kf(s0) & (s0["age"] >= a0) & (s0["age"] < a1)
                        m1 = (s1["tag"] == tag) & kf(s1) & (s1["age"] >= a0 + dt) & (s1["age"] < a1 + dt)
                        n0, n1 = int(m0.sum()), int(m1.sum())
                        if n0 < 10 or n1 < 10:
                            continue
                        w = min(n0, n1)
                        h0 = np.maximum(s0["z"][m0] - s0["sea"], 0.0).mean()
                        h1 = np.maximum(s1["z"][m1] - s1["sea"], 0.0).mean()
                        d = acc.setdefault((tag, kname, a0), np.zeros(9))
                        d += [1, n0, n1, w, w * s0["T"][m0].mean(), w * (s0["T"][m0].mean() - s1["T"][m1].mean()) / dt,
                              w * h0, w * (h0 - h1) / dt, w * (h0 + h1) / 2]
                    if tr:
                        # Thick columns (crossed since the trace began) that went under 45 km in the interval.
                        m = (s1["tag"] == tag) & kf(s1) & (s1["T"] < 45_000) & np.isfinite(s1["cross_t"]) \
                            & (s1["touch_t"] > s0["t"]) & (s1["touch_t"] <= s1["t"])
                        dd = drops.setdefault((tag, kname), {})
                        for p in np.rint(s1["touch"][m]).astype(int):
                            dd[p] = dd.get(p, 0) + 1
            if tr:
                # The same for untagged columns.
                m = (s1["tag"] == 0) & (s1["T"] < 45_000) & np.isfinite(s1["cross_t"]) \
                    & (s1["touch_t"] > s0["t"]) & (s1["touch_t"] <= s1["t"])
                kk = s1["K"][m] >= 100
                for kname, sel in (("keel", kk), ("no keel", ~kk)):
                    dd = drops.setdefault((0, kname), {})
                    for p in np.rint(s1["touch"][m][sel]).astype(int):
                        dd[p] = dd.get(p, 0) + 1
    print(f"{len(runs)} runs ({'front trace' if traced else 'crust-budget maps'})")
    print("tag, keel, start age Myr: pairs, n start, share left, T start km, E_c m/Myr (thinning +), H start m,"
          " dH m/Myr (lowering +), 0.061 H m/Myr, E_c / PS, H e-fold Myr at dH")
    for tag, tname in ((2, "collision"), (1, "andean")):
        for key in sorted(k for k in acc if k[0] == tag):
            _, kname, a0 = key
            p, n0, n1, w, wT, wE, wH, wdH, wHm = acc[key]
            a1 = AGES[AGES.index(a0) + 1]
            ec, hm, dh = wE / w, wHm / w, wdH / w
            print(f"  {tname:9s} {kname:7s} {a0:g}-{a1:g}: pairs {int(p):3d}  n {int(n0):7d}  left {n1 / n0:4.2f}"
                  f"  T {wT / w / 1000:5.1f}  E_c {ec:6.1f}  H {wH / w:6.0f}  dH {dh:6.1f}  PS {PS * hm:6.1f}"
                  f"  E_c/PS {ec / max(PS * hm, 1e-9):5.2f}  e-fold {hm / dh if dh > 0 else float('inf'):7.0f}")
    if drops:
        print("\nthick columns gone under 45 km in an interval, by the part that took them under: tag, keel: n; "
              "top parts (share)")
        for (tag, kname), dd in sorted(drops.items()):
            n = sum(dd.values())
            if n == 0:
                continue
            top = sorted(((c, p) for p, c in dd.items()), reverse=True)[:6]
            name = {2: "collision", 1: "andean", 0: "untagged"}[tag]
            print(f"  {name:9s} {kname:7s}: n {n:6d}; " + ", ".join(
                f"{PART_NAMES[p] if 0 <= p < len(PART_NAMES) else p}: {c / n:.2f}" for c, p in top))


if __name__ == "__main__":
    main()
