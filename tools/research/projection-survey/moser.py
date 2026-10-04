#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Equal-area worlds that stay smooth: a conformal map made equal-area by Moser's flow.

Research tool for docs/research/notes/seamless-projection-survey.md (2026-10-03). Not shipped.

Start from a conformal tile map F (the conformal cube's face, or Peirce's hemisphere square), whose
planet area per world area rho (normalised to mean 1 on the tile) is smooth except at the tile's
singular corners. Solve the Neumann problem  Lap u = rho - 1  on the tile (a cosine transform: the
tile's edges and diagonals are mirror lines of the whole world, so Neumann is exact), and flow every
world point along  dx/dt = -grad u / ((1 - t) + t rho)  from t = 0 to 1 (Moser 1965; the same flow
that diffusion cartograms use). The time-1 map phi pushes the uniform world measure onto rho, so
G = F o phi is equal-area. phi keeps the tile's edges and is mirror-symmetric across them, so G is
as smooth across every seam and tile edge as F is: no creases. The construction is the author's
(this note's) composition of two published pieces; it is not a published projection.

Outputs a table of G on the tile (for the survey harness and the benchmark) and its measurements.
"""

import json
import math
import sys

import numpy as np
from scipy.fft import dctn, idctn
from scipy.interpolate import RectBivariateSpline
from scipy.ndimage import map_coordinates

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import conformal  # noqa: E402
import survey  # noqa: E402


def conformal_tile(case, U, V):
    """Conformal tile map on [-1,1]^2 for case 'cube' (face centre at the pole, edge midpoints at
    distance 45 deg, corners singular) or 'peirce' (hemisphere square: pole at the centre, equator on
    the boundary). Returns unit vectors (face frame: z = tile centre) and the scale sigma."""
    fa, sb, d01, _, _, d02 = conformal.CASES[case]
    S = conformal.Schwarz(fa, sb, d01)
    au, av = np.abs(U), np.abs(V)
    swap = av > au
    x = np.where(swap, av, au)
    y = np.where(swap, au, av)
    corner = (np.abs(x - 1) < 1e-9) & (np.abs(y - 1) < 1e-9)
    # keep off the edge E-V (t real > 1 is the hypergeometric branch cut) by a hair
    xe = np.minimum(x, 1 - 1e-11)
    ye = np.minimum(y, xe)
    Z = (xe + 1j * ye).ravel()
    t, err = S.invert(Z)
    sig, zt = S.scale(t)
    zt = zt.reshape(U.shape)
    sig = sig.reshape(U.shape)
    # the singular corner (t -> infinity) is not reached by Newton: put it where it belongs
    zt = np.where(corner, math.tan(d02 / 2) * np.exp(1j * math.pi * sb[0]), zt)
    zt = np.where((x == 0) & (y == 0), 0j, zt)  # the tile centre is the pole
    zt = np.where(swap, 1j * np.conj(zt), zt)
    zt = np.where(U < 0, -np.conj(zt), zt)
    zt = np.where(V < 0, np.conj(zt), zt)
    bad = ~np.isfinite(zt) & ~corner
    if bad.any():
        print(f"conformal_tile: {int(bad.sum())} non-finite of {bad.size}; e.g. x, y =",
              x[bad][:5], y[bad][:5], file=sys.stderr)
    return survey.zeta2xyz(zt), sig, float(np.max(err[np.isfinite(err)]))


def solve_neumann(f, h):
    """Lap u = f on a cell-centred grid with zero normal derivative (DCT-II)."""
    n, m = f.shape
    fh = dctn(f, type=2, norm="ortho")
    kx = (2 * np.cos(np.pi * np.arange(n) / n) - 2) / h ** 2
    ky = (2 * np.cos(np.pi * np.arange(m) / m) - 2) / h ** 2
    lam = kx[:, None] + ky[None, :]
    lam[0, 0] = 1
    uh = fh / lam
    uh[0, 0] = 0
    return idctn(uh, type=2, norm="ortho")


def build(case, n=512, steps=400):
    h = 2.0 / n
    c = -1 + (np.arange(n) + 0.5) * h  # cell centres
    U, V = np.meshgrid(c, c, indexing="ij")
    P, sig, err = conformal_tile(case, U, V)
    tile_planet = 4 * math.pi / (6 if case == "cube" else 2)
    rho = sig ** 2 / (tile_planet / 4.0)
    rho_mean = rho.mean()
    rho = rho / rho_mean  # remove the grid's quadrature error at the singular corners
    u = solve_neumann(rho - 1, h)
    gu = np.gradient(u, h, edge_order=2)
    gx, gy = gu[0], gu[1]

    def interp(field, X, Y):
        # grid index coordinates for cell-centred samples; mirror at the edges (Neumann)
        ix = (X + 1) / h - 0.5
        iy = (Y + 1) / h - 0.5
        return map_coordinates(field, [ix, iy], order=1, mode="mirror")

    def vel(X, Y, t):
        r = (1 - t) + t * interp(rho, X, Y)
        return -interp(gx, X, Y) / r, -interp(gy, X, Y) / r

    # start points: a vertex grid including the edges, so the table spans the closed tile
    m = n + 1
    s = np.linspace(-1, 1, m)
    X0, Y0 = np.meshgrid(s, s, indexing="ij")
    X, Y = X0.copy(), Y0.copy()
    dt = 1.0 / steps
    for k in range(steps):
        t = k * dt
        k1 = vel(X, Y, t)
        k2 = vel(X + 0.5 * dt * k1[0], Y + 0.5 * dt * k1[1], t + 0.5 * dt)
        k3 = vel(X + 0.5 * dt * k2[0], Y + 0.5 * dt * k2[1], t + 0.5 * dt)
        k4 = vel(X + dt * k3[0], Y + dt * k3[1], t + dt)
        X = X + dt / 6 * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0])
        Y = Y + dt / 6 * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1])
        X = np.clip(X, -1, 1)
        Y = np.clip(Y, -1, 1)
    # keep the edges exactly on the edges (the flow preserves them up to interpolation error)
    X[0, :], X[-1, :] = -1, 1
    Y[:, 0], Y[:, -1] = -1, 1
    G, _, err2 = conformal_tile(case, X, Y)
    return s, G, {"newton_err": max(err, err2), "rho_grid_mean_before_renorm": float(rho_mean),
                  "rho_min": float(rho.min()), "rho_max": float(rho.max())}


def spline_face(s, G):
    sps = [RectBivariateSpline(s, s, G[..., i], kx=3, ky=3) for i in range(3)]

    def face(u, v):
        out = np.stack([sp.ev(u, v) for sp in sps], axis=-1)
        return survey.normalize(out)
    return face


def main(argv):
    case = argv[1] if len(argv) > 1 else "cube"
    n = int(argv[2]) if len(argv) > 2 else 512
    s, G, info = build(case, n=n)
    np.savez_compressed(f"/tmp/moser-{case}-{n}.npz", s=s, G=G)
    # the conformal map itself through the same harness (spline of the same grid), for comparison
    X0, Y0 = np.meshgrid(s, s, indexing="ij")
    G0, _, _ = conformal_tile(case, X0, Y0)
    face0 = spline_face(s, G0)
    if case == "cube":
        w0 = survey.cube_world("cube-conformal", face0, "conformal cube (Lee 1976 / Rancic et al. 1996), Schwarz maps")
    else:
        w0 = survey.pillowcase("pillowcase-peirce-schwarz", face0, "Peirce via Schwarz maps (check)")
    d0 = survey.distortion(w0, n=300)
    cr0 = survey.creases(w0)
    print(json.dumps({w0.name: {"distortion": d0, "creases": survey.summarize_creases(cr0)}}, indent=1), flush=True)
    face = spline_face(s, G)
    if case == "cube":
        w = survey.cube_world(f"cube-moser-conformal-n{n}", face,
                              "conformal cube made equal-area by Moser's flow (this note)")
    else:
        w = survey.pillowcase(f"pillowcase-moser-peirce-n{n}", face,
                              "Peirce made equal-area by Moser's flow (this note)")
    d = survey.distortion(w, n=300)
    cr = survey.creases(w)
    print(json.dumps({w.name: {"build": info, "distortion": d,
                               "creases": survey.summarize_creases(cr)}}, indent=1), flush=True)


if __name__ == "__main__":
    main(sys.argv)
