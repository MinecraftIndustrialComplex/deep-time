#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The cube world's map table (docs/plans/cube-world-plan.md M1; ledger SCL-080, SCL-081, SCL-101).

The production copy of the mountains study's cube_table.py, which builds on the projection
survey's moser.py and conformal.py (tools/research/projection-survey/, this repository). The pieces
it needs are copied here so the generator does not depend on the research folders.

The map family: the conformal cube (Schwarz triangle functions; Lee 1976, Rancic, Purser and
Mesinger 1996) moved by Moser's flow (Moser 1965) stopped at time tau. tau = 0 is the conformal
cube, tau = 1 the survey's equal-area cube; blocks per planet area are tau + (1 - tau) / rho, rho
the conformal map's planet area per world area. Swapping tau changes only the table.

What is stored is one eighth of a face, the triangle 0 <= v <= u <= 1 of the face's own
coordinates (u, v in [-1, 1] across the face): the cube's symmetry gives the rest by sign flips
and a swap. The generator builds on that triangle and mirrors it, so the symmetry is exact by
construction, not the result of a full-face solve:
  - rho is computed on the triangle's cells of one quadrant and copied to the other cells by
    index (no arithmetic);
  - the Neumann solve runs on the whole face (a cosine transform), and its result and gradient
    are then rebuilt from the triangle by index;
  - only the triangle's nodes are flowed, with the nodes on the axis, the diagonal and the edge
    put back on their lines after every step;
  - edge nodes are written exactly (a = 1), axis nodes b = 0, diagonal nodes a = b, the corner
    (1, 1).
Each node stores the planet point in gnomonic coordinates of the face frame (a, b) = (y / x,
z / x) with x along the face centre. A second table over the gnomonic triangle 0 <= b <= a <= 1
holds the (u, v) whose bicubic forward is that point: the inverse's start.

