#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Pictures and data for the world-shape decision page (tools/review/world-shape/, 2026-10-03).

Runs on the test VM (numpy, scipy, mpmath, pillow), through run.sh. Every candidate world of
docs/research/notes/seamless-projection-survey.md is rebuilt here on top of that survey's own
projection code (tools/research/projection-survey/), and a real Deep Time bake is resampled onto it:

  * <id>-map.webp     the planet drawn in the world (hypsometric colours, world-space hill shade);
  * <id>-area.webp    blocks per planet area relative to the mean (log scale, diverging);
  * <id>-shape.webp   shape ratio a/b (how many times too long one direction is drawn);
  * <id>-grat.webp    a 15 degree graticule (kinks show creases);
  * <id>-crease.webp  hill shade of a smooth synthetic planet (creases show as lines);
  * data.js           seams (pairs, transforms, arrows), corners, Tissot ellipses, walks (equator,
                      over a pole, either side of a corner), globe overlays, fold meshes, the corner
                      placement sweeps, self-checks.

usage: compute.py <heightmap.npy> <out dir> [moser cache dir]
The heightmap is `deeptime heightmap --world-data <bake> --box=-8192,-2624,8192,2624 --step 4
--layers climate/ice_cover,climate/p_mm` of a 16k Lambert bake (format 0.4).
"""

import base64
import json
import math
import os
import sys
import time

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "research", "projection-survey"))
import survey  # noqa: E402

TAU = 2 * math.pi
C16 = 16384.0
R16 = C16 / TAU  # planet radius in blocks at the 16k scale (equal-area mean)
T0 = time.time()


def log(*a):
    print(f"[{time.time() - T0:7.1f}s]", *a, file=sys.stderr, flush=True)


# ------------------------------------------------------------------------------------------
# the planet


class Planet:
    def __init__(self, path):
        a = np.load(path, mmap_mode="r")
        self.H = np.array(a[..., 0], dtype=np.float32)
        self.ice = np.array(a[..., 3], dtype=np.float32)
        self.p = np.array(a[..., 4], dtype=np.float32)
        self.rows, self.cols = self.H.shape
        self.x0, self.z0, self.step = -8192.0, -2624.0, 4.0
        mx = float(np.nanmax(self.ice))
        self.ice_full = mx if mx > 1.5 else 1.0
        log("planet", self.H.shape, "H", float(self.H.min()), float(self.H.max()),
            "ice max", mx, "share>half", float((self.ice > 0.5 * self.ice_full).mean()),
            "p_mm p50", float(np.median(self.p)))

    def _rc(self, xyz):
        lat = np.arcsin(np.clip(xyz[..., 2], -1, 1))
        lon = np.arctan2(xyz[..., 1], xyz[..., 0])
        x = R16 * lon
        z = -R16 * np.sin(lat)
        return (z - self.z0) / self.step, (x - self.x0) / self.step

    def _bil(self, A, r, c):
        r = np.clip(r, 0, self.rows - 1.001)
        r0 = np.floor(r).astype(int)
        fr = r - r0
        c0 = np.floor(c).astype(int)
        fc = c - c0
        c0m = np.mod(c0, self.cols)
        c1m = np.mod(c0 + 1, self.cols)
        return ((A[r0, c0m] * (1 - fc) + A[r0, c1m] * fc) * (1 - fr)
                + (A[r0 + 1, c0m] * (1 - fc) + A[r0 + 1, c1m] * fc) * fr)

    def sample(self, xyz):
        ok = np.isfinite(xyz[..., 0])
        out = np.full(xyz.shape[:-1] + (3,), np.nan, np.float32)
        q = xyz[ok]
        r, c = self._rc(q)
        out[ok, 0] = self._bil(self.H, r, c)
        out[ok, 1] = self._bil(self.ice, r, c) / self.ice_full
        out[ok, 2] = self._bil(self.p, r, c)
        return out

    def land(self, xyz):
        r, c = self._rc(xyz)
        return self._bil(self.H, r, c) > 63.0


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


def hillshade(H, px_blocks, exag=6.0, cosfix=None):
    """Lambertian shade of a height grid (rows = north..south), light from the north-west."""
    Hf = np.where(np.isfinite(H), H, np.nan)
    def nb(a, dr, dc):
        b = np.roll(a, (dr, dc), axis=(0, 1))
        return np.where(np.isfinite(b), b, a)
    Hc = np.where(np.isfinite(Hf), Hf, 0)
    gx = (nb(Hc, 0, -1) - nb(Hc, 0, 1)) / (2 * px_blocks)
    gy = (nb(Hc, 1, 0) - nb(Hc, -1, 0)) / (2 * px_blocks)  # north positive
    if cosfix is not None:
        gx = gx / np.maximum(cosfix, 0.05)
    gx *= exag
    gy *= exag
    L = np.array([-1.0, 1.0, 1.3])
    L /= np.linalg.norm(L)
    n = np.stack([-gx, -gy, np.ones_like(gx)], -1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    s = (n @ L) / L[2]
    return np.clip(s, 0.35, 1.45)


def colorize(S, shade):
    H, ice, p = S[..., 0], S[..., 1], S[..., 2]
    sea = 63.0
    depth = np.clip((sea - H) / 260.0, 0, 1) ** 0.6
    ocean = (np.array([0.20, 0.42, 0.66]) * (1 - depth[..., None])
             + np.array([0.05, 0.12, 0.33]) * depth[..., None])
    elev = np.clip((H - sea) / 420.0, 0, 1)
    wet = smoothstep(150, 1300, p)[..., None]
    low = np.array([0.80, 0.70, 0.47]) * (1 - wet) + np.array([0.27, 0.50, 0.24]) * wet
    rock = np.array([0.52, 0.47, 0.40])
    land = low * (1 - smoothstep(0.25, 0.75, elev)[..., None]) + rock * smoothstep(0.25, 0.75, elev)[..., None]
    snow = smoothstep(0.78, 0.95, elev)[..., None]
    land = land * (1 - snow) + np.array([0.93, 0.94, 0.96]) * snow
    icy = smoothstep(0.35, 0.6, ice)[..., None]
    land = land * (1 - icy) + np.array([0.90, 0.94, 0.98]) * icy
    oc_ice = icy * 0.85
    ocean = ocean * (1 - oc_ice) + np.array([0.82, 0.88, 0.94]) * oc_ice
    sh = shade[..., None]
    rgb = np.where((H > sea)[..., None], land * sh, ocean * (0.82 + 0.18 * sh))
    return np.clip(rgb, 0, 1)


# synthetic smooth planet for the crease test: Gaussian bumps of several widths
_rng = np.random.default_rng(7)
_SYN_C = survey.normalize(_rng.normal(size=(70, 3)))
_SYN_K = _rng.uniform(6, 40, size=70)
_SYN_A = _rng.normal(size=70) * 400 / np.sqrt(_SYN_K)


def synthetic(xyz):
    out = np.zeros(xyz.shape[:-1])
    for c, k, a in zip(_SYN_C, _SYN_K, _SYN_A):
        out = out + a * np.exp(k * (xyz @ c - 1))
    return out


# ------------------------------------------------------------------------------------------
# layouts: tiles in a world plane (Y up = north where it applies) + affine seams


class Seam:
    """One side of a seam: world edge a0 -> a1 (counter-clockwise round its tile, so the inside is
    on the left), outward normal n, and the motion g(x) = R x + t taking points beyond it inside."""

    def __init__(self, a0, a1, R, t, pair, kind, partner):
        self.a0, self.a1 = np.array(a0, float), np.array(a1, float)
        d = self.a1 - self.a0
        self.n = np.array([d[1], -d[0]]) / np.linalg.norm(d)
        self.R, self.t = np.array(R, float), np.array(t, float)
        self.pair, self.kind, self.partner = pair, kind, partner

    def g(self, X, Y):
        return self.R[0, 0] * X + self.R[0, 1] * Y + self.t[0], self.R[1, 0] * X + self.R[1, 1] * Y + self.t[1]


def kind_of(R):
    a = round(math.degrees(math.atan2(R[1, 0], R[0, 0]))) % 360
    return {0: "slide", 90: "quarter", 270: "quarter", 180: "half"}[a], a


def rot(k):
    k %= 4
    return np.array([[1, 0], [0, 1]], float) if k == 0 else (
        np.array([[0, -1], [1, 0]], float) if k == 1 else (
            np.array([[-1, 0], [0, -1]], float) if k == 2 else np.array([[0, 1], [-1, 0]], float)))


class Layout:
    def __init__(self, lid, tiles, seams, unit_blocks, cls, walls=False):
        self.id, self.tiles, self.seams = lid, tiles, seams
        self.unit_blocks, self.cls, self.walls = unit_blocks, cls, walls
        xs = [t[0] for t in tiles] + [t[0] + t[2] for t in tiles]
        ys = [t[1] for t in tiles] + [t[1] + t[3] for t in tiles]
        self.bbox = (min(xs), min(ys), max(xs), max(ys))
        self.area = sum(t[2] * t[3] for t in tiles)
        self.corners = []      # (X, Y, vertex id, cone angle deg)
        self.internal = []     # internal tile edges (p0, p1)
        self.hinges = []       # (parent tile, child tile, p0, p1)
        self.lines = []        # crease scan lines for survey.creases (p0, p1, n)
        self.fold = None       # 'cube' | 'pillow' | 'cylinder' | None
        self.mesh_cells = 32
        self.extra = {}

    def inside(self, X, Y, tol=1e-12):
        m = np.zeros(np.shape(X), bool)
        for (x0, y0, w, h, _) in self.tiles:
            m |= (X >= x0 - tol) & (X <= x0 + w + tol) & (Y >= y0 - tol) & (Y <= y0 + h + tol)
        return m

    def F(self, X, Y):
        X = np.asarray(X, float)
        Y = np.asarray(Y, float)
        out = np.full(X.shape + (3,), np.nan)
        done = np.zeros(X.shape, bool)
        for (x0, y0, w, h, fn) in self.tiles:
            m = (~done) & (X >= x0 - 1e-12) & (X <= x0 + w + 1e-12) & (Y >= y0 - 1e-12) & (Y <= y0 + h + 1e-12)
            if m.any():
                out[m] = fn(X[m], Y[m])
                done |= m
        return out

    def Fx(self, X, Y, depth=1.0):
        """F continued beyond the edges through the seams (for derivatives at edges)."""
        X = np.array(X, float)
        Y = np.array(Y, float)
        for _ in range(3):
            out = ~self.inside(X, Y)
            if not out.any():
                break
            for s in self.seams:
                d = s.a1 - s.a0
                L2 = d @ d
                tt = ((X - s.a0[0]) * d[0] + (Y - s.a0[1]) * d[1]) / L2
                dist = (X - s.a0[0]) * s.n[0] + (Y - s.a0[1]) * s.n[1]
                m = out & (tt >= -1e-9) & (tt <= 1 + 1e-9) & (dist > 0) & (dist <= depth)
                if m.any():
                    X[m], Y[m] = s.g(X[m], Y[m])
                    out &= ~m
        return self.F(X, Y)

    def tile_of(self, X, Y, tol=1e-12):
        X = np.asarray(X, float)
        Y = np.asarray(Y, float)
        out = np.full(X.shape, -1)
        for k, (x0, y0, w, h, _) in enumerate(self.tiles):
            m = (out < 0) & (X >= x0 - tol) & (X <= x0 + w + tol) & (Y >= y0 - tol) & (Y <= y0 + h + tol)
            out[m] = k
        return out

    def jac(self, X, Y, h):
        """Derivatives from samples in the same tile only (one-sided at tile edges), so no difference
        straddles a seam, an interior seam line or the world's edge."""
        p = self.F(X, Y)
        t0 = self.tile_of(X, Y)

        def d(dx, dy):
            fp, fm = self.F(X + dx, Y + dy), self.F(X - dx, Y - dy)
            tp, tm = self.tile_of(X + dx, Y + dy) == t0, self.tile_of(X - dx, Y - dy) == t0
            c = (fp - fm) / (2 * h)
            fwd = (fp - p) / h
            bwd = (p - fm) / h
            return np.where((tp & tm)[..., None], c, np.where(tp[..., None], fwd, bwd))
        return p, d(h, 0.0), d(0.0, h)

    def survey_world(self):
        seams = [(tuple(s.a0), tuple(s.a1), tuple(s.n), s.g) for s in self.seams]
        return survey.World(self.id, self.tiles, seams, self.lines, self.cls)


