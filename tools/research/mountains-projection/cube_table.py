#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The cube maps of the mountains study as tables: the conformal cube (Lee 1976; Rancic, Purser and
Mesinger 1996), the survey's equal-area cube (that map made equal-area by Moser's flow,
seamless-projection-survey.md §4), and the maps in between, Moser's flow stopped at time tau.

Research tool for docs/research/notes/mountains-and-the-projection.md. Not shipped. The flow is the
survey's (tools/research/projection-survey/moser.py, the same grid, the same RK4 steps): world points
move along dx/dt = -grad u / ((1 - t) + t rho), Lap u = rho - 1, rho the conformal map's planet area
per world area (mean 1). Stopped at tau, the map G_tau = F o phi_tau draws planet area per world area
rho / ((1 - tau) + tau rho) at the planet point F(phi_tau(x)): its blocks per planet area are
tau + (1 - tau) / rho, the conformal map's blocks per planet area moved a share tau of the way to 1.
tau = 0 is the conformal cube, tau = 1 the survey's equal-area cube. The in-between maps are this
study's own construction (a published flow stopped early), not published projections.

    cube_table.py OUT_DIR [n] [tau ...]

writes OUT_DIR/cube-tau<tau>.npz for each tau (default 0 0.25 0.5 0.625 0.75 0.875 1): the
(n + 1)^2 vertex table of unit vectors (face frame: z the face centre) and its nodes.
"""

import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "projection-survey"))
import moser  # noqa: E402
from scipy.ndimage import map_coordinates  # noqa: E402

import math  # noqa: E402


def build(n, taus, steps=400):
    h = 2.0 / n
    c = -1 + (np.arange(n) + 0.5) * h
    U, V = np.meshgrid(c, c, indexing="ij")
    _, sig, err = moser.conformal_tile("cube", U, V)
    rho = sig ** 2 / ((4 * math.pi / 6) / 4.0)
    rho = rho / rho.mean()
    u = moser.solve_neumann(rho - 1, h)
    gx, gy = np.gradient(u, h, edge_order=2)

    def interp(field, X, Y):
        return map_coordinates(field, [(X + 1) / h - 0.5, (Y + 1) / h - 0.5], order=1, mode="mirror")

    def vel(X, Y, t):
        r = (1 - t) + t * interp(rho, X, Y)
        return -interp(gx, X, Y) / r, -interp(gy, X, Y) / r

    s = np.linspace(-1, 1, n + 1)
    X0, Y0 = np.meshgrid(s, s, indexing="ij")
    X, Y = X0.copy(), Y0.copy()
    stops = {int(round(t * steps)): t for t in taus}
    out = {}
    dt = 1.0 / steps
    for k in range(steps + 1):
        if k in stops:
            Xs, Ys = X.copy(), Y.copy()
            Xs[0, :], Xs[-1, :] = -1, 1
            Ys[:, 0], Ys[:, -1] = -1, 1
            out[stops[k]] = (Xs, Ys)
        if k == steps:
            break
        t = k * dt
        k1 = vel(X, Y, t)
        k2 = vel(X + 0.5 * dt * k1[0], Y + 0.5 * dt * k1[1], t + 0.5 * dt)
        k3 = vel(X + 0.5 * dt * k2[0], Y + 0.5 * dt * k2[1], t + 0.5 * dt)
        k4 = vel(X + dt * k3[0], Y + dt * k3[1], t + dt)
        X = np.clip(X + dt / 6 * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]), -1, 1)
        Y = np.clip(Y + dt / 6 * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]), -1, 1)
    tables = {}
    for t, (Xs, Ys) in out.items():
        G, _, e2 = moser.conformal_tile("cube", Xs, Ys)
        tables[t] = G
        print(f"tau {t}: newton err {max(err, e2):.2e}", flush=True)
    return s, tables


def main(argv):
    out = argv[1]
    n = int(argv[2]) if len(argv) > 2 else 512
    taus = [float(a) for a in argv[3:]] or [0.0, 0.25, 0.5, 0.625, 0.75, 0.875, 1.0]
    os.makedirs(out, exist_ok=True)
    s, tables = build(n, taus)
    for t, G in tables.items():
        np.savez_compressed(os.path.join(out, f"cube-tau{t:.3f}.npz"), s=s, G=G, tau=t)


if __name__ == "__main__":
    main(sys.argv)
