#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Force-balance tasks 7 and 8: a plate run's history in time bins, from its dynamics record.

    deeptime plate-check --run <run.ck> --record <rec.json>
    force-balance-series.py <rec.json> [bin Myr, default 20] [first Myr shown, default 0] [last]

Per bin: plates alive; plate speed (visible-sample-weighted RMS, and the 10th / 50th / 90th
percentile of plate-steps of plates of 30 samples or more); plate-steps at the speed guard;
oceanic slabs (count and mean length); collisions; continental crust and the largest continental
piece's share of it (the supercontinent measure: 0.7 assembled, under 0.5 dispersed); land.
Then the supercontinent episodes by plate-check's definition. Runs on the test VM (the record is
1-2 MB of JSON per 1000 Myr).
"""
import json
import math
import sys


def pct(v, q):
    if not v:
        return float("nan")
    v = sorted(v)
    i = min(len(v) - 1, max(0, int(round(q * (len(v) - 1)))))
    return v[i]


def main():
    path = sys.argv[1]
    width = float(sys.argv[2]) if len(sys.argv) > 2 else 20.0
    lo = float(sys.argv[3]) if len(sys.argv) > 3 else -1e9
    hi = float(sys.argv[4]) if len(sys.argv) > 4 else 1e9
    rec = json.load(open(path))
    d = rec.get("dynamics", rec)
    steps = d["steps"]
    t0 = steps[0]["t"] - 2.0
    bins = {}
    for s in steps:
        if not (lo <= s["t"] <= hi):
            continue
        b = math.floor((s["t"] - t0 - 1e-9) / width)
        bins.setdefault(b, []).append(s)
    print(f"{'Myr':>9} {'plates':>6} {'rms':>5} {'p10':>5} {'p50':>5} {'p90':>5} {'guard':>5} "
          f"{'slabs':>5} {'slabkm':>6} {'coll':>4} {'crust':>5} {'piece':>5} {'land':>5}")
    for b in sorted(bins):
        ss = bins[b]
        speeds, w2, wn, guard, nplates = [], 0.0, 0.0, 0, 0
        slabs, slabkm, coll, crust, piece, land = 0, 0.0, 0, 0.0, 0.0, 0.0
        for s in ss:
            nplates += len(s["plates"])
            for p in s["plates"]:
                pid, vis, cont, v, front, bnd, rot, flags = p
                w2 += vis * v * v
                wn += vis
                if vis >= 30:
                    speeds.append(v)
                if int(flags) & 2:
                    guard += 1
            slabs += len(s.get("slabs_km", []))
            slabkm += sum(s.get("slabs_km", []))
            coll += s.get("collisions", 0)
            crust += s["continental"] / max(1, s["visible"])
            pieces = s.get("pieces", [])
            piece += (pieces[0] / s["continental"]) if pieces and s["continental"] else 0.0
            land += s["land"] / max(1, s["visible"])
        n = len(ss)
        a = t0 + b * width
        print(f"{a:4.0f}-{a + width:<4.0f} {nplates / n:6.1f} {math.sqrt(w2 / max(wn, 1)):5.1f} "
              f"{pct(speeds, .1):5.1f} {pct(speeds, .5):5.1f} {pct(speeds, .9):5.1f} {guard:5d} "
              f"{slabs / n:5.1f} {slabkm / max(slabs, 1):6.0f} {coll:4d} {crust / n:5.3f} "
              f"{piece / n:5.3f} {land / n:5.3f}")
    # Supercontinent episodes: assembled at 0.7 of the crust in one piece, dispersed under 0.5.
    on, start, eps = False, None, []
    for s in steps:
        pieces = s.get("pieces", [])
        share = (pieces[0] / s["continental"]) if pieces and s["continental"] else 0.0
        if not on and share >= 0.7:
            on, start = True, s["t"]
        elif on and share < 0.5:
            on = False
            eps.append((start, s["t"]))
    if on:
        eps.append((start, None))
    # A re-assembly within 20 Myr of a dispersal is the same stay (plate-check's
    # dispersal_hold_myr, CHK-060).
    merged = []
    for a, b in eps:
        if merged and merged[-1][1] is not None and a - merged[-1][1] < 20.0:
            merged[-1] = (merged[-1][0], b)
        else:
            merged.append((a, b))
    eps = merged
    print("supercontinent episodes (assembled, dispersed):",
          ", ".join(f"{a:.0f}-{'' if b is None else f'{b:.0f}'}" for a, b in eps) or "none")
    gaps = [eps[i + 1][0] - eps[i][0] for i in range(len(eps) - 1)]
    tenures = [b - a for a, b in eps if b is not None]
    if tenures:
        print(f"tenure median {pct(tenures, .5):.0f} Myr; assembly-to-assembly "
              f"{'median %.0f Myr' % pct(gaps, .5) if gaps else 'n/a'}")


if __name__ == "__main__":
    main()
