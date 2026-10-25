#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""floorage_th.py <arm folder>...: where the old floor sits, from the floor's thickness.

Bare oceanic crust is born 7 km thick (pools.rs oceanic_column: 2 km mafic volcanic, 5 km plutonic) with pelagic
sediment of 5 m/Myr times its age (ridges.rs, PLT-103), and gains 5 m/Myr while it is visible (pools.rs). Everything
else that lands on the floor (the continents' sediment, arc magma, hotspot crust) only adds thickness, so
(thickness - 7 km - arc magma) / 5 m/Myr is an upper bound on the age of oceanic crust; it is read only on crust under
10 km (600 Myr of pelagic sediment). The bound is checked one-sidedly against the record's age histogram at the same
time: the proxy's count under each age, over the thin floor, can never exceed the record's count under it.

Then the old floor (over 180 Myr; the record's exact count O) splits by place: the thin floor's proxy-old count U is
an upper bound on the thin floor's true old count, so O - U is a lower bound on the old floor in the thick crust.

One JSON object per run on stdout, counts summed over the run's snapshots (t >= -800, not at a resampling).

The check FAILS (2026-10-25): over the thin floor the proxy finds 1.6 to 1.9 times the record's count under 10 Myr,
and 4 % of the thin floor reads under 7 km, so some floor lost sediment or crust and the proxy is no upper bound; its
split of the old floor by place is not to be used. The areas (the ocean and the thick crust by distance) are exact."""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

FIELDS = 11
R_KM = 6371.0
TRENCH, ARC, HOT, DIV = 2, 4, 8, 16
CRUST0_M = 7000.0
PELAGIC_M_PER_MYR = 5.0
THIN_M = 10000.0
OLD = 180.0
CUM = [10, 30, 60, 100, 150, 180, 250]
DC = [0, 330, 1000, 2000, 1e9]
DT = [0, 500, 1000, 1500, 1e9]
TH = [0, 8000, 10000, 15000, 25000, 1e9]


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def nearest_km(p_from, p_to):
    if len(p_to) == 0 or len(p_from) == 0:
        return np.full(len(p_from), np.inf)
    d, _ = cKDTree(p_to).query(p_from)
    return 2.0 * R_KM * np.arcsin(np.minimum(d / 2.0, 1.0))


def band(x, edges):
    return np.clip(np.searchsorted(edges, x, side="right") - 1, 0, len(edges) - 2)


def counts(sel, idx, n):
    return np.bincount(idx[sel], minlength=n).astype(int).tolist()


def add(acc, key, vals):
    if key not in acc:
        acc[key] = [0] * len(vals)
    acc[key] = [a + b for a, b in zip(acc[key], vals)]


def run_part(run, r):
    N = r["samples"]
    ages = r["ages"]
    age_t = np.array([a["t"] for a in ages])
    acc = {"snapshots": 0}
    for path in sorted((run / "maps").glob("t*.bin")):
        t = float(path.stem[1:])
        if t < -800.0:
            continue
        a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
        n = len(a)
        if n >= N:
            continue
        pl_path = run / "maps" / "plate" / path.name
        if not pl_path.exists():
            continue
        plate = np.fromfile(pl_path, "<u4").astype(np.int64)
        if len(plate) != n:
            continue
        k = int(np.argmin(np.abs(age_t - t)))
        if abs(age_t[k] - t) > 1e-6:
            continue
        hist = ages[k]["hist"]
        p = xyz(a[:, 0], a[:, 1])
        th = a[:, 3]
        cont = a[:, 4] > 0.5
        oc = ~cont
        flags = np.rint(a[:, 10]).astype(np.int64)
        arc_m = a[:, 7] + a[:, 8]
        age = (th - CRUST0_M - arc_m) / PELAGIC_M_PER_MYR
        thin = oc & (th < THIN_M)
        thick = oc & ~thin
        acc["snapshots"] += 1
        add(acc, "ocean", [int(oc.sum())])
        add(acc, "record_old", [int(sum(hist[int(OLD // 10):]))])
        add(acc, "record_hist_sum", [int(sum(hist))])
        add(acc, "record_cum", [int(sum(hist[: c // 10])) for c in CUM])
        add(acc, "proxy_cum_thin", [int((thin & (age < c)).sum()) for c in CUM])
        add(acc, "proxy_negative_thin", [int((thin & (age < 0)).sum())])
        add(acc, "thin", [int(thin.sum())])
        old_thin = thin & (age >= OLD)
        add(acc, "proxy_old_thin", [int(old_thin.sum())])
        # Distances: to any continental crust, to the plate's own, to a trench, to a divergent boundary.
        dc = nearest_km(p, p[cont])
        dt = nearest_km(p, p[(flags & TRENCH) != 0])
        dco = np.full(n, np.inf)
        ptr = np.zeros(n, bool)
        for pid in np.unique(plate):
            m = plate == pid
            mc, mo = m & cont, m & oc
            ptr[m] = bool((m & ((flags & TRENCH) != 0)).any())
            if mc.any() and mo.any():
                dco[mo] = nearest_km(p[mo], p[mc])
        bdc, bdco, bdt, bth = band(dc, DC), band(dco, DC), band(dt, DT), band(th, TH)
        for name, sel in (("oc", oc), ("thin", thin), ("thick", thick), ("old_thin", old_thin)):
            add(acc, f"{name}_dc", counts(sel, bdc, 4))
            add(acc, f"{name}_dco", counts(sel, bdco, 4))
            add(acc, f"{name}_dt", counts(sel, bdt, 4))
            add(acc, f"{name}_th", counts(sel, bth, 5))
            add(acc, f"{name}_notrench", [int((sel & ~ptr).sum())])
        # The proxy's age bands on the thin floor beside the plate's own continent and away from it.
        for nm, sel in (("near", thin & (dco <= 330.0)), ("far", thin & (dco > 330.0))):
            add(acc, f"thin_{nm}_cum", [int((sel & (age < c)).sum()) for c in CUM] + [int(sel.sum())])
    return acc


def main():
    for arm in sys.argv[1:]:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file() and (p / "maps").is_dir()):
            r = json.loads((run / "rec.json").read_text())
            print(json.dumps({"arm": str(Path(arm)), "run": run.name, "acc": run_part(run, r)}), flush=True)


main()