def seams_from_pairs(raw):
    """raw: list of (a0, a1, b0, b1, pair) meaning edge a0->a1 (ccw round tile A) is the same planet
    curve as b1->b0 (ccw round tile B runs b0->b1). Returns both sides."""
    out = []
    for (a0, a1, b0, b1, pair) in raw:
        a0, a1, b0, b1 = (np.array(v, float) for v in (a0, a1, b0, b1))
        da, db = a1 - a0, b0 - b1
        ang = math.atan2(db[1], db[0]) - math.atan2(da[1], da[0])
        k = int(round(ang / (math.pi / 2))) % 4
        R = rot(k)
        t = b1 - R @ a0
        kind, _ = kind_of(R)
        out.append(Seam(a0, a1, R, t, pair, kind, "B"))
        Ri = R.T
        ti = a0 - Ri @ b1
        out.append(Seam(b0, b1, Ri, ti, pair, kind, "A"))
    return out


# ---- cube nets ------------------------------------------------------------------------------

FRAMES = {k: survey.BAND_FRAMES[k] for k in range(4)}
FRAMES["N"] = survey.N_FRAME
FRAMES["S"] = survey.S_FRAME


def zrot(lam):
    c, s = math.cos(lam), math.sin(lam)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])


def cube_net(lid, placements, face_fn, lam0, tile_fns=None):
    """placements: (face, i, j, quarter-turns ccw of the face's picture in the net)."""
    Rz = zrot(lam0)
    tiles = []
    corners_world = {}
    edges = {}
    for ti, (f, i, j, r) in enumerate(placements):
        c, e1, e2 = FRAMES[f]
        Rinv = rot(-r)

        def mk(i=i, j=j, Rinv=Rinv, c=c, e1=e1, e2=e2, f=f):
            def fn(X, Y):
                a = 2 * (X - i) - 1
                b = 2 * (Y - j) - 1
                u = Rinv[0, 0] * a + Rinv[0, 1] * b
                v = Rinv[1, 0] * a + Rinv[1, 1] * b
                if tile_fns is not None:
                    p = tile_fns[f](u, v)
                else:
                    abc = face_fn(u, v)
                    p = abc[..., 0:1] * e1 + abc[..., 1:2] * e2 + abc[..., 2:3] * c
                return p @ Rz.T
            return fn
        tiles.append((i, j, 1, 1, mk()))
        # topology from the gnomonic frame (exact corners)
        cw = [(i + 1, j), (i + 1, j + 1), (i, j + 1), (i, j)]   # E side starts at (i+1, j), ccw
        keys = []
        for (X, Y) in cw:
            a, b = 2 * (X - i) - 1, 2 * (Y - j) - 1
            u = Rinv[0, 0] * a + Rinv[0, 1] * b
            v = Rinv[1, 0] * a + Rinv[1, 1] * b
            p = u * e1 + v * e2 + c
            key = tuple(int(round(q)) for q in p)
            keys.append(key)
            corners_world.setdefault(key, []).append((X, Y))
        for s in range(4):
            p0, p1 = cw[s], cw[(s + 1) % 4]
            k0, k1 = keys[s], keys[(s + 1) % 4]
            edges.setdefault(frozenset([k0, k1]), []).append((ti, p0, p1, k0, k1))
    raw, internal, hinges = [], [], []
    pair = 0
    for ek, lst in edges.items():
        assert len(lst) == 2, lst
        (ta, a0, a1, ka0, ka1), (tb, b0, b1, kb0, kb1) = lst
        if tuple(a0) == tuple(b1) and tuple(a1) == tuple(b0):
            internal.append((a0, a1))
            hinges.append((ta, tb, a0, a1))
            continue
        assert ka0 == kb1 and ka1 == kb0
        raw.append((a0, a1, b0, b1, pair))
        pair += 1
    L = Layout(lid, tiles, seams_from_pairs(raw), R16 * math.sqrt(4 * math.pi / 6), "cube")
    L.internal = internal
    L.hinges = hinges
    L.fold = "cube"
    vid = {k: n for n, k in enumerate(sorted(corners_world))}
    for k, pts in corners_world.items():
        for (X, Y) in set(pts):
            L.corners.append((X, Y, vid[k], 270))
    lines = [(a0, a1, (a1[1] - a0[1], -(a1[0] - a0[0]))) for (a0, a1) in internal]
    for (i, j, w, h, _) in tiles:
        lines += survey.tile_lines(i, j, 1, 1)
    L.lines = [(tuple(map(float, p0)), tuple(map(float, p1)), tuple(map(float, n))) for (p0, p1, n) in lines]
    return L


