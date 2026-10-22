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

def corner_distances(Q, land_tree):
    """Each canonical corner's angular distance to land (degrees, capped) for rotations Q[n,3,3]
    planet -> canonical; corner in the planet frame = Q^T c."""
    pts = np.einsum("nji,kj->nki", Q, CORNERS)  # (n, 8, 3): Q^T c
    d, _ = land_tree.query(pts.reshape(-1, 3), workers=-1)
    return np.minimum(ang_deg(d), LAND_CAP_DEG).reshape(pts.shape[:2])


def polar_search(land_tree):
    lam = np.arange(0, 90, STEP_DEG)
    Q = np.stack([geom.rot_z(-x) for x in lam])
    D = corner_distances(Q, land_tree)
    mn = D.min(1)
    sm = D.sum(1)
    best = int(np.lexsort((-sm, -mn))[0])
    return lam, D, {"lambda0_deg": float(lam[best]), "min_distance_deg": round(float(mn[best]), 2),
                    "distances_deg": [round(float(x), 2) for x in D[best]],
                    "share_at_sea": round(float((mn > 0).mean()), 4),
                    "share_3deg": round(float((mn >= 3).mean()), 4),
                    "share_5deg": round(float((mn >= 5).mean()), 4),
                    "share_10deg": round(float((mn >= 10).mean()), 4)}


def free_search(land_tree, n=60000, seed=7):
    Q = Rotation.random(n, random_state=seed).as_matrix()
    D = corner_distances(Q, land_tree)
    mn = D.min(1)
    best = int(np.argmax(mn + 1e-3 * D.sum(1)))
    return Q[best], {"rotations": n, "best_min_distance_deg": round(float(mn[best]), 2),
                     "share_at_sea": round(float((mn > 0).mean()), 4),
                     "share_3deg": round(float((mn >= 3).mean()), 4),
                     "share_5deg": round(float((mn >= 5).mean()), 4),
                     "share_10deg": round(float((mn >= 10).mean()), 4)}


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
    land_tree = cKDTree(p[land])
    lam, D, polar = polar_search(land_tree)
    Qfree, free = free_search(land_tree)
    res["corners_this_planet"] = {"polar_aspect": polar, "free_rotation": free}
    np.save(os.path.join(out, "polar-corner-distances.npy"), D)

    # Other planets: the same two searches on their land.
    op = {}
    for name, (lat_o, lon_o, land_o) in others.items():
        tr = cKDTree(geom.ll2xyz(lat_o[land_o], lon_o[land_o]))
        _, _, pol = polar_search(tr)
        _, fr = free_search(tr, n=30000)
        op[name] = {"land_share": round(float(land_o.mean()), 3), "polar_aspect": pol, "free_rotation": fr}
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
            if aspect == "polar":
                idx = np.flatnonzero(m)
                idx = rng.choice(idx, size=min(40000, idx.size), replace=False)
                for dist in (256, 1024, 4096):
                    vc = view_change(cube, p[idx], f["k"][idx], Q, r_m, dist)
                    a["exaggeration_mountain_1km"][f"change_within_{dist}_blocks_q"] = q(vc, [0.5, 0.9, 0.99, 1.0])
            entry[aspect] = a
        # The whole planet with no placement (the map's own figures; survey §3.1 check).
        fp = cube_fields(cube, p, geom.rot_z(0.0))
        entry["planet_check"] = {"bpa": q(fp["bpa"]), "shape": q(fp["shape"]),
                                 "within_1.25": round(float(((fp["bpa"] <= 1.25) & (fp["bpa"] >= 0.8)).mean()), 4),
                                 "within_1.5": round(float(((fp["bpa"] <= 1.5) & (fp["bpa"] >= 1 / 1.5)).mean()), 4)}
        # How the worst land bpa grows as the nearest corner comes closer to land (any aspect).
        cube_res[key] = entry
        print(key, json.dumps(entry["polar"]["mountain_1km"]["bpa"]), flush=True)
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
        for lo_ in (0, 2, 3, 5, 8, 10, 15, 20, 30):
            sel = dc >= lo_
            rows[f">={lo_}deg"] = {"bpa_max": round(float(bpa[sel].max()), 4), "bpa_min": round(float(bpa[sel].min()), 4),
                                   "shape_max": round(float(sh[sel].max()), 4)}
        prof[f"tau={tau:.3f}"] = rows
    res["cube_by_corner_distance"] = prof
    return p, land, masks, polar, Qpolar


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