The corner patch (the conformal cube only; ledger SCL-104). The conformal map behaves like
(z - z_corner)^(4/3) at a cube corner, so the bicubic is only O(h^(4/3)) accurate in the cells
next to it (3.3e-5 rad, 0.17 blocks at 32k, at m = 256, against 1e-8 rad elsewhere). A finer table
of the same kind covers the square of the last K coarse cells at the corner, F fine cells to a
coarse cell, with its ghost nodes stored (the edge mirror and the corner extrapolation, the coarse
table's rules). The forward is the fine bicubic within K - 1 coarse cells of the corner, the
coarse one beyond K, and between them coarse + s (fine - coarse) with s = w^2 (3 - 2 w), w the
Chebyshev distance inward from K cells, in coarse cells: continuous with a continuous Jacobian.

The table file (crates/deeptime-format/data/cube-map-<name>.bin), little-endian:

  offset  size  field
       0     4  magic "DTCM"
       4     4  u32 layout version (1)
       8     4  u32 m: forward cells along the triangle's side (the face has n = 2 m + 1 nodes a side)
      12     4  u32 q: inverse cells along the gnomonic triangle's side
      16     8  f64 tau
      24     4  u32 flow steps (the RK4 steps of the whole flow, t from 0 to 1; 0 for tau 0, no flow)
      28     4  u32 K: corner patch, coarse cells from the corner it covers (0: no patch)
      32     4  u32 F: corner patch, fine cells to a coarse cell (0 when K is 0)
      36     4  u32 reserved (0)
      40    32  name, ASCII, NUL padded
      72    40  generator commit, ASCII hex, NUL padded
     112        forward nodes: for i in 0..=m, for j in 0..=i: a, b (f64)
                inverse nodes: for k in 0..=q, for l in 0..=k: u, v (f64)
                patch nodes (n = K F): for i in -1..=n+1, for j in -1..=i: a, b (f64), node (i, j)
                at u = 1 - K / m + i / (m F), v likewise; i = n + 1 and (n + 1, n) are ghosts

and <file>.sha256 beside it (sha256sum format). Readers: Rust deeptime_format::cube_map, Java
worlddata/CubeMap, JS DTCubeMap in projection.js; FORMAT.md "The cube map table".

    cube_map.py generate --m 256 --q 128 --tau 0 --name conformal-n513 --commit <hash> --out DIR
    cube_map.py generate --m 256 --q 128 --tau 1 --name moser-t1-n513 --commit <hash> --out DIR
"""

import argparse
import hashlib
import math
import os
import struct
import sys

import numpy as np
import scipy.special as sp
from scipy.fft import dctn, idctn
from scipy.ndimage import map_coordinates
from scipy.spatial import cKDTree

MAGIC = b"DTCM"
LAYOUT_VERSION = 1
HEADER_BYTES = 112

# ---------------------------------------------------------------------------------------------
# The conformal cube by Schwarz triangle functions (copied from the survey's conformal.py, the
# "cube" case: flat triangle angles pi (1/4, 1/2, 1/4), spherical pi (1/4, 1/2, 1/3), face centre
# 45 degrees from the edge midpoint; ledger SCL-080).

CUBE_FLAT = (0.25, 0.5, 0.25)
CUBE_SPHERE = (0.25, 0.5, 1.0 / 3.0)
CUBE_D01 = math.pi / 4  # face centre to edge midpoint
CUBE_D02 = math.atan(math.sqrt(2.0))  # face centre to corner


def _gauss1(a, b, c):
    return math.gamma(c) * math.gamma(c - a - b) / (math.gamma(c - a) * math.gamma(c - b))


class Schwarz:
    def __init__(self, fa=CUBE_FLAT, sb=CUBE_SPHERE, d01=CUBE_D01):
        self.fa, self.sb = fa, sb
        a0, a1, _ = fa
        b0, b1, b2 = sb
        self.c = 1 - b0
        self.a = (1 - b0 - b1 + b2) / 2
        self.b = (1 - b0 - b1 - b2) / 2
        z1 = _gauss1(a0, 1 - a1, a0 + 1) / a0
        self.A = 1 / z1
        r1 = _gauss1(self.a - self.c + 1, self.b - self.c + 1, 2 - self.c) / _gauss1(self.a, self.b, self.c)
        self.B = math.tan(d01 / 2) / r1

    def z(self, t):
        a0, a1, _ = self.fa
        return self.A * t ** a0 / a0 * sp.hyp2f1(a0, 1 - a1, a0 + 1, t)

    def dz(self, t):
        a0, a1, _ = self.fa
        return self.A * t ** (a0 - 1) * (1 - t) ** (a1 - 1)

    def zeta(self, t):
        a, b, c = self.a, self.b, self.c
        return self.B * t ** self.sb[0] * sp.hyp2f1(a - c + 1, b - c + 1, 2 - c, t) / sp.hyp2f1(a, b, c, t)

    def invert(self, Z, iters=40):
        rr = np.logspace(-8, 4, 500)
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
        a0 = self.fa[0]
        t0 = (Z * a0 / self.A) ** (1 / a0)
        t = np.where(np.abs(t0) < 1e-6, t0, t)
        for _ in range(iters):
            tn = t - (self.z(t) - Z) / self.dz(t)
            t = np.where(tn.imag < 0, tn.real + 1e-14j, tn)
        return t, np.abs(self.z(t) - Z)

    def scale(self, t):
        d = 1e-6 * np.maximum(np.abs(t), 1e-280)
        dzeta = (self.zeta(t + d) - self.zeta(t - d)) / (2 * d)
        zt = self.zeta(t)
        return np.abs(dzeta / self.dz(t)) * 2 / (1 + np.abs(zt) ** 2), zt


_SCHWARZ = None


def conformal_triangle(x, y):
    """The conformal cube at face points with x >= y >= 0 (x, y <= 1): the stereographic
    coordinate zeta from the face centre (a complex array) and the scale sigma (planet radians
    per face unit). As moser.conformal_tile, restricted to the triangle."""
    global _SCHWARZ
    if _SCHWARZ is None:
        _SCHWARZ = Schwarz()
    S = _SCHWARZ
    x = np.asarray(x, dtype=float)
    y = np.asarray(y, dtype=float)
    corner = (np.abs(x - 1) < 1e-9) & (np.abs(y - 1) < 1e-9)
    xe = np.minimum(x, 1 - 1e-11)  # off the branch cut on the edge, as the survey
    ye = np.minimum(y, xe)
    Z = (xe + 1j * ye).ravel()
    t, err = S.invert(Z)
    sig, zt = S.scale(t)
    zt = zt.reshape(x.shape)
    sig = sig.reshape(x.shape)
    # the edge midpoint (t = 1, the other singular point of the hypergeometric) likewise
    midpoint = (np.abs(x - 1) < 1e-9) & (y == 0)
    zt = np.where(corner, math.tan(CUBE_D02 / 2) * np.exp(1j * math.pi * CUBE_SPHERE[0]), zt)
    zt = np.where(midpoint, math.tan(CUBE_D01 / 2) + 0j, zt)
    zt = np.where((x == 0) & (y == 0), 0j, zt)
    if not np.all(np.isfinite(zt)):
        raise RuntimeError("conformal_triangle: non-finite points")
    err = np.where((corner | midpoint).ravel(), 0.0, err)  # placed, not inverted
    if os.environ.get("CM_DEBUG"):
        o = np.argsort(-np.nan_to_num(err, nan=np.inf))[:5]
        print("conformal_triangle worst:", [(float(x.ravel()[k]), float(y.ravel()[k]), float(err[k])) for k in o],
              file=sys.stderr)
    return zt, sig, float(np.max(err[np.isfinite(err)]))


def gnomonic_of_zeta(zt):
    """Stereographic zeta (from the face centre) to gnomonic (a, b) of the same planet point."""
    r2 = zt.real * zt.real + zt.imag * zt.imag
    den = 1.0 - r2
    return 2.0 * zt.real / den, 2.0 * zt.imag / den


# ---------------------------------------------------------------------------------------------
# Moser's flow (the survey's moser.py solve_neumann and the study's flow stopped at tau).


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


def mirror_scalar(q):
    """A whole face (2m x 2m cells) from one quadrant's cells (m x m, x >= 0, y >= 0) of a field
    even in x and y, by index."""
    m = q.shape[0]
    full = np.empty((2 * m, 2 * m))
    full[m:, m:] = q
    full[:m, m:] = q[::-1, :]
    full[m:, :m] = q[:, ::-1]
    full[:m, :m] = q[::-1, ::-1]
    return full


def quadrant_from_triangle(q):
    """The quadrant's cells with j > i rebuilt from the triangle j <= i (a field symmetric under
    the swap)."""
    m = q.shape[0]
    iu = np.triu_indices(m, 1)  # (i, j) with j > i
    out = q.copy()
    out[iu] = q.T[iu]
    return out


def flow_field(m):
    """Moser's flow on the face: the velocity (X, Y, t) -> (dX, dY) and the Schwarz Newton error of
    the density it was built from."""
    n = 2 * m  # cells across the face for the Neumann solve (the study's n)
    h = 2.0 / n
    # rho on the triangle cells of the quadrant x, y > 0; cell centre (k + 0.5) h
    cq = (np.arange(m) + 0.5) * h
    I, J = np.meshgrid(np.arange(m), np.arange(m), indexing="ij")
    tri = J <= I
    _, sig, err = conformal_triangle(cq[I[tri]], cq[J[tri]])
    q = np.zeros((m, m))
    q[I[tri], J[tri]] = sig ** 2
    q = quadrant_from_triangle(q)
    rho = mirror_scalar(q)
    rho = rho / rho.mean()  # the grid's quadrature error at the singular corners, as the survey
    u = solve_neumann(rho - 1, h)
    uq = quadrant_from_triangle(np.where(tri, u[m:, m:], 0.0))
    u = mirror_scalar(uq)
    gx, gy = np.gradient(u, h, edge_order=2)
    # rebuild the gradient from the triangle by index: odd in its own axis, swapped by the swap
    gxq = gx[m:, m:].copy()
    gyq = gy[m:, m:].copy()
    iu = np.triu_indices(m, 1)
    gxq[iu], gyq[iu] = gy[m:, m:].T[iu], gx[m:, m:].T[iu]
    gx = np.empty((n, n))
    gy = np.empty((n, n))
    gx[m:, m:], gy[m:, m:] = gxq, gyq
    gx[:m, m:], gy[:m, m:] = -gxq[::-1, :], gyq[::-1, :]
    gx[m:, :m], gy[m:, :m] = gxq[:, ::-1], -gyq[:, ::-1]
    gx[:m, :m], gy[:m, :m] = -gxq[::-1, ::-1], -gyq[::-1, ::-1]

    def interp(field, X, Y):
        return map_coordinates(field, [(X + 1) / h - 0.5, (Y + 1) / h - 0.5], order=1, mode="mirror")

    def vel(X, Y, t):
        r = (1 - t) + t * interp(rho, X, Y)
        return -interp(gx, X, Y) / r, -interp(gy, X, Y) / r

    return vel, err


def build(m, tau, steps=400):
    """The forward table on the triangle: a, b at nodes (i / m, j / m), 0 <= j <= i <= m, as two
    (m + 1) x (m + 1) arrays (entries with j > i unused). tau = 0 (the conformal cube) skips the
    flow: its nodes are the Schwarz map at the nodes themselves."""
    k_stop = round(tau * steps)
    if abs(k_stop - tau * steps) > 1e-9:
        raise ValueError(f"tau {tau} is not a whole number of the {steps} flow steps")
    vel, err = flow_field(m) if k_stop > 0 else (None, 0.0)
    Ni, Nj = np.meshgrid(np.arange(m + 1), np.arange(m + 1), indexing="ij")
    node = Nj <= Ni
    ii, jj = Ni[node], Nj[node]
    X = ii / m
    Y = jj / m
    edge, axis, diag = ii == m, jj == 0, ii == jj

    def pin(X, Y):
        X = np.clip(X, -1, 1)
        Y = np.clip(Y, -1, 1)
        X = np.where(edge, 1.0, X)
        Y = np.where(axis, 0.0, Y)
        mid = 0.5 * (X + Y)
        return np.where(diag, mid, X), np.where(diag, mid, Y)

    dt = 1.0 / steps
    for k in range(k_stop):
        t = k * dt
        k1 = vel(X, Y, t)
        k2 = vel(X + 0.5 * dt * k1[0], Y + 0.5 * dt * k1[1], t + 0.5 * dt)
        k3 = vel(X + 0.5 * dt * k2[0], Y + 0.5 * dt * k2[1], t + 0.5 * dt)
        k4 = vel(X + dt * k3[0], Y + dt * k3[1], t + dt)
        X, Y = pin(X + dt / 6 * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]),
                   Y + dt / 6 * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]))
    zt, _, err2 = conformal_triangle(X, Y)
    a, b = gnomonic_of_zeta(zt)
    a = np.where(edge, 1.0, a)
    b = np.where(axis, 0.0, b)
    mid = 0.5 * (a + b)
    a = np.where(diag, mid, a)
    b = np.where(diag, mid, b)
    a = np.where(edge & diag, 1.0, a)
    b = np.where(edge & diag, 1.0, b)
    A = np.full((m + 1, m + 1), np.nan)
    B = np.full((m + 1, m + 1), np.nan)
    A[ii, jj], B[ii, jj] = a, b
    return A, B, {"newton_err": max(err, err2), "steps": steps}


# ---------------------------------------------------------------------------------------------
# The bicubic evaluator: the Python twin of deeptime_format::cube_map (same ghost rules, same
# Catmull-Rom form, same corner patch and blend). Used for the inverse table and the checks.


def padded(A, B):
    """The node array over [-1, m + 1]^2 (index offset 1), from the triangle by the ghost rules of
    FORMAT.md "The cube map table"."""
    m = A.shape[0] - 1
    P = np.full((m + 3, m + 3, 2), np.nan)
    I, J = np.meshgrid(np.arange(m + 1), np.arange(m + 1), indexing="ij")
    low = J <= I
    # the triangle, and above the diagonal its swap
    P[1:m + 2, 1:m + 2, 0] = np.where(low, A, B.T)
    P[1:m + 2, 1:m + 2, 1] = np.where(low, B, A.T)
    # beyond u = 1 (rows j < m): the mirror across the edge's plane, (a, b) -> (1 / a, b / a)
    a, b = P[m, 1:m + 1, 0], P[m, 1:m + 1, 1]  # i = m - 1
    P[m + 2, 1:m + 1, 0] = 1.0 / a
    P[m + 2, 1:m + 1, 1] = b / a
    # beyond v = 1 (columns i < m): the swap of the above
    P[1:m + 1, m + 2, 0] = P[m + 2, 1:m + 1, 1]
    P[1:m + 1, m + 2, 1] = P[m + 2, 1:m + 1, 0]
    # past the corner: linear along the edge, then the swap; the diagonal ghost linear along it
    pmm, pm1 = P[m + 1, m + 1].copy(), P[m + 1, m].copy()  # (m, m), (m, m - 1)
    P[m + 1, m + 2] = 2.0 * pmm - pm1  # (m, m + 1): a stays exactly 1
    P[m + 2, m + 1] = P[m + 1, m + 2][::-1]  # (m + 1, m)
    P[m + 2, m + 2] = 2.0 * pmm - P[m, m]  # (m + 1, m + 1) from (m - 1, m - 1)
    # beyond u = 0 (rows 0..m + 1): (-a, b) of column 1
    P[0, 1:m + 3, 0] = -P[2, 1:m + 3, 0]
    P[0, 1:m + 3, 1] = P[2, 1:m + 3, 1]
    # beyond v = 0 (columns -1..m + 1): (a, -b) of row 1
    P[:, 0, 0] = P[:, 2, 0]
    P[:, 0, 1] = -P[:, 2, 1]
    assert np.all(np.isfinite(P)), "padded table has holes"
    return P


def patch_nodes(m, K, F):
    """The corner patch: nodes (i, j), i, j in [-1, n + 1] (n = K F, index offset 1) at
    u = 1 - K / m + i / (m F), from the exact conformal map inside the face and the coarse table's
    ghost rules beyond it (the edge mirror, the swap, the corner extrapolation)."""
    n = K * F
    u0 = 1.0 - K / m
    hf = 1.0 / (m * F)
    idx = np.arange(-1, n + 1)
    I, J = np.meshgrid(idx, idx, indexing="ij")
    low = J <= I
    ii, jj = I[low], J[low]
    zt, _, err = conformal_triangle(u0 + ii * hf, u0 + jj * hf)
    a, b = gnomonic_of_zeta(zt)
    edge, diag = ii == n, ii == jj
    a = np.where(edge, 1.0, a)
    mid = 0.5 * (a + b)
    a = np.where(diag, mid, a)
    b = np.where(diag, mid, b)
    a = np.where(edge & diag, 1.0, a)
    b = np.where(edge & diag, 1.0, b)
    Q = np.full((n + 3, n + 3, 2), np.nan)
    Q[ii + 1, jj + 1, 0], Q[ii + 1, jj + 1, 1] = a, b
    Q[jj + 1, ii + 1, 0], Q[jj + 1, ii + 1, 1] = b, a
    # beyond u = 1 (j = -1 .. n - 1): the mirror of node (n - 1, j); beyond v = 1 its swap
    ar, br = Q[n, 0:n + 1, 0].copy(), Q[n, 0:n + 1, 1].copy()
    Q[n + 2, 0:n + 1, 0] = 1.0 / ar
    Q[n + 2, 0:n + 1, 1] = br / ar
    Q[0:n + 1, n + 2, 0] = Q[n + 2, 0:n + 1, 1]
    Q[0:n + 1, n + 2, 1] = Q[n + 2, 0:n + 1, 0]
    pnn, pn1 = Q[n + 1, n + 1].copy(), Q[n + 1, n].copy()  # (n, n), (n, n - 1)
    Q[n + 1, n + 2] = 2.0 * pnn - pn1  # (n, n + 1)
    Q[n + 2, n + 1] = Q[n + 1, n + 2][::-1]  # (n + 1, n)
    Q[n + 2, n + 2] = 2.0 * pnn - Q[n, n]  # (n + 1, n + 1) from (n - 1, n - 1)
    assert np.all(np.isfinite(Q)), "patch has holes"
    return Q, err


def weights(t):
    """The Catmull-Rom weights of the four nodes at t in [0, 1] (the readers' form and order:
    crates/deeptime-format/src/cube_map.rs weights)."""
    t2 = t * t
    t3 = t2 * t
    return (0.5 * ((2.0 * t2 - t) - t3),
            0.5 * ((2.0 - 5.0 * t2) + 3.0 * t3),
            0.5 * ((t + 4.0 * t2) - 3.0 * t3),
            0.5 * (t3 - t2))


def dweights(t):
    """The weights' derivatives along t."""
    t2 = t * t
    return (0.5 * ((4.0 * t - 1.0) - 3.0 * t2),
            0.5 * (9.0 * t2 - 10.0 * t),
            0.5 * ((1.0 + 8.0 * t) - 9.0 * t2),
            0.5 * (3.0 * t2 - 2.0 * t))