CROSS = [(0, 0, 0, 0), (1, 1, 0, 0), (2, 2, 0, 0), (3, 3, 0, 0), ("N", 0, 1, 0), ("S", 0, -1, 0)]
# 3 x 2: top row the band faces 0, 1, 2 (north up); bottom row the meridian strip N, 3, S (turned)
NET32 = [(0, 0, 1, 0), (1, 1, 1, 0), (2, 2, 1, 0), ("N", 0, 0, 2), (3, 1, 0, 1), ("S", 2, 0, 0)]


def healpix_tile_fns():
    Q = math.pi / 2
    R135 = zrot(math.radians(135))
    def band(k):
        def fn(u, v):
            X = k + (u + 1) / 2
            Y = (v + 1) / 2
            return survey.healpix_inv(-math.pi + Q * X, Q * Y - math.pi / 4) @ R135.T
        return fn
    hw = survey.healpix_world()
    fN = hw.tiles[4][4]
    fS = hw.tiles[5][4]
    def pol(fp, y0):
        def fn(u, v):
            return fp((u + 1) / 2, y0 + (v + 1) / 2) @ R135.T
        return fn
    d = {k: band(k) for k in range(4)}
    d["N"] = pol(fN, 1)
    d["S"] = pol(fS, -1)
    return d


