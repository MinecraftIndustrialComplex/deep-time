#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How the plate check's collisions end and how long they last (the collision length checks,
CHK-012/013), from the records of tools/review/force-balance-ensemble.sh runs.

  python3 tools/review/collision-ends.py <run.rec.json> ...

Per run and pooled: the collisions the check counts (`CollisionRec.collision`), how each ended
(stalled into a weld, a terrane transferred, a plate died), their length (first to last step plus
one step, as the check counts it), the share of the contact's life before the collision began,
the convergence at the end and the shortening; and the 90th-percentile tail's ends.
"""
import json
import sys


def pct(xs, q):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    i = (len(xs) - 1) * q
    lo = int(i)
    hi = min(lo + 1, len(xs) - 1)
    return xs[lo] + (xs[hi] - xs[lo]) * (i - lo)


def end_of(c):
    if c.get("died") is not None:
        return "died"
    if c.get("stalled"):
        return "stalled" + ("+sutured" if c.get("sutured") else "")
    if c.get("sutured"):
        return "sutured"
    return "transfer"


def main():
    pooled = []
    for p in sys.argv[1:]:
        d = json.load(open(p))
        dt = d.get("dt_myr", 2.0)
        cols = [c for c in d.get("collisions", []) if c.get("collision")]
        rows = []
        for c in cols:
            rows.append(
                {
                    "len": c["end_myr"] - c["start_myr"] + dt,
                    "pre": c["start_myr"] - c.get("contact_onset_myr", c["start_myr"]),
                    "end": end_of(c),
                    "conv": c.get("convergence_mm_yr", float("nan")),
                    "short": c.get("front_shortening_km", float("nan")),
                    "onset": c.get("onset_convergence_mm_yr") or float("nan"),
                    "frac": c.get("frac", float("nan")),
                }
            )
        pooled += rows
        lens = [r["len"] for r in rows]
        print(f"{p.rsplit('/', 1)[-1]}: {len(rows)} collisions, length p50 {pct(lens, 0.5):.0f} p90 {pct(lens, 0.9):.0f} Myr")
    lens = [r["len"] for r in pooled]
    p90 = pct(lens, 0.9)
    print(f"pooled: {len(pooled)} collisions, length p50 {pct(lens, 0.5):.0f} p90 {p90:.0f} Myr")
    ends = {}
    for r in pooled:
        ends.setdefault(r["end"], []).append(r)
    for k, rs in sorted(ends.items()):
        ls = [r["len"] for r in rs]
        print(f"  ended {k:16s} {len(rs):4d}  length p50 {pct(ls, 0.5):5.0f} p90 {pct(ls, 0.9):5.0f}  conv at end p50 {pct([r['conv'] for r in rs], 0.5):5.1f} mm/yr  shortening p50 {pct([r['short'] for r in rs], 0.5):6.0f} km  frac p50 {pct([r['frac'] for r in rs], 0.5):.2f}")
    tail = sorted([r for r in pooled if r["len"] >= p90], key=lambda r: -r["len"])
    print(f"  the tail (length >= p90, {len(tail)}):")
    for r in tail[:20]:
        print(f"    {r['len']:5.0f} Myr  {r['end']:16s} onset {r['onset']:5.1f} end {r['conv']:5.1f} mm/yr  shortening {r['short']:6.0f} km  frac {r['frac']:.2f}  contact before {r['pre']:5.0f} Myr")


if __name__ == "__main__":
    main()