def mix(w, p):
    """Sum of w_k p_k, summed from k = 0 (the readers' order)."""
    return ((w[0] * p[0] + w[1] * p[1]) + w[2] * p[2]) + w[3] * p[3]


def grid_eval(P, x, y, n):
    """The Catmull-Rom bicubic of the padded node array P (index offset 1, n cells a side) at grid
    coordinates (x, y) in node units, the cell from x, y clamped to [0, n - 1] and the cubic
    evaluated at the point itself; with its derivatives per node unit. The cubic along x on each
    of the four rows, then along y, as the readers do it."""
    x = np.asarray(x, dtype=float)
    y = np.asarray(y, dtype=float)
    i0 = np.clip(np.floor(x), 0, n - 1).astype(int)
    j0 = np.clip(np.floor(y), 0, n - 1).astype(int)
    tu, tv = x - i0, y - j0
    wu, dwu = [w[..., None] for w in weights(tu)], [w[..., None] for w in dweights(tu)]
    wv, dwv = [w[..., None] for w in weights(tv)], [w[..., None] for w in dweights(tv)]
    rows, drows = [], []
    for dj in range(-1, 3):
        p = [P[i0 + di + 1, j0 + dj + 1] for di in range(-1, 3)]
        rows.append(mix(wu, p))
        drows.append(mix(dwu, p))
    return mix(wv, rows), mix(wv, drows), mix(dwv, rows)


