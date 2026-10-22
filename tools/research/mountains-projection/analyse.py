#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The measurements of docs/research/notes/mountains-and-the-projection.md. Research tool, not shipped;
run on the test VM through run.sh (numpy, scipy, matplotlib).

    analyse.py --work W --out O [--bin deeptime] [--others DIR,DIR...]

Reads W/lambert-s2.npy and W/pg-s2.npy (`deeptime heightmap --scale --layers meta/province`, every 2
blocks, of one simulation baked twice: the Lambert world in the simulation's own frame and the
Petroff-Guyou world the game plays), W/lambert/manifest.json and the cube tables W/cube/cube-tau*.npz
(cube_table.py). With --others, it first takes a land mask of each other world-data folder (every 16
blocks) for the corner-placement shares. Writes O/numbers.json and charts.

Definitions:
- land: elevation E at or above the sea (blocks); mountain land: E at least 1 km (and 2 km) above
  the sea at the world's vertical scale; orogen land: land in an Orogen province (meta/province, the
  manifest's province table).
- The Lambert world's blocks are all the same planet area, as are Petroff-Guyou's, so plain
  quantiles over either grid's samples are quantiles by planet area.
- A = d(east, north metres)/d(world x, z blocks) of the Petroff-Guyou world, from the block lat/lon
  by central differences; L = the same for the Lambert raster the surface solvers run on
  (diag(m cos phi, m / cos phi), m = 300). Shape ratio of a 2 x 2 map: its larger singular value over
  its smaller. A planet-true shape is drawn with ratio sr(A); a shape the block-metric solvers make
  round on the Lambert raster is drawn with ratio sr(A^-1 L).
- Cube maps (cube_table.py): blocks per planet area relative to the mean (bpa; 1 everywhere on an
  equal-area map), shape ratio, metres per block along the most and least stretched directions,
  for a cube world with the Petroff-Guyou world's mean block (294.6 m).
