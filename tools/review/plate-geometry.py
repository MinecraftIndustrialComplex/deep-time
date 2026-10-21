#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plate-geometry task (build log 2026-10-21): a plate run's boundary mix, plate count and size
distribution, plate births and deaths and collisions, from its dynamics record, against Bird 2003.

    deeptime plate-check --run <run.ck> --record <rec.json>
    plate-geometry.py <rec.json>... [--skip 100] [--max-plates 40] [--label NAME]

Each record is one run; the runs are pooled (means over runs of each run's mean over its steps after
the first --skip Myr). Prints:

- the boundary mix by class (dynamics::mix), with the subducting share, the ocean-ocean share of the
  subducting boundary and of all boundary, and the converging boundary with no contact;
- the ocean-ocean and ocean-continent fronts by slab length, contact age and convergence across
  the boundary (FrontBands);
- plates alive and the cumulative count N(>A) at the areas of Bird's Table 1 counts (A in sr;
  4 pi sr is the visible planet), with Bird's own count at each area and his N = 7 A^(-1/3);
- the steps with max_plates alive, plates by crust (oceanic under 5 % continental);
- births by origin and deaths by fate per 100 Myr, the median life of plates that died;
- collisions (terranes of collision_min_frac or more) per 100 Myr, partings per 100 Myr;
- land, continental crust and the largest continental piece's share of the planet.

