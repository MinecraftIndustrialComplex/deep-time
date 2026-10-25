#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""floorage_age.py <arm folder>...: where the sea floor of each age sits, from the crust-budget age map.

Reads each run folder's maps (maps/t*.bin, maps/plate/t*.bin and maps/age/t*.bin, the last written since 2026-10-25,
crust_budget.rs write_ages: one f32 per map record, the floor's age, NaN on continental crust) and rec.json (the checks).
Prints one JSON object per run on stdout. Needs numpy and scipy (the VM's pool numpy env).

Per snapshot from t = -500 Myr on (none taken at a resampling), the visible oceanic samples by age band (AGE_EDGES)
and by place, the places not exclusive:
  margin    within the margin of its own plate's continent (PLT-503: arc distance + width + one spacing, 331 km on the
            16k lattice), the floor initiation keeps on the parent;
  interior  farther than 1,000 km from every TRENCH-flagged visible sample (PLT-064's clearance);
  giant     on a plate over 0.2 of the planet;
  notrench  on a plate none of whose visible samples is TRENCH-flagged (it subducts nowhere);
  nofront   on a plate with no TRENCH, front-subduction or front-collision flag (it touches no convergent front);
  and two exclusive partitions: margin / interior-not-margin / the rest, and the initiation status of floor older than
  each gate (PLT-062's 150 Myr and Crameri et al. 2020's 117.4 Myr) as rifting.rs maybe_initiate_subduction reads it.
The status reads the map's end-of-step flags, without hidden trench samples, so it is close, not exact; the ages and the
places are exact. The checks per snapshot: the age map's records equal the map's, its oceanic count equals the map's
kind, and its count older than 180 Myr equals the step record's old180."""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

FIELDS = 11
R_KM = 6371.0
TRENCH = 2
FRONT_SUB, FRONT_COLL = 256, 512
CLEAR_KM = 1000.0  # PLT-064
MARGIN_KM = 200.0 + 60.0  # PLT-503: arc distance + width, plus one spacing (added per run)
GIANT = 0.2
GATES = (150.0, 117.4)  # PLT-062; Crameri et al. 2020's oldest initiation age
AGE_EDGES = [0, 10, 30, 60, 100, 150, 180, 250, 1e9]
NB = len(AGE_EDGES) - 1
T_FROM = -500.0


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def km(chord):
    return 2.0 * R_KM * np.arcsin(np.minimum(chord / 2.0, 1.0))


def nearest_km(p_from, p_to):
    if len(p_to) == 0 or len(p_from) == 0:
        return np.full(len(p_from), np.inf)
    d, _ = cKDTree(p_to).query(p_from)
    return km(d)


def band(x):
    return np.clip(np.searchsorted(AGE_EDGES, x, side="right") - 1, 0, NB - 1)


def bands(age, sel):
    return np.bincount(band(age[sel]), minlength=NB).astype(int).tolist()


def snapshot(run, path, N, step_of):
    t = float(path.stem[1:])
    a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
    n = len(a)
    if n >= N:
        return None  # taken at a resampling: nothing hidden, no front flagged
    if not ((run / "maps" / "plate" / path.name).is_file() and (run / "maps" / "age" / path.name).is_file()):
        return {"check": {"t": t, "missing": True}}
    plate = np.fromfile(run / "maps" / "plate" / path.name, "<u4").astype(np.int64)
    age = np.fromfile(run / "maps" / "age" / path.name, "<f4").astype(np.float64)
    cont = a[:, 4] > 0.5
    oc = ~cont
    st = step_of(t)
    check = {"t": t, "records": [n, len(plate), len(age)], "nan_is_cont": bool((np.isnan(age) == cont).all()),
             "rec_ocean": st["ocean"], "map_ocean": int(oc.sum()),
             "rec_old180": st["old180"], "age_old180": int((oc & (age > 180.0)).sum())}
    if len(plate) != n or len(age) != n:
        return {"check": check}
    spacing_km = math.sqrt(4.0 * math.pi / N) * R_KM
    margin_km = MARGIN_KM + spacing_km
    p = xyz(a[:, 0], a[:, 1])
    flags = np.rint(a[:, 10]).astype(np.int64)
    trench = (flags & TRENCH) != 0
    front = trench | ((flags & (FRONT_SUB | FRONT_COLL)) != 0)
    dt = nearest_km(p, p[trench])
    dco = np.full(n, np.inf)
    ids, inv = np.unique(plate, return_inverse=True)
    share = np.bincount(inv, minlength=len(ids)) / N
    has_tr = np.bincount(inv, weights=trench, minlength=len(ids)) > 0
    has_fr = np.bincount(inv, weights=front, minlength=len(ids)) > 0
    for k in range(len(ids)):
        m = inv == k
        mc, mo = m & cont, m & oc
        if mc.any() and mo.any():
            dco[mo] = nearest_km(p[mo], p[mc])
    margin = oc & (dco <= margin_km)
    interior = oc & (dt > CLEAR_KM)
    giant = oc & (share[inv] > GIANT)
    notrench = oc & ~has_tr[inv]
    nofront = oc & ~has_fr[inv]
    out = {"check": check, "t": t, "plates": len(ids), "giant_plates": int((share > GIANT).sum()),
           "ocean": bands(age, oc), "margin": bands(age, margin), "interior": bands(age, interior),
           "giant": bands(age, giant), "notrench": bands(age, notrench), "nofront": bands(age, nofront),
           "x_interior_rest": bands(age, interior & ~margin), "x_neither": bands(age, oc & ~margin & ~interior),
           "x_giant_notrench": bands(age, giant & notrench), "x_margin_notrench": bands(age, margin & notrench)}
    # The initiation status of floor older than each gate, as maybe_initiate_subduction reads it.
    for g in GATES:
        old = oc & (age > g)
        clear_old = old & (dt > CLEAR_KM)
        per = np.bincount(inv, weights=clear_old, minlength=len(ids))
        far = np.bincount(inv, weights=clear_old & (dco > margin_km), minlength=len(ids))
        el, fa = (per >= 16)[inv], (far > 0)[inv]
        status = [int((old & (dt <= CLEAR_KM)).sum()), int((clear_old & ~el).sum()),
                  int((clear_old & el & ~fa).sum()), int((clear_old & el & fa & (dco <= margin_km)).sum()),
                  int((clear_old & el & fa & (dco > margin_km)).sum())]
        out[f"status_{g:g}"] = status
        out[f"eligible_plates_{g:g}"] = int(((per >= 16) & (far > 0)).sum())
    return out


def main():
    for arm in sys.argv[1:]:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file()):
            if not (run / "maps" / "age").is_dir():
                print(json.dumps({"arm": arm, "run": run.name, "error": "no maps/age"}), flush=True)
                continue
            r = json.loads((run / "rec.json").read_text())
            N = r["samples"]
            steps = r["steps"]
            step_t = np.array([s["t"] for s in steps])

            def step_of(t):
                return steps[int(np.argmin(np.abs(step_t - t)))]

            snaps = []
            for path in sorted((run / "maps").glob("t*.bin")):
                if float(path.stem[1:]) < T_FROM:
                    continue
                s = snapshot(run, path, N, step_of)
                if s is not None:
                    snaps.append(s)
            print(json.dumps({"arm": arm, "run": run.name, "samples": N, "snaps": snaps,
                              "status_names": ["trench<=1000", "plate<16", "no_seed", "kept_margin", "exposed"]}),
                  flush=True)


main()