"""

import argparse
import glob
import json
import math
import os
import subprocess
import sys

import numpy as np
from scipy import ndimage
from scipy.spatial import cKDTree
from scipy.spatial.transform import Rotation

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import geom  # noqa: E402

QS = [0.0, 0.01, 0.05, 0.1, 0.5, 0.9, 0.95, 0.99, 0.999, 1.0]
CORNERS = np.array([[sx, sy, sz] for sx in (1, -1) for sy in (1, -1) for sz in (1, -1)], float) / math.sqrt(3)
LAND_CAP_DEG = 20.0  # as the bake's lambda-c search (project.rs LAND_SEARCH_DEG)
STEP_DEG = 0.25      # as the bake's LAMBDA_STEP_DEG


def q(v, qs=QS):
    v = np.asarray(v, float)
    v = v[np.isfinite(v)]
    if v.size == 0:
        return None
    return {str(k): round(float(x), 4) for k, x in zip(qs, np.quantile(v, qs))}


def load(work, name):
    a = np.load(os.path.join(work, name + ".npy"), mmap_mode="r")
    meta = json.load(open(os.path.join(work, name + ".npy.json")))
    ch = {n: i for i, n in enumerate(meta["channels"])}
    return a, ch, meta


def find_key(obj, key):
    if isinstance(obj, dict):
        if key in obj:
            return obj[key]
        for v in obj.values():
            r = find_key(v, key)
            if r is not None:
                return r
    elif isinstance(obj, list):
        for v in obj:
            r = find_key(v, key)
            if r is not None:
                return r
    return None


def dicts_with(obj, keys):
    """Every dict inside obj holding all of keys."""
    if isinstance(obj, dict):
        if all(k in obj for k in keys):
            yield obj
        for v in obj.values():
            yield from dicts_with(v, keys)
    elif isinstance(obj, list):
        for v in obj:
            yield from dicts_with(v, keys)


def province_classes(manifest):
    for prov in dicts_with(manifest.get("notes", {}), ("province", "classes")):
        if isinstance(prov["province"], list):
            return {int(r["id"]): r["class"] for r in prov["province"]}
    return None


def ang_deg(chord):
    return np.degrees(2 * np.arcsin(np.clip(chord / 2, 0, 1)))


# ----------------------------------------------------------------------------------------------
# Part 1 and 2: today's distortion, and its trace in the relief
# ----------------------------------------------------------------------------------------------

def part_today(L, Lc, P, Pc, sea, mv, r_m, classes, res, out):
    # Petroff-Guyou at step 4 (every other sample of the step-2 map).
    lat = np.asarray(P[::2, ::2, Pc["lat_deg"]], np.float64)
    lon = np.asarray(P[::2, ::2, Pc["lon_deg"]], np.float64)
    E = np.asarray(P[::2, ::2, Pc["elevation"]], np.float64)
    prov = np.asarray(P[::2, ::2, Pc["meta/province"]])
    U = geom.ll2xyz(lat, lon)
    a, b, c, d = geom.jacobian_from_units(U, 4.0, r_m, wrap_x=True)
    s1, s2, det = geom.sv2(a, b, c, d)
    res["pg_block_m_check"] = q(np.sqrt(np.abs(det)), [0.01, 0.5, 0.99])
    srA = s1 / s2
    # A^-1 L: L = diag(m cos phi, m / cos phi) (signs do not change singular values).
    cphi = np.cos(np.radians(lat))
    l11, l22 = 300.0 * cphi, 300.0 / cphi
    inv = 1.0 / det
    m11, m12 = inv * d * l11, -inv * b * l22
    m21, m22 = -inv * c * l11, inv * a * l22
    srM = geom.shape_ratio(m11, m12, m21, m22)
    srL = 1.0 / cphi ** 2
    land = E >= sea
    m1 = E >= sea + 1000.0 / mv
    m2 = E >= sea + 2000.0 / mv
    orog = land & np.isin(prov, [k for k, v in (classes or {}).items() if v == "Orogen"])
    inner = np.ones_like(land)
    inner[:3], inner[-3:] = False, False  # one-sided differences at the half-turn edges
    masks = {"planet": inner, "land": land & inner, "mountain_1km": m1 & inner,
             "mountain_2km": m2 & inner, "orogen_land": orog & inner}
    t = {}
    for k, m in masks.items():
        t[k] = {"share_of_samples": round(float(m.mean()), 4),
                "petroff_alone": q(srA[m]), "lambert_alone": q(srL[m]),
                "sim_texture_in_world": q(srM[m])}
    bands = {}
    alat = np.abs(lat)
    for lo_, hi_ in ((0, 20), (20, 40), (40, 55), (55, 70), (70, 90)):
        m = m1 & inner & (alat >= lo_) & (alat < hi_)
        if m.sum() > 50:
            bands[f"{lo_}-{hi_}"] = {"samples": int(m.sum()),
                                     "petroff_alone_q50_q90": [round(float(x), 3) for x in np.quantile(srA[m], [0.5, 0.9])],
                                     "lambert_alone_q50": round(float(np.quantile(srL[m], 0.5)), 3),
                                     "sim_texture_q50_q90": [round(float(x), 3) for x in np.quantile(srM[m], [0.5, 0.9])]}
    m = masks["mountain_1km"]
    t["exaggeration_mountain_1km"] = {
        "exaggeration_steepest_direction_q": q(s1[m] / mv, [0.0, 0.05, 0.5, 0.95, 1.0]),
        "exaggeration_gentlest_direction_q": q(s2[m] / mv, [0.0, 0.05, 0.5, 0.95, 1.0]),
        "within_place_directions_q": q(srA[m], [0.5, 0.9, 0.99, 1.0]),
        "all_directions_all_places_max_over_min": round(float(s1[m].max() / s2[m].min()), 4)}
    t["mountain_1km_by_abs_lat"] = bands
    res["today"] = t
    res["mountain_lat_share"] = {f"{lo_}-{hi_}": round(float((m1 & (alat >= lo_) & (alat < hi_)).sum() / max(1, m1.sum())), 4)
                                 for lo_, hi_ in ((0, 20), (20, 40), (40, 55), (55, 70), (70, 90))}

    # Part 2: slope energy along x (east) and z (north) on the Lambert raster, mountain land, by band.
    El = np.asarray(L[:, :, Lc["elevation"]], np.float32)
    latl = np.asarray(L[:, 0, Lc["lat_deg"]], np.float64)
    gx = (np.roll(El, -1, 1) - np.roll(El, 1, 1)) / 4.0   # blocks per Lambert block
    gz = np.gradient(El, 2.0, axis=0)
    ml = El >= sea + 1000.0 / mv
    rows = {}
    for lo_, hi_ in ((0, 20), (20, 40), (40, 55), (55, 70), (70, 85)):
        rsel = (np.abs(latl) >= lo_) & (np.abs(latl) < hi_)
        m = ml & rsel[:, None]
        if m.sum() < 200:
            continue
        rx = float(np.sqrt(np.mean(gx[m] ** 2)))
        rz = float(np.sqrt(np.mean(gz[m] ** 2)))
        c2 = float(np.mean(np.cos(np.radians(latl[rsel])) ** 2))
        rows[f"{lo_}-{hi_}"] = {"samples": int(m.sum()), "lambert_blocks_x_over_z": round(rx / rz, 3),
                                "true_east_over_north": round(rx / rz / c2, 3),
                                "isotropic_in_lambert_would_give": round(1 / c2, 3)}
    res["slope_anisotropy_mountain_1km"] = rows


# ----------------------------------------------------------------------------------------------
# Part 3: the cube maps
# ----------------------------------------------------------------------------------------------

class Coast:
    """Land and sea samples of a planet (unit vectors, roughly equal-area). A point's signed
    distance to the coast, degrees: positive at sea (the distance to the nearest land sample),
    negative on land (minus the distance to the nearest sea sample); a point is at sea when its
    nearest sample is sea. Capped at LAND_CAP_DEG as the bake's lambda-c search."""

    def __init__(self, p, land):
        self.land = cKDTree(p[land])
        self.sea = cKDTree(p[~land]) if (~land).any() else None

    def signed(self, pts):
        dl, _ = self.land.query(pts, workers=-1)
        if self.sea is None:
            return -np.zeros(len(pts))
        ds, _ = self.sea.query(pts, workers=-1)
        return np.minimum(np.where(dl > ds, ang_deg(dl), -ang_deg(ds)), LAND_CAP_DEG)


def corner_distances(Q, coast):
    """Each canonical corner's signed distance to the coast (degrees) for rotations Q[n,3,3]
    planet -> canonical; corner in the planet frame = Q^T c."""
    pts = np.einsum("nji,kj->nki", Q, CORNERS)  # (n, 8, 3): Q^T c
    return coast.signed(pts.reshape(-1, 3)).reshape(pts.shape[:2])


def shares(mn):
    return {"share_at_sea": round(float((mn > 0).mean()), 4),
            "share_3deg": round(float((mn >= 3).mean()), 4),
            "share_5deg": round(float((mn >= 5).mean()), 4),
            "share_10deg": round(float((mn >= 10).mean()), 4)}


