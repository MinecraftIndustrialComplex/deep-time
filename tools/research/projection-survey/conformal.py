#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Conformal maps of flat polyhedral worlds onto the sphere, by Schwarz triangle functions.

Research tool for docs/research/notes/seamless-projection-survey.md (2026-10-03). Not shipped.

A conformal world built on a polyhedron (cube, tetrahedron, the Peirce pillowcase) is, on one
fundamental triangle of its symmetry group, the conformal map from a flat triangle with angles
pi*(a0, a1, a2) onto the spherical triangle with angles pi*(b0, b1, b2) that sends vertex to
vertex. Both go to the upper half plane: the flat one by Schwarz-Christoffel,
    z(t) = t^a0 / a0 * 2F1(a0, 1 - a1; a0 + 1; t),
the spherical one by Schwarz's s-function (exponent differences b0, b1, b2 at 0, 1, infinity),
    zeta(t) = B t^b0 2F1(a - c + 1, b - c + 1; 2 - c; t) / 2F1(a, b; c; t),
    c = 1 - b0, a = (1 - b0 - b1 + b2)/2, b = (1 - b0 - b1 - b2)/2,
with zeta the stereographic coordinate (vertex 0 at the pole) and B fixed by the spherical
distance from vertex 0 to vertex 1 (Gauss's sum gives both 2F1 at t = 1). The world point z is
inverted to t by Newton's method, started from the nearest point of a table.

The Peirce case (flat 1/4, 1/2, 1/4; spherical 1/4, 1/2, 1/2) checks the machinery against
Peirce's elliptic-function formula (survey.py).
"""

import json
import math
import sys

import mpmath as mp
import numpy as np
import scipy.special as sp
from scipy.spatial import cKDTree


def gauss1(a, b, c):
    return math.gamma(c) * math.gamma(c - a - b) / (math.gamma(c - a) * math.gamma(c - b))


class Schwarz:
    def __init__(self, fa, sb, d01, use_mp=False):
        self.fa, self.sb = fa, sb
        a0, a1, a2 = fa
        assert abs(sum(fa) - 1) < 1e-12
        b0, b1, b2 = sb
        self.c = 1 - b0
        self.a = (1 - b0 - b1 + b2) / 2
        self.b = (1 - b0 - b1 - b2) / 2
        self.use_mp = use_mp
        # flat: z(1) on the positive real axis; scale so that |z(1)| = 1 (vertex 1 at z = 1)
        z1 = gauss1(a0, 1 - a1, a0 + 1) / a0
        self.A = 1 / z1
        r1 = gauss1(self.a - self.c + 1, self.b - self.c + 1, 2 - self.c) / gauss1(self.a, self.b, self.c)
        self.B = math.tan(d01 / 2) / r1
        self.sphere_area = (sum(sb) - 1) * math.pi

    def h(self, a, b, c, t):
        if self.use_mp:
            return np.array([complex(mp.hyp2f1(a, b, c, complex(x))) for x in np.ravel(t)]).reshape(np.shape(t))
        return sp.hyp2f1(a, b, c, t)

    def z(self, t):
        a0, a1, _ = self.fa
        return self.A * t ** a0 / a0 * self.h(a0, 1 - a1, a0 + 1, t)

    def dz(self, t):
        a0, a1, _ = self.fa
        return self.A * t ** (a0 - 1) * (1 - t) ** (a1 - 1)

    def zeta(self, t):
        a, b, c = self.a, self.b, self.c
        b0 = self.sb[0]
        return self.B * t ** b0 * self.h(a - c + 1, b - c + 1, 2 - c, t) / self.h(a, b, c, t)

    def vertex2(self):
        """z of vertex 2 (t -> infinity): the flat triangle's third corner from its angles."""
        a0, a1, a2 = self.fa
        # triangle with vertex 0 at 0, vertex 1 at 1, angle pi*a0 at 0 and pi*a1 at 1
        ang0, ang1 = math.pi * a0, math.pi * a1
        L = math.sin(ang1) / math.sin(math.pi * a2)
        return L * complex(math.cos(ang0), math.sin(ang0))

    def invert(self, Z, iters=40):
        # table of t in the upper half plane, log-polar about 0 and about 1
        rr = np.concatenate([np.logspace(-8, 4, 500)])
        th = np.linspace(1e-4, math.pi - 1e-4, 200)
        T0 = (rr[:, None] * np.exp(1j * th[None, :])).ravel()
        T1 = 1 + (np.logspace(-8, 0, 300)[:, None] * np.exp(1j * th[None, :])).ravel()
        T = np.concatenate([T0, T1])
        ZT = self.z(T)
        good = np.isfinite(ZT)
        T, ZT = T[good], ZT[good]
        tree = cKDTree(np.stack([ZT.real, ZT.imag], 1))
        _, idx = tree.query(np.stack([Z.real, Z.imag], 1))
        t = T[idx].copy()
        # near vertex 0 the table is too coarse: start from the leading term z ~ A t^a0 / a0
        a0 = self.fa[0]
        t0 = (Z * a0 / self.A) ** (1 / a0)
        t = np.where(np.abs(t0) < 1e-6, t0, t)
        for _ in range(iters):
            f = self.z(t) - Z
            step = f / self.dz(t)
            # damp steps that would leave the closed upper half plane
            tn = t - step
            tn = np.where(tn.imag < 0, tn.real + 1e-14j, tn)
            t = tn
        err = np.abs(self.z(t) - Z)
        return t, err

    def scale(self, t):
        """Planet radians per world unit (conformal: the same in every direction)."""
        d = 1e-6 * np.maximum(np.abs(t), 1e-280)
        dzeta = (self.zeta(t + d) - self.zeta(t - d)) / (2 * d)
        zt = self.zeta(t)
        return np.abs(dzeta / self.dz(t)) * 2 / (1 + np.abs(zt) ** 2), zt


def triangle_grid(v2, n):
    """World-uniform sample of the flat triangle (0, 1, v2): cell centroids of an n-subdivision."""
    pts = []
    for i in range(n):
        for j in range(n - i):
            # two kinds of small triangles
            for (a, b) in ((i + 1 / 3, j + 1 / 3),) + (((i + 2 / 3, j + 2 / 3),) if i + j < n - 1 else ()):
                u, v = a / n, b / n
                pts.append(u * 1 + v * v2)
    return np.array(pts)


QS = [0.0, 0.01, 0.05, 0.10, 0.50, 0.90, 0.95, 0.99, 0.999, 1.0]


def wquant(vals, w, qs):
    o = np.argsort(vals)
    v, ww = vals[o], w[o]
    c = np.cumsum(ww)
    c /= c[-1]
    return [float(v[min(np.searchsorted(c, q), len(v) - 1)]) for q in qs]


CASES = {
    # name: flat angles, spherical angles, spherical distance vertex0 -> vertex1, copies of the
    # triangle on the sphere, note, spherical distance vertex0 -> vertex2
    "peirce": ((0.25, 0.5, 0.25), (0.25, 0.5, 0.5), math.pi / 2, 16,
               "pillowcase (Peirce/Guyou): pole, equator point under the tile edge midpoint, singular corner",
               math.pi / 2),
    "cube": ((0.25, 0.5, 0.25), (0.25, 0.5, 1 / 3), math.pi / 4, 48,
             "conformal cube: face centre, edge midpoint, corner", math.atan(math.sqrt(2))),
    "tetra": ((1 / 3, 0.5, 1 / 6), (1 / 3, 0.5, 1 / 3), math.atan(math.sqrt(2)), 24,
              "Lee conformal tetrahedral: face centre, edge midpoint, vertex", math.acos(1 / 3)),
    "octa": ((1 / 3, 0.5, 1 / 6), (1 / 3, 0.5, 0.25), math.atan(1 / math.sqrt(2)), 48,
             "conformal octahedral (cone points 240 deg: not grid-compatible), reference",
             math.atan(math.sqrt(2))),
}
# spherical distances: cube face centre to edge midpoint 45 deg, to corner atan(sqrt 2) = 54.74 deg;
# tetrahedron face centre -(1,1,1)/sqrt3 to edge midpoint (0,0,-1): acos(1/sqrt3) = 54.74 deg, to a
# vertex acos(1/3) = 70.53 deg; octahedron face centre (1,1,1)/sqrt3 to edge midpoint (1,1,0)/sqrt2:
# atan(1/sqrt2) = 35.26 deg, to a vertex (1,0,0): 54.74 deg.


def run(name, n=300, check_mp=0):
    fa, sb, d01, copies, note, d02 = CASES[name]
    S = Schwarz(fa, sb, d01)
    v2 = S.vertex2()
    Z = triangle_grid(v2, n)
    t, err = S.invert(Z)
    sc, zt = S.scale(t)
    world_area = abs(v2.imag) / 2  # triangle (0, 1, v2)
    mean = S.sphere_area / world_area  # planet area per world area, mean
    ar = sc ** 2
    rel = ar / mean
    ok = np.isfinite(rel) & (err < 1e-8)
    bpa = 1 / rel
    w = ar
    # vertex 2's image: check the spherical distance pole -> vertex 2
    tt = np.array([1e6 * complex(math.cos(math.pi / 2), math.sin(math.pi / 2)) + 0.0])
    out = {
        "note": note,
        "newton_err_max": float(np.max(err)),
        "share_ok": float(ok.mean()),
        "planet_area_check": float((ar[ok].sum() * world_area / ok.sum()) / S.sphere_area),
        "blocks_per_planet_area_rel_q": dict(zip([str(q) for q in QS], wquant(bpa[ok], w[ok], QS))),
        "share_area_within_x1.25": float(w[ok & (bpa <= 1.25) & (bpa >= 0.8)].sum() / w[ok].sum()),
        "share_area_within_x2": float(w[ok & (bpa <= 2) & (bpa >= 0.5)].sum() / w[ok].sum()),
        "max_over_min_excluding_5deg_discs": None,
        "zeta_vertex1": complex(S.zeta(np.array([1 - 1e-12 + 0j]))[0]).__repr__(),
    }
    # exclude discs round the singular vertex (vertex 2 for peirce, cube, tetra): the planet-angular
    # distance from vertex 2's image, approximated from zeta
    ang = math.pi * sb[0]
    pv = np.array([math.sin(d02) * math.cos(ang), math.sin(d02) * math.sin(ang), math.cos(d02)])
    zv2 = S.zeta(np.array([1e6j]))[0]
    p = np.stack([2 * zt.real, 2 * zt.imag, 1 - np.abs(zt) ** 2], 1) / (1 + np.abs(zt) ** 2)[:, None]
    pv_num = np.array([2 * zv2.real, 2 * zv2.imag, 1 - abs(zv2) ** 2]) / (1 + abs(zv2) ** 2)
    out["vertex2_check_deg"] = float(np.degrees(np.arccos(np.clip(pv_num @ pv, -1, 1))))
    dist = np.degrees(np.arccos(np.clip(p @ pv, -1, 1)))
    out["vertex2_lat_from_vertex0_deg"] = float(90 - np.degrees(np.arccos(pv[2])))
    o = np.argsort(-np.where(ok, bpa, -1))[:5]
    out["worst_points"] = [[float(bpa[i]), float(dist[i]), float(abs(t[i])), float(err[i])] for i in o]
    for r in (5, 10, 20):
        m = ok & (dist > r)
        out[f"max_over_min_beyond_{r}deg"] = float(bpa[m].max() / bpa[m].min())
        out[f"planet_share_within_{r}deg"] = float(w[ok & (dist <= r)].sum() / w[ok].sum())
    out.pop("max_over_min_excluding_5deg_discs")
    if check_mp:
        Smp = Schwarz(fa, sb, d01, use_mp=True)
        sel = np.linspace(0, len(t) - 1, check_mp).astype(int)
        zz = Smp.zeta(t[sel])
        out["mp_check_zeta_maxdiff"] = float(np.max(np.abs(zz - zt[sel])))
        z2 = Smp.z(t[sel])
        out["mp_check_z_maxdiff"] = float(np.max(np.abs(z2 - Z[sel])))
        tw = t[o]
        scw, _ = Smp.scale(tw)
        out["worst_points_mp_bpa"] = [float(mean / x ** 2) for x in scw]
    return out


if __name__ == "__main__":
    names = sys.argv[1:] or list(CASES)
    res = {}
    for nm in names:
        res[nm] = run(nm, n=300, check_mp=200)
        print(json.dumps({nm: res[nm]}, indent=1), flush=True)
