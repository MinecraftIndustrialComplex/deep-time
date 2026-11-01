#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The intra-oceanic front by how its contact began against the resamplings (no simulation): gapfronts.py <arm folder>[@N] ...

Needs the resampling trace's records (rec.json `resamples` and the onsets' `since_resample_myr`, `prov`,
`front_km_myr`; branch oceanic-fronts) and the crust-budget maps (`DEEPTIME_CRUST_BUDGET_MAPS`, maps/t*.bin with
maps/plate/ and maps/age/). A run without the records gives the maps' part only.

Each onset (plates/dynamics.rs `OnsetRec`) is classed by
  when: the time since the last resampling at the onset (0 its own step, 2 the step after, 4 two steps after, 6 to
        10, over 10, before any);
  floor: its first front samples on the subducting side (`prov.under_bits`): gap fill (PLT-502) for half of them or
        more, less, or no front sample at the onset (unknown);
  history: whether the pair's boundary spread since the resampling before the onset's last one (the interval whose
        gap the fill closed), only before that, or never (`since_spread_myr`, PairHist).
It prints, pooled over the runs:
  the onsets that make intra-oceanic front by class, and the front each class makes over the run (OnsetRec
  `front_km_myr[0]`, a split's copies counted, as a mean length over the run);
  the intra-oceanic front on the maps (-950 to -50 Myr every 50; ocean-ocean as oofronts.py classes it: no continental
  crust of the overriding plate within 200 km, PLT-370) by the floor going under (younger than 55 Myr or not) and by
  the class of its contact's onset (the pair's latest onset at or before the map's time and still active then; a
  split's copy, which carries a new plate id, through the plates' lineage, rec.json's `parent`; or unmatched);
  snapshots taken at a resampling carry no front flags and are left out, as oofronts.py leaves them;
  for the onsets on the step after a resampling on gap fill: their pair's history, the boundary the step before, the
  floor's age at the onset (`slab_age_myr`), the age that floor had when the fill made it (the onset's age less the
  time since the fill) against the interval the fill closed (a gap point's age is alpha times that interval, PLT-502,
  so the ratio is alpha), and the plates' convergence across the overlap at the onset (the overlap trace).
"""
import json
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import oofronts  # noqa: E402
import probe  # noqa: E402

YOUNG_MYR = 55.0  # the young floor of the intra-oceanic fronts' diagnosis (plates.md, the floor going under)
WHEN = ["own step", "step after", "two after", "6-10 Myr", ">10 Myr", "before any"]
FLOOR = ["gap fill", "other", "unknown"]
HIST = ["spread in interval", "spread before", "never spread"]
KINDS = ["spreading", "converging", "transform", "front", "collision", "none", "unknown"]


def when_of(since, dt):
    # By the run's step (`dt`, Myr): its own step, the step after, two after.
    if since < 0:
        return 5
    if since < 1e-6:
        return 0
    if since <= dt + 1e-3:
        return 1
    if since <= 2.0 * dt + 1e-3:
        return 2
    if since <= 10.001:
        return 3
    return 4


def floor_of(o):
    p = o.get("prov") or {}
    n = p.get("n", 0)
    if n <= 0:
        return 2
    return 0 if p["under_bits"][0] * 2 >= n else 1


def classify(o, res_ts, t0, dt):
    since = o.get("since_resample_myr", -1.0)
    o["when"] = when_of(since, dt)
    o["floor"] = floor_of(o)
    # The interval the last fill closed: from the resampling before it (or the run's start) to it.
    o["interval"] = float("nan")
    window = 0.0
    if since >= 0:
        t_fill = o["t"] - since
        prev = [r for r in res_ts if r < t_fill - 1e-9]
        o["interval"] = t_fill - (prev[-1] if prev else t0)
        window = since + o["interval"]
    ss = o.get("since_spread_myr", -1.0)
    o["hist"] = 2 if ss < 0 else (0 if ss <= window + 1e-6 else 1)


def front_samples(run, t):
    """The intra-oceanic front samples of the map at t: the overriding plate, the subducting plate (the nearest
    sample of another plate among the 12 nearest) and the floor's age there; with the spacing, km."""
    a, _ = probe.read(run, t)
    nm = probe.name_of(t)
    plate = np.fromfile(run / "maps" / "plate" / nm, "<u4").astype(np.int64)
    age = np.fromfile(run / "maps" / "age" / nm, "<f4").astype(np.float64)
    if len(plate) != len(a) or len(age) != len(a):
        return None
    p = oofronts.xyz(a)
    cont = a[:, 4] > 0.5
    front = (a[:, 10].astype(np.int64) & oofronts.FRONT) > 0
    idx = np.nonzero(front)[0]
    spacing = float(np.sqrt(4 * np.pi * oofronts.R_KM ** 2 / len(a)))
    if not len(idx):
        # A snapshot at a resampling carries no front flags (crust_budget.rs): not a planet with no front.
        return None
    trees = {}
    for q in np.unique(plate[cont]):
        m = np.nonzero(cont & (plate == q))[0]
        trees[q] = cKDTree(p[m])
    all_tree = cKDTree(p)
    out = []
    for i in idx:
        q = int(plate[i])
        if q in trees:
            dd, _ = trees[q].query(p[i])
            if oofronts.arc_km(dd) < oofronts.ARC_KM:
                continue
        _, ks = all_tree.query(p[i], k=12)
        other = [k for k in np.atleast_1d(ks) if plate[k] != q]
        if not other:
            continue
        out.append((q, int(plate[other[0]]), float(age[other[0]])))
    return out, spacing


