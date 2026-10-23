#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `collcoast` measure (V1 gap 2, 2026-10-23): why collision belts stand on the coast,
from one pool run folder's dynamics record and final fields, printed as a JSON object on stdout.

    python3 -I measure_collcoast.py <run folder> [--fields DIR]

From `rec.json`, over the run after its first 100 Myr (ledger PLT-508):
  hand_<size>_per100   collision states handed off (a terrane transferred or the plates welded,
                       `collision::collide`) per 100 Myr, by the terrane's own size in samples
                       (about 5,100 km2 each on the quick planet): lt50, 50_200, 200_1000, ge1000
  hand_<size>_crossed  the share of the crust that crossed collision fronts (the records'
                       `area_km2`, hand-offs and partings together) in that size's hand-offs
  counted_per100       the hand-offs the plate check counts as collisions (`collision`: a terrane
                       of at least `collision_min_frac` of a starting plate, PLT-077)
  partings_per100, partings_crossed
                       collision states that ended with no hand-off, and their share of the crust
  parted, reversed, closed
                       shares of the partings by their reason
From the final fields (coastset's coast and high ground: coast cells within 1.6 cell spacings of
the sea, high ground land above 2 km of the same landmass within 200 km; a collision high coast has
a collision-regime cell within 150 km):
  coll_land            the collision regime's share of the land
  coll_land_dsea_km    its median distance to the sea, km
  coll_land_near       its share within 150 km of the sea
  geom_side, geom_end, geom_round
                       the collision high coasts by the local belt's shape: the collision cells
                       within 400 km, their long axis in the tangent plane (axis ratio 1.5 or
                       more, else `round`), and the direction to the coast's nearest sea cell
                       across it (`side`: the belt runs along the coast) or along it (`end`: the
                       coast is where the belt runs out), at 45 degrees
  landmass_small, landmass_large
                       the collision high coasts on landmasses under 0.02, and of 0.10 or more, of
                       the land

Needs numpy and scipy (the pool resolves them with nix). Version 1.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402
from scipy.spatial import cKDTree  # noqa: E402

import continents_census as cc  # noqa: E402

SIZES = (("lt50", 0, 50), ("50_200", 50, 200), ("200_1000", 200, 1000), ("ge1000", 1000, 10**12))


def from_record(rec):
    t_end = rec["steps"][-1]["t"]
    t0 = rec["t0_myr"] + 100.0
    span = max(t_end - t0, 1e-9)
    hands = [c for c in rec["collisions"] if c["end_myr"] >= t0]
    parts = [p for p in rec["partings"] if p["end_myr"] >= t0]

    def area(c):
        return float(c.get("area_km2") or 0.0)

    total = sum(area(c) for c in hands + parts) or 1e-9
    out = {}
    for name, lo, hi in SIZES:
        sel = [c for c in hands if lo <= c["terrane_samples"] < hi]
        out[f"hand_{name}_per100"] = 100.0 * len(sel) / span
        out[f"hand_{name}_crossed"] = sum(area(c) for c in sel) / total
    out["counted_per100"] = 100.0 * sum(1 for c in hands if c["collision"]) / span
    out["partings_per100"] = 100.0 * len(parts) / span
    out["partings_crossed"] = sum(area(p) for p in parts) / total
    for reason in ("parted", "reversed", "closed"):
        out[reason] = sum(1 for p in parts if p.get("reason") == reason) / max(len(parts), 1)
    return out


def from_fields(fields_dir):
    f, _g, e = cc.load_dump(fields_dir)
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    spacing = np.sqrt(cc.EARTH_KM2 / int(ok.sum()))
    p = cc.xyz(lat[ok], lon[ok])
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[ok]
    reg = np.rint(np.nan_to_num(f["regime"][ok], nan=0.0)).astype(int)
    land = z > 0
    sea_idx = np.nonzero(~land)[0]
    ds, js = cKDTree(p[~land]).query(p[land])
    d_sea = np.zeros(z.size)
    d_sea[land] = cc.arc_km(ds)
    near_sea = np.full(z.size, -1)
    near_sea[land] = sea_idx[js]
    coast = land & (d_sea <= 1.6 * spacing)
    hi = land & (z > 2000)
    comp = cc.components(p, land, spacing)
    has = np.zeros(z.size, bool)
    for c in np.unique(comp[hi]):
        hm, cm = np.nonzero(hi & (comp == c))[0], np.nonzero(coast & (comp == c))[0]
        if cm.size == 0:
            continue
        d, _j = cKDTree(p[hm]).query(p[cm], distance_upper_bound=cc.chord(200.0))
        has[cm[np.isfinite(d)]] = True
    coll = reg == 5  # regime::COLLISION
    out = {"coll_land": float((coll & land).sum() / max(int(land.sum()), 1))}
    cl = coll & land
    out["coll_land_dsea_km"] = float(np.median(d_sea[cl])) if cl.any() else None
    out["coll_land_near"] = float((d_sea[cl] <= 150.0).mean()) if cl.any() else None
    if not coll.any():
        return out
    coll_idx = np.nonzero(coll)[0]
    tree = cKDTree(p[coll])
    hc = np.nonzero(coast & has)[0]
    near = tree.query_ball_point(p[hc], cc.chord(150.0)) if hc.size else []
    chc = np.array([i for i, n in zip(hc, near) if n], int)
    if chc.size == 0:
        return out
    wide = tree.query_ball_point(p[chc], cc.chord(400.0))
    kinds = {"side": 0, "end": 0, "round": 0}
    for i, nb in zip(chc, wide):
        c0 = p[i]
        e1 = np.cross(c0, [0.0, 0.0, 1.0])
        if np.linalg.norm(e1) < 1e-6:
            e1 = np.cross(c0, [1.0, 0.0, 0.0])
        e1 /= np.linalg.norm(e1)
        e2 = np.cross(c0, e1)
        q = p[coll_idx[nb]] - c0
        uv = np.stack([q @ e1, q @ e2], 1)
        if len(uv) < 5:
            kinds["round"] += 1
            continue
        lam, vec = np.linalg.eigh(np.cov((uv - uv.mean(0)).T))
        if lam[0] <= 0.0 or np.sqrt(lam[1] / lam[0]) < 1.5:
            kinds["round"] += 1
            continue
        s = p[near_sea[i]] - c0
        n = np.array([s @ e1, s @ e2])
        n /= max(np.linalg.norm(n), 1e-12)
        kinds["end" if abs(vec[:, 1] @ n) >= np.cos(np.radians(45.0)) else "side"] += 1
    for k, v in kinds.items():
        out[f"geom_{k}"] = v / chc.size
    n_land = max(int(land.sum()), 1)
    labels, counts = np.unique(comp[land], return_counts=True)
    share = dict(zip(labels.tolist(), (counts / n_land).tolist()))
    sizes = np.array([share[comp[i]] for i in chc])
    out["landmass_small"] = float((sizes < 0.02).mean())
    out["landmass_large"] = float((sizes >= 0.10).mean())
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--fields")
    a = ap.parse_args()
    run = Path(a.run)
    values = from_record(json.loads((run / "rec.json").read_text()))
    values.update(from_fields(a.fields or run / "fields"))
    print(json.dumps({k: (None if v is None else float(v)) for k, v in values.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