class Map:
    """A table ready to evaluate (the readers' CubeMap): the padded coarse nodes, the corner patch
    and the inverse table."""

    def __init__(self, t):
        self.m, self.q, self.K, self.F = t["m"], t["q"], t["K"], t["F"]
        self.P = padded(t["A"], t["B"])
        self.Q = t.get("Q")
        self.U, self.V = t.get("U"), t.get("V")
        self.t = t


def tri_eval_raw(T, u, v):
    """(a, b) of the triangle's bicubic at (u, v) and its derivatives along u and v (per unit of
    u), the corner patch blended in; without the edge's a = 1 (a Newton iterate may step a hair
    outside the triangle)."""
    m = T.m
    u = np.asarray(u, dtype=float)
    v = np.asarray(v, dtype=float)
    val, du, dv = grid_eval(T.P, u * m, v * m, m)
    du, dv = du * m, dv * m
    if not T.K:
        return val, du, dv
    n = T.K * T.F
    mf = float(m * T.F)
    u0 = 1.0 - T.K / m
    fv, fdu, fdv = grid_eval(T.Q, (u - u0) * mf, (v - u0) * mf, n)
    fdu, fdv = fdu * mf, fdv * mf
    w = ((v - u0) * m)[..., None]  # coarse cells inward from the patch's edge (v is the smaller)
    s = w * w * (3.0 - 2.0 * w)
    ds = 6.0 * w * (1.0 - w) * m
    rv = val + s * (fv - val)
    rdu = du + s * (fdu - du)
    rdv = (dv + s * (fdv - dv)) + (fv - val) * ds
    inner, ring = w >= 1.0, (w > 0.0) & (w < 1.0)
    val = np.where(inner, fv, np.where(ring, rv, val))
    du = np.where(inner, fdu, np.where(ring, rdu, du))
    dv = np.where(inner, fdv, np.where(ring, rdv, dv))
    return val, du, dv