def charts(res, out):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    cm = res["cube_maps"]
    taus = sorted(float(k.split("=")[1]) for k in cm)
    fig, axs = plt.subplots(1, 3, figsize=(15, 4.2))
    for mk, col in (("land", "#4a7"), ("mountain_1km", "#a54")):
        hi = [cm[f"tau={t:.3f}"]["polar"][mk]["bpa"]["1.0"] for t in taus]
        lo = [cm[f"tau={t:.3f}"]["polar"][mk]["bpa"]["0.0"] for t in taus]
        axs[0].plot(taus, hi, "-o", color=col, label=f"{mk}: max")
        axs[0].plot(taus, lo, "--o", color=col, label=f"{mk}: min")
        sh = [cm[f"tau={t:.3f}"]["polar"][mk]["shape"]["1.0"] for t in taus]
        sm = [cm[f"tau={t:.3f}"]["polar"][mk]["shape"]["0.5"] for t in taus]
        axs[1].plot(taus, sh, "-o", color=col, label=f"{mk}: max")
        axs[1].plot(taus, sm, "--o", color=col, label=f"{mk}: median")
        ex = [cm[f"tau={t:.3f}"]["polar"]["exaggeration_mountain_1km"]["between_places_max_over_min"] for t in taus]
        ew = [cm[f"tau={t:.3f}"]["polar"]["exaggeration_mountain_1km"]["within_place_directions_q"]["0.9"] for t in taus]
    axs[2].plot(taus, ex, "-o", color="#a54", label="between places, max/min")
    axs[2].plot(taus, ew, "--o", color="#47a", label="within a place, directions, p90")
    axs[0].set_title("Blocks per planet area on land (corners at sea)")
    axs[1].set_title("Shape ratio on land")
    axs[2].set_title("Slope spread on mountain land")
    for ax in axs:
        ax.set_xlabel("tau (0 conformal, 1 equal-area)")
        ax.grid(alpha=0.3)
        ax.legend(fontsize=8)
    fig.tight_layout()
    fig.savefig(os.path.join(out, "cube-maps-by-tau.png"), dpi=110)
    plt.close(fig)


def net_images(tables, sea, L, Lc, polar, out):
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
        cube = geom.Cube(path, lambda0_deg=polar["lambda0_deg"])
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
        f = cube_fields(cube, U[good], None)
        B = np.full(X.shape, np.nan)
        B[good] = f["bpa"]
        ax.imshow(img, extent=(0, 4, -1, 2), origin="upper")
        cs = ax.contour(Xs, Ys, B, levels=[0.8, 0.9, 1.1, 1.25, 1.5, 2.0], colors="k", linewidths=0.6)
        ax.clabel(cs, fontsize=6)
        ax.set_title(f"tau = {tau:g}: blocks per planet area")
        ax.set_xlim(0, 4)
        ax.set_ylim(-1, 2)
        ax.set_aspect("equal")
        ax.axis("off")
    fig.tight_layout()
    fig.savefig(os.path.join(out, "cube-nets.png"), dpi=110)
    plt.close(fig)


# ----------------------------------------------------------------------------------------------

def other_planets(dirs, work, bin_):
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
            if r.returncode != 0:
                print(f"other planet {d}: heightmap failed: {r.stderr.strip()[-300:]}", flush=True)
                continue
        a = np.load(dst)
        meta = json.load(open(dst + ".json"))
        ch = {n: i for i, n in enumerate(meta["channels"])}
        sea = m["scale"]["sea_level_y"]
        lat = a[:, :, ch["lat_deg"]].astype(np.float64)
        lon = a[:, :, ch["lon_deg"]].astype(np.float64)
        E = a[:, :, ch["elevation"]].astype(np.float64)
        ok = np.isfinite(lat) & np.isfinite(E)
        out[name] = (lat[ok], lon[ok], E[ok] >= sea)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--bin", default="deeptime")
    ap.add_argument("--others", default="")
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
        others = other_planets(a.others.split(","), a.work, a.bin) if a.others else {}
        _, _, _, polar, _ = part_cube(L, Lc, sea, mv, r_m, classes, tables, res, a.out, others)
        json.dump(res, open(res_path, "w"), indent=1)
        net_images(tables, sea, L, Lc, polar, a.out)
    if "other" in parts:
        part_other(L, Lc, P, Pc, sea, mv, r_m, manifest, res)
        json.dump(res, open(res_path, "w"), indent=1)
    if "charts" in parts and "cube_maps" in res:
        charts(res, a.out)
    json.dump(res, open(res_path, "w"), indent=1)
    print("wrote", res_path)


if __name__ == "__main__":
    main()