def ancestors(x, parents):
    out = [x]
    while len(out) < 64 and parents.get(out[-1]) is not None:
        out.append(parents[out[-1]])
    return out


def link(onsets_by_pair, onsets, u, q, t, parents=None):
    """The onset of the contact (under u, over q) at map time t: the pair's latest onset at or before t still active
    then; else (a split's copy carries a new plate id) the latest such onset whose plates are u's and q's ancestors
    (rec.json's plate lives, `parent`); else None."""
    best = None
    for i in onsets_by_pair.get((u, q), []):
        o = onsets[i]
        if o["t"] <= t + 1e-6 and o.get("last_myr", o["t"]) >= t - 1e-6:
            if best is None or o["t"] > onsets[best]["t"]:
                best = i
    if best is not None:
        return best, "pair"
    if parents:
        au, aq = ancestors(u, parents), ancestors(q, parents)
        for i, o in enumerate(onsets):
            if (o["under"] in au and o["over"] in aq and o["t"] <= t + 1e-6
                    and o.get("last_myr", o["t"]) >= t - 1e-6):
                if best is None or o["t"] > onsets[best]["t"]:
                    best = i
    return (best, "lineage") if best is not None else (None, "none")


def main(argv):
    ts = list(range(-950, 0, 50))
    runs, length_myr = 0, 0.0
    onset_rows = []
    maps = {}  # (young, when, floor, hist, how) -> km summed over snapshots
    snapshots = 0
    traced_runs = 0
    for arg in argv:
        for r in probe.runs_of(arg):
            rec = json.loads((r / "rec.json").read_text())
            steps = rec.get("steps") or []
            if not steps:
                continue
            runs += 1
            res_ts = [float(x["t"]) for x in rec.get("resamples") or []]
            onsets = [o for o in rec.get("onsets") or [] if "since_resample_myr" in o]
            traced = bool(res_ts) and bool(onsets)
            traced_runs += int(traced)
            t0 = float(rec.get("t0_myr", steps[0]["t"]))
            if traced:
                length_myr += float(steps[-1]["t"]) - t0
            dt = float(rec.get("dt_myr", 2.0))
            parents = {int(x["id"]): x.get("parent") for x in rec.get("plates") or []}
            for o in onsets:
                classify(o, res_ts, t0, dt)
            onset_rows.extend(onsets)
            by_pair = {}
            for i, o in enumerate(onsets):
                by_pair.setdefault((o["under"], o["over"]), []).append(i)
            for t in ts:
                try:
                    got = front_samples(r, t)
                except (FileNotFoundError, ValueError):
                    got = None
                if got is None:
                    continue
                snapshots += 1
                fs, sp = got
                for q, u, age in fs:
                    young = int(age < YOUNG_MYR)
                    if traced:
                        i, how = link(by_pair, onsets, u, q, t, parents)
                        key = (young, onsets[i]["when"], onsets[i]["floor"], onsets[i]["hist"], how) if i is not None \
                            else (young, -1, -1, -1, how)
                    else:
                        key = (young, -1, -1, -1, "untraced")
                    maps[key] = maps.get(key, 0.0) + sp
    if not runs:
        print("no runs")
        return 1
    print(f"# {runs} runs ({traced_runs} with the resampling trace), {snapshots} map snapshots")
    tot = sum(maps.values())
    young = sum(v for k, v in maps.items() if k[0] == 1)
    print(f"intra-oceanic front on the maps, mean km at a snapshot: {tot / max(snapshots, 1):.0f}; on floor younger than"
          f" {YOUNG_MYR:.0f} Myr {young / max(snapshots, 1):.0f} ({young / max(tot, 1e-9):.3f})")
    if not traced_runs:
        return 0
    how_tot = {}
    for k, v in maps.items():
        how_tot[(k[0], k[4])] = how_tot.get((k[0], k[4]), 0.0) + v
    for yv, name in ((1, "young"), (0, "old")):
        den = sum(v for (y, _), v in how_tot.items() if y == yv)
        print(f"  {name} floor: linked to its contact's onset by the pair " + "  ".join(
            f"{h} {how_tot.get((yv, h), 0.0) / max(den, 1e-9):.3f}" for h in ("pair", "lineage", "none")))
    print("# the intra-oceanic front on the maps by its contact's onset, shares of the young-floor / old-floor front"
          " (linked by the pair or its lineage)")
    for yv, name in ((1, f"floor under {YOUNG_MYR:.0f} Myr"), (0, f"floor {YOUNG_MYR:.0f} Myr or older")):
        sel = {k: v for k, v in maps.items() if k[0] == yv and k[4] in ("pair", "lineage")}
        den = max(sum(sel.values()), 1e-9)
        print(f"  {name} ({den / max(snapshots, 1):.0f} km at a snapshot)")
        for wi, w in enumerate(WHEN):
            row = [sum(v for k, v in sel.items() if k[1] == wi and k[2] == fi) / den for fi in range(3)]
            print(f"    {w:11s} " + "  ".join(f"{f} {x:.3f}" for f, x in zip(FLOOR, row)) + f"  (all {sum(row):.3f})")
        near_gap = sum(v for k, v in sel.items() if k[1] in (0, 1, 2) and k[2] == 0) / den
        print(f"    within two steps of a resampling on gap fill: {near_gap:.3f}; by the pair's history: " + "  ".join(
            f"{h} {sum(v for k, v in sel.items() if k[1] in (0, 1, 2) and k[2] == 0 and k[3] == hi) / den:.3f}"
            for hi, h in enumerate(HIST)))
    # The onsets that made intra-oceanic front, and what each class made over the run.
    made = [o for o in onset_rows if (o.get("front_km_myr") or [0.0])[0] > 0.0]
    tot_km = sum(o["front_km_myr"][0] for o in made)
    per = 1.0 / max(length_myr, 1e-9)
    print(f"# onsets that made intra-oceanic front ({len(made)} of {len(onset_rows)}); front made over the run, mean km"
          f" {tot_km * per:.0f} per run (the traced runs)")
    print("  share of the intra-oceanic front made (front_km_myr, a split's copies counted), by when and floor:")
    for wi, w in enumerate(WHEN):
        row = [sum(o["front_km_myr"][0] for o in made if o["when"] == wi and o["floor"] == fi) / max(tot_km, 1e-9)
               for fi in range(3)]
        n = sum(1 for o in made if o["when"] == wi)
        print(f"    {w:11s} {n:5d} onsets  " + "  ".join(f"{f} {x:.3f}" for f, x in zip(FLOOR, row)) +
              f"  (all {sum(row):.3f})")
    sub = [o for o in onset_rows if o["when"] == 1 and o["floor"] == 0 and o["setting"] == 0]
    if not sub:
        return 0
    print(f"# the ocean-ocean onsets on the step after a resampling on gap fill ({len(sub)}; all routes)")
    km = sum(o["front_km_myr"][0] for o in sub)
    print(f"  intra-oceanic front made: {km / max(tot_km, 1e-9):.3f} of all; passed incipient"
          f" {sum(1 for o in sub if o.get('subducting_after_myr', -1) >= 0) / len(sub):.3f}")
    for hi, h in enumerate(HIST):
        s2 = [o for o in sub if o["hist"] == hi]
        k2 = sum(o["front_km_myr"][0] for o in s2)
        print(f"  {h:19s} {len(s2) / len(sub):.3f} of the onsets, {k2 / max(km, 1e-9):.3f} of their front")
    print("  the boundary the step before: " + "  ".join(
        f"{k} {sum(1 for o in sub if KINDS[o['kind']] == k) / len(sub):.3f}" for k in KINDS[:6]))
    ages = [o["slab_age_myr"] for o in sub]
    at_fill = [o["slab_age_myr"] - o["since_resample_myr"] for o in sub]
    alpha = [a / o["interval"] for a, o in zip(at_fill, sub) if o["interval"] == o["interval"] and o["interval"] > 0]
    q = lambda xs: "  ".join(f"p{p} {np.percentile(xs, p):.2f}" for p in (10, 25, 50, 75, 90)) if xs else "-"
    print(f"  the floor's age at the onset, Myr: {q(ages)}")
    print(f"  its age when the fill made it, Myr: {q(at_fill)}")
    print(f"  the interval the fill closed, Myr: {q([o['interval'] for o in sub if o['interval'] == o['interval']])}")
    print(f"  alpha (the age at the fill over the interval): {q(alpha)}; under 0.1 {sum(1 for a in alpha if a < 0.1) / max(len(alpha), 1):.3f}"
          f", under 0.25 {sum(1 for a in alpha if a < 0.25) / max(len(alpha), 1):.3f}")
    ov = [o for o in sub if (o.get("overlap") or {}).get("n", 0) > 0]
    if ov:
        print(f"  the pair across the overlap at the onset ({len(ov)} read): converging"
              f" {sum(1 for o in ov if o['overlap']['pair_class'] == 0) / len(ov):.3f}, median"
              f" {statistics.median([o['overlap']['pair_conv_mm_yr'] for o in ov]):.1f} mm/yr")
    for hi, h in enumerate(HIST):
        s2 = [o for o in sub if o["hist"] == hi]
        if not s2:
            continue
        a2 = [o["slab_age_myr"] - o["since_resample_myr"] for o in s2]
        print(f"  {h}: the boundary the step before " + "  ".join(
            f"{k} {sum(1 for o in s2 if KINDS[o['kind']] == k) / len(s2):.3f}" for k in KINDS[:6]) +
              f"; age at the fill median {statistics.median(a2):.2f} Myr")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