def polar_search(coast):
    """The cube with its north and south faces centred on the poles, turned by lambda0 about the
    axis (0.25 degree steps over the 90 degree period): as the bake's lambda-c search, maximise the
    least corner distance, then the sum."""
    lam = np.arange(0, 90, STEP_DEG)
    Q = np.stack([geom.rot_z(-x) for x in lam])
    D = corner_distances(Q, coast)
    mn = D.min(1)
    sm = D.sum(1)
    best = int(np.lexsort((-sm, -mn))[0])
    return lam, D, {"lambda0_deg": float(lam[best]), "min_distance_deg": round(float(mn[best]), 2),
                    "distances_deg": [round(float(x), 2) for x in D[best]], **shares(mn)}


def free_search(coast, n=60000, seed=7, keep=24, rounds=120, extra=None):
    """Any orientation: n random rotations (their shares say how easy it is), then the best `keep`
    refined by a shrinking random walk (accept a turn that raises the least corner distance)."""
    rng = np.random.default_rng(seed)
    Q = Rotation.random(n, random_state=seed).as_matrix()
    D = corner_distances(Q, coast)
    mn = D.min(1)
    random_best = float(mn.max())
    if extra is not None:  # seed the walk with the polar aspect's best too, so free >= polar
        De = corner_distances(extra[None], coast)
        Q, D = np.concatenate([extra[None], Q]), np.concatenate([De, D])
        mn = np.concatenate([De.min(1), mn])
    top = np.argsort(-(mn + 1e-3 * D.sum(1)))[:keep]
    Qs, score = Q[top], (mn + 1e-3 * D.sum(1))[top]
    for r in range(rounds):
        sig = math.radians(4.0 * (0.03 / 4.0) ** (r / max(1, rounds - 1)))  # 4 degrees down to 0.03
        tries = 32
        R = Rotation.from_rotvec(rng.normal(scale=sig, size=(keep * tries, 3))).as_matrix()
        Qn = np.einsum("nij,njk->nik", np.repeat(Qs, tries, axis=0), R)
        Dn = corner_distances(Qn, coast)
        sn = (Dn.min(1) + 1e-3 * Dn.sum(1)).reshape(keep, tries)
        j = np.argmax(sn, axis=1)
        better = sn[np.arange(keep), j] > score
        Qs[better] = Qn.reshape(keep, tries, 3, 3)[np.arange(keep), j][better]
        score[better] = sn[np.arange(keep), j][better]
    b = int(np.argmax(score))
    Db = corner_distances(Qs[b:b + 1], coast)[0]
    return Qs[b], {"rotations": n, "random_best_min_distance_deg": round(random_best, 2),
                   "best_min_distance_deg": round(float(Db.min()), 2),
                   "distances_deg": [round(float(x), 2) for x in Db],
                   "rotation_planet_to_canonical": [[round(float(x), 6) for x in row] for row in Qs[b]],
                   **{"random_" + k: v for k, v in shares(mn).items()}}


def pole_tilt_deg(Q):
    """The north pole's angle from the nearest cube face centre, degrees, for rotations Q[n,3,3]
    (planet -> canonical; the pole's canonical image is Q[:, :, 2])."""
    q = np.abs(np.asarray(Q)[..., :, 2])
    return np.degrees(np.arccos(np.clip(q.max(-1), -1, 1)))


def limited_search(coast, limit_deg, seed=11, keep=24, rounds=80):
    """As free_search, but the pole may lean at most limit_deg from the centre of the face that
    holds it (0 is the polar aspect): a grid of tilts, tilt directions and turns, then the shrinking
    random walk with turns past the limit refused."""
    rng = np.random.default_rng(seed)
    Qs = []
    for b in np.arange(0.0, limit_deg + 1e-9, 2.0):
        for a in (np.arange(0.0, 360.0, 10.0) if b > 0 else [0.0]):
            R = geom.rot_zyz(a, b, -a)
            for lam_ in np.arange(0.0, 90.0, 1.0):
                Qs.append(geom.rot_z(-lam_) @ R)
    Q = np.stack(Qs)
    D = corner_distances(Q, coast)
    sc = D.min(1) + 1e-3 * D.sum(1)
    top = np.argsort(-sc)[:keep]
    Qk, score = Q[top].copy(), sc[top].copy()
    for r in range(rounds):
        sig = math.radians(2.0 * (0.03 / 2.0) ** (r / max(1, rounds - 1)))
        tries = 32
        Rr = Rotation.from_rotvec(rng.normal(scale=sig, size=(keep * tries, 3))).as_matrix()
        Qn = np.einsum("nij,njk->nik", np.repeat(Qk, tries, axis=0), Rr)
        Dn = corner_distances(Qn, coast)
        sn = Dn.min(1) + 1e-3 * Dn.sum(1)
        sn[pole_tilt_deg(Qn) > limit_deg + 1e-9] = -1e9
        sn = sn.reshape(keep, tries)
        j = np.argmax(sn, axis=1)
        better = sn[np.arange(keep), j] > score
        Qk[better] = Qn.reshape(keep, tries, 3, 3)[np.arange(keep), j][better]
        score[better] = sn[np.arange(keep), j][better]
    b_ = int(np.argmax(score))
    Db = corner_distances(Qk[b_:b_ + 1], coast)[0]
    return Qk[b_], {"tilt_limit_deg": limit_deg, "tilt_deg": round(float(pole_tilt_deg(Qk[b_])), 2),
                    "best_min_distance_deg": round(float(Db.min()), 2),
                    "distances_deg": [round(float(x), 2) for x in Db]}


TILT_LIMITS = (10.0, 20.0, 30.0)


