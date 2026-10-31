#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The subduction fronts by setting and state, from the runs' step records (no simulation): fronts.py <arm folder> ...

Reads each run's rec.json (`steps[*].mix_km` and `steps[*].front_bands`, plates/dynamics.rs `boundary_mix`) and prints
the mean length at any time, km (Earth-normalized, as the record's), over the steps from --from (Myr, default -1000)
to the end, as the median and range of the runs:

  the ocean-ocean front past incipient (`mix_km[OCEAN_OCEAN]`: an oceanic slab under oceanic crust with no continental
  crust of the overriding plate within the arc's distance, PLT-370), the ocean-continent one (`mix_km[OCEAN_CONTINENT]`),
  and the incipient fronts of each setting (`front_bands.incipient`, PLT-610, PLT-614, PLT-681; of them `slow`, the
  thermal floor's alone); the sum is what the arcs' counters `plates/arc_front_{oceanic,continental}_km_myr` take
  (subduction::fronts_finish counts every oceanic-slab front sample, incipient ones included);
  the subducting fronts with a Benioff zone (`benioff`, PLT-615);
  the subducting fronts by their convergence across the boundary (`normal`, NORMAL_BANDS_MM_YR, mm/yr);
  the ocean-ocean front past incipient by its contact's age (`age[0]`, AGE_BANDS_MYR), by its slab (`slab[0]`), by the
  overriding plate's continental share (`oo_over_cont`), by the distance to that plate's nearest continental crust
  (`oo_coast`, COAST_BANDS_KM; the last band holds plates with none) and by how its two plates are related (`oo_pair`:
  split by an initiation, rifted, neither);
  the front's turnover (`gain`, `loss`, km a Myr, by setting and state);
  and, in records made since the intra-oceanic fronts diagnosis (2026-10-31), the subducting fronts by their arc's age
  (`arc_age`, the contact's PLT-669 onset, which a polarity reversal resets), by their contact's slab age at its onset
  (`onset_slab`), and the ocean-ocean front by its contact's age against the overriding floor's age at the front
  (`oo_over_floor`).

Earth: Bird 2003, subduction zones 51,310 km; Stern & Scholl 2010, intra-oceanic arcs 14,500 km and continental arcs
27,500; Clift et al. 2009's Table 1, 40,225 km, of it 13,550 intra-oceanic by the crust-budget lane's split.
"""
import json
import statistics
import sys
from pathlib import Path

AGE = ["0-6", "6-12", "12-24", "24-50", "50-100", ">100"]
SLAB = ["0-50", "50-100", "100-150", "150-300", "300-600", ">600"]
SHARE = ["0-1%", "1-5%", "5-15%", "15-30%", "30-50%", ">50%"]
NORMAL = ["0-5", "5-10", "10-20", "20-40", "40-80", ">80"]
COAST = ["0-150", "150-300", "300-600", "600-1200", "1200-2400", ">2400/none"]
PAIR = ["initiation", "rift", "other"]


def runs_of(folder):
    return sorted(p for p in Path(folder).iterdir() if (p / "rec.json").is_file())


def one(run, t_from):
    steps = json.loads((run / "rec.json").read_text()).get("steps") or []
    steps = [s for s in steps if float(s["t"]) >= t_from and s.get("mix_km")]
    if not steps:
        return None
    n = len(steps)
    dt = float(steps[1]["t"]) - float(steps[0]["t"]) if n > 1 else 1.0
    acc = {}

    def add(key, v):
        acc[key] = acc.get(key, 0.0) + v / n

    for s in steps:
        mix, fb = s["mix_km"], s.get("front_bands") or {}
        add("oo", mix[0])
        add("oc", mix[1])
        inc = fb.get("incipient", [0.0, 0.0])
        slow = fb.get("slow", [0.0, 0.0])
        ben = fb.get("benioff", [0.0, 0.0])
        add("oo_inc", inc[0])
        add("oc_inc", inc[1])
        add("oo_slow", slow[0])
        add("oc_slow", slow[1])
        add("oo_ben", ben[0])
        add("oc_ben", ben[1])
        for name, key, labels in (("age", "age", AGE), ("slab", "slab", SLAB), ("normal", "normal", NORMAL)):
            rows = fb.get(key)
            if rows:
                for i, lab in enumerate(labels):
                    add(f"oo_{name}_{lab}", rows[0][i])
                    add(f"oc_{name}_{lab}", rows[1][i])
        for key, labels in (("oo_over_cont", SHARE), ("oo_under_cont", SHARE), ("oo_coast", COAST), ("oo_pair", PAIR)):
            row = fb.get(key)
            if row:
                for i, lab in enumerate(labels):
                    add(f"{key}_{lab}", row[i])
        # The trace's fields (the intra-oceanic fronts diagnosis, 2026-10-31; absent from older records): the
        # subducting fronts by their arc's age and by their contact's slab age at the onset, and the ocean-ocean front
        # by its contact's age against the overriding floor's age at the front. (The trace oo-trace, key 28820e933ba1,
        # also wrote no_arc, always 0, and oo_age_ocean and oo_arc_ocean, the ocean left from the subducting side's
        # lattice neighbour, which is mostly a sample gone under and so mostly read "none"; both dropped.)
        for key in ("arc_age", "onset_slab"):
            rows = fb.get(key)
            if rows:
                for i, lab in enumerate(AGE):
                    add(f"oo_{key}_{lab}", rows[0][i])
                    add(f"oc_{key}_{lab}", rows[1][i])
        for key, labels in (("oo_over_floor", AGE),):
            m = fb.get(key)
            if m:
                for i, row_lab in enumerate(labels):
                    for j, lab in enumerate(AGE):
                        add(f"{key}_{row_lab}_{lab}", m[i][j])
        for key in ("gain", "loss"):
            g = fb.get(key)
            if g:
                for si, setting in enumerate(("oo", "oc")):
                    for st, state in enumerate(("inc", "sub")):
                        add(f"{key}_{setting}_{state}", g[si][st] / dt)
    return acc


def main(argv):
    t_from = -1000.0
    if "--from" in argv:
        i = argv.index("--from")
        t_from = float(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    rows = []
    for folder in argv:
        for r in runs_of(folder):
            a = one(r, t_from)
            if a:
                rows.append(a)
    keys = list(rows[0].keys())
    print(f"# {len(rows)} runs, steps from t {t_from:.0f}; mean km at any time (gain/loss km a Myr): median [min, max]")
    for k in keys:
        xs = [r.get(k, 0.0) for r in rows]
        print(f"{k:28s} {statistics.median(xs):9.0f} [{min(xs):8.0f}, {max(xs):8.0f}]")
    tot = [r["oo"] + r["oo_inc"] for r in rows]
    print(f"{'oo + oo_inc (the arcs count)':28s} {statistics.median(tot):9.0f} [{min(tot):8.0f}, {max(tot):8.0f}]")
    sub = [r["oo"] + r["oc"] for r in rows]
    share = [r["oo"] / max(r["oo"] + r["oc"], 1e-9) for r in rows]
    print(f"{'subducting oo + oc':28s} {statistics.median(sub):9.0f} [{min(sub):8.0f}, {max(sub):8.0f}]")
    print(f"{'oo share of subducting':28s} {statistics.median(share):9.3f} [{min(share):8.3f}, {max(share):8.3f}]")
    # Over time: the ocean-ocean front past incipient and its share of the subducting front, every step of every
    # run pooled, by start and by 200 Myr window.
    series = {}
    for folder in argv:
        for r in runs_of(folder):
            start = r.name.rsplit("-", 1)[0]
            for s in json.loads((r / "rec.json").read_text()).get("steps") or []:
                t, mix = float(s["t"]), s.get("mix_km")
                if not mix or t < t_from:
                    continue
                w = int((t + 1000.0) // 200.0)
                series.setdefault((start, w), []).append((mix[0], mix[0] / max(mix[0] + mix[1], 1e-9)))
    print("# over time (every step pooled): oo km p10 / p50 / p90, oo share p50")
    for key in sorted(series):
        xs = sorted(x for x, _ in series[key])
        sh = sorted(y for _, y in series[key])
        q = lambda v, f: v[min(len(v) - 1, int(f * (len(v) - 1)))]
        lo = -1000 + 200 * key[1]
        print(f"  {key[0]:22s} t {lo:5d} to {lo + 200:5d}: {q(xs, 0.1):7.0f} {q(xs, 0.5):7.0f} {q(xs, 0.9):7.0f}"
              f"  share {q(sh, 0.5):.3f}")


if __name__ == "__main__":
    main(sys.argv[1:])