def tri_eval(T, u, v, derivs=False):
    """(a, b) of the triangle's bicubic at (u, v) in [0, 1]^2 (arrays), and with derivs the
    derivatives along u and v (per unit of u). On the edge u = 1, a is exactly 1."""
    val, du, dv = tri_eval_raw(T, u, v)
    val[..., 0] = np.where(np.asarray(u) == 1.0, 1.0, val[..., 0])
    return (val, du, dv) if derivs else val


def newton_step(T, a, b, u, v):
    val, du, dv = tri_eval_raw(T, u, v)
    ra, rb = a - val[..., 0], b - val[..., 1]
    det = du[..., 0] * dv[..., 1] - dv[..., 0] * du[..., 1]
    return u + (dv[..., 1] * ra - dv[..., 0] * rb) / det, v + (du[..., 0] * rb - du[..., 1] * ra) / det


def tri_inverse(T, a, b, start=None, iters=60):
    """(u, v) on the triangle whose bicubic is (a, b), by Newton to convergence."""
    a = np.asarray(a, dtype=float)
    b = np.asarray(b, dtype=float)
    if start is None:
        u = np.arctan(a) * (4 / math.pi)
        v = np.arctan(b) * (4 / math.pi)
    else:
        u, v = start
    for _ in range(iters):
        u, v = newton_step(T, a, b, u, v)
    val = tri_eval_raw(T, u, v)[0]
    res = np.maximum(np.abs(val[..., 0] - a), np.abs(val[..., 1] - b))
    return u, v, res