Runs on the test VM (records are 1-3 MB of JSON per 1000 Myr).
"""
import json
import math
import sys

# Bird 2003, Table 1 (G3 4(3):1027): the 52 plates' areas, steradians (read 2026-10-21).
BIRD_SR = [
    1.44065, 0.13066, 1.43268, 0.02050, 0.12082, 0.00793, 0.01418, 1.13294, 0.01295, 0.00481,
    0.01715, 0.01270, 0.07304, 0.03765, 0.07223, 0.00356, 0.00411, 1.19630, 0.00079, 0.00036,
    0.30637, 0.00632, 0.00241, 0.01245, 0.01037, 0.00020, 0.00284, 0.01030, 1.36559, 0.00956,
    0.02394, 0.01585, 0.00306, 0.39669, 0.07482, 0.00802, 2.57685, 0.00674, 0.13409, 0.00249,
    1.03045, 0.00762, 0.04190, 0.00178, 0.47192, 0.00317, 0.21967, 0.00454, 0.00870, 0.00625,
    0.01116, 0.05425,
]
AREAS_SR = [1.0, 0.3, 0.1, 0.0377, 0.01]
NAMES = ["ocean_ocean", "ocean_continent", "collision", "ridge", "rift", "other"]
SLAB_BANDS = ["0", "50", "100", "150", "300", "600"]
NORMAL_BANDS = ["0", "5", "10", "20", "40", "80"]
AGE_BANDS = ["0", "6", "12", "24", "50", "100"]
CONT_BANDS = ["0", "0.01", "0.05", "0.15", "0.3", "0.5"]
COAST_BANDS = ["0", "150", "300", "600", "1200", "2400"]
PAIRS = ["initiation", "rift", "other"]


def pct(v, q):
    if not v:
        return float("nan")
    v = sorted(v)
    i = min(len(v) - 1, max(0, int(round(q * (len(v) - 1)))))
    return v[i]


def mean(v):
    return sum(v) / len(v) if v else float("nan")


def run_stats(path, skip, max_plates):
    rec = json.load(open(path))
    d = rec.get("dynamics", rec)
    steps = d["steps"]
    if not steps:
        return None
    t_first = steps[0]["t"]
    t_last = steps[-1]["t"]
    span = t_last - t_first + d.get("dt_myr", 2.0)
    use = [s for s in steps if s["t"] > t_first + skip]
    r = {}
    # Boundary mix.
    mix = [0.0] * 6
    for s in use:
        for k, x in enumerate(s.get("mix_km", [0.0] * 6)):
            mix[k] += x
    total = sum(mix) or 1.0
    for k, n in enumerate(NAMES):
        r[f"mix_{n}"] = mix[k] / total
    r["mix_subducting"] = (mix[0] + mix[1]) / total
    r["oo_of_subducting"] = mix[0] / max(mix[0] + mix[1], 1e-9)
    conv_other = sum(s.get("front_bands", {}).get("other_converging", 0.0) for s in use)
    r["other_converging"] = conv_other / total
    # Fronts by band (0 ocean-ocean, 1 ocean-continent), shares of each setting's front.
    for which, bands in (("slab", SLAB_BANDS), ("age", AGE_BANDS), ("normal", NORMAL_BANDS)):
        for k, setting in ((0, "oo"), (1, "oc")):
            acc = [0.0] * 6
            for s in use:
                fb = s.get("front_bands", {})
                row = fb.get(which, [[0.0] * 6, [0.0] * 6])[k]
                for j, x in enumerate(row):
                    acc[j] += x
            tot = sum(acc) or 1.0
            for j, b in enumerate(bands):
                r[f"{setting}_{which}_{b}"] = acc[j] / tot
    # The ocean-ocean front's diagnostics (records of the plate-geometry task's builds only).
    for which, bands in (("oo_over_cont", CONT_BANDS), ("oo_under_cont", CONT_BANDS),
                         ("oo_coast", COAST_BANDS), ("oo_pair", PAIRS)):
        acc = [0.0] * len(bands)
        for s in use:
            row = s.get("front_bands", {}).get(which)
            if row:
                for j, x in enumerate(row):
                    acc[j] += x
        tot = sum(acc)
        if tot > 0:
            for j, b in enumerate(bands):
                r[f"{which}_{b}"] = acc[j] / tot
    # Plates and their sizes.
    alive, at_max, oceanic_n, oceanic_area = [], 0, [], []
    over = {a: [] for a in AREAS_SR}
    above_rift_floor = []
    for s in use:
        vis = sum(p[1] for p in s["plates"]) or 1
        ps = [p for p in s["plates"] if p[1] > 0]
        alive.append(len(ps))
        if len(ps) >= max_plates:
            at_max += 1
        sizes = [4.0 * math.pi * p[1] / vis for p in ps]
        for a in AREAS_SR:
            over[a].append(sum(1 for x in sizes if x >= a))
        oc = [p for p in ps if p[2] < 0.05 * p[1]]
        oceanic_n.append(len(oc))
        oceanic_area.append(sum(p[1] for p in oc) / vis)
        above_rift_floor.append(sum(1 for p in ps if p[1] / vis >= 0.024))
    r["plates"] = mean(alive)
    r["plates_min"] = min(alive)
    r["plates_max"] = max(alive)
    r["steps_at_max"] = at_max / max(len(use), 1)
    for a in AREAS_SR:
        r[f"n_over_{a}"] = mean(over[a])
    r["oceanic_plates"] = mean(oceanic_n)
    r["oceanic_plate_area"] = mean(oceanic_area)
    r["plates_over_rift_floor"] = mean(above_rift_floor)
    # Births, deaths.
    births, fates, lives = {}, {}, []
    for p in d["plates"]:
        o = p.get("origin", "?")
        if o != "initial":
            births[o] = births.get(o, 0) + 1
        f = p.get("fate")
        if f:
            fates[f] = fates.get(f, 0) + 1
            if p.get("died_myr") is not None:
                lives.append(p["died_myr"] - p["born_myr"])
    for o, n in births.items():
        r[f"born_{o}"] = 100.0 * n / span
    for f, n in fates.items():
        r[f"died_{f}"] = 100.0 * n / span
    r["life_p50"] = pct(lives, 0.5)
    # Collisions.
    cols = [c for c in d.get("collisions", []) if c.get("collision")]
    r["collisions_per_100myr"] = 100.0 * len(cols) / span
    r["welds_per_100myr"] = 100.0 * sum(1 for c in cols if c.get("sutured")) / span
    r["accretions_per_100myr"] = 100.0 * sum(1 for c in d.get("collisions", []) if not c.get("collision")) / span
    r["partings_per_100myr"] = 100.0 * len(d.get("partings", [])) / span
    for p in d.get("partings", []):
        key = f"parting_{p.get('reason', '?')}"
        r[key] = r.get(key, 0.0) + 100.0 / span
    # Collision terranes by size (A/A0) and ends.
    for c in d.get("collisions", []):
        f = c.get("frac", 0.0)
        band = "ge0.5" if f >= 0.5 else "ge0.1" if f >= 0.1 else "ge0.02" if f >= 0.02 else "lt0.02"
        key = f"terrane_{band}{'_weld' if c.get('sutured') else ''}"
        r[key] = r.get(key, 0.0) + 100.0 / span
    # Land.
    r["land"] = mean([s["land"] / max(1, s["visible"]) for s in use])
    r["continental"] = mean([s["continental"] / max(1, s["visible"]) for s in use])
    r["piece1"] = mean([(s["pieces"][0] if s.get("pieces") else 0) / max(1, s["visible"]) for s in use])
    r["land_end"] = steps[-1]["land"] / max(1, steps[-1]["visible"])
    # The run summary's counters (the stage's printed `plates_final=` line), per 100 Myr.
    txt = path[: -len(".rec.json")] + ".txt" if path.endswith(".rec.json") else None
    if txt:
        try:
            for line in open(txt, errors="replace"):
                at = line.find("plates_final=")
                if at < 0:
                    continue
                for kv in line[at:].split():
                    k, _, v = kv.partition("=")
                    if k in ("rifts", "initiations", "reversals", "subductions", "collisions",
                             "sutures", "healed", "accretions", "births", "deaths", "buoyant"):
                        r[f"n_{k}_per_100myr"] = 100.0 * float(v) / span
                break
        except OSError:
            pass
    return r


def main():
    args = sys.argv[1:]
    skip, max_plates, label = 100.0, 40, ""
    paths = []
    i = 0
    while i < len(args):
        a = args[i]
        if a == "--skip":
            skip = float(args[i + 1]); i += 2
        elif a == "--max-plates":
            max_plates = int(args[i + 1]); i += 2
        elif a == "--label":
            label = args[i + 1]; i += 2
        else:
            paths.append(a); i += 1
    runs = []
    for p in paths:
        try:
            s = run_stats(p, skip, max_plates)
        except (OSError, ValueError, KeyError) as e:
            print(f"# skip {p}: {e}", file=sys.stderr)
            continue
        if s:
            runs.append((p, s))
    if not runs:
        print("no runs")
        return
    keys = []
    for _, s in runs:
        for k in s:
            if k not in keys:
                keys.append(k)
    print(f"# {label} runs={len(runs)} skip={skip} Myr")
    names = [p.rsplit("/", 1)[-1].replace(".rec.json", "") for p, _ in runs]
    print(f"{'measure':28} {'mean':>8} " + " ".join(f"{n[-22:]:>22}" for n in names))
    for k in keys:
        vals = [s.get(k, 0.0) for _, s in runs]
        print(f"{k:28} {mean(vals):8.4f} " + " ".join(f"{v:22.4f}" for v in vals))
    print("# Bird 2003 Table 1: N(>A) " + ", ".join(
        f"{a} sr {sum(1 for x in BIRD_SR if x >= a)} (7A^-1/3 {7.0 * a ** (-1.0 / 3.0):.1f})"
        for a in AREAS_SR))


if __name__ == "__main__":
    main()