# ---- pillowcase (Guyou) and Peirce square ---------------------------------------------------


def guyou(lid, hemi, lamc):
    """The survey's polar-aspect pillowcase (tiles N [-1,1]^2, S [1,3]x[-1,1]) with the sphere turned
    so the tile centre goes to (0, lamc) and the top-edge midpoint to the north pole (Guyou)."""
    a = np.array([0.0, 0, 1])
    e = survey.normalize(hemi(np.array([0.0]), np.array([1.0 - 1e-9])))[0]
    e[2] = 0
    e /= np.linalg.norm(e)
    f = np.cross(a, e)
    a2 = np.array([math.cos(lamc), math.sin(lamc), 0])
    e2 = np.array([0.0, 0, 1])
    f2 = np.cross(a2, e2)
    Rs = np.stack([a2, e2, f2], 1) @ np.stack([a, e, f], 1).T

    def fN(X, Y):
        return hemi(X, Y) @ Rs.T

    def fS(X, Y):
        return survey.reflect_equator(hemi(2 - X, Y)) @ Rs.T

    tiles = [(-1, -1, 2, 2, fN), (1, -1, 2, 2, fS)]
    raw = [((3, -1), (3, 1), (-1, 1), (-1, -1), 0),           # S east -> N west (slide)
           ((1, 1), (-1, 1), (3, 1), (1, 1), 1),              # top: N half onto S half (half-turn about (1, 1))
           ((-1, -1), (1, -1), (1, -1), (3, -1), 2)]          # bottom: half-turn about (1, -1)
    L = Layout(lid, tiles, seams_from_pairs(raw), R16 * math.sqrt(4 * math.pi / 8), "pillowcase")
    L.internal = [((1, -1), (1, 1))]
    L.hinges = [(0, 1, (1, -1), (1, 1))]
    L.fold = "pillow"
    L.corners = [(1, 1, 0, 180), (1, -1, 1, 180), (-1, 1, 2, 180), (3, 1, 2, 180),
                 (-1, -1, 3, 180), (3, -1, 3, 180)]
    lines = [((1, -1), (1, 1), (1, 0))] + survey.tile_lines(-1, -1, 2, 2) + survey.tile_lines(1, -1, 2, 2)
    L.lines = lines
    L.extra["Rs"] = Rs
    return L


def peirce_square(lid, hemi):
    """Peirce's own aspect: the pole at the centre of a square of side 2*sqrt2 (tile units), each
    edge glued to itself by a half-turn about its midpoint (a singular point on the equator)."""
    base = guyou("tmp", hemi, 0.0)
    # polar aspect: undo the Guyou turn
    Rs = base.extra["Rs"]
    q = 1 / math.sqrt(2)

    def fn(P, Q):
        X = q * (P - Q)
        Y = q * (P + Q)
        return base.Fx(X, Y, depth=2.0) @ Rs  # (row vectors) @ Rs = Rs^T applied: back to polar
    s = math.sqrt(2)
    tiles = [(-s, -s, 2 * s, 2 * s, fn)]
    raw = []
    k = 0
    for (c0, c1) in [((s, -s), (s, s)), ((s, s), (-s, s)), ((-s, s), (-s, -s)), ((-s, -s), (s, -s))]:
        c0, c1 = np.array(c0), np.array(c1)
        m = (c0 + c1) / 2
        # half a0=c0..m glued to the other half reversed: the half-turn about m
        raw.append((tuple(c0), tuple(m), tuple(m), tuple(c1), k))
        k += 1
    L = Layout(lid, tiles, seams_from_pairs(raw), R16 * math.sqrt(4 * math.pi / 8), "pillowcase")
    L.corners = [(s, 0, 0, 180), (0, s, 1, 180), (-s, 0, 2, 180), (0, -s, 3, 180)]
    L.extra["southpole"] = [(s, s), (-s, s), (-s, -s), (s, -s)]
    L.fold = None
    L.lines = []
    return L


def lambert(lid):
    zc = math.sin(math.radians(80))

    def fn(X, Y):
        return survey.ll2xyz(np.arcsin(np.clip(Y, -1, 1)), X)
    tiles = [(-math.pi, -zc, TAU, 2 * zc, fn)]
    raw = [((math.pi, -zc), (math.pi, zc), (-math.pi, zc), (-math.pi, -zc), 0)]
    L = Layout(lid, tiles, seams_from_pairs(raw), R16, "cylinder", walls=True)
    L.fold = "cylinder"
    L.extra["band70"] = math.sin(math.radians(70))
    L.extra["stop80"] = zc
    return L


# ------------------------------------------------------------------------------------------
# rendering


def grid_for(L, px_long):
    bx0, by0, bx1, by1 = L.bbox
    sc = px_long / max(bx1 - bx0, by1 - by0)
    nx, ny = int(math.ceil((bx1 - bx0) * sc)), int(math.ceil((by1 - by0) * sc))
    xs = bx0 + (np.arange(nx) + 0.5) / sc
    ys = by1 - (np.arange(ny) + 0.5) / sc
    X, Y = np.meshgrid(xs, ys)
    return X, Y, sc


def save(img, path, q=82):
    im = Image.fromarray(np.clip(img * 255 + 0.5, 0, 255).astype(np.uint8))
    im.save(path, "WEBP", quality=q, method=5)


def palette(stops, t):
    t = np.clip(t, 0, 1)
    xs = np.linspace(0, 1, len(stops))
    st = np.array(stops, float) / 255
    return np.stack([np.interp(t, xs, st[:, k]) for k in range(3)], -1)


DIVERGE = [(33, 102, 172), (103, 169, 207), (209, 229, 240), (247, 247, 247), (253, 219, 199),
           (239, 138, 98), (178, 24, 43)]
SEQ = [(250, 250, 240), (254, 232, 160), (253, 174, 97), (230, 97, 60), (190, 40, 70), (120, 20, 90), (50, 10, 70)]