def inverse_start(T, a, b):
    """The inverse table's bilinear start at (a, b) on the gnomonic triangle (the readers' start)."""
    q = T.q
    x, y = a * q, b * q
    k = np.clip(np.floor(x), 0, q - 1).astype(int)
    l = np.clip(np.floor(y), 0, q - 1).astype(int)
    tx, ty = x - k, y - l

    def node(i, j):
        lo = j <= i
        return (np.where(lo, T.U[np.maximum(i, j), np.minimum(i, j)], T.V[np.maximum(i, j), np.minimum(i, j)]),
                np.where(lo, T.V[np.maximum(i, j), np.minimum(i, j)], T.U[np.maximum(i, j), np.minimum(i, j)]))

    u00, v00 = node(k, l)
    u10, v10 = node(k + 1, l)
    u01, v01 = node(k, l + 1)
    u11, v11 = node(k + 1, l + 1)
    u0 = u00 + tx * (u10 - u00)
    u1 = u01 + tx * (u11 - u01)
    w0 = v00 + tx * (v10 - v00)
    w1 = v01 + tx * (v11 - v01)
    return u0 + ty * (u1 - u0), w0 + ty * (w1 - w0)


def inverse_table(T, q):
    """(u, v) at the gnomonic triangle's nodes (k / q, l / q), 0 <= l <= k <= q."""
    K, L = np.meshgrid(np.arange(q + 1), np.arange(q + 1), indexing="ij")
    node = L <= K
    kk, ll = K[node], L[node]
    a, b = kk / q, ll / q
    u, v, res = tri_inverse(T, a, b)
    u = np.where(kk == q, 1.0, u)
    v = np.where(ll == 0, 0.0, v)
    mid = 0.5 * (u + v)
    u = np.where(kk == ll, mid, u)
    v = np.where(kk == ll, mid, v)
    u = np.where((kk == q) & (ll == q), 1.0, u)
    v = np.where((kk == q) & (ll == q), 1.0, v)
    U = np.full((q + 1, q + 1), np.nan)
    V = np.full((q + 1, q + 1), np.nan)
    U[kk, ll], V[kk, ll] = u, v
    return U, V, float(np.max(res))


