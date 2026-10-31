#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The sea's rise split into its terms on cached pool runs (no simulation):

    sea_split.py <arm folder> <base folder>

Each folder is a pool run set (`.../pool/<key>/<preset>-<length>/<argkey>`); runs supercontinent-1..4 and
scattered_continents-1..4, the final snapshot of the crust budget's maps (`maps/t0000.bin`, `maps/age/t0000.bin`).
The samples are equal areas. Per run, the water W (m as a layer) is what its own sea (rec.json's last `sea_m`) holds
over its samples' heights, and the sea is filled again with that water over the ground with one term undone at a
time (a static reading, not a counterfactual run):

  S_A  the ground as it stands (= the run's sea)
  S_B  ocean crust under 10 km at GDH1's bare floor of its age (pools::gdh1_depth_m): A - B is the floor's height
       over its age's curve
  S_C  and ocean crust of 10 km or more there too: B - C is the thick ocean crust over its bare floor (plateaus,
       seamount chains, island arcs, stacked crust)
  S_D  and every ocean sample at one depth, the base arm's mean bare-floor depth: C - D is the floor's age
       distribution against the base's
  S_D  itself: the continents' hypsometry (area, heights) and the water

The arm's mean of each term less the base's splits the arm's rise of the sea (means, so the terms add up). Then the
floor's lift beside thick crust: for ocean samples of the ocean's own column (under 7.2 km), the mean height over
GDH1's curve by the share of their neighbours within 150 km that are ocean crust 7.2 km or more (seamounts, the
tails' and heads' rock, arcs). Needs numpy and scipy (the VM's .pyenv-numpy, run with -I).
"""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

NF = 11
RUNS = [f"{s}-{n}" for s in ("supercontinent", "scattered_continents") for n in (1, 2, 3, 4)]
R_KM = 6371.0


def gdh1_depth(age):
    t = np.maximum(np.nan_to_num(age, nan=0.0), 0.0)
    return np.where(t < 20.0, 2600.0 + 365.0 * np.sqrt(t), 5651.0 - 2473.0 * np.exp(-0.0278 * t))


def load(run):
    m = np.fromfile(run / "maps" / "t0000.bin", "<f4").reshape(-1, NF).astype(np.float64)
    age = np.fromfile(run / "maps" / "age" / "t0000.bin", "<f4").astype(np.float64)
    sea = float(json.loads((run / "rec.json").read_text())["steps"][-1]["sea_m"])
    return m, age, sea


def water(z, sea):
    return float(np.maximum(sea - z, 0.0).mean())


def fill(z, w):
    """The level s with mean((s - z)+) = w."""
    zs = np.sort(z)
    n = len(zs)
    cum = np.cumsum(zs)
    # Over the k lowest: s*k - cum[k-1] = w*n, with zs[k-1] <= s <= zs[k].
    for k in range(1, n + 1):
        s = (w * n + cum[k - 1]) / k
        if k == n or s <= zs[k]:
            return float(s)
    return float("nan")


def ocean_bare_depth(folder):
    ds = []
    for r in RUNS:
        m, age, _ = load(folder / r)
        oc = m[:, 4] <= 0.5
        ds.append(float(gdh1_depth(age[oc]).mean()))
    return float(np.mean(ds))


def terms(run, d_ref):
    m, age, sea = load(run)
    z, th = m[:, 2], m[:, 3]
    oc = m[:, 4] <= 0.5
    w = water(z, sea)
    bare = -gdh1_depth(age)
    s_a = fill(z, w)
    z1 = np.where(oc & (th < 10000.0), bare, z)
    s_b = fill(z1, w)
    z2 = np.where(oc, bare, z)
    s_c = fill(z2, w)
    z3 = np.where(oc, -d_ref, z)
    s_d = fill(z3, w)
    return {"sea": sea, "S_A": s_a, "floor_over_gdh1": s_a - s_b, "thick_over_bare": s_b - s_c,
            "floor_age": s_c - s_d, "S_D": s_d, "water_m": w, "cont_share": float((~oc).mean())}


def beside(run):
    m, age, _ = load(run)
    lat, lon, z, th = np.radians(m[:, 0]), np.radians(m[:, 1]), m[:, 2], m[:, 3]
    oc = m[:, 4] <= 0.5
    p = np.stack([np.cos(lat) * np.cos(lon), np.cos(lat) * np.sin(lon), np.sin(lat)], -1)
    tree = cKDTree(p)
    o7 = np.nonzero(oc & (th < 7200.0))[0]
    thick = oc & (th >= 7200.0)
    res = z + gdh1_depth(age)
    chord = 2.0 * np.sin(150.0 / R_KM / 2.0)
    nb = tree.query_ball_point(p[o7], chord)
    share = np.array([thick[np.array(a)].mean() if len(a) else 0.0 for a in nb])
    out = {}
    for lo, hi in [(0.0, 1e-9), (1e-9, 0.25), (0.25, 0.5), (0.5, 1.01)]:
        sel = (share >= lo) & (share < hi) if lo > 0 else share <= hi
        out[f"nb{lo:g}"] = (float(sel.mean()), float(res[o7][sel].mean()) if sel.any() else float("nan"))
    return out


def main():
    arm, base = Path(sys.argv[1]), Path(sys.argv[2])
    d_ref = ocean_bare_depth(base)
    print(f"the base's mean bare-floor depth {d_ref:.0f} m")
    means = {}
    for name, folder in (("arm", arm), ("base", base)):
        rows = [terms(folder / r, d_ref) for r in RUNS]
        keys = list(rows[0])
        print(f"== {name} {folder}")
        print("run " + " ".join(keys))
        for r, v in zip(RUNS, rows):
            print(r + " " + " ".join(f"{v[k]:.4f}" if abs(v[k]) < 1 else f"{v[k]:.0f}" for k in keys))
        means[name] = {k: float(np.mean([v[k] for v in rows])) for k in keys}
        print("mean " + " ".join(f"{means[name][k]:.4f}" if abs(means[name][k]) < 1 else f"{means[name][k]:.0f}"
                                 for k in keys))
    print("== the arm's rise of the sea, means, arm less base")
    for k in ("sea", "floor_over_gdh1", "thick_over_bare", "floor_age", "S_D"):
        print(f"{k} {means['arm'][k] - means['base'][k]:+.0f}")
    print("== the 7 km floor's height over GDH1 by its neighbours' thick share (share of samples, mean m)")
    for name, folder in (("arm", arm), ("base", base)):
        acc = {}
        for r in RUNS:
            for k, (f, res) in beside(folder / r).items():
                acc.setdefault(k, []).append((f, res))
        print(name + " " + " ".join(f"{k}: {np.mean([a for a, _ in v]):.3f} {np.nanmean([b for _, b in v]):.0f}"
                                    for k, v in acc.items()))


if __name__ == "__main__":
    main()
