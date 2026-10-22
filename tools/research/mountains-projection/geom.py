# SPDX-License-Identifier: LGPL-3.0-or-later
"""Geometry shared by the mountains-and-the-projection tools: sphere helpers, 2 x 2 Jacobians and
their shape ratio, and the projection survey's smooth equal-area cube in the cross net (its Moser
table from cube_table.py), with the forward map, the seams unfolded for local views, and its shape
ratio anywhere on the sphere. Research code, not shipped."""

import math
import os
import sys

import numpy as np
from scipy.interpolate import RectBivariateSpline
from scipy.spatial import cKDTree

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "projection-survey"))
import survey  # noqa: E402

D2R = math.pi / 180.0


def ll2xyz(lat_deg, lon_deg):
    la = np.asarray(lat_deg, np.float64) * D2R
    lo = np.asarray(lon_deg, np.float64) * D2R
    cl = np.cos(la)
    return np.stack([cl * np.cos(lo), cl * np.sin(lo), np.sin(la)], axis=-1)


def xyz2ll(p):
    p = np.asarray(p, np.float64)
    lat = np.degrees(np.arctan2(p[..., 2], np.hypot(p[..., 0], p[..., 1])))
    lon = np.degrees(np.arctan2(p[..., 1], p[..., 0])) % 360.0
    return lat, lon


def east_north(p):
    """Unit east and north vectors at unit vectors p (undefined at the poles)."""
    lat, lon = xyz2ll(p)
    la, lo = lat * D2R, lon * D2R
    e = np.stack([-np.sin(lo), np.cos(lo), np.zeros_like(lo)], axis=-1)
    n = np.stack([-np.sin(la) * np.cos(lo), -np.sin(la) * np.sin(lo), np.cos(la)], axis=-1)
    return e, n


def rot_z(deg):
    a = deg * D2R
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])


def rot_zyz(a, b, g):
    """R = Rz(a) Ry(b) Rz(g), degrees."""
    def ry(deg):
        t = deg * D2R
        c, s = math.cos(t), math.sin(t)
        return np.array([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]])
    return rot_z(a) @ ry(b) @ rot_z(g)


def sv2(a, b, c, d):
    """Singular values of [[a, b], [c, d]] (arrays)."""
    e = a * a + b * b + c * c + d * d
    det = a * d - b * c
    disc = np.sqrt(np.maximum(e * e - 4 * det * det, 0.0))
    s1 = np.sqrt((e + disc) / 2)
    s2 = np.sqrt(np.maximum((e - disc) / 2, 0.0))
    return s1, s2, det


def shape_ratio(a, b, c, d):
    s1, s2, _ = sv2(a, b, c, d)
    return s1 / np.maximum(s2, 1e-300)


def jacobian_from_units(P, h_blocks, r_m, wrap_x=True):
    """A = d(east, north metres) / d(x, z blocks) of a grid of unit vectors P[z, x, 3] sampled every
    h_blocks. Returns a, b, c, d with A = [[a, b], [c, d]] (columns x and z)."""
    if wrap_x:
        dPdx = (np.roll(P, -1, axis=1) - np.roll(P, 1, axis=1)) / (2 * h_blocks)
    else:
        dPdx = np.gradient(P, h_blocks, axis=1)
    dPdz = np.gradient(P, h_blocks, axis=0)
    e, n = east_north(P)
    a = (dPdx * e).sum(-1) * r_m
    b = (dPdz * e).sum(-1) * r_m
    c = (dPdx * n).sum(-1) * r_m
    d = (dPdz * n).sum(-1) * r_m
    return a, b, c, d


# ----------------------------------------------------------------------------------------------
# The smooth equal-area cube (survey §4, §6.1) in the cross net. World units: face units (one face
# is 1 x 1); band face k at X in [k, k+1], Y in [0, 1]; N at [0, 1] x [1, 2]; S at [0, 1] x [-1, 0].
# Y is north on the band, so block z = -Y * S_blocks.