def render_layout(L, planet, out, px=1400, hpx=700):
    log("render", L.id)
    X, Y, sc = grid_for(L, px)
    P = L.F(X, Y)
    ok = np.isfinite(P[..., 0])
    px_blocks = L.unit_blocks / sc
    S = planet.sample(P)
    shade = hillshade(S[..., 0], px_blocks)
    rgb = colorize(np.nan_to_num(S), shade)
    a = ok.astype(float)[..., None]
    save(np.concatenate([rgb, a], -1), f"{out}/{L.id}-map.webp")
    # graticule
    lat = np.degrees(np.arcsin(np.clip(P[..., 2], -1, 1)))
    lon = np.degrees(np.arctan2(P[..., 1], P[..., 0]))
    g = np.zeros(lat.shape, bool)
    eq = np.zeros(lat.shape, bool)
    for arr, step, lim in ((lat, 15.0, None), (lon, 15.0, 80.0)):
        b = np.floor(np.nan_to_num(arr) / step)
        for dr, dc in ((0, 1), (1, 0)):
            nb = np.roll(b, (-dr, -dc), axis=(0, 1))
            okn = np.roll(ok, (-dr, -dc), axis=(0, 1))
            ch = (nb != b) & ok & okn
            if lim is not None:
                ch &= np.abs(lat) < lim
                # the antimeridian jump of atan2 is a line too (180 deg), keep it
            g |= ch
    beq = np.sign(np.nan_to_num(lat))
    for dr, dc in ((0, 1), (1, 0)):
        nb = np.roll(beq, (-dr, -dc), axis=(0, 1))
        okn = np.roll(ok, (-dr, -dc), axis=(0, 1))
        eq |= (nb != beq) & ok & okn
    G = np.zeros(lat.shape + (4,))
    G[g] = [1, 1, 1, 0.75]
    G[eq] = [1, 0.85, 0.2, 1.0]
    save(G, f"{out}/{L.id}-grat.webp", q=90)
    # crease test
    syn = synthetic(np.nan_to_num(P))
    syn = np.where(ok, syn, np.nan)
    sh = hillshade(syn, px_blocks, exag=4.0)
    cr = np.stack([sh * 0.62, sh * 0.62, sh * 0.66], -1)
    save(np.concatenate([np.clip(cr, 0, 1), a], -1), f"{out}/{L.id}-crease.webp")
    # distortion heat maps (half resolution)
    Xh, Yh, sch = grid_for(L, hpx)
    h = 0.15 / sch
    p, da, db = L.jac(Xh, Yh, h)
    okh = np.isfinite(p[..., 0]) & L.inside(Xh, Yh)
    s1, s2 = survey.sv(da, db)
    mean = 1.0 if L.cls == "cylinder" else 4 * math.pi / L.area
    bpa = mean / np.maximum(s1 * s2, 1e-300)
    shape = s1 / np.maximum(s2, 1e-300)
    t = (np.log2(np.clip(np.nan_to_num(bpa, nan=1), 0.25, 4)) + 2) / 4
    A = np.concatenate([palette(DIVERGE, t), okh.astype(float)[..., None]], -1)
    save(A, f"{out}/{L.id}-area.webp", q=88)
    ts = np.log(np.clip(np.nan_to_num(shape, nan=1), 1, 4)) / math.log(4)
    B = np.concatenate([palette(SEQ, ts), okh.astype(float)[..., None]], -1)
    save(B, f"{out}/{L.id}-shape.webp", q=88)
    return {"px": [int(X.shape[1]), int(X.shape[0])], "hpx": [int(Xh.shape[1]), int(Xh.shape[0])]}


def tissot(L, k, rho=math.radians(6.0)):
    pts = []
    for (x0, y0, w, hh, _) in L.tiles:
        nx = max(1, int(round(k * w / min(w, hh)))) if L.cls == "cylinder" else k
        ny = max(1, int(round(k * hh / min(w, hh)))) if L.cls == "cylinder" else k
        if L.cls == "cylinder":
            nx, ny = 16, 5
        for i in range(nx):
            for j in range(ny):
                pts.append((x0 + (i + 0.5) * w / nx, y0 + (j + 0.5) * hh / ny))
    P = np.array(pts)
    p, a, b = L.jac(P[:, 0], P[:, 1], 1e-5)
    E = (a * a).sum(-1)
    Fm = (a * b).sum(-1)
    G = (b * b).sum(-1)
    out = []
    for n in range(len(P)):
        M = np.array([[E[n], Fm[n]], [Fm[n], G[n]]])
        if not np.isfinite(M).all():
            continue
        w_, V = np.linalg.eigh(M)  # world metric: planet length^2 per world length^2
        w_ = np.maximum(w_, 1e-12)
        r = rho / np.sqrt(w_) * L.unit_blocks  # semi-axes in blocks
        ang = math.degrees(math.atan2(V[1, 0], V[0, 0]))
        out.append([round(P[n, 0] * L.unit_blocks, 1), round(P[n, 1] * L.unit_blocks, 1),
                    round(float(min(r[0], 4000)), 1), round(float(min(r[1], 4000)), 1), round(ang, 1)])
    return out


# ------------------------------------------------------------------------------------------
# walks


