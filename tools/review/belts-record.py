#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Summarise the collisions of plate dynamics records (plate tasks 3 and 4: belts, resistance,
weld on stall).

    deeptime plate-check --run <checkpoint> --record rec.json
    tools/review/belts-record.py rec.json [rec2.json ...]

Per record: how collision states ended (welds, transfers, partings by reason), their lengths,
shortening, front lengths, belt heights and wedges, the convergence slowdown, and the
continental crust over the run. Reads only the JSON the check writes; nothing heavy.
"""

import json
import statistics as st
import sys


def pct(xs, q):
    xs = sorted(x for x in xs if x is not None)
    if not xs:
        return float("nan")
    i = min(len(xs) - 1, max(0, int(round(q * (len(xs) - 1)))))
    return xs[i]


def line(name, xs, unit=""):
    xs = [x for x in xs if x is not None]
    if not xs:
        print(f"  {name:<44} n=0")
        return
    print(f"  {name:<44} n={len(xs):<4} p10={pct(xs, .1):9.1f} p50={pct(xs, .5):9.1f} "
          f"p90={pct(xs, .9):9.1f} max={max(xs):9.1f} {unit}")


def main(paths):
    for path in paths:
        d = json.load(open(path))
        n = d["samples"]
        dt = d["dt_myr"]
        steps = d["steps"]
        span = steps[-1]["t"] - d["t0_myr"] if steps else 0
        cols = d["collisions"]
        parts = d.get("partings", [])
        belts = d.get("belts", [])
        big = [c for c in cols if c["collision"]]
        welds = [c for c in cols if c["sutured"]]
        print(f"== {path}: {span:.0f} Myr, {len(cols)} transfers and welds "
              f"({len(big)} collisions, {len(welds)} welds), {len(parts)} partings")
        reasons = {}
        for p in parts:
            reasons[p["reason"]] = reasons.get(p["reason"], 0) + 1
        print("  partings by reason:", reasons)
        line("collision length, Myr", [c["end_myr"] - c["start_myr"] + dt for c in big])
        line("weld length, Myr", [c["end_myr"] - c["start_myr"] + dt for c in welds])
        line("collision shortening, km", [c["front_shortening_km"] for c in big])
        line("collision front, km", [c["front_km"] for c in big])
        line("parting length, Myr", [p["end_myr"] - p["start_myr"] + dt for p in parts])
        line("parting shortening, km", [p["shortening_km"] for p in parts])
        line("parting front, km", [p["front_km"] for p in parts])
        line("parting peak convergence, mm/yr", [p["peak_convergence_mm_yr"] for p in parts])
        line("parting belt peak height, m", [p.get("peak_height_m") for p in parts])
        line("belt peak height, m", [b.get("peak_height_m") for b in belts])
        line("belt width, km", [b.get("belt_width_km") for b in belts])
        line("belt length, km", [b.get("belt_length_km") for b in belts])
        line("wedge width, km", [b.get("wedge_width_km") for b in belts])
        line("foredeep width, km", [b.get("foredeep_width_km") for b in belts])
        line("foredeep depth, m", [b.get("foredeep_depth_m") for b in belts])
        ratios = [b["convergence_20myr_mm_yr"] / b["convergence_onset_mm_yr"]
                  for b in belts if b.get("convergence_20myr_mm_yr") is not None
                  and b.get("convergence_onset_mm_yr")]
        line("convergence 20 Myr after onset / onset", ratios)
        line("peak uplift (front), m", [c["peak_uplift_m"] for c in big])
        line("most raised above sea in a step, % planet",
             [100.0 * c["emerged"]["samples"] / n for c in big])
        cont = [s["continental"] / n for s in steps]
        land = [s["land"] / n for s in steps]
        if cont:
            print(f"  continental crust: start {cont[0]:.3f} min {min(cont):.3f} "
                  f"max {max(cont):.3f} end {cont[-1]:.3f}; land median {st.median(land):.3f}")
        crossed = sum(c.get("area_km2", 0.0) for c in cols) + sum(p["area_km2"] for p in parts)
        print(f"  continental crust that crossed collision fronts: {crossed / 1e6:.1f} million km2 "
              f"({crossed / (4 * 3.141592653589793 * 6370.0 ** 2):.3f} of the sphere)")


if __name__ == "__main__":
    main(sys.argv[1:])