FACES = [(k, 0.0, survey.BAND_FRAMES[k]) for k in range(4)]
FACES += [(0, 1.0, survey.N_FRAME), (0, -1.0, survey.S_FRAME)]


class Cube:
    def __init__(self, table_npz, lambda0_deg=0.0, Q=None):
        """Q, when given, is the rotation planet -> canonical cube frame (any orientation);
        otherwise the polar aspect turned by lambda0 about the axis."""
        t = np.load(table_npz)
        s, G = t["s"], t["G"]
        self.sps = [RectBivariateSpline(s, s, G[..., i], kx=3, ky=3) for i in range(3)]
        self.lambda0 = lambda0_deg
        self.Q = np.asarray(Q, float) if Q is not None else rot_z(-lambda0_deg)
        self._ab_tree = None

    # --- the face map
    def face_vec(self, u, v):
        g = np.stack([sp.ev(u, v) for sp in self.sps], axis=-1)
        return g / np.linalg.norm(g, axis=-1, keepdims=True)

    def face_sv(self, n=513):
        """The singular values s1 >= s2 of d(planet)/d(u, v) (planet radians per face unit, the face
        being [-1, 1]^2) at n x n nodes of one face (the same on all six), with the face-local unit
        vectors there."""
        s = np.linspace(-1, 1, n)
        U, V = np.meshgrid(s, s, indexing="ij")
        u, v = U.ravel(), V.ravel()
        g = np.stack([sp.ev(u, v) for sp in self.sps], axis=-1)
        gu = np.stack([sp.ev(u, v, dx=1) for sp in self.sps], axis=-1)
        gv = np.stack([sp.ev(u, v, dy=1) for sp in self.sps], axis=-1)
        r = np.linalg.norm(g, axis=-1, keepdims=True)
        p = g / r
        pu = (gu - p * (p * gu).sum(-1, keepdims=True)) / r
        pv = (gv - p * (p * gv).sum(-1, keepdims=True)) / r
        s1, s2 = survey.sv(pu, pv)
        return p, s1, s2

    def tree(self):
        if self._ab_tree is None:
            p, s1, s2 = self.face_sv()
            ok = np.isfinite(s1) & np.isfinite(s2) & (s2 > 0)
            self._ab_tree = (cKDTree(p[ok]), s1[ok], s2[ok])
        return self._ab_tree

    def sv_at(self, p, Q=None):
        """s1, s2 (planet radians per face unit) at planet unit vectors p; Q, when given, is the
        rotation planet -> canonical cube frame (replacing the turn lambda0 about the axis)."""
        q = (p @ Q.T) if Q is not None else self.canonical(p)
        tree, s1, s2 = self.tree()
        _, loc = self.face_of(q)
        _, i = tree.query(loc, workers=-1)
        return s1[i], s2[i]

    @staticmethod
    def face_of(q):
        """Index into FACES of the face holding each canonical unit vector q, and q in that face's
        frame (e1, e2, c)."""
        cs = np.stack([f[2][0] for f in FACES])  # face centres
        k = np.argmax(q @ cs.T, axis=-1)
        e1 = np.stack([f[2][1] for f in FACES])[k]
        e2 = np.stack([f[2][2] for f in FACES])[k]
        c = cs[k]
        loc = np.stack([(q * e1).sum(-1), (q * e2).sum(-1), (q * c).sum(-1)], axis=-1)
        return k, loc

    def canonical(self, p):
        """Planet unit vectors into the cube's canonical frame."""
        return p @ self.Q.T

    # --- the world in face units
    def _tiles(self, X, Y):
        out = np.full(np.shape(X) + (3,), np.nan)
        for (x0, y0, fr) in FACES:
            m = (X >= x0) & (X <= x0 + 1) & (Y >= y0) & (Y <= y0 + 1) & np.isnan(out[..., 0])
            if m.any():
                u = 2 * (X[m] - x0) - 1
                v = 2 * (Y[m] - y0) - 1
                abc = self.face_vec(u, v)
                c, e1, e2 = fr
                out[m] = abc[:, 0:1] * e1 + abc[:, 1:2] * e2 + abc[:, 2:3] * c
        return out

    def unit(self, X, Y):
        """World (X, Y) in face units to planet unit vectors; points a little past a seam are
        carried through it (the seams are isometries, so a local view is continuous)."""
        X = np.asarray(X, np.float64)
        Y = np.asarray(Y, np.float64)
        band = (Y >= 0) & (Y <= 1)
        X = np.where(band, np.mod(X, 4.0), X)
        out = self._tiles(X, Y)
        rem = np.isnan(out[..., 0])
        if rem.any():
            # A point outside the net goes through the NEAREST seam it lies past (the band wraps,
            # so X - 4, X and X + 4 are all tried). Taking the first seam whose strip holds the
            # point sent points beside a cube corner through the wrong edge (a 37 degree jump).
            Xr, Yr = X[rem], Y[rem]
            best = np.full(Xr.shape, np.inf)
            pick = np.full(Xr.shape, -1)
            shift = np.zeros(Xr.shape)
            for si, (p0, p1, nn, g) in enumerate(survey.CUBE_SEAMS):
                p0, p1, nn = np.array(p0, float), np.array(p1, float), np.array(nn, float)
                t = p1 - p0
                for sh in (-4.0, 0.0, 4.0):
                    dX, dY = Xr + sh - p0[0], Yr - p0[1]
                    d = dX * nn[0] + dY * nn[1]
                    tt = (dX * t[0] + dY * t[1]) / (t @ t)
                    m = (d > 0) & (d < 0.5) & (tt >= 0) & (tt <= 1) & (d < best)
                    best[m], pick[m], shift[m] = d[m], si, sh
            sub = out[rem]
            for si, (p0, p1, nn, g) in enumerate(survey.CUBE_SEAMS):
                m = pick == si
                if not m.any():
                    continue
                X2, Y2 = g(Xr[m] + shift[m], Yr[m])
                X2 = np.asarray(X2, float)
                Y2 = np.asarray(Y2, float)
                b2 = (Y2 >= 0) & (Y2 <= 1)
                X2 = np.where(b2, np.mod(X2, 4.0), X2)
                sub[m] = self._tiles(X2, Y2)
            out[rem] = sub
        return out @ self.Q

    def forward(self, p):
        """Planet unit vector(s) to world (X, Y) in face units (Newton on the face table)."""
        p = np.atleast_2d(p)
        q = self.canonical(p)
        k, loc = self.face_of(q)
        gx, gy = loc[:, 0] / loc[:, 2], loc[:, 1] / loc[:, 2]
        u = np.arctan(gx) * 4 / math.pi
        v = np.arctan(gy) * 4 / math.pi

        def f(u, v):
            w = self.face_vec(u, v)
            return w[:, 0] / w[:, 2], w[:, 1] / w[:, 2]
        h = 1e-6
        for _ in range(30):
            fx, fy = f(u, v)
            rx, ry = fx - gx, fy - gy
            ax, ay = f(u + h, v)
            bx, by = f(u, v + h)
            j11, j21 = (ax - fx) / h, (ay - fy) / h
            j12, j22 = (bx - fx) / h, (by - fy) / h
            det = j11 * j22 - j12 * j21
            du = (j22 * rx - j12 * ry) / det
            dv = (-j21 * rx + j11 * ry) / det
            u = np.clip(u - du, -1, 1)
            v = np.clip(v - dv, -1, 1)
            if np.max(np.abs(du) + np.abs(dv)) < 1e-12:
                break
        x0 = np.array([FACES[i][0] for i in k], float)
        y0 = np.array([FACES[i][1] for i in k], float)
        return x0 + (u + 1) / 2, y0 + (v + 1) / 2, k