def sph_tri(a, b, c):
    """Area of the spherical triangle abc (unit vectors), Van Oosterom and Strackee."""
    num = abs(float(a @ np.cross(b, c)))
    den = 1.0 + float(a @ b) + float(b @ c) + float(c @ a)
    return 2.0 * math.atan2(num, den)


def cube_fields(cube, p, Q=None, block_m=294.6):
    s1, s2 = cube.sv_at(p, Q)
    bpa = (math.pi / 6) / (s1 * s2)
    k = block_m / np.sqrt(bpa)                     # metres per block, the isotropic part
    f = block_m / math.sqrt(math.pi / 6)
    return {"bpa": bpa, "shape": s1 / s2, "k": k, "m_hi": s1 * f, "m_lo": s2 * f}


def view_change(cube, p, k, Q, r_m, dist_blocks, n_dir=8, block_m=294.6):
    """The largest ratio of the isotropic metres-per-block between a point and the points
    dist_blocks away in n_dir directions (planet distance dist_blocks * k)."""
    e, nn = geom.east_north(p)
    worst = np.zeros(len(p))
    for j in range(n_dir):
        t = 2 * math.pi * j / n_dir
        dirv = math.cos(t) * e + math.sin(t) * nn
        a = (dist_blocks * k / r_m)[:, None]
        p2 = p * np.cos(a) + dirv * np.sin(a)
        k2 = cube_fields(cube, p2, Q, block_m)["k"]
        worst = np.maximum(worst, np.abs(np.log(k2 / k)))
    return np.exp(worst)


