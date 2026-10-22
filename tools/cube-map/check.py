#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The map table's checks (docs/plans/cube-world-plan.md M1), on the bicubic the game reads:

  - convergence: tables at n = 257, 513 and 1,025 (m = 128, 256, 512) agree to the bound printed,
    as planet angle and as blocks at the 16k and 32k face sizes;
  - positivity: the bicubic's Jacobian determinant is positive at 8 x 8 points in every cell of
    the triangle and on a 400 x 400 grid within one block of the corner at 32k (no fold-overs);
  - statistics: area and shape on 300 x 300 cells of a face, weighted by planet area as the
    projection survey's distortion() (seamless-projection-survey.md §3.1), to compare with its
    numbers for the chosen tau;
  - slope jumps across a face edge (the survey's crease measure) and the exact edge values.

    check.py --tables T257.bin T513.bin T1025.bin [--committed T513.bin] --out check.json

Runs on the test VM (tools/cube-map/run.sh check).
"""

import argparse
import json
import math
import sys

import numpy as np

import cube_map as cm

# The face sides at 16k and 32k (ledger SCL-078, SCL-100) and the world radius in blocks they draw
# (6 s^2 = 4 pi R^2).
FACE = {"16k": 3840, "32k": 7424}


def radius_blocks(s):
    return s * math.sqrt(6.0 / (4.0 * math.pi))


def face_forward(P, u, v):
    """Unit vectors (components along the face centre, e_u, e_v) at face points (u, v) in
    [-1, 1]^2, as deeptime_format::cube_map::CubeMap::forward."""
    u = np.asarray(u, dtype=float)
    v = np.asarray(v, dtype=float)
    au, av = np.abs(u), np.abs(v)
    swap = av > au
    x = np.where(swap, av, au)
    y = np.where(swap, au, av)
    val = cm.tri_eval(P, x, y)
    a, b = val[..., 0], val[..., 1]
    r = np.sqrt((1.0 + a * a) + b * b)
    A = np.where(swap, b, a)
    B = np.where(swap, a, b)
    A = np.where(u < 0, -A, A)
    B = np.where(v < 0, -B, B)
    return np.stack([1.0 / r, A / r, B / r], -1)


def angle(p, q):
    return 2.0 * np.arcsin(np.clip(np.linalg.norm(p - q, axis=-1) / 2.0, 0, 1))


def sample_points():
    g = np.linspace(-1, 1, 401)
    U, V = np.meshgrid(g, g, indexing="ij")
    pts = [np.stack([U.ravel(), V.ravel()], -1)]
    t = np.linspace(-1, 1, 2001)
    for e in (-1.0, 1.0):
        pts.append(np.stack([np.full_like(t, e), t], -1))
        pts.append(np.stack([t, np.full_like(t, e)], -1))
    c = 1.0 - np.linspace(0, 2.0 / 3712, 41)  # within one block of the corner at 32k
    CU, CV = np.meshgrid(c, c, indexing="ij")
    pts.append(np.stack([CU.ravel(), CV.ravel()], -1))
    return np.concatenate(pts)


def convergence(tables):
    pts = sample_points()
    ps = {}
    for t in tables:
        P = cm.padded(t["A"], t["B"])
        ps[t["m"]] = face_forward(P, pts[:, 0], pts[:, 1])
    ms = sorted(ps)
    out = {}
    for lo in ms[:-1]:
        hi = ms[-1]
        ang = angle(ps[lo], ps[hi])
        key = f"n{2 * lo + 1}_vs_n{2 * hi + 1}"
        out[key] = {"max_rad": float(ang.max()), "rms_rad": float(np.sqrt(np.mean(ang ** 2)))}
        for k, s in FACE.items():
            out[key][f"max_blocks_{k}"] = float(ang.max() * radius_blocks(s))
    if len(ms) >= 3:
        a = angle(ps[ms[0]], ps[ms[-1]]).max()
        b = angle(ps[ms[1]], ps[ms[-1]]).max()
        out["observed_order"] = float(math.log2(a / b)) if b > 0 else None
    return out


def positivity(t, s32=FACE["32k"]):
    P = cm.padded(t["A"], t["B"])
    m = t["m"]
    sub = (np.arange(8) + 0.0) / 7.0  # cell corners and six points between
    out = {}
    worst = (math.inf, None)
    total = 0
    for i0 in range(m):
        uu = (i0 + sub) / m
        U, V = np.meshgrid(uu, np.concatenate([(j + sub) / m for j in range(i0 + 1)]), indexing="ij")
        keep = V <= U + 1e-15
        u, v = U[keep], np.minimum(V[keep], U[keep])
        _, du, dv = cm.tri_eval(P, u, v, derivs=True)
        det = du[:, 0] * dv[:, 1] - dv[:, 0] * du[:, 1]
        total += det.size
        k = int(np.argmin(det))
        if det[k] < worst[0]:
            worst = (float(det[k]), [float(u[k]), float(v[k])])
    out["cells_min_det"] = worst[0]
    out["cells_min_det_at"] = worst[1]
    out["cells_points"] = total
    one = 2.0 / s32  # one block at 32k in face units
    c = 1.0 - np.linspace(0, one, 400)
    U, V = np.meshgrid(c, c, indexing="ij")
    keep = V <= U
    _, du, dv = cm.tri_eval(P, U[keep], V[keep], derivs=True)
    det = du[:, 0] * dv[:, 1] - dv[:, 0] * du[:, 1]
    out["corner_32k_min_det"] = float(det.min())
    out["corner_32k_points"] = int(det.size)
    _, du, dv = cm.tri_eval(P, np.array([0.0]), np.array([0.0]), derivs=True)
    out["centre_det"] = float(du[0, 0] * dv[0, 1] - dv[0, 0] * du[0, 1])
    out["positive"] = bool(out["cells_min_det"] > 0 and out["corner_32k_min_det"] > 0)
    return out


QS = [0.0, 0.01, 0.5, 0.9, 0.99, 0.999, 1.0]


def wquant(vals, w, qs):
    o = np.argsort(vals)
    v, ww = vals[o], w[o]
    c = np.cumsum(ww)
    c /= c[-1]
    return [float(v[min(np.searchsorted(c, q), len(v) - 1)]) for q in qs]


def stats(t, n=300, h=1e-7):
    P = cm.padded(t["A"], t["B"])
    c = -1 + (np.arange(n) + 0.5) * (2.0 / n)
    U, V = np.meshgrid(c, c, indexing="ij")
    u, v = U.ravel(), V.ravel()
    pu = (face_forward(P, u + h, v) - face_forward(P, u - h, v)) / (2 * h)
    pv = (face_forward(P, u, v + h) - face_forward(P, u, v - h)) / (2 * h)
    E = np.einsum("ij,ij->i", pu, pu)
    F = np.einsum("ij,ij->i", pu, pv)
    G = np.einsum("ij,ij->i", pv, pv)
    area = np.sqrt(E * G - F * F)
    disc = np.sqrt((E - G) ** 2 + 4 * F * F)
    s1 = np.sqrt((E + G + disc) / 2)
    s2 = np.sqrt(np.maximum((E + G - disc) / 2, 0))
    mean = area.mean()
    rel = area / mean
    bpa = 1 / rel
    shape = s1 / s2
    w = area
    face_area = float(area.sum() * (2.0 / n) ** 2)
    return {
        "planet_area_of_face_over_4pi_6": face_area / (4 * math.pi / 6),
        "blocks_per_planet_area_rel_q": dict(zip([str(q) for q in QS], wquant(bpa, w, QS))),
        "area_cv": float(np.sqrt(np.mean((rel - 1) ** 2))),
        "shape_ratio_q": dict(zip([str(q) for q in QS], wquant(shape, w, QS))),
        "two_omega_max_deg": float(np.degrees(2 * np.arcsin((s1 - s2) / (s1 + s2))).max()),
        "samples": int(u.size),
    }


def edge_jump(t, d=1e-5, k=2001):
    """The survey's crease measure across the edge u = 1: the derivative across the edge from
    this face against the neighbour's (the reflection of this face across the edge's plane,
    (x, y, z) -> (y, x, z) in the frame (centre, e_u, e_v))."""
    P = cm.padded(t["A"], t["B"])
    v = np.linspace(-1, 1, k)[1:-1]
    p1 = face_forward(P, np.ones_like(v), v)
    p0 = face_forward(P, np.full_like(v, 1 - d), v)
    refl = p0[:, [1, 0, 2]]
    da = (p1 - p0) / d
    db = (refl - p1) / d
    jump = np.linalg.norm(da - db, axis=-1) / np.linalg.norm(da, axis=-1)
    gap = angle(p1, p1[:, [1, 0, 2]])  # the edge point and its own reflection: one point
    return {"slope_jump_max": float(jump.max()), "slope_jump_median": float(np.median(jump)),
            "slope_jump_max_at_v": float(v[int(np.argmax(jump))]), "edge_self_gap_rad": float(gap.max())}


def exactness(t):
    A, B = t["A"], t["B"]
    m = t["m"]
    return {
        "edge_a_is_1": bool(np.all(A[m, : m + 1] == 1.0)),
        "axis_b_is_0": bool(np.all(B[:, 0] == 0.0)),
        "diagonal_a_eq_b": bool(np.all(np.diag(A) == np.diag(B))),
        "corner": [float(A[m, m]), float(B[m, m])],
    }


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--tables", nargs="+", required=True)
    ap.add_argument("--committed", default=None)
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv[1:])
    tables = [cm.read_table(p) for p in args.tables]
    res = {"tables": [{k: t[k] for k in ("name", "m", "q", "tau", "commit", "sha256", "bytes")} for t in tables]}
    res["convergence"] = convergence(tables)
    print(json.dumps({"convergence": res["convergence"]}, indent=1), flush=True)
    main_t = cm.read_table(args.committed) if args.committed else tables[len(tables) // 2]
    res["checked"] = main_t["name"]
    res["exact"] = exactness(main_t)
    res["positivity"] = positivity(main_t)
    print(json.dumps({"positivity": res["positivity"]}, indent=1), flush=True)
    res["stats"] = stats(main_t)
    res["edge"] = edge_jump(main_t)
    res["positivity_all"] = {t["name"]: positivity(t)["positive"] for t in tables}
    print(json.dumps({k: res[k] for k in ("exact", "stats", "edge", "positivity_all")}, indent=1), flush=True)
    with open(args.out, "w") as f:
        json.dump(res, f, indent=1)


if __name__ == "__main__":
    main(sys.argv)