# ---------------------------------------------------------------------------------------------
# The file.


def triangle_values(A, B, lo=0):
    """The triangle's values in file order: for i, for j <= i (from index lo), a then b."""
    n = A.shape[0]
    out = []
    for i in range(n):
        for j in range(i + 1):
            out.append((A[i, j], B[i, j]))
    return np.array(out, dtype="<f8")


def write_table(path, name, commit, m, q, tau, steps, A, B, U, V, K=0, F=0, Q=None):
    if len(name.encode()) > 32 or len(commit.encode()) > 40:
        raise ValueError("name or commit too long")
    head = MAGIC + struct.pack("<IIIdIIII", LAYOUT_VERSION, m, q, tau, steps, K, F, 0)
    head += name.encode().ljust(32, b"\0") + commit.encode().ljust(40, b"\0")
    assert len(head) == HEADER_BYTES
    body = triangle_values(A, B).tobytes() + triangle_values(U, V).tobytes()
    if K:
        body += triangle_values(Q[..., 0], Q[..., 1]).tobytes()  # i, j from -1 (index offset 1)
    data = head + body
    with open(path, "wb") as f:
        f.write(data)
    digest = hashlib.sha256(data).hexdigest()
    with open(path + ".sha256", "w") as f:
        f.write(f"{digest}  {os.path.basename(path)}\n")
    return digest, len(data)