def part_cube(L, Lc, sea, mv, r_m, classes, tables, res, out, others):
    lat = np.asarray(L[::4, ::4, Lc["lat_deg"]], np.float64)
    lon = np.asarray(L[::4, ::4, Lc["lon_deg"]], np.float64)
    E = np.asarray(L[::4, ::4, Lc["elevation"]], np.float64)
    prov = np.asarray(L[::4, ::4, Lc["meta/province"]])
    ok = np.isfinite(lat) & (np.abs(lat) <= 90)
    p = geom.ll2xyz(lat[ok], lon[ok])
    E = E[ok]
    prov = prov[ok]
    land = E >= sea
    masks = {"planet": np.ones_like(land), "land": land, "mountain_1km": E >= sea + 1000 / mv,
             "mountain_2km": E >= sea + 2000 / mv,
             "orogen_land": land & np.isin(prov, [k for k, v in (classes or {}).items() if v == "Orogen"])}
    coast = Coast(p, land)
    lam, D, polar = polar_search(coast)
    Qfree, free = free_search(coast, extra=geom.rot_z(-polar["lambda0_deg"]))
    free["tilt_deg"] = round(float(pole_tilt_deg(Qfree[None])[0]), 2)
    Qlim, lim = {}, {}
    for tl in TILT_LIMITS:
        Qlim[tl], lim[f"tilt<={tl:g}"] = limited_search(coast, tl)
        print("this planet", lim[f"tilt<={tl:g}"], flush=True)
    res["corners_this_planet"] = {"polar_aspect": polar, "free_rotation": free, "tilt_limited": lim}
    np.save(os.path.join(out, "polar-corner-distances.npy"), D)

    # Other planets: the same two searches on their land.
    op = {}
    other_land = {}
    for name, (lat_o, lon_o, land_o, chk) in others.items():
        tr = Coast(geom.ll2xyz(lat_o, lon_o), land_o)
        _, _, pol = polar_search(tr)
        Qo, fr = free_search(tr, n=30000, extra=geom.rot_z(-pol["lambda0_deg"]))
        fr["tilt_deg"] = round(float(pole_tilt_deg(Qo[None])[0]), 2)
        limd, limQ = {}, []
        for tl in TILT_LIMITS:
            Ql, info = limited_search(tr, tl)
            limd[f"tilt<={tl:g}"] = info
            limQ.append(Ql)
        other_land[name] = (geom.ll2xyz(lat_o[land_o], lon_o[land_o]), Qo, geom.rot_z(-pol["lambda0_deg"]), *limQ)
        op[name] = {"land_share": round(float(land_o.mean()), 3), "polar_aspect": pol, "free_rotation": fr,
                    "tilt_limited": limd, "petroff_cone_check": chk}
        print("other", name, json.dumps(op[name]), flush=True)
    res["corners_other_planets"] = op

    Qpolar = geom.rot_z(-polar["lambda0_deg"])
    rng = np.random.default_rng(3)
    cube_res = {}
    for path in tables:
        cube = geom.Cube(path)
        tau = float(np.load(path)["tau"])
        key = f"tau={tau:.3f}"
        entry = {}
        for aspect, Q in (("polar", Qpolar), ("free", Qfree)):
            f = cube_fields(cube, p, Q)
            a = {}
            for mk, m in masks.items():
                a[mk] = {"bpa": q(f["bpa"][m]), "shape": q(f["shape"][m]),
                         "k_m_per_block": q(f["k"][m], [0.0, 0.05, 0.5, 0.95, 1.0]),
                         "relaxed_factor_max": round(float(max(f["bpa"][m].max(), 1 / f["bpa"][m].min())), 4),
                         "relaxed_factor_p999": round(float(max(np.quantile(f["bpa"][m], 0.999),
                                                                1 / np.quantile(f["bpa"][m], 0.001))), 4)}
            # Exaggeration on mountain land: vertical 35 m per block against the horizontal metres
            # per block; between places (isotropic part) and within one place (directions).
            m = masks["mountain_1km"]
            k = f["k"][m]
            a["exaggeration_mountain_1km"] = {
                "isotropic_q": q(k / mv, [0.0, 0.05, 0.5, 0.95, 1.0]),
                "between_places_p95_over_p5": round(float(np.quantile(k, 0.95) / np.quantile(k, 0.05)), 4),
                "between_places_max_over_min": round(float(k.max() / k.min()), 4),
                "within_place_directions_q": q(f["shape"][m], [0.5, 0.9, 0.99, 1.0]),
                "all_directions_all_places_max_over_min": round(float(f["m_hi"][m].max() / f["m_lo"][m].min()), 4),
            }
            if True:  # how fast the scale changes as a player walks (both placements)
                idx = np.flatnonzero(m)
                idx = rng.choice(idx, size=min(40000, idx.size), replace=False)
                for dist in (256, 1024, 4096):
                    vc = view_change(cube, p[idx], f["k"][idx], Q, r_m, dist)
                    a["exaggeration_mountain_1km"][f"change_within_{dist}_blocks_q"] = q(vc, [0.5, 0.9, 0.99, 1.0])
            entry[aspect] = a
        # Other planets' land, each with its own best free orientation.
        for tl in TILT_LIMITS:
            f = cube_fields(cube, p, Qlim[tl])
            entry[f"tilt<={tl:g}"] = {mk: {"bpa_max": round(float(f["bpa"][masks[mk]].max()), 4),
                                           "bpa_p999": round(float(np.quantile(f["bpa"][masks[mk]], 0.999)), 4),
                                           "bpa_min": round(float(f["bpa"][masks[mk]].min()), 4)}
                                      for mk in ("land", "mountain_1km")}
        asps = [("free", 1), ("polar", 2)] + [(f"tilt<={tl:g}", 3 + i) for i, tl in enumerate(TILT_LIMITS)]
        for asp, qi in asps:
            ol = {}
            for name, item in other_land.items():
                pl, Qa = item[0], item[qi]
                b = cube_fields(cube, pl, Qa)["bpa"]
                # land within 5 degrees of a corner (share of the planet's land samples)
                cpl = CORNERS @ Qa  # corners in the planet frame (rows)
                dmin = ang_deg(np.min(np.linalg.norm(pl[:, None, :] - cpl[None], axis=-1), axis=1))
                ol[name] = {"bpa_min": round(float(b.min()), 4), "bpa_p99": round(float(np.quantile(b, 0.99)), 4),
                            "bpa_p999": round(float(np.quantile(b, 0.999)), 4),
                            "bpa_max": round(float(b.max()), 4),
                            "relaxed_factor_max": round(float(max(b.max(), 1 / b.min())), 4),
                            "land_share_within_5deg_of_corner": round(float((dmin < 5).mean()), 5)}
            if ol:
                entry[f"other_planets_land_{asp}"] = ol
                entry[f"other_planets_land_{asp}_worst_factor"] = max(v["relaxed_factor_max"] for v in ol.values())
                entry[f"other_planets_land_{asp}_worst_p999"] = max(v["bpa_p999"] for v in ol.values())
        # The smallest cell: the corner cell of an n x n face grid, its planet area over the mean
        # cell's (spherical quadrilateral from the four corner points, as a finite-volume grid
        # would take it), and its shortest edge over the mean edge.
        cc = {}
        for n_ in (60, 120, 240, 480):
            uu = np.linspace(-1, 1, n_ + 1)[-2:]
            P00, P10 = cube.face_vec(np.array([uu[0]]), np.array([uu[0]]))[0], cube.face_vec(np.array([uu[1]]), np.array([uu[0]]))[0]
            P11, P01 = cube.face_vec(np.array([uu[1]]), np.array([uu[1]]))[0], cube.face_vec(np.array([uu[0]]), np.array([uu[1]]))[0]
            area = sph_tri(P00, P10, P11) + sph_tri(P00, P11, P01)
            mean_area = 4 * math.pi / (6 * n_ * n_)
            edges = [math.acos(min(1.0, float(a @ b))) for a, b in ((P00, P10), (P10, P11), (P11, P01), (P01, P00))]
            mean_edge = math.sqrt(mean_area)
            cc[str(n_)] = {"area_over_mean": round(area / mean_area, 5), "shortest_edge_over_mean": round(min(edges) / mean_edge, 5)}
        entry["corner_cell"] = cc
        # The whole planet with no placement (the map's own figures; survey §3.1 check).
        fp = cube_fields(cube, p, geom.rot_z(0.0))
        entry["planet_check"] = {"bpa": q(fp["bpa"]), "shape": q(fp["shape"]),
                                 "within_1.25": round(float(((fp["bpa"] <= 1.25) & (fp["bpa"] >= 0.8)).mean()), 4),
                                 "within_1.5": round(float(((fp["bpa"] <= 1.5) & (fp["bpa"] >= 1 / 1.5)).mean()), 4)}
        # How the worst land bpa grows as the nearest corner comes closer to land (any aspect).
        cube_res[key] = entry
        print(key, "polar", json.dumps(entry["polar"]["mountain_1km"]["bpa"]), flush=True)
        print(key, "free", json.dumps(entry["free"]["mountain_1km"]["bpa"]), flush=True)
    res["cube_maps"] = cube_res

    # bpa and shape as a function of the distance from the nearest corner, per map.
    prof = {}
    for path in tables:
        cube = geom.Cube(path)
        tau = float(np.load(path)["tau"])
        pts, s1, s2 = cube.face_sv(257)
        ok2 = np.isfinite(s1) & (s2 > 0)
        dc = ang_deg(np.min(np.linalg.norm(pts[ok2][:, None, :] - CORNERS[None], axis=-1), axis=1))
        bpa = (math.pi / 6) / (s1[ok2] * s2[ok2])
        sh = s1[ok2] / s2[ok2]
        rows = {}
        pa = s1[ok2] * s2[ok2]  # planet area per node (the nodes are equal world areas)
        for lo_ in (0, 2, 3, 5, 8, 10, 15, 20, 30):
            sel = dc >= lo_
            rows[f">={lo_}deg"] = {"bpa_max": round(float(bpa[sel].max()), 4), "bpa_min": round(float(bpa[sel].min()), 4),
                                   "shape_max": round(float(sh[sel].max()), 4),
                                   "world_share_nearer": round(float((~sel).mean()), 4),
                                   "planet_share_nearer": round(float(pa[~sel].sum() / pa.sum()), 4)}
        prof[f"tau={tau:.3f}"] = rows
    res["cube_by_corner_distance"] = prof
    return p, land, masks, {"polar": Qpolar, "free": Qfree}


