#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The crust standing behind the model's subduction fronts, as earthisl.py reads Earth's (no simulation):
forecrust.py <arm folder>[@N] ...

Reads each run's crust-budget maps (maps/t*.bin, maps/plate/) at -950 to -50 Myr, every 50 Myr; a snapshot taken at a
resampling has no front flags and is skipped. The front samples are the overriding side's samples of a subducting
front (the map flag 256, as oofronts.py takes them), each one lattice spacing of front. For each, the distance on its
own plate to the nearest visible sample whose whole crust (the map's thickness: every pool, sediment included, as
CRUST1.0's whole crust below water and ice) is at least T km, for T of 10 to 30 km: the share of the front with no
such crust within each reach is earthisl.py's "island km by threshold and reach, whole crust, own" over Earth's
51,309 km of SUB steps. Split as the record splits the fronts (PLT-370): intra-oceanic where the plate has no
continental crust within the arc's distance (200 km), continental otherwise. Pooled over the snapshots and runs.

Beside it, how long the front has stood where it is, read off the arc it built: the arc's magma (the map's island-arc
and continental-arc magma, m of rock a sample) on the own plate's samples in the arc's band, 140 to 260 km from the
front (`arc_distance_km` 200 with `arc_width_km` 60, PLT-042, PLT-043), its largest; at the arc's rate per km of front
R (km3/km/Myr) spread over the band's Gaussian, sqrt(pi) x 60 km wide, a front that stood T Myr has about
R T / 106 km at the axis (our arithmetic). And the floor's age under the forearc: the median age of the own plate's
oceanic samples within 200 km.
"""
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "crustcycle"))
import probe  # noqa: E402

FRONT = 256
R_KM = 6371.0
ARC_KM = 200.0  # ledger: PLT-042 (the arc's distance, the class's line, PLT-370)
THRESH_KM = [10.0, 15.0, 20.0, 25.0, 30.0]
REACH_KM = [77.0, 143.0, 200.0, 260.0, 400.0]  # earthisl.py's columns (77, 143: a model spacing and two at 16k)
TIMES = range(-950, 0, 50)
BAND_KM = (140.0, 260.0)  # the arc's band: arc_distance_km 200 +- arc_width_km 60 (PLT-042, PLT-043)


def xyz(a):
    la, lo = np.radians(a[:, 0]), np.radians(a[:, 1])
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def arc_km(chord):
    return 2.0 * np.arcsin(np.clip(chord / 2.0, 0.0, 1.0)) * R_KM


def own_distance(p, plate, idx, member):
    """For each front sample idx[j], the distance to the nearest sample of its own plate in `member`, km."""
    d = np.full(len(idx), np.inf)
    for q in np.unique(plate[idx]):
        m = np.nonzero(member & (plate == q))[0]
        if not len(m):
            continue
        sel = plate[idx] == q
        dd, _ = cKDTree(p[m]).query(p[idx[sel]])
        d[sel] = arc_km(dd)
    return d


def snapshot(run, t):
    nm = probe.name_of(t)
    if not (run / "maps" / nm).exists():
        return None
    a, _ = probe.read(run, t)
    plate = np.fromfile(run / "maps" / "plate" / nm, "<u4").astype(np.int64)
    if len(plate) != len(a):
        return None
    idx = np.nonzero((a[:, 10].astype(np.int64) & FRONT) > 0)[0]
    if not len(idx):
        return None
    p = xyz(a)
    io = own_distance(p, plate, idx, a[:, 4] > 0.5) >= ARC_KM
    dist = {th: own_distance(p, plate, idx, a[:, 3] >= th * 1000.0) for th in THRESH_KM}
    # The thickest crust of the own plate within the arc's distance, km.
    tree = cKDTree(p)
    near = tree.query_ball_point(p[idx], 2.0 * np.sin(ARC_KM / R_KM / 2.0))
    thick = np.array([max((a[k, 3] for k in ks if plate[k] == plate[i]), default=0.0)
                      for i, ks in zip(idx, near)]) / 1000.0
    age = np.fromfile(run / "maps" / "age" / nm, "<f4").astype(np.float64)
    fore_age = np.array([np.nanmedian([age[k] for k in ks if plate[k] == plate[i] and a[k, 4] < 0.5] or [np.nan])
                         for i, ks in zip(idx, near)])
    band = tree.query_ball_point(p[idx], 2.0 * np.sin(BAND_KM[1] / R_KM / 2.0))
    lo = 2.0 * np.sin(BAND_KM[0] / R_KM / 2.0)
    magma = np.array([max((a[k, 7] + a[k, 8] for k in ks
                           if plate[k] == plate[i] and np.linalg.norm(p[k] - p[i]) >= lo), default=0.0)
                      for i, ks in zip(idx, band)]) / 1000.0
    return io, dist, thick, fore_age, magma


def main(argv):
    for arg in argv:
        runs = probe.runs_of(arg)
        io_all, thick_all, age_all, magma_all = [], [], [], []
        dist_all = {th: [] for th in THRESH_KM}
        snaps = 0
        for run in runs:
            for t in TIMES:
                r = snapshot(run, t)
                if r is None:
                    continue
                snaps += 1
                io_all.append(r[0])
                thick_all.append(r[2])
                age_all.append(r[3])
                magma_all.append(r[4])
                for th in THRESH_KM:
                    dist_all[th].append(r[1][th])
        if not snaps:
            print(f"== {arg}: no snapshots with fronts")
            continue
        io = np.concatenate(io_all)
        thick = np.concatenate(thick_all)
        fore_age = np.concatenate(age_all)
        magma = np.concatenate(magma_all)
        dist = {th: np.concatenate(v) for th, v in dist_all.items()}
        print(f"== {arg}: {len(runs)} runs, {snaps} snapshots, {len(io)} front samples "
              f"({io.mean():.3f} intra-oceanic)")
        for name, sel in (("all fronts", np.ones(len(io), bool)), ("intra-oceanic", io), ("continental", ~io)):
            print(f"  {name}: share of the front with no crust of T km on its own plate within the reach")
            print("     T " + "".join(f"{r:>8.0f}" for r in REACH_KM))
            for th in THRESH_KM:
                d = dist[th][sel]
                print(f"  {th:>4.0f} " + "".join(f"{(d > r).mean():>8.3f}" for r in REACH_KM))
            tk = thick[sel]
            q = np.percentile(tk, [10, 25, 50, 75, 90])
            print("    thickest crust within 200 km, km, p10 p25 p50 p75 p90: " + " ".join(f"{x:.1f}" for x in q))
            q = np.percentile(magma[sel], [10, 25, 50, 75, 90])
            print("    most arc magma on a sample 140 to 260 km behind, km, p10 p25 p50 p75 p90: "
                  + " ".join(f"{x:.2f}" for x in q))
            fa = fore_age[sel]
            fa = fa[np.isfinite(fa)]
            if len(fa):
                q = np.percentile(fa, [10, 25, 50, 75, 90])
                print(f"    median floor age within 200 km, Myr ({len(fa) / sel.sum():.2f} of the front has floor there), "
                      "p10 p25 p50 p75 p90: " + " ".join(f"{x:.0f}" for x in q))


if __name__ == "__main__":
    main(sys.argv[1:])
