#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How small terranes dock, from the crust-budget maps (the docking lane, 2026-10-30). Measurement only,
cached pool folders, nothing simulated.

    python3 -I dock_maps.py <arm folder> <out.jsonl> [workers] [run,run,...]
    python3 -I dock_maps.py --summary <out.jsonl> [<out.jsonl>...]   # medians and means per field, side by side

<arm folder> as dock_records.py's (runs with maps/, DEEPTIME_CRUST_BUDGET_MAPS). Per run:

1. What the small terranes are made of (`makeup`): at every map snapshot from -900 to -50 Myr, the
   connected bodies of continental crust of one plate (samples of the same plate within 1.6 sample
   spacings, cc.components' rule) under 50 samples that are not their plate's largest body: their own
   crust (thickness less the ocean basement a converted arc keeps under it, PLT-331, from pools/), its
   quartiles and its shares under 15 km (Cloos 1993's ~15 km, PLT-047), 15 to 25, 25 to 35 and 35 km or
   more; the share of their samples whose arc magma is island-arc magma (arc_oceanic_m >= 1 km and
   above the continental arc's: the m5fields rule); their height over the snapshot's sea.
2. Which collisions make the coast with high ground (`m5`): the final map (t0000) on 100,000 Fibonacci
   points (m5maps.py's way: the nearest sample, its elevation less the last step's sea), the census's
   coast and high ground (m5lib), and every high point with a collision orogeny matched to the
   collision state that stamped it: the states (hand-offs and partings of rec.json, open ones included)
   on the point's plate (or a plate that welded into it later) whose window holds the stamp's time
   (t - orogeny age, within a step), the overriding plate first, then the incoming one; of several, the
   one ending nearest the stamp. The qualified coast is split by the class of its nearest qualifying
   high point (the hand-off's terrane size lt50 / 50_200 / 200_1000 / ge1000, partings, open states,
   unmatched collision, Andean, other), and M5 is recomputed without each class (`m5_without`).
"""
import json
import sys
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "m5diag"))
sys.path.insert(0, str(HERE.parent))
import m5lib  # noqa: E402
from m5lib import cc  # noqa: E402

FIELDS = 11
POOLS = 9
SNAP = list(range(-900, 0, 50))
NPTS = 100_000
CLASSES = ["lt50", "50_200", "200_1000", "ge1000", "parting", "open", "coll_unmatched", "andean", "other"]
SIZES = (("lt50", 0, 50), ("50_200", 50, 200), ("200_1000", 200, 1000), ("ge1000", 1000, 10**12))


def load(run, t):
    name = f"t{t:04.0f}.bin"
    a = np.fromfile(run / "maps" / name, "<f4").reshape(-1, FIELDS).astype(np.float64)
    pl = np.fromfile(run / "maps" / "plate" / name, "<u4").astype(np.int64)
    pf = run / "maps" / "pools" / name
    pools = np.fromfile(pf, "<f4").reshape(-1, POOLS).astype(np.float64) if pf.exists() else None
    return a, pl, pools


def sea_at(rec, t):
    ts = np.array([s["t"] for s in rec["steps"]])
    sea = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
    return float(sea[int(np.argmin(np.abs(ts - t)))])


def bodies(p, cont, plate, spacing_km):
    """Continental bodies of one plate each: a label per sample (-1 off continental crust)."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    idx = np.nonzero(cont)[0]
    out = np.full(cont.size, -1)
    pairs = cKDTree(p[idx]).query_pairs(cc.chord(1.6 * spacing_km), output_type="ndarray")
    same = plate[idx[pairs[:, 0]]] == plate[idx[pairs[:, 1]]]
    pairs = pairs[same]
    g = coo_matrix((np.ones(len(pairs)), (pairs[:, 0], pairs[:, 1])), shape=(idx.size, idx.size))
    _, lab = connected_components(g, directed=False)
    out[idx] = lab
    return out


def makeup(run, rec):
    own_all, isl_all, hgt_all, n_small, mix_all = [], [], [], [], []
    for t in SNAP:
        try:
            a, pl, pools = load(run, t)
        except FileNotFoundError:
            continue
        p = cc.xyz(a[:, 0], a[:, 1])
        sp = float(np.sqrt(cc.EARTH_KM2 / a.shape[0]))
        cont = a[:, 4] > 0.5
        lab = bodies(p, cont, pl, sp)
        sizes = np.bincount(lab[cont])
        # Each plate's largest body.
        largest = {}
        for b in np.unique(lab[cont]):
            pid = int(pl[np.nonzero(lab == b)[0][0]])
            if pid not in largest or sizes[b] > sizes[largest[pid]]:
                largest[pid] = b
        big = set(largest.values())
        small = [b for b in range(sizes.size) if sizes[b] < 50 and b not in big]
        n_small.append(len(small))
        sel = np.isin(lab, small)
        basement = pools[:, 7] if pools is not None else np.zeros(a.shape[0])
        own = (a[:, 3] - basement)[sel] / 1000.0
        isl = ((a[:, 7] >= 1000.0) & (a[:, 7] > a[:, 8]))[sel]
        own_all.append(own)
        isl_all.append(isl)
        hgt_all.append(a[sel, 2] - sea_at(rec, t))
        if pools is not None:
            mix_all.append(pools[sel, :7])
    own = np.concatenate(own_all) if own_all else np.zeros(0)
    isl = np.concatenate(isl_all) if isl_all else np.zeros(0, bool)
    hgt = np.concatenate(hgt_all) if hgt_all else np.zeros(0)
    if own.size == 0:
        return {}
    mix = np.concatenate(mix_all) if mix_all else np.zeros((0, 7))
    tot = np.maximum(mix.clip(0).sum(1), 1.0)

    def share(cols):
        return float(np.median(mix[:, cols].clip(0).sum(1) / tot)) if mix.size else None

    def vol_share(cols):
        return float(mix[:, cols].clip(0).sum() / max(mix.clip(0).sum(), 1.0)) if mix.size else None

    return {
        # The own crust's rock (pools: sediment, sedimentary, metamorphic, felsic plutonic and
        # volcanic, mafic volcanic and plutonic; the basement apart): the medians of each sample's
        # shares, and the shares of the summed volume.
        "mix_median": {"sed": share([0, 1]), "meta": share([2]), "felsic": share([3, 4]),
                       "mafic": share([5, 6])},
        "mix_volume": {"sed": vol_share([0, 1]), "meta": vol_share([2]), "felsic": vol_share([3, 4]),
                       "mafic": vol_share([5, 6])},
        "small_bodies_per_snapshot": float(np.median(n_small)),
        "samples": int(own.size),
        "own_km_q": [float(np.percentile(own, x)) for x in (10, 25, 50, 75, 90)],
        "own_lt15": float((own < 15).mean()), "own_15_25": float(((own >= 15) & (own < 25)).mean()),
        "own_25_35": float(((own >= 25) & (own < 35)).mean()), "own_ge35": float((own >= 35).mean()),
        "island_arc_share": float(isl.mean()),
        "own_km_island_arc_q": [float(np.percentile(own[isl], x)) for x in (25, 50, 75)] if isl.any() else None,
        "own_km_other_q": [float(np.percentile(own[~isl], x)) for x in (25, 50, 75)] if (~isl).any() else None,
        "height_m_q": [float(np.percentile(hgt, x)) for x in (25, 50, 75)],
        "dry_share": float((hgt > 0).mean()),
    }


def states(rec):
    out = []
    for c in rec["collisions"]:
        cls = next(n for n, lo, hi in SIZES if lo <= c["terrane_samples"] < hi)
        out.append((c["start_myr"], c["end_myr"], c["under"], c["over"], cls, c.get("area_km2") or 0.0))
    for p in rec["partings"]:
        cls = "open" if p.get("reason") == "open" else "parting"
        out.append((p["start_myr"], p["end_myr"], p["under"], p["over"], cls, p.get("area_km2") or 0.0))
    return out


def welded_into(rec):
    """plate -> the plates that welded into it (directly or through others)."""
    into = {}
    for c in rec["collisions"]:
        if c.get("sutured") and c.get("died") is not None:
            keep = c["over"] if c["died"] == c["under"] else c["under"]
            into.setdefault(keep, set()).add(c["died"])
    def anc(pid, seen=None):
        seen = seen or {pid}
        for d in into.get(pid, ()):
            if d not in seen:
                seen.add(d)
                anc(d, seen)
        return seen
    return anc


def m5(run, rec):
    a, pl, _ = load(run, 0)
    sea = sea_at(rec, 0.0)
    dt = rec["dt_myr"]
    flat, flon = m5lib.fib(NPTS)
    pf = cc.xyz(flat, flon)
    sp = float(np.sqrt(cc.EARTH_KM2 / NPTS))
    _, nn = cKDTree(cc.xyz(a[:, 0], a[:, 1])).query(pf)
    oro = np.rint(a[nn, 5]).astype(int)
    age = a[nn, 6]
    z = a[nn, 2] - sea
    plate = pl[nn]
    st = states(rec)
    anc = welded_into(rec)
    cls = np.full(NPTS, CLASSES.index("other"))
    cls[oro == 1] = CLASSES.index("andean")
    coll = np.nonzero((oro == 2) & (z > 0.0))[0]
    cache = {}
    ambiguous = 0
    for i in coll:
        key = (int(plate[i]), round(float(age[i]) / dt))
        if key not in cache:
            ts = 0.0 - float(age[i])
            fam = anc(int(plate[i]))
            cands = []
            for (s0, s1, u, o, c, ar) in st:
                if s0 - dt <= ts <= s1 + dt:
                    if o in fam:
                        cands.append((0, abs(s1 - ts), -ar, c))
                    elif u in fam:
                        cands.append((1, abs(s1 - ts), -ar, c))
            cands.sort()
            cache[key] = (cands[0][3] if cands else "coll_unmatched", len(cands))
        c, nc = cache[key]
        ambiguous += nc > 1
        cls[i] = CLASSES.index(c)
    cls[(oro == 2) & (z <= 0.0)] = CLASSES.index("coll_unmatched")
    det = m5lib.coast_detail(pf, z, sp, cls=cls, ncls=len(CLASSES))
    land_all = z > 0
    # Each collision class's land: its share of the land, its height over the sea (quartiles), its
    # share above 2 km and its median distance to the sea.
    heights = {}
    for j, k in enumerate(CLASSES[:6]):
        m = land_all & (cls == j)
        if m.sum() == 0:
            heights[k] = None
            continue
        heights[k] = {"land_share": float(m.sum() / max(land_all.sum(), 1)),
                      "z_q": [float(np.percentile(z[m], x)) for x in (25, 50, 75, 90)],
                      "above2km": float((z[m] > 2000).mean()),
                      "dsea_km_median": float(np.median(det["d_sea"][m]))}
    D = det["D"]
    C = D.shape[0]
    dmin = D.min(1)
    qual = np.isfinite(dmin)
    near = np.argmin(D, 1)
    land = det["land"]
    L = int(land.sum())
    return {
        "M5": float(qual.sum() / max(C, 1)), "M4": float((z[land] > 2000).mean()) if L else 0.0,
        "m5_by_nearest": {k: float((qual & (near == j)).sum() / max(C, 1)) for j, k in enumerate(CLASSES)},
        "m5_without": {k: float(np.isfinite(np.delete(D, j, 1)).any(1).sum() / max(C, 1))
                       for j, k in enumerate(CLASSES)},
        "m5_without_small": float(np.isfinite(np.delete(D, [0, 1], 1)).any(1).sum() / max(C, 1)),
        "m4_by": {k: float((det["hi"] & (cls == j)).sum() / max(L, 1)) for j, k in enumerate(CLASSES)},
        "heights": heights,
        "coll_hi_points": int(coll.size),
        "coll_hi_ambiguous": float(ambiguous / max(coll.size, 1)),
        # The small terranes' qualifying high ground by the age of its stamp (Myr).
        "m5_small_by_age": {
            f"{a0:g}-{a1:g}": float((qual & np.isin(near, [0, 1])
                                     & (age[det["J"][np.arange(C), near]] >= a0)
                                     & (age[det["J"][np.arange(C), near]] < a1)).sum() / max(C, 1))
            for a0, a1 in ((-1e9, 2.5), (2.5, 10), (10, 30), (30, 1e9))},
    }


def one(run):
    rec = json.loads((run / "rec.json").read_text())
    out = {"run": run.name}
    out["makeup"] = makeup(run, rec)
    out["m5"] = m5(run, rec)
    return out


def flat(d, prefix=""):
    """{"a": {"b": 1}} as {"a.b": 1}; lists of numbers are kept whole."""
    out = {}
    for k, v in d.items():
        if isinstance(v, dict):
            out.update(flat(v, f"{prefix}{k}."))
        else:
            out[f"{prefix}{k}"] = v
    return out


def summary(paths):
    """Medians and means over runs of every field of one or more out.jsonl files, side by side."""
    tables = []
    for p in paths:
        rows = [flat(json.loads(line)) for line in Path(p).read_text().splitlines() if line.strip()]
        tables.append(rows)
    keys = [k for k in flat(json.loads(Path(paths[0]).read_text().splitlines()[0])) if k != "run"]
    print("field", *[f"{Path(p).stem} (n={len(t)}) median / mean" for p, t in zip(paths, tables)], sep="\t")
    for k in keys:
        cells = []
        for rows in tables:
            vals = [r[k] for r in rows if k in r and r[k] is not None]
            if not vals:
                cells.append("-")
            elif isinstance(vals[0], list):
                a = np.array([v for v in vals if len(v) == len(vals[0])], dtype=float)
                cells.append("[" + ", ".join(f"{x:.4g}" for x in np.median(a, axis=0)) + "]")
            else:
                a = np.array(vals, dtype=float)
                cells.append(f"{np.median(a):.4g} / {a.mean():.4g}")
        print(k, *cells, sep="\t")


def main():
    if sys.argv[1] == "--summary":
        summary(sys.argv[2:])
        return
    arm = Path(sys.argv[1])
    dest = Path(sys.argv[2])
    workers = int(sys.argv[3]) if len(sys.argv) > 3 else 3
    runs = sorted(p for p in arm.glob("*-*") if p.is_dir() and (p / "maps" / "t0000.bin").exists())
    if len(sys.argv) > 4:
        runs = [p for p in runs if p.name in sys.argv[4].split(",")]
    print(f"{len(runs)} runs", flush=True)
    with Pool(workers) as pool, dest.open("w") as fh:
        for o in pool.imap_unordered(one, runs):
            fh.write(json.dumps(o) + "\n")
            fh.flush()
            print(o["run"], f"M5 {o['m5']['M5']:.3f}", flush=True)
    print("DOCKMAPS_DONE")


if __name__ == "__main__":
    main()