# ----------------------------------------------------------------------------------------------
# Part 4: what else makes mountains weak
# ----------------------------------------------------------------------------------------------

def part_other(L, Lc, P, Pc, sea, mv, r_m, manifest, res):
    E = np.asarray(L[:, :, Lc["elevation"]], np.float32)
    H = np.asarray(L[:, :, Lc["surface"]], np.float32)
    lat = np.asarray(L[:, 0, Lc["lat_deg"]], np.float64)
    land = E >= sea
    # Summits: highest points of distinct ranges (one per 200-block square).
    hm = (H - sea) * mv
    mx = ndimage.maximum_filter(np.where(land, hm, -1e9), size=101, mode="wrap")
    peaks = np.argwhere((hm == mx) & land & (hm > 2000))
    vals = hm[peaks[:, 0], peaks[:, 1]]
    o = np.argsort(-vals)[:12]
    res["summits_m"] = [{"m": round(float(vals[i])), "lat": round(float(lat[peaks[i, 0]]), 1)} for i in o]
    res["highest_E_m"] = round(float(((E[land] - sea) * mv).max()))
    res["land_above_km_share"] = {str(k): round(float((E[land] >= sea + k * 1000 / mv).mean()), 4) for k in (1, 2, 3, 4, 5, 6)}
    # Belt widths: half-width = distance to the edge of the >1 km mask, in true km, per band.
    m1 = E >= sea + 1000 / mv
    widths = {}
    for lo_, hi_ in ((0, 30), (30, 55), (55, 75)):
        rsel = (np.abs(lat) >= lo_) & (np.abs(lat) < hi_)
        if not rsel.any():
            continue
        cph = math.cos(math.radians((lo_ + hi_) / 2))
        sub = m1[rsel]
        dt = ndimage.distance_transform_edt(sub, sampling=(2 * 300.0 / cph / 1000, 2 * 300.0 * cph / 1000))
        if sub.sum() < 100:
            continue
        lab, n = ndimage.label(sub)
        if n == 0:
            continue
        mxw = ndimage.maximum(dt, lab, index=np.arange(1, n + 1))
        sizes = ndimage.sum(sub, lab, index=np.arange(1, n + 1))
        big = mxw[sizes >= 50]
        widths[f"{lo_}-{hi_}"] = {"belts": int(big.size),
                                  "full_width_km_q50_q90_max": [round(float(x) * 2, 1) for x in np.quantile(big, [0.5, 0.9, 1.0])] if big.size else None}
    res["belt_widths_1km"] = widths
    # Slopes as the game draws them (Petroff-Guyou, terrain height H, every 2 blocks): angle over a
    # 2-block run, so a lower bound on the block-to-block slope.
    Hp = np.asarray(P[:, :, Pc["surface"]], np.float32)
    Ep = np.asarray(P[:, :, Pc["elevation"]], np.float32)
    gx = (np.roll(Hp, -1, 1) - np.roll(Hp, 1, 1)) / 4.0
    gz = np.gradient(Hp, 2.0, axis=0)
    ang = np.degrees(np.arctan(np.hypot(gx, gz)))
    lp = Ep >= sea
    mp = Ep >= sea + 1000 / mv
    res["slopes_world_deg"] = {"land": q(ang[lp], [0.5, 0.9, 0.99]), "mountain_1km": q(ang[mp], [0.5, 0.9, 0.99]),
                               "land_steeper_than_45": round(float((ang[lp] > 45).mean()), 4),
                               "mountain_steeper_than_45": round(float((ang[mp] > 45).mean()), 4),
                               "mountain_steeper_than_30": round(float((ang[mp] > 30).mean()), 4)}
    # True slopes at the simulation's resolution (E, metres over metres, Lambert step 2).
    gxl = (np.roll(E, -1, 1) - np.roll(E, 1, 1)) / 4.0 * mv / (300.0 * np.cos(np.radians(lat)))[:, None]
    gzl = np.gradient(E, 2.0, axis=0) * mv / (300.0 / np.cos(np.radians(lat)))[:, None]
    tang = np.degrees(np.arctan(np.hypot(gxl, gzl)))
    res["true_slopes_sim_deg_mountain_1km"] = q(tang[m1 & (np.abs(lat) < 70)[:, None]], [0.5, 0.9, 0.99])
    res["grids"] = {k: {kk: v.get(kk) for kk in ("width", "height", "cell_m", "cell_blocks", "dx_blocks")
                        if isinstance(v, dict)} for k, v in manifest.get("grids", {}).items()}
    res["scale"] = manifest.get("scale")


# ----------------------------------------------------------------------------------------------
# Charts
# ----------------------------------------------------------------------------------------------