def walk(L, start, heading, length, ds=0.002):
    pos = np.array(start, float)
    d = np.array(heading, float)
    d /= np.linalg.norm(d)
    segs, cur, crossings = [], [pos.copy()], []
    travelled, stopped = 0.0, None
    guard = 0
    seam_tile = []
    for s in L.seams:
        m = (s.a0 + s.a1) / 2 - 1e-6 * s.n
        seam_tile.append(int(L.tile_of(m[0], m[1])))
    tile = int(L.tile_of(pos[0] + 1e-9 * d[0], pos[1] + 1e-9 * d[1]))

    def in_tile(q, k):
        x0, y0, w, h, _ = L.tiles[k]
        return x0 - 1e-12 <= q[0] <= x0 + w + 1e-12 and y0 - 1e-12 <= q[1] <= y0 + h + 1e-12

    while travelled < length - 1e-12 and guard < 200000:
        guard += 1
        step = min(ds, length - travelled)
        new = pos + step * d
        if in_tile(new, tile):
            pos = new
            cur.append(pos.copy())
            travelled += step
            continue
        best = None
        for si, s in enumerate(L.seams):
            if seam_tile[si] != tile:
                continue
            dd = s.a1 - s.a0
            M = np.array([[step * d[0], -dd[0]], [step * d[1], -dd[1]]])
            if abs(np.linalg.det(M)) < 1e-14 or d @ s.n <= 0:
                continue
            u, v = np.linalg.solve(M, s.a0 - pos)
            if -1e-9 <= u <= 1 + 1e-9 and -1e-9 <= v <= 1 + 1e-9 and (best is None or u < best[0]):
                best = (u, s)
        if best is None and L.inside(new[0], new[1]):
            # an internal edge: the next face is the neighbour on the planet too
            tile = int(L.tile_of(new[0], new[1]))
            pos = new
            cur.append(pos.copy())
            travelled += step
            continue
        if best is None:
            # a wall (Lambert's stop line)
            lo, hi = 0.0, 1.0
            for _ in range(50):
                mid = (lo + hi) / 2
                q = pos + mid * step * d
                if L.inside(q[0], q[1]):
                    lo = mid
                else:
                    hi = mid
            pos = pos + lo * step * d
            cur.append(pos.copy())
            stopped = pos.copy()
            travelled += lo * step
            break
        u, s = best
        cp = pos + u * step * d
        cur.append(cp - 1e-9 * d)
        segs.append(cur)
        mx, my = s.g(cp[0], cp[1])
        d2 = s.R @ d
        crossings.append({"at": [round(cp[0] * L.unit_blocks, 1), round(cp[1] * L.unit_blocks, 1)],
                          "to": [round(mx * L.unit_blocks, 1), round(my * L.unit_blocks, 1)],
                          "kind": s.kind, "pair": s.pair,
                          "turn": kind_of(s.R)[1], "dist": round((travelled + u * step) * L.unit_blocks)})
        pos = np.array([mx, my]) + 1e-10 * d2
        d = d2
        tile = int(L.tile_of(pos[0] + 1e-9 * d[0], pos[1] + 1e-9 * d[1]))
        cur = [pos.copy()]
        travelled += u * step
    segs.append(cur)
    allp = np.concatenate([np.array(sg) for sg in segs])
    xyz = L.F(allp[:, 0], allp[:, 1])
    # planet arc per step within segments
    arcs, k = [], 0
    for sg in segs:
        q = xyz[k:k + len(sg)]
        if len(sg) > 1:
            dots = np.clip((q[1:] * q[:-1]).sum(-1), -1, 1)
            ar = np.nan_to_num(np.arccos(dots))
            wl = np.linalg.norm(np.diff(np.array(sg), axis=0), axis=1) * L.unit_blocks
            arcs.append(np.stack([wl, ar], 1))
        k += len(sg)
    arcs = np.concatenate(arcs) if arcs else np.zeros((0, 2))
    cum_b = np.cumsum(arcs[:, 0])
    good = arcs[:, 1] > 1e-12
    bpd = np.where(good, arcs[:, 0] / np.degrees(np.maximum(arcs[:, 1], 1e-12)), np.nan)
    # profile: 160 bins of blocks per degree along the walk
    prof = []
    if len(cum_b):
        edges = np.linspace(0, cum_b[-1], 161)
        idx = np.digitize(cum_b, edges[1:-1])
        for b_ in range(160):
            m = (idx == b_) & good
            if m.any():
                prof.append([round(float(edges[b_ + 1]), 1),
                             round(float(arcs[m, 0].sum() / np.degrees(arcs[m, 1].sum())), 2)])
    lat = np.degrees(np.arcsin(np.clip(xyz[:, 2], -1, 1)))
    lon = np.degrees(np.arctan2(xyz[:, 1], xyz[:, 0]))
    sub = np.unique(np.concatenate([np.arange(0, len(lat), max(1, len(lat) // 400)), [len(lat) - 1]]))
    track = [[round(float(lat[i]), 3), round(float(lon[i]), 3)] for i in sub if np.isfinite(lat[i])]
    segs_b = []
    for sg in segs:
        a = np.array(sg) * L.unit_blocks
        sub2 = np.unique(np.concatenate([np.arange(0, len(a), max(1, len(a) // 60)), [len(a) - 1]]))
        segs_b.append([[round(float(a[i, 0]), 1), round(float(a[i, 1]), 1)] for i in sub2])
    return {"segs": segs_b, "crossings": crossings, "track": track, "profile": prof,
            "blocks": round(float(cum_b[-1]) if len(cum_b) else 0, 1),
            "planet_deg": round(float(np.degrees(np.nansum(arcs[:, 1]))), 2),
            "stopped": None if stopped is None else [round(stopped[0] * L.unit_blocks, 1), round(stopped[1] * L.unit_blocks, 1)]}


# ------------------------------------------------------------------------------------------
# globe overlays, meshes


def latlon(xyz):
    return [[round(float(math.degrees(math.asin(max(-1, min(1, p[2]))))), 3),
             round(float(math.degrees(math.atan2(p[1], p[0]))), 3)] for p in xyz if np.isfinite(p[0])]


def overlays(L):
    out = {"seams": [], "edges": [], "corners": []}
    t = np.linspace(0, 1, 49)
    for s in L.seams:
        if s.partner != "B":
            continue
        P = s.a0[None, :] + t[:, None] * (s.a1 - s.a0)[None, :] - 1e-7 * s.n[None, :]
        out["seams"].append({"pair": s.pair, "kind": s.kind, "ll": latlon(L.F(P[:, 0], P[:, 1]))})
    for (a0, a1) in L.internal:
        a0, a1 = np.array(a0, float), np.array(a1, float)
        P = a0[None, :] + t[:, None] * (a1 - a0)[None, :]
        out["edges"].append(latlon(L.Fx(P[:, 0], P[:, 1])))
    seen = set()
    for (X, Y, v, cone) in L.corners:
        if v in seen:
            continue
        seen.add(v)
        q = L.Fx(np.array([X * 0.999999 + 1e-7]), np.array([Y * 0.999999 + 1e-7]))
        c = survey.normalize(q)[0]
        out["corners"].append({"v": v, "cone": cone, "ll": latlon([c])[0]})
    return out


def mesh(L):
    if L.fold is None and L.cls != "pillowcase":
        return None
    n = L.mesh_cells
    tiles = []
    for (x0, y0, w, h, _) in L.tiles:
        if L.cls == "cylinder":
            nx, ny = 128, 24
        else:
            nx = ny = n
        xs = x0 + np.linspace(0, 1, nx + 1) * w
        ys = y0 + np.linspace(0, 1, ny + 1) * h
        X, Y = np.meshgrid(xs, ys)  # rows = y
        eps = 1e-6 * max(w, h)
        Xi = np.clip(X, x0 + eps, x0 + w - eps)
        Yi = np.clip(Y, y0 + eps, y0 + h - eps)
        P = L.F(Xi, Yi)
        P = survey.normalize(np.nan_to_num(P))
        q = np.clip(np.round(P * 32767), -32767, 32767).astype("<i2")
        tiles.append({"x0": x0, "y0": y0, "w": w, "h": h, "nx": nx, "ny": ny,
                      "xyz": base64.b64encode(q.tobytes()).decode()})
    hinges = [{"parent": a, "child": b, "p0": list(map(float, p0)), "p1": list(map(float, p1))}
              for (a, b, p0, p1) in L.hinges]
    return {"tiles": tiles, "hinges": hinges, "fold": L.fold,
            "radius": math.sqrt(L.area / (4 * math.pi)) if L.cls != "cylinder" else 1.0}


def seams_json(L):
    out = []
    for s in L.seams:
        out.append({"pair": s.pair, "kind": s.kind, "side": s.partner,
                    "a0": [round(float(v) * L.unit_blocks, 2) for v in s.a0],
                    "a1": [round(float(v) * L.unit_blocks, 2) for v in s.a1],
                    "turn": kind_of(s.R)[1]})
    return out


def self_check(L):
    """Every seam closes: F just outside (folded) equals F just inside, along every seam."""
    worst = 0.0
    t = np.linspace(0.02, 0.98, 50)
    for s in L.seams:
        P = s.a0[None, :] + t[:, None] * (s.a1 - s.a0)[None, :]
        inside = L.F(P[:, 0] - 1e-9 * s.n[0], P[:, 1] - 1e-9 * s.n[1])
        gx, gy = s.g(P[:, 0] + 1e-9 * s.n[0], P[:, 1] + 1e-9 * s.n[1])
        other = L.F(gx, gy)
        dd = np.linalg.norm(inside - other, axis=-1)
        worst = max(worst, float(np.nanmax(dd)))
    return worst


# ------------------------------------------------------------------------------------------
# corner placement sweeps


def disc_points(c, rad, n=40):
    c = c / np.linalg.norm(c)
    a = np.cross(c, [0, 0, 1.0]) if abs(c[2]) < 0.9 else np.cross(c, [1.0, 0, 0])
    a /= np.linalg.norm(a)
    b = np.cross(c, a)
    rr = rad * np.sqrt((np.arange(n) + 0.5) / n)
    th = np.arange(n) * 2.39996
    return (np.cos(rr)[:, None] * c[None, :] + np.sin(rr)[:, None]
            * (np.cos(th)[:, None] * a[None, :] + np.sin(th)[:, None] * b[None, :]))


def sweep(planet, corners_fn, lams, rad=math.radians(3)):
    rows = []
    for lam in lams:
        cs = corners_fn(lam)
        shares = [float(planet.land(disc_points(c, rad)).mean()) for c in cs]
        rows.append([round(math.degrees(lam), 2), round(max(shares), 3), round(float(np.mean(shares)), 3)])
    return rows


def cube_corner_dirs(lam):
    out = []
    for zs in (1, -1):
        for k in range(4):
            a = lam + math.pi / 4 + k * math.pi / 2
            v = np.array([math.cos(a), math.sin(a), zs / math.sqrt(2)])
            out.append(v / np.linalg.norm(v))
    return out


def guyou_corner_dirs(lam):
    # singular points at +-45 deg latitude on the meridians 90 deg from the hemisphere centres
    out = []
    for zs in (1, -1):
        for dl in (math.pi / 2, -math.pi / 2):
            a = lam + dl
            out.append(np.array([math.cos(a) * math.cos(math.pi / 4), math.sin(a) * math.cos(math.pi / 4),
                                 zs * math.sin(math.pi / 4)]))
    return out


# ------------------------------------------------------------------------------------------


def main():
    hm, out = sys.argv[1], sys.argv[2]
    cache = sys.argv[3] if len(sys.argv) > 3 else out
    os.makedirs(out, exist_ok=True)
    planet = Planet(hm)

    # corner placement
    lam_c = np.radians(np.arange(0, 90, 1.0))
    sw_cube = sweep(planet, cube_corner_dirs, lam_c)
    lam_g = np.radians(np.arange(0, 360, 2.0))
    sw_guyou = sweep(planet, guyou_corner_dirs, lam_g)
    best_cube = min(sw_cube, key=lambda r: (r[1], r[2]))
    best_guyou = min(sw_guyou, key=lambda r: (r[1], r[2]))
    log("best cube lam0", best_cube, "best guyou lamc", best_guyou)
    lam0 = math.radians(best_cube[0])
    lamg = math.radians(best_guyou[0])

    # face maps
    mpath = os.path.join(cache, "moser-cube-512.npz")
    if os.path.exists(mpath):
        z = np.load(mpath)
        s_, G = z["s"], z["G"]
        X0, Y0 = np.meshgrid(s_, s_, indexing="ij")
        G0 = z["G0"]
    else:
        import moser
        s_, G, info = moser.build("cube", n=512)
        X0, Y0 = np.meshgrid(s_, s_, indexing="ij")
        G0, _, _ = moser.conformal_tile("cube", X0, Y0)
        np.savez_compressed(mpath, s=s_, G=G, G0=G0)
        log("moser built", info)
    from scipy.interpolate import RectBivariateSpline

    def spline_face(GG):
        sps = [RectBivariateSpline(s_, s_, GG[..., i], kx=3, ky=3) for i in range(3)]
        return lambda u, v: survey.normalize(np.stack([sp.ev(u, v) for sp in sps], axis=-1))
    face_moser = spline_face(G)
    face_conf = spline_face(G0)

    layouts = [
        lambert("lambert"),
        guyou("guyou-petroff", survey.hemi_petroff, lamg),
        guyou("guyou-peirce", survey.hemi_peirce, lamg),
        peirce_square("peirce-square", survey.hemi_peirce),
        guyou("guyou-gringorten", survey.hemi_gringorten, lamg),
        guyou("guyou-collignon", survey.hemi_collignon, lamg),
        cube_net("cube-moser-cross", CROSS, face_moser, lam0),
        cube_net("cube-moser-3x2", NET32, face_moser, lam0),
        cube_net("cube-qsc-cross", CROSS, survey.face_rosca, lam0),
        cube_net("cube-healpix-cross", CROSS, None, lam0, tile_fns=healpix_tile_fns()),
        cube_net("cube-eac-cross", CROSS, survey.face_eac, lam0),
        cube_net("cube-conformal-cross", CROSS, face_conf, lam0),
    ]
    only = os.environ.get("ONLY")
    if only:
        layouts = [L for L in layouts if L.id in only.split(",")]

    data = {"generated": time.strftime("%Y-%m-%d %H:%M UTC", time.gmtime()), "R16": R16,
            "lam0_deg": best_cube[0], "lamc_deg": best_guyou[0],
            "sweep_cube": sw_cube, "sweep_guyou": sw_guyou, "layouts": {}}
    for L in layouts:
        chk = self_check(L)
        log(L.id, "seam closure", chk)
        info = render_layout(L, planet, out)
        d = {"cls": L.cls, "unit_blocks": L.unit_blocks,
             "bbox": [v * L.unit_blocks for v in L.bbox],
             "tiles": [[t[0] * L.unit_blocks, t[1] * L.unit_blocks, t[2] * L.unit_blocks, t[3] * L.unit_blocks]
                       for t in L.tiles],
             "seams": seams_json(L), "seam_closure_rad": chk,
             "internal": [[[float(v) * L.unit_blocks for v in a0], [float(v) * L.unit_blocks for v in a1]]
                          for (a0, a1) in L.internal],
             "corners": [[X * L.unit_blocks, Y * L.unit_blocks, v, c] for (X, Y, v, c) in L.corners],
             "img": info, "tissot": tissot(L, 3 if L.cls == "cube" else 4),
             "overlay": overlays(L), "mesh": mesh(L),
             "extra": {k: (v * L.unit_blocks if isinstance(v, float) else None) for k, v in L.extra.items()
                       if k in ("band70", "stop80")}}
        if "southpole" in L.extra:
            d["extra"]["southpole"] = [[a * L.unit_blocks, b * L.unit_blocks] for a, b in L.extra["southpole"]]
        W = {}
        if L.cls == "cube":
            if L.id.endswith("3x2"):
                W["equator"] = walk(L, (1e-6, 1.5), (1, 0), 4.0)
                W["pole"] = walk(L, (0.5, 1.5), (0, 1), 2.0)
                W["cornerA"] = walk(L, (0.9, 1.5), (0, 1), 1.6)
                W["cornerB"] = walk(L, (1.1, 1.5), (0, 1), 1.6)
            else:
                W["equator"] = walk(L, (1e-6, 0.5), (1, 0), 4.0)
                W["pole"] = walk(L, (0.5, 0.5), (0, 1), 2.0)
                W["poleB"] = walk(L, (1.5, 0.5), (0, 1), 2.0)
                W["cornerA"] = walk(L, (0.9, 0.5), (0, 1), 1.6)
                W["cornerB"] = walk(L, (1.1, 0.5), (0, 1), 1.6)
        elif L.id == "peirce-square":
            s = math.sqrt(2)
            W["pole"] = walk(L, (-s / 2 + 1e-6, -s / 2 + 1e-6), (1, 1), 2.0)
            W["cornerA"] = walk(L, (0.9 * s - 0.1, -0.6), (0, 1), 1.6)
        elif L.cls == "pillowcase":
            W["equator"] = walk(L, (-1 + 1e-6, 0.0), (1, 0), 4.0)
            W["pole"] = walk(L, (0.0, 0.0), (0, 1), 2.0)
            W["cornerA"] = walk(L, (0.9, 0.0), (0, 1), 1.6)
            W["cornerB"] = walk(L, (1.1, 0.0), (0, 1), 1.6)
        else:
            W["equator"] = walk(L, (-math.pi + 1e-6, 0.0), (1, 0), TAU, ds=0.004)
            W["pole"] = walk(L, (0.0, 0.0), (0, 1), 2.0)
        d["walks"] = W
        data["layouts"][L.id] = d
        log(L.id, "walks", {k: (v["blocks"], v["planet_deg"], len(v["crossings"])) for k, v in W.items()})

    # the equirectangular globe texture
    log("globe texture")
    nlon, nlat = 2048, 1024
    lon = (np.arange(nlon) + 0.5) / nlon * TAU - math.pi
    lat = math.pi / 2 - (np.arange(nlat) + 0.5) / nlat * math.pi
    LO, LA = np.meshgrid(lon, lat)
    P = survey.ll2xyz(LA, LO)
    S = planet.sample(P)
    px_blocks = TAU * R16 / nlon
    shade = hillshade(S[..., 0], px_blocks, cosfix=np.cos(LA))
    save(colorize(np.nan_to_num(S), shade), f"{out}/globe.webp", q=85)
    with open(f"{out}/data.js", "w") as f:
        f.write("window.WS_DATA = ")
        json.dump(data, f, separators=(",", ":"))
        f.write(";\n")
    log("done", os.path.getsize(f"{out}/data.js"), "bytes of data.js")


if __name__ == "__main__":
    main()
