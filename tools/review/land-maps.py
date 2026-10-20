#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The land-shape task's map reading (build log 2026-10-20): from the crust budget's maps
(DEEPTIME_CRUST_BUDGET_MAPS, crates/deeptime-core/src/plates/crust_budget.rs MAP_FIELDS), the
shape of the land and of the continents at the end of a run: the pieces of land (neighbours within
1.5 lattice spacings) and how many hold 90 % of it, the same for the continental crust, the plates
a continent is cut into, the land above 2 km by the age of its last orogeny, and where the land
stands from the continent's own edge (the distance from each dry sample to the nearest oceanic
sample, against its distance to the sea).

    python3 -I tools/review/land-maps.py <maps dir> <sea level m> [<t Myr>]

Needs numpy and scipy.
"""
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components
from scipy.spatial import cKDTree

R = 6371.0
FIELDS = 11


def pieces(xyz, mask, r):
    idx = np.flatnonzero(mask)
    if len(idx) == 0:
        return idx, np.array([], dtype=int), 0
    pairs = cKDTree(xyz[idx]).query_pairs(r, output_type="ndarray")
    n = len(idx)
    g = coo_matrix((np.ones(len(pairs)), (pairs[:, 0], pairs[:, 1])), shape=(n, n))
    k, lab = connected_components(g, directed=False)
    return idx, lab, k


def holding(lab, k, share=0.9):
    sizes = np.sort(np.bincount(lab, minlength=k))[::-1]
    c = np.cumsum(sizes) / sizes.sum()
    return int(np.searchsorted(c, share) + 1), float(sizes[0] / sizes.sum())


def dist_km(tree, pts):
    d, _ = tree.query(pts)
    return 2 * np.arcsin(np.minimum(d / 2, 1)) * R


def main():
    folder, sea = Path(sys.argv[1]), float(sys.argv[2])
    files = sorted(folder.glob("t*.bin"), key=lambda f: float(f.stem[1:]))
    f = files[-1] if len(sys.argv) < 4 else folder / f"t{int(sys.argv[3]):04d}.bin"
    a = np.fromfile(f, dtype="<f4").reshape(-1, FIELDS).astype(np.float64)
    lat, lon, z, t, kind, orog, age, arc_o, arc_c, group, flags = a.T
    la, lo = np.radians(lat), np.radians(lon)
    xyz = np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], axis=1)
    n_all = len(z)
    spacing = np.sqrt(4 * np.pi / n_all * 2 / np.sqrt(3))
    r = 1.5 * spacing
    land = z > sea
    cont = kind > 0.5
    print(f"{f.name}: {n_all} visible samples, land {land.mean():.4f}, continental {cont.mean():.4f}, "
          f"dry share {np.sum(land & cont) / max(1, cont.sum()):.3f}")
    idx, lab, k = pieces(xyz, land, r)
    h, big = holding(lab, k)
    print(f"land pieces: {k}, holding 90 %: {h}, largest {big:.3f} of the land")
    cidx, clab, ck = pieces(xyz, cont, r)
    ch, cbig = holding(clab, ck)
    print(f"continental pieces: {ck}, holding 90 %: {ch}, largest {cbig:.3f} of the continental crust")
    # The plates (groups) of the largest continental pieces holding 90 % of it.
    sizes = np.bincount(clab, minlength=ck)
    order = np.argsort(sizes)[::-1][:ch]
    rows = []
    for p in order:
        members = cidx[clab == p]
        g, cnt = np.unique(group[members], return_counts=True)
        dry = np.mean(land[members])
        rows.append(f"{len(members) / cont.sum():.3f} ({len(g)} groups, the largest {cnt.max() / len(members):.2f}; dry {dry:.2f})")
    print("  the pieces holding 90 %: " + "; ".join(rows))
    # Coast distance and the continent's own edge.
    sea_tree = cKDTree(xyz[~land])
    ocean_tree = cKDTree(xyz[~cont])
    d_sea = dist_km(sea_tree, xyz[land])
    d_ocean = dist_km(ocean_tree, xyz[land])
    print(f"land to the sea p50 {np.median(d_sea):.0f} km; to the ocean floor (non-continental) p50 "
          f"{np.median(d_ocean):.0f} km; continental crust to the ocean floor p50 "
          f"{np.median(dist_km(ocean_tree, xyz[cont])):.0f} km")
    # Drowned continental crust: how far from the ocean floor it lies.
    wet = cont & ~land
    if wet.any():
        dw = dist_km(ocean_tree, xyz[wet])
        print(f"drowned continental crust {wet.sum() / cont.sum():.3f} of it: to the ocean floor p10/50/90 "
              f"{np.percentile(dw, 10):.0f}/{np.median(dw):.0f}/{np.percentile(dw, 90):.0f} km, "
              f"thickness p50 {np.median(t[wet]) / 1000:.1f} km, inside 800 km of the ocean floor {np.mean(dw < 800):.2f}")
    # The high ground by the age of its last orogeny (orogeny 0: none, age meaningless).
    hi = land & (z > sea + 2000)
    print(f"land above 2 km: {hi.sum() / max(1, land.sum()):.3f} of the land")
    for name, sel in [("no orogeny", orog == 0), ("Andean", orog == 1), ("Himalayan", orog == 2)]:
        s = hi & sel
        if not s.any():
            continue
        ages = age[s]
        print(f"  {name}: {s.sum() / hi.sum():.3f} of it; orogeny age p50 {np.median(ages):.0f} Myr, "
              f"older than 100 / 200 / 400 Myr {np.mean(ages > 100):.2f} / {np.mean(ages > 200):.2f} / {np.mean(ages > 400):.2f}; "
              f"thickness p50 {np.median(t[s]) / 1000:.1f} km")
    island = (arc_o >= 1000) & (arc_o > arc_c)
    print(f"  island-arc crust: {np.sum(hi & island) / max(1, hi.sum()):.3f} of it, "
          f"{np.sum(land & island) / max(1, land.sum()):.3f} of the land")
    # Thickness of the land, and of the dry continental crust by distance to the ocean floor.
    tl = t[land & cont] / 1000
    print(f"dry continental crust thickness p10/25/50/75/90: "
          + "/".join(f"{np.percentile(tl, q):.1f}" for q in (10, 25, 50, 75, 90)) + " km")
    tc = t[cont] / 1000
    print(f"all visible continental crust thickness p10/25/50/75/90: "
          + "/".join(f"{np.percentile(tc, q):.1f}" for q in (10, 25, 50, 75, 90)) + " km")


if __name__ == "__main__":
    main()