def charts(res, out, aspect="free"):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    cm = res["cube_maps"]
    taus = sorted(float(k.split("=")[1]) for k in cm)
    fig, axs = plt.subplots(1, 3, figsize=(15, 4.2))
    for mk, col in (("land", "#4a7"), ("mountain_1km", "#a54")):
        hi = [cm[f"tau={t:.3f}"][aspect][mk]["bpa"]["1.0"] for t in taus]
        lo = [cm[f"tau={t:.3f}"][aspect][mk]["bpa"]["0.0"] for t in taus]
        axs[0].plot(taus, hi, "-o", color=col, label=f"{mk}: max")
        axs[0].plot(taus, lo, "--o", color=col, label=f"{mk}: min")
        sh = [cm[f"tau={t:.3f}"][aspect][mk]["shape"]["1.0"] for t in taus]
        sm = [cm[f"tau={t:.3f}"][aspect][mk]["shape"]["0.5"] for t in taus]
        axs[1].plot(taus, sh, "-o", color=col, label=f"{mk}: max")
        axs[1].plot(taus, sm, "--o", color=col, label=f"{mk}: median")
        ex = [cm[f"tau={t:.3f}"][aspect]["exaggeration_mountain_1km"]["between_places_max_over_min"] for t in taus]
        ew = [cm[f"tau={t:.3f}"][aspect]["exaggeration_mountain_1km"]["within_place_directions_q"]["0.9"] for t in taus]
    axs[2].plot(taus, ex, "-o", color="#a54", label="between places, max/min")
    axs[2].plot(taus, ew, "--o", color="#47a", label="within a place, directions, p90")
    axs[0].set_title(f"Blocks per planet area on land ({aspect} corners)")
    axs[1].set_title("Shape ratio on land")
    axs[2].set_title("Slope spread on mountain land")
    for ax in axs:
        ax.set_xlabel("tau (0 conformal, 1 equal-area)")
        ax.grid(alpha=0.3)
        ax.legend(fontsize=8)
    fig.tight_layout()
    fig.savefig(os.path.join(out, "cube-maps-by-tau.png"), dpi=110)
    plt.close(fig)