def _triangle(vals, n):
    A = np.full((n, n), np.nan)
    B = np.full((n, n), np.nan)
    k = 0
    for i in range(n):
        A[i, : i + 1], B[i, : i + 1] = vals[k:k + i + 1, 0], vals[k:k + i + 1, 1]
        k += i + 1
    return A, B


def read_table(path):
    with open(path, "rb") as f:
        data = f.read()
    if data[:4] != MAGIC:
        raise ValueError(f"{path}: not a cube map table")
    version, m, q, tau, steps, K, F, _ = struct.unpack_from("<IIIdIIII", data, 4)
    if version != LAYOUT_VERSION:
        raise ValueError(f"{path}: layout version {version}")
    name = data[40:72].rstrip(b"\0").decode()
    commit = data[72:112].rstrip(b"\0").decode()
    nf = (m + 1) * (m + 2) // 2
    ni = (q + 1) * (q + 2) // 2
    side = K * F + 3 if K else 0
    npch = side * (side + 1) // 2
    vals = np.frombuffer(data, dtype="<f8", offset=HEADER_BYTES)
    if vals.size != 2 * (nf + ni + npch):
        raise ValueError(f"{path}: {vals.size} values, want {2 * (nf + ni + npch)}")
    vals = vals.reshape(-1, 2)
    A, B = _triangle(vals[:nf], m + 1)
    U, V = _triangle(vals[nf:nf + ni], q + 1)
    t = {"name": name, "commit": commit, "m": m, "q": q, "tau": tau, "steps": steps, "K": K, "F": F,
         "A": A, "B": B, "U": U, "V": V, "Q": None,
         "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
    if K:
        QA, QB = _triangle(vals[nf + ni:], side)
        low = np.tril(np.ones((side, side), dtype=bool))
        Q = np.empty((side, side, 2))
        Q[..., 0] = np.where(low, QA, QB.T)
        Q[..., 1] = np.where(low, QB, QA.T)
        t["Q"] = Q
    return t


def generate(m, q, tau, name, commit, out, steps=400, K=None, F=16):
    if K is None:
        K = 4 if tau == 0 else 0  # the corner patch is for the conformal cube's singular corner
    if K and tau != 0:
        raise ValueError("the corner patch holds the exact conformal map: tau must be 0")
    A, B, info = build(m, tau, steps)
    t = {"m": m, "q": q, "K": K, "F": F if K else 0, "A": A, "B": B, "Q": None}
    perr = 0.0
    if K:
        t["Q"], perr = patch_nodes(m, K, F)
    T = Map(t)
    U, V, res = inverse_table(T, q)
    os.makedirs(out, exist_ok=True)
    path = os.path.join(out, f"cube-map-{name}.bin")
    # tau 0 runs no flow: the header says 0 steps
    digest, size = write_table(path, name, commit, m, q, tau, steps if tau else 0, A, B, U, V, K, t["F"], t["Q"])
    print(f"{path}: m {m} (n {2 * m + 1}), q {q}, tau {tau}, corner patch K {K} F {t['F']}, {size} bytes, "
          f"sha256 {digest}, schwarz newton {max(info['newton_err'], perr):.2e}, "
          f"inverse-table newton residual {res:.2e}", flush=True)
    return path


def main(argv):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("generate")
    g.add_argument("--m", type=int, default=256)
    g.add_argument("--q", type=int, default=128)
    g.add_argument("--tau", type=float, default=1.0)
    g.add_argument("--steps", type=int, default=400)
    g.add_argument("--patch-cells", type=int, default=None, help="K (default 4 for tau 0, else 0)")
    g.add_argument("--patch-fine", type=int, default=16, help="F")
    g.add_argument("--name", required=True)
    g.add_argument("--commit", required=True)
    g.add_argument("--out", required=True)
    args = ap.parse_args(argv[1:])
    if args.cmd == "generate":
        generate(args.m, args.q, args.tau, args.name, args.commit, args.out, args.steps,
                 args.patch_cells, args.patch_fine)


if __name__ == "__main__":
    main(sys.argv)