def net_images(tables, sea, L, Lc, Q, aspect, out):
    """The cube world's cross net for a few maps: land and the blocks-per-planet-area contours."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    lat = np.asarray(L[::4, ::4, Lc["lat_deg"]], np.float64)
    lon = np.asarray(L[::4, ::4, Lc["lon_deg"]], np.float64)
    E = np.asarray(L[::4, ::4, Lc["elevation"]], np.float64)
    ok = np.isfinite(lat)
    tree = cKDTree(geom.ll2xyz(lat[ok], lon[ok]))
    Ev = E[ok]
    n = 200
    picks = [t for t in tables if any(f"tau{x}" in t for x in ("0.000", "0.500", "1.000"))]
    fig, axs = plt.subplots(1, len(picks), figsize=(6 * len(picks), 4.8))
    axs = np.atleast_1d(axs)
    for ax, path in zip(axs, picks):
        cube = geom.Cube(path, Q=Q)
        tau = float(np.load(path)["tau"])
        Xs = (np.arange(4 * n) + 0.5) / n
        Ys = 2 - (np.arange(3 * n) + 0.5) / n
        X, Y = np.meshgrid(Xs, Ys)
        inside = ((Y >= 0) & (Y <= 1)) | ((X <= 1) & (Y >= -1) & (Y <= 2))
        U = np.full(X.shape + (3,), np.nan)
        U[inside] = cube.unit(X[inside], Y[inside])
        img = np.full(X.shape + (3,), 1.0)
        good = inside & np.isfinite(U[..., 0])
        _, i = tree.query(U[good], workers=-1)
        e = Ev[i]
        col = np.where((e >= sea)[:, None], np.array([0.55, 0.62, 0.42]), np.array([0.62, 0.74, 0.86]))
        col = np.where((e >= sea + 1000 / 35)[:, None], np.array([0.55, 0.45, 0.35]), col)
        img[good] = col
        f = cube_fields(cube, U[good], Q)
        B = np.full(X.shape, np.nan)
        B[good] = f["bpa"]
        ax.imshow(img, extent=(0, 4, -1, 2), origin="upper")
        cs = ax.contour(Xs, Ys, B, levels=[0.8, 0.9, 1.1, 1.25, 1.5, 2.0], colors="k", linewidths=0.6)
        ax.clabel(cs, fontsize=6)
        ax.set_title(f"tau = {tau:g} ({aspect} corners): blocks per planet area")
        ax.set_xlim(0, 4)
        ax.set_ylim(-1, 2)
        ax.set_aspect("equal")
        ax.axis("off")
    fig.tight_layout()
    fig.savefig(os.path.join(out, f"cube-nets-{aspect}.png"), dpi=110)
    plt.close(fig)


# ----------------------------------------------------------------------------------------------

def other_planets(dirs, work, bin_, old_bin=None, pg=None):
    """Land masks (lat, lon, land) of other world-data folders, every 16 blocks. A world this
    binary cannot read (an older format) is read with old_bin without --scale, when it is a
    Petroff-Guyou world of this study's hemisphere side: its blocks' lat/lon are this study's
    Petroff-Guyou world's (pg = (P, Pc, b, lambda_c)) turned by the difference in lambda_c, since
    lambda_c only turns the planet about its axis (projection.rs PetroffGuyou::new)."""
    out = {}
    for d in dirs:
        d = d.strip()
        if not d or not os.path.exists(os.path.join(d, "manifest.json")):
            continue
        m = json.load(open(os.path.join(d, "manifest.json")))
        name = d.rstrip("/").replace("/data/ban/", "").replace("/", "_")
        dst = os.path.join(work, "others", name + ".npy")
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if not os.path.exists(dst):
            pr = m.get("projection") or {}
            dom = pr.get("domain")
            if dom:
                box = f"{int(dom['x_min'])},{int(dom['z_min'])},{int(dom['x_max'])},{int(dom['z_max'])}"
            else:
                c = m["scale"]["circumference_blocks"]
                zr = int(math.ceil(c / (2 * math.pi) / 16) * 16)
                box = f"{-c // 2},{-zr},{c // 2},{zr}"
            r = subprocess.run([bin_, "heightmap", "--world-data", d, f"--box={box}", "--step", "16", "--scale",
                                "--out", dst], capture_output=True, text=True)
            if r.returncode != 0 and old_bin and pg and pr.get("kind") == "petroff_guyou" \
                    and int(pr.get("hemisphere_blocks", 0)) == pg[2] and dom:
                r = subprocess.run([old_bin, "heightmap", "--world-data", d, f"--box={box}", "--step", "16",
                                    "--out", dst], capture_output=True, text=True)
            if r.returncode != 0:
                print(f"other planet {d}: heightmap failed: {r.stderr.strip()[-300:]}", flush=True)
                continue
        a = np.load(dst)
        meta = json.load(open(dst + ".json"))
        ch = {n: i for i, n in enumerate(meta["channels"])}
        sea = m["scale"]["sea_level_y"]
        E = a[:, :, ch["elevation"]].astype(np.float64)
        if "lat_deg" in ch:
            lat = a[:, :, ch["lat_deg"]].astype(np.float64)
            lon = a[:, :, ch["lon_deg"]].astype(np.float64)
        else:
            P, Pc, _, lc0 = pg
            lat = np.asarray(P[::8, ::8, Pc["lat_deg"]], np.float64)[:E.shape[0], :E.shape[1]]
            lon = np.asarray(P[::8, ::8, Pc["lon_deg"]], np.float64)[:E.shape[0], :E.shape[1]]
            lon = lon + (float((m.get("projection") or {})["lambda_c_deg"]) - lc0)
            if lat.shape != E.shape:
                print(f"other planet {d}: grid {E.shape} against {lat.shape}, skipped", flush=True)
                continue
        ok = np.isfinite(lat) & np.isfinite(E)
        land = E[ok] >= sea
        # Check of the lat/lon: Petroff-Guyou's four cone points (lat +-45, lon lambda_c +-90)
        # against the distances to land the bake recorded for them.
        chk = None
        pr = m.get("projection") or {}
        if pr.get("kind") == "petroff_guyou" and land.any():
            lc = float(pr["lambda_c_deg"])
            cones = geom.ll2xyz(np.array([45.0, -45.0, 45.0, -45.0]), np.array([lc + 90, lc + 90, lc - 90, lc - 90]))
            dd = Coast(geom.ll2xyz(lat[ok], lon[ok]), land).signed(cones)
            rec = find_key(m.get("notes", {}), "cone_distances_deg")
            chk = {"ours_deg": sorted(round(float(x), 2) for x in dd),
                   "bake_recorded_deg": sorted(rec) if rec else None}
        out[name] = (lat[ok], lon[ok], land, chk)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--bin", default="deeptime")
    ap.add_argument("--others", default="")
    ap.add_argument("--old-bin", default="", help="a deeptime binary that reads older worlds (no --scale needed)")
    ap.add_argument("--parts", default="today,cube,other,charts")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    L, Lc, _ = load(a.work, "lambert-s2")
    P, Pc, _ = load(a.work, "pg-s2")
    manifest = json.load(open(os.path.join(a.work, "lambert", "manifest.json")))
    pgm = json.load(open(os.path.join(a.work, "pg", "manifest.json")))
    sc = manifest["scale"]
    sea, mv = sc["sea_level_y"], sc["m_per_block_v"]
    r_m = sc["circumference_blocks"] * sc["m_per_block_h"] / (2 * math.pi)
    classes = province_classes(manifest)
    res_path = os.path.join(a.out, "numbers.json")
    res = json.load(open(res_path)) if os.path.exists(res_path) else {}
    res["planet"] = {"r_m": r_m, "sea": sea, "mv": mv, "classes_found": classes is not None,
                     "pg_lambda_c": find_key(pgm.get("notes", {}), "lambda_c"),
                     "pg_projection": {k: (pgm.get("projection") or {}).get(k) for k in ("hemisphere_blocks", "lambda_c_deg")}}
    parts = a.parts.split(",")
    if "today" in parts:
        part_today(L, Lc, P, Pc, sea, mv, r_m, classes, res, a.out)
        json.dump(res, open(res_path, "w"), indent=1)
    tables = sorted(glob.glob(os.path.join(a.work, "cube", "cube-tau*.npz")))
    if "cube" in parts:
        pgp = pgm.get("projection") or {}
        pg = (P, Pc, int(pgp.get("hemisphere_blocks", 0)), float(pgp.get("lambda_c_deg") or 0.0))
        others = other_planets(a.others.split(","), a.work, a.bin, a.old_bin, pg) if a.others else {}
        _, _, _, aspects = part_cube(L, Lc, sea, mv, r_m, classes, tables, res, a.out, others)
        json.dump(res, open(res_path, "w"), indent=1)
        for name, Q in aspects.items():
            net_images(tables, sea, L, Lc, Q, name, a.out)
    if "other" in parts:
        part_other(L, Lc, P, Pc, sea, mv, r_m, manifest, res)
        json.dump(res, open(res_path, "w"), indent=1)
    if "charts" in parts and "cube_maps" in res:
        charts(res, a.out)
    json.dump(res, open(res_path, "w"), indent=1)
    print("wrote", res_path)


if __name__ == "__main__":
    main()
