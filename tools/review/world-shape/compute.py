#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Pictures and data for the world-shape decision page (tools/review/world-shape/, 2026-10-03; reworked
2026-10-04 into a head-to-head of Petroff 2021 in the Guyou 2:1 layout against the Moser equal-area cube in
the cross net).

Runs on the test VM (numpy, scipy, mpmath, pillow), through run.sh. Every world is rebuilt on top of the
projection survey's own code (tools/research/projection-survey/) and a real Deep Time bake is resampled
onto it. For the two finalists:

  * <id>-{map,area,shape,crease,grat}.webp  the planet, blocks per planet area, shape ratio, a crease
                      test, a 15 degree graticule;
  * <id>-north.webp   how far F3's "north" (-z) is from true north;
  * <id>-bands.webp   latitude (climate) bands over the planet;
  * <id>-coast.webp   coastlines (an overlay);
  * <id>-dist.webp, <id>-pdist.webp, <id>-compass.webp   walking distance from a common spawn (rays
                      cast through the seams), the same planet distance as lines, and how far a vanilla
                      compass (raw coordinates) points from the true way back;
  * corner-<id>-{nbhd,raw,view,viewtint}.webp   a corner up close: the faces round it with the missing
                      wedge, the single raw copy the wrap engine's edge order (or a generator fold) shows
                      there, and what a player standing near it sees (rays traced through the seams);
  * tiling-guyou.webp, tiling-cube-{a,b,mix}.webp, rawext-cube.webp, rawext-cube-tear.webp   the plane
                      tiled by Guyou copies, the cube corner's two candidate pictures, the cube net's raw
                      surroundings under the engine's edge order and where they tear;
  * data.js           seams, corners, walks (equator, pole, diagonal, corner walkers, a ring round a
                      corner, straight trips by heading), F3 jumps per seam, block grids at a corner,
                      distortion and distance statistics on this planet, fold meshes, self-checks.
The other candidates get thumbnails for the appendix.

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
PXS = float(os.environ.get("WS_PXS", "1"))  # image-size factor (a small value for a quick smoke run)


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
# latitude (climate) bands: equatorial, tropics, subtropics, temperate, boreal, polar
BAND_EDGES = [10.0, 23.44, 35.0, 55.0, 66.56]
BAND_RGB = np.array([(214, 60, 46), (240, 140, 50), (235, 205, 80), (90, 170, 90), (60, 130, 170), (175, 160, 220)], float) / 255
DIST_STOPS = [(255, 247, 236), (253, 212, 158), (252, 141, 89), (215, 48, 31), (127, 0, 0), (40, 0, 40)]


def wq(vals, w, qs):
    o = np.argsort(vals)
    v, ww = vals[o], w[o]
    c = np.cumsum(ww)
    c /= c[-1]
    return [float(np.interp(q, c, v)) for q in qs]


def edges_of(mask, ok):
    """Pixels where a field changes against the right or lower neighbour (both valid)."""
    e = np.zeros(mask.shape, bool)
    for dr, dc in ((0, 1), (1, 0)):
        nb = np.roll(mask, (-dr, -dc), axis=(0, 1))
        okn = np.roll(ok, (-dr, -dc), axis=(0, 1))
        e |= (nb != mask) & ok & okn
    return e


def render_layout(L, planet, out, px=1400, hpx=700, full=True):
    log("render", L.id, "full" if full else "thumb")
    X, Y, sc = grid_for(L, px)
    P = L.F(X, Y)
    ok = np.isfinite(P[..., 0])
    px_blocks = L.unit_blocks / sc
    S = planet.sample(P)
    shade = hillshade(S[..., 0], px_blocks)
    rgb = colorize(np.nan_to_num(S), shade)
    a = ok.astype(float)[..., None]
    save(np.concatenate([rgb, a], -1), f"{out}/{L.id}-map.webp")
    syn = synthetic(np.nan_to_num(P))
    syn = np.where(ok, syn, np.nan)
    sh = hillshade(syn, px_blocks, exag=4.0)
    cr = np.stack([sh * 0.62, sh * 0.62, sh * 0.66], -1)
    save(np.concatenate([np.clip(cr, 0, 1), a], -1), f"{out}/{L.id}-crease.webp")
    info = {"px": [int(X.shape[1]), int(X.shape[0])]}
    if not full:
        return info, None
    # graticule
    lat = np.degrees(np.arcsin(np.clip(P[..., 2], -1, 1)))
    lon = np.degrees(np.arctan2(P[..., 1], P[..., 0]))
    g = np.zeros(lat.shape, bool)
    for arr, lim in ((lat, None), (lon, 80.0)):
        b = np.floor(np.nan_to_num(arr) / 15.0)
        e = edges_of(b, ok)
        if lim is not None:
            e &= np.abs(lat) < lim
        g |= e
    eq = edges_of(np.sign(np.nan_to_num(lat)), ok)
    G = np.zeros(lat.shape + (4,))
    G[g] = [1, 1, 1, 0.75]
    G[eq] = [1, 0.85, 0.2, 1.0]
    save(G, f"{out}/{L.id}-grat.webp", q=90)
    # coastlines
    land = (np.nan_to_num(S[..., 0]) > 63.0) & ok
    C = np.zeros(lat.shape + (4,))
    C[edges_of(land, ok)] = [1, 1, 1, 0.95]
    save(C, f"{out}/{L.id}-coast.webp", q=90)
    # latitude bands over the planet
    zone = np.digitize(np.abs(np.nan_to_num(lat)), BAND_EDGES)
    lum = rgb.mean(-1, keepdims=True)
    col = BAND_RGB[zone] * (0.45 + 0.75 * lum)
    col = np.where(land[..., None], col, col * 0.62 + 0.06)
    save(np.concatenate([np.clip(col, 0, 1), a], -1), f"{out}/{L.id}-bands.webp")
    # distortion and north error (half resolution)
    Xh, Yh, sch = grid_for(L, hpx)
    h = 0.15 / sch
    p, da, db = L.jac(Xh, Yh, h)
    okh = np.isfinite(p[..., 0]) & L.inside(Xh, Yh)
    s1, s2 = survey.sv(da, db)
    mean = 4 * math.pi / L.area
    bpa = mean / np.maximum(s1 * s2, 1e-300)
    shape = s1 / np.maximum(s2, 1e-300)
    t = (np.log2(np.clip(np.nan_to_num(bpa, nan=1), 0.25, 4)) + 2) / 4
    save(np.concatenate([palette(DIVERGE, t), okh.astype(float)[..., None]], -1), f"{out}/{L.id}-area.webp", q=88)
    ts = np.log(np.clip(np.nan_to_num(shape, nan=1), 1, 4)) / math.log(4)
    save(np.concatenate([palette(SEQ, ts), okh.astype(float)[..., None]], -1), f"{out}/{L.id}-shape.webp", q=88)
    zhat = np.array([0.0, 0, 1])
    nn = zhat[None, None, :] - p[..., 2:3] * p
    nn /= np.maximum(np.linalg.norm(nn, axis=-1, keepdims=True), 1e-12)
    E, Fm, Gm = (da * da).sum(-1), (da * db).sum(-1), (db * db).sum(-1)
    b1, b2 = (da * nn).sum(-1), (db * nn).sum(-1)
    det = E * Gm - Fm * Fm
    wx = (Gm * b1 - Fm * b2) / det
    wy = (E * b2 - Fm * b1) / det
    nerr = np.degrees(np.arctan2(wx, wy))       # clockwise from raw north (-z) to true north
    an = np.abs(np.nan_to_num(nerr))
    save(np.concatenate([palette(SEQ, an / 180.0), okh.astype(float)[..., None]], -1), f"{out}/{L.id}-north.webp", q=88)
    landh = planet.sample(np.nan_to_num(p))[..., 0] > 63.0
    info["hpx"] = [int(Xh.shape[1]), int(Xh.shape[0])]
    R = {"X": X, "Y": Y, "sc": sc, "P": P, "ok": ok, "lat": lat, "Xh": Xh, "Yh": Yh, "sch": sch, "p": p,
         "okh": okh, "shape": shape, "w": s1 * s2, "nerr": nerr, "landh": landh & okh}
    return info, R


def distortion_stats(L, R):
    m = R["okh"] & np.isfinite(R["shape"]) & np.isfinite(R["nerr"])
    land = m & R["landh"]
    w = R["w"]
    sh, an = R["shape"], np.abs(R["nerr"])

    def share(mask, cond):
        return round(float(w[mask & cond].sum() / w[mask].sum()), 4)
    return {"land_area_share": round(float(w[land].sum() / w[m].sum()), 4),
            "land_shape_q50_90_99": [round(v, 3) for v in wq(sh[land], w[land], [0.5, 0.9, 0.99])],
            "all_shape_q50_90_99": [round(v, 3) for v in wq(sh[m], w[m], [0.5, 0.9, 0.99])],
            "land_shape_gt": {str(k): share(land, sh > k) for k in (1.25, 1.5, 2.0, 2.5)},
            "all_shape_gt": {str(k): share(m, sh > k) for k in (1.25, 1.5, 2.0, 2.5)},
            "north_within": {str(k): share(m, an <= k) for k in (5, 15, 45, 90)},
            "land_north_within": {str(k): share(land, an <= k) for k in (5, 15, 45, 90)},
            "north_q50_90": [round(v, 1) for v in wq(an[m], w[m], [0.5, 0.9])]}


def parallel_bow(L, R, lat_deg, y_eq, y_top):
    """How far (blocks, along z) a parallel moves across the world between the equator line y_eq and y_top."""
    X, Y, lat, ok = R["X"], R["Y"], R["lat"], R["ok"]
    ys = []
    for j in range(X.shape[1]):
        yy, la, oo = Y[:, j], lat[:, j], ok[:, j]
        m = oo & (yy >= y_eq) & (yy <= y_top) & np.isfinite(la)
        if m.sum() < 4:
            continue
        o = np.argsort(la[m])
        lv, yv = la[m][o], yy[m][o]
        if lv[0] > lat_deg or lv[-1] < lat_deg:
            continue
        ys.append(float(np.interp(lat_deg, lv, yv)))
    ys = np.array(ys)
    return {"lat": lat_deg, "min_blocks_from_eq": round(float((ys.min() - y_eq) * L.unit_blocks)),
            "max_blocks_from_eq": round(float((ys.max() - y_eq) * L.unit_blocks)),
            "bow_blocks": round(float((ys.max() - ys.min()) * L.unit_blocks)), "columns": len(ys)}


def tissot(L, k, rho=math.radians(6.0)):
    pts = []
    for (x0, y0, w, hh, _) in L.tiles:
        nx, ny = (16, 5) if L.cls == "cylinder" else (k, k)
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
        w_, V = np.linalg.eigh(M)
        w_ = np.maximum(w_, 1e-12)
        r = rho / np.sqrt(w_) * L.unit_blocks
        ang = math.degrees(math.atan2(V[1, 0], V[0, 0]))
        out.append([round(P[n, 0] * L.unit_blocks, 1), round(P[n, 1] * L.unit_blocks, 1),
                    round(float(min(r[0], 4000)), 1), round(float(min(r[1], 4000)), 1), round(ang, 1)])
    return out


# ------------------------------------------------------------------------------------------
# Minecraft coordinates (the wrap engine's layouts: WrapLayouts.guyou centred, cubeCross with F0 at
# x, z in [0, s)), yaw and facing


def mc_frame(L):
    """(A, c): block (x, z) = A (X, Y) + c for tile coordinates (X east, Y north)."""
    u = L.mc_unit
    if L.cls == "cube":
        return np.array([[u, 0.0], [0.0, -u]]), np.array([0.0, u])
    return np.array([[u, 0.0], [0.0, -u]]), np.array([-u, 0.0])


def to_mc(L, X, Y):
    A, c = mc_frame(L)
    return A[0, 0] * X + c[0], A[1, 1] * Y + c[1]


def mc_yaw(d):
    """Minecraft yaw of a heading (dX east, dY north): 0 south, 90 west, 180 north, -90 east."""
    return math.degrees(math.atan2(-d[0], -d[1]))


def seam_mc(L):
    """Each seam pair's motion on Minecraft points, (x, z) -> M (x, z) + T, and the yaw change."""
    A, c = mc_frame(L)
    Ai = np.linalg.inv(A)
    out = []
    for s in L.seams:
        if s.partner != "B":
            continue
        M = A @ s.R @ Ai
        T = A @ s.t + c - M @ c
        ang = kind_of(s.R)[1]
        dyaw = (-ang + 540) % 360 - 180
        mid = (s.a0 + s.a1) / 2
        q = to_mc(L, mid[0], mid[1])
        gx, gy = s.g(mid[0], mid[1])
        q2 = to_mc(L, gx, gy)
        out.append({"pair": s.pair, "kind": s.kind, "name": L.seam_names.get(s.pair, ""),
                    "M": [[int(round(v)) for v in row] for row in M], "T": [int(round(v)) for v in T],
                    "dyaw": int(dyaw), "mid": [int(round(q[0])), int(round(q[1]))],
                    "mid_to": [int(round(q2[0])), int(round(q2[1]))]})
    return out


def name_seams(L, tile_names):
    """Name each seam pair by the two tile edges it joins, e.g. 'F1 top / N east'."""
    def side(seam):
        mid = (seam.a0 + seam.a1) / 2 - 1e-6 * seam.n
        k = int(L.tile_of(mid[0], mid[1]))
        n = seam.n
        sd = "east" if n[0] > 0.5 else "west" if n[0] < -0.5 else "top" if n[1] > 0.5 else "bottom"
        return f"{tile_names[k]} {sd}"
    names = {}
    for s in L.seams:
        names.setdefault(s.pair, []).append(side(s))
    L.seam_names = {k: " / ".join(v) for k, v in names.items()}


# ------------------------------------------------------------------------------------------
# walks (straight, or curving at a constant rate), with what F3 shows at each crossing


def walk(L, start, heading, length, ds=0.002, curv=0.0):
    pos = np.array(start, float)
    d = np.array(heading, float)
    d /= np.linalg.norm(d)
    d0 = d.copy()
    segs, cur, crossings = [], [pos.copy()], []
    travelled, stopped = 0.0, None
    guard = 0
    seam_tile = []
    for s in L.seams:
        m = (s.a0 + s.a1) / 2 - 1e-6 * s.n
        seam_tile.append(int(L.tile_of(m[0], m[1])))
    tile = int(L.tile_of(pos[0] + 1e-9 * d[0], pos[1] + 1e-9 * d[1]))
    felt = 0.0

    def in_tile(q, k):
        x0, y0, w, h, _ = L.tiles[k]
        return x0 - 1e-12 <= q[0] <= x0 + w + 1e-12 and y0 - 1e-12 <= q[1] <= y0 + h + 1e-12

    def turn(v, a):
        c, s_ = math.cos(a), math.sin(a)
        return np.array([c * v[0] - s_ * v[1], s_ * v[0] + c * v[1]])

    while travelled < length - 1e-12 and guard < 400000:
        guard += 1
        step = min(ds, length - travelled)
        if curv:
            d = turn(d, curv * step / 2)
        new = pos + step * d
        if curv:
            d = turn(d, curv * step / 2)
            felt += curv * step
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
            M = np.array([[new[0] - pos[0], -dd[0]], [new[1] - pos[1], -dd[1]]])
            if abs(np.linalg.det(M)) < 1e-14 or d @ s.n <= 0:
                continue
            u, v = np.linalg.solve(M, s.a0 - pos)
            if -1e-9 <= u <= 1 + 1e-9 and -1e-9 <= v <= 1 + 1e-9 and (best is None or u < best[0]):
                best = (u, s)
        if best is None and L.inside(new[0], new[1]):
            tile = int(L.tile_of(new[0], new[1]))
            pos = new
            cur.append(pos.copy())
            travelled += step
            continue
        if best is None:
            lo, hi = 0.0, 1.0
            for _ in range(50):
                mid = (lo + hi) / 2
                q = pos + mid * (new - pos)
                if L.inside(q[0], q[1]):
                    lo = mid
                else:
                    hi = mid
            pos = pos + lo * (new - pos)
            cur.append(pos.copy())
            stopped = pos.copy()
            travelled += lo * step
            break
        u, s = best
        cp = pos + u * (new - pos)
        cur.append(cp - 1e-9 * d)
        segs.append(cur)
        mx, my = s.g(cp[0], cp[1])
        d2 = s.R @ d
        a_mc = to_mc(L, cp[0], cp[1]) if hasattr(L, "mc_unit") else (0, 0)
        b_mc = to_mc(L, mx, my) if hasattr(L, "mc_unit") else (0, 0)
        crossings.append({"at": [round(cp[0] * L.unit_blocks, 1), round(cp[1] * L.unit_blocks, 1)],
                          "to": [round(mx * L.unit_blocks, 1), round(my * L.unit_blocks, 1)],
                          "mc_at": [int(round(a_mc[0])), int(round(a_mc[1]))], "mc_to": [int(round(b_mc[0])), int(round(b_mc[1]))],
                          "kind": s.kind, "pair": s.pair, "turn": kind_of(s.R)[1],
                          "yaw": [round(mc_yaw(d)), round(mc_yaw(d2))],
                          "dist": round((travelled + u * step) * L.unit_blocks)})
        pos = np.array([mx, my]) + 1e-10 * d2
        d = d2
        tile = int(L.tile_of(pos[0] + 1e-9 * d[0], pos[1] + 1e-9 * d[1]))
        cur = [pos.copy()]
        travelled += u * step
    segs.append(cur)
    allp = np.concatenate([np.array(sg) for sg in segs])
    xyz = L.F(allp[:, 0], allp[:, 1])
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
    sub = np.unique(np.concatenate([np.arange(0, len(lat), max(1, len(lat) // 500)), [len(lat) - 1]]))
    track = [[round(float(lat[i]), 3), round(float(lon[i]), 3)] for i in sub if np.isfinite(lat[i])]
    segs_b, segs_t = [], []
    for sg in segs:
        a = np.array(sg)
        sub2 = np.unique(np.concatenate([np.arange(0, len(a), max(1, len(a) // 80)), [len(a) - 1]]))
        segs_b.append([[round(float(a[i, 0] * L.unit_blocks), 1), round(float(a[i, 1] * L.unit_blocks), 1)] for i in sub2])
        segs_t.append([[round(float(a[i, 0]), 5), round(float(a[i, 1]), 5)] for i in sub2])
    endp = allp[-1]
    return {"segs": segs_b, "segs_t": segs_t, "crossings": crossings, "track": track, "profile": prof,
            "blocks": round(float(cum_b[-1]) if len(cum_b) else 0, 1),
            "planet_deg": round(float(np.degrees(np.nansum(arcs[:, 1]))), 2),
            "planet_blocks": round(float(np.nansum(arcs[:, 1]) * R16), 1),
            "felt_turn_deg": round(math.degrees(felt), 1),
            "yaw_start": round(mc_yaw(d0)), "yaw_end": round(mc_yaw(d)),
            "end_t": [round(float(endp[0]), 6), round(float(endp[1]), 6)],
            "close_blocks": round(float(np.linalg.norm(endp - np.array(start, float)) * L.unit_blocks), 2),
            "stopped": None if stopped is None else [round(stopped[0] * L.unit_blocks, 1), round(stopped[1] * L.unit_blocks, 1)]}


def travel_table(L, start, headings_deg, length_blocks):
    rows = []
    for hd in headings_deg:
        d = (math.cos(math.radians(hd)), math.sin(math.radians(hd)))
        w = walk(L, start, d, length_blocks / L.unit_blocks, ds=0.004)
        kinds = {"slide": 0, "half": 0, "quarter": 0}
        for c in w["crossings"]:
            kinds[c["kind"]] += 1
        per = {k: round(v * 10000 / max(w["blocks"], 1), 2) for k, v in kinds.items()}
        first = [{"kind": c["kind"], "dist": c["dist"], "yaw": c["yaw"], "mc_at": c["mc_at"], "mc_to": c["mc_to"]}
                 for c in w["crossings"][:8]]
        rows.append({"heading": hd, "blocks": w["blocks"], "per10k": per, "first": first,
                     "planet_deg": w["planet_deg"], "segs": [sg[::2] for sg in w["segs"][:60]]})
        log(L.id, "trip", hd, per)
    return rows


# ------------------------------------------------------------------------------------------
# rays through the seams (the exponential map from a point: what a player sees, walking distances)


def seam_arrays(L):
    A0 = np.array([s.a0 for s in L.seams])
    A1 = np.array([s.a1 for s in L.seams])
    NN = np.array([s.n for s in L.seams])
    RR = np.array([s.R for s in L.seams])
    TT = np.array([s.t for s in L.seams])
    return A0, A1 - A0, NN, RR, TT


def march_rays(L, start, ang, n_steps, step):
    """Straight rays from `start` (tile units) at angles `ang`, `n_steps` steps of `step` tile units,
    carried through every seam by its motion. Returns positions and headings per step (NaN once a ray
    is lost, e.g. exactly into a cone point) and the number of seams crossed so far."""
    n = len(ang)
    pos = np.tile(np.asarray(start, float), (n, 1))
    d = np.stack([np.cos(ang), np.sin(ang)], 1)
    out = np.full((n, n_steps + 1, 2), np.nan, np.float32)
    hd = np.full((n, n_steps + 1, 2), np.nan, np.float32)
    nc = np.zeros((n, n_steps + 1), np.int16)
    out[:, 0], hd[:, 0] = pos, d
    alive = np.ones(n, bool)
    cnt = np.zeros(n, np.int16)
    A0, E, NN, RR, TT = seam_arrays(L)
    for k in range(1, n_steps + 1):
        idx = np.nonzero(alive)[0]
        if not len(idx):
            break
        p, dd = pos[idx].copy(), d[idx].copy()
        new = p + step * dd
        bad = np.nonzero(~L.inside(new[:, 0], new[:, 1]))[0]
        dead = []
        for _rep in range(3):
            if not len(bad):
                break
            pb, db, nb = p[bad], dd[bad], new[bad]
            seg = nb - pb
            det = -seg[:, None, 0] * E[None, :, 1] + E[None, :, 0] * seg[:, None, 1]
            rhs = A0[None] - pb[:, None]
            with np.errstate(divide="ignore", invalid="ignore"):
                u = (-rhs[..., 0] * E[None, :, 1] + E[None, :, 0] * rhs[..., 1]) / det
                v = (seg[:, None, 0] * rhs[..., 1] - seg[:, None, 1] * rhs[..., 0]) / det
            fac = (db[:, None, 0] * NN[None, :, 0] + db[:, None, 1] * NN[None, :, 1]) > 0
            valid = np.isfinite(u) & (u >= -1e-9) & (u <= 1 + 1e-9) & (v >= -1e-7) & (v <= 1 + 1e-7) & fac
            uu = np.where(valid, u, np.inf)
            j = np.argmin(uu, 1)
            has = np.isfinite(uu[np.arange(len(bad)), j])
            dead.extend(bad[~has].tolist())
            bh, jh = bad[has], j[has]
            uh = uu[has, jh][:, None]
            cp = pb[has] + uh * seg[has]
            Rj, Tj = RR[jh], TT[jh]
            cp2 = np.einsum("nij,nj->ni", Rj, cp) + Tj
            n2 = np.einsum("nij,nj->ni", Rj, nb[has]) + Tj
            d2 = np.einsum("nij,nj->ni", Rj, db[has])
            p[bh] = cp2 + 1e-9 * d2
            new[bh] = n2
            dd[bh] = d2
            cnt[idx[bh]] += 1
            still = ~L.inside(n2[:, 0], n2[:, 1])
            bad = bh[still]
        dead.extend(bad.tolist())
        keep = np.ones(len(idx), bool)
        if dead:
            dead = np.array(sorted(set(dead)))
            alive[idx[dead]] = False
            keep[dead] = False
        ik = idx[keep]
        pos[ik], d[ik] = new[keep], dd[keep]
        out[ik, k], hd[ik, k] = new[keep], dd[keep]
        nc[ik, k] = cnt[ik]
    return out, hd, nc


def nearest_fill(A, valid, region):
    from scipy import ndimage
    if valid.all():
        return A
    _, (ii, jj) = ndimage.distance_transform_edt(~valid, return_indices=True)
    return np.where(region, A[ii, jj], np.nan)


def distance_field(L, R, start, rmax_blocks=10500.0, step_blocks=10.0, ndir=4320):
    """Walking distance from `start` to every half-resolution pixel (the shortest straight ray through the
    seams), the planet's own distance, and how far a vanilla compass is off the way back."""
    u = L.unit_blocks
    ang = TAU * (np.arange(ndir) + 0.5) / ndir
    n_steps = int(rmax_blocks / step_blocks)
    pos, hd, _ = march_rays(L, start, ang, n_steps, step_blocks / u)
    Xh, Yh, sch, okh = R["Xh"], R["Yh"], R["sch"], R["okh"]
    bx0, by0, bx1, by1 = L.bbox
    H, W = Xh.shape
    dist = np.broadcast_to(np.arange(n_steps + 1)[None, :] * step_blocks, pos.shape[:2])
    m = np.isfinite(pos[..., 0])
    px, hh, dd = pos[m], hd[m], dist[m]
    col = np.floor((px[:, 0] - bx0) * sch).astype(int)
    row = np.floor((by1 - px[:, 1]) * sch).astype(int)
    ok = (col >= 0) & (col < W) & (row >= 0) & (row < H)
    key = row[ok] * W + col[ok]
    dd, hh = dd[ok], hh[ok]
    o = np.lexsort((dd, key))
    key, dd, hh = key[o], dd[o], hh[o]
    first = np.r_[True, key[1:] != key[:-1]]
    D = np.full(H * W, np.nan)
    HX = np.full(H * W, np.nan)
    HY = np.full(H * W, np.nan)
    D[key[first]] = dd[first]
    HX[key[first]] = hh[first, 0]
    HY[key[first]] = hh[first, 1]
    D, HX, HY = D.reshape(H, W), HX.reshape(H, W), HY.reshape(H, W)
    valid = np.isfinite(D) & okh
    filled = float((okh & ~valid).sum() / okh.sum())
    D = nearest_fill(D, valid, okh)
    HX = nearest_fill(HX, valid, okh)
    HY = nearest_fill(HY, valid, okh)
    Ps = survey.normalize(L.F(np.array([start[0]]), np.array([start[1]])))[0]
    pd = np.arccos(np.clip(np.nan_to_num(R["p"]) @ Ps, -1, 1)) * R16
    cx, cy = start[0] - Xh, start[1] - Yh
    bx, by = -HX, -HY
    cerr = np.degrees(np.abs(np.arctan2(cx * by - cy * bx, cx * bx + cy * by)))
    return D, pd, cerr, filled


def dist_images(L, R, D, pd, cerr, out):
    okh = R["okh"]
    a = okh.astype(float)[..., None]
    col = palette(DIST_STOPS, np.nan_to_num(D) / 9000.0)
    stripe = (np.floor(np.nan_to_num(D) / 1000.0) % 2 == 1)
    col = np.where(stripe[..., None], col * 0.86, col)
    iso = edges_of(np.floor(np.nan_to_num(D) / 1000.0), okh)
    col[iso] = col[iso] * 0.35
    save(np.concatenate([np.clip(col, 0, 1), a], -1), f"{out}/{L.id}-dist.webp", q=88)
    G = np.zeros(okh.shape + (4,))
    G[edges_of(np.floor(np.nan_to_num(pd) / 1000.0), okh)] = [0.1, 0.35, 0.95, 1.0]
    save(G, f"{out}/{L.id}-pdist.webp", q=92)
    save(np.concatenate([palette(SEQ, np.nan_to_num(cerr) / 180.0), a], -1), f"{out}/{L.id}-compass.webp", q=88)


# ------------------------------------------------------------------------------------------
# the wrap engine's edge order (WrapLayouts, WrapTopology.fold), in continuous coordinates


class Iso:
    """CellIsometry on points: k clockwise quarter-turns seen from above (x east, z south), then a
    translation; compose(o) = self after o (as CellIsometry.compose)."""

    def __init__(self, k, tx, tz):
        self.k, self.tx, self.tz = k % 4, tx, tz

    def apply(self, x, z):
        k = self.k
        X, Z = (x, z) if k == 0 else (-z, x) if k == 1 else (-x, -z) if k == 2 else (z, -x)
        return X + self.tx, Z + self.tz

    def apply_cell(self, x, z):
        k = self.k
        X, Z = (x, z) if k == 0 else (-z - 1, x) if k == 1 else (-x - 1, -z - 1) if k == 2 else (z, -x - 1)
        return X + self.tx, Z + self.tz

    def compose(self, o):
        tx, tz = Iso(self.k, 0, 0).apply(o.tx, o.tz)
        return Iso(self.k + o.k, tx + self.tx, tz + self.tz)

    def inverse(self):
        r = Iso(-self.k, 0, 0)
        tx, tz = r.apply(-self.tx, -self.tz)
        return Iso(-self.k, tx, tz)


def quarter_about(k, a, b):
    ra, rb = Iso(k, 0, 0).apply(a, b)
    return Iso(k, a - ra, b - rb)


def half_about(a, b):
    return Iso(2, 2 * a, 2 * b)


def cube_cross_edges(s, depth, slides=True):
    """WrapLayouts.cubeCross, in the same order."""
    slideEast = Iso(0, 4 * s, 0)
    nWest = slideEast.compose(quarter_about(3, 0, 0))
    sWest = slideEast.compose(quarter_about(1, 0, s))
    north, south = Iso(2, 3 * s, -s), Iso(2, 3 * s, 3 * s)
    e = []
    if slides:
        e += [("F0-west-slide", True, True, 0, 0, s, depth, slideEast),
              ("F3-east-slide", True, False, 4 * s, 0, s, depth, slideEast.inverse())]
    e += [("N-east/F1", True, False, s, -s, 0, depth, quarter_about(1, s, 0)),
          ("F1-top/N", False, True, 0, s, 2 * s, depth, quarter_about(3, s, 0)),
          ("N-west/F3", True, True, 0, -s, 0, depth, nWest),
          ("F3-top/N", False, True, 0, 3 * s, 4 * s, depth, nWest.inverse()),
          ("N-north/F2", False, True, -s, 0, s, depth, north),
          ("F2-top/N", False, True, 0, 2 * s, 3 * s, depth, north),
          ("S-east/F1", True, False, s, s, 2 * s, depth, quarter_about(3, s, s)),
          ("F1-bottom/S", False, False, s, s, 2 * s, depth, quarter_about(1, s, s)),
          ("S-west/F3", True, True, 0, s, 2 * s, depth, sWest),
          ("F3-bottom/S", False, False, s, 3 * s, 4 * s, depth, sWest.inverse()),
          ("S-south/F2", False, False, 2 * s, 0, s, depth, south),
          ("F2-bottom/S", False, False, s, 2 * s, 3 * s, depth, south)]
    return e


def guyou_edges(h, depth, slides=True):
    """WrapLayouts.guyou, in the same order."""
    hz = h / 2 if isinstance(h, float) else h // 2
    e = []
    if slides:
        e += [("west-slide", True, True, -h, -hz - depth, hz + depth, depth, Iso(0, 2 * h, 0)),
              ("east-slide", True, False, h, -hz - depth, hz + depth, depth, Iso(0, -2 * h, 0))]
    e += [("north-half-turn", False, True, -hz, -h, h, depth, half_about(0, -hz)),
          ("south-half-turn", False, False, hz, -h, h, depth, half_about(0, hz))]
    return e


def beyond(e, x, z):
    _, normalX, minSide, coord, lo, hi, depth, _ = e
    v, u = (x, z) if normalX else (z, x)
    ins = (u >= lo) & (u < hi)
    return ins & ((v < coord) & (v >= coord - depth) if minSide else (v >= coord) & (v < coord + depth))


def engine_fold(edges, x, z, max_steps=8):
    """WrapTopology.fold on points: the first edge (in table order) the point is beyond moves it, until it
    is beyond none. Returns the folded point, the first edge used (-1: none) and whether it closed."""
    x = np.array(x, float)
    z = np.array(z, float)
    first = np.full(x.shape, -1)
    for step in range(max_steps):
        hit = np.full(x.shape, -1)
        for ei, e in enumerate(edges):
            m = (hit < 0) & beyond(e, x, z)
            hit[m] = ei
        if not (hit >= 0).any():
            return x, z, first, np.ones(x.shape, bool)
        for ei in np.unique(hit[hit >= 0]):
            m = hit == ei
            x[m], z[m] = edges[ei][7].apply(x[m], z[m])
            if step == 0:
                first[m] = ei
    hit = np.zeros(x.shape, bool)
    for e in edges:
        hit |= beyond(e, x, z)
    return x, z, first, ~hit


def engine_conv(L):
    """(tile -> engine units, engine units -> tile); engine units are s (cube) or h/2 (Guyou)."""
    if L.cls == "cube":
        return (lambda X, Y: (X, 1 - Y)), (lambda x, z: (x, 1 - z))
    return (lambda X, Y: (X - 1, -Y)), (lambda x, z: (x + 1, -z))


def engine_edges_unit(L, depth, slides=True):
    return cube_cross_edges(1.0, depth, slides) if L.cls == "cube" else guyou_edges(2.0, depth, slides)


def engine_check(L):
    """The engine's edge table agrees with the survey's seams: just beyond every edge, the engine's fold and
    the survey's continuation through its seams are the same planet point."""
    t2e, e2t = engine_conv(L)
    worst = 0.0
    edges = engine_edges_unit(L, 0.05)
    for e in edges:
        _, normalX, minSide, coord, lo, hi, depth, iso = e
        uu = lo + (hi - lo) * np.linspace(0.05, 0.95, 40)
        vv = coord - 0.01 if minSide else coord + 0.01
        x, z = (np.full_like(uu, vv), uu) if normalX else (uu, np.full_like(uu, vv))
        fx, fz, _, ok = engine_fold(edges, x, z)
        X, Y = e2t(fx, fz)
        Xo, Yo = e2t(x, z)
        a = L.F(X, Y)
        b = L.Fx(Xo, Yo, depth=0.05)
        worst = max(worst, float(np.nanmax(np.linalg.norm(a - b, axis=-1))))
    return worst


# ------------------------------------------------------------------------------------------
# windows round a corner


def window_grid(cx, cy, half, npx):
    xs = cx - half + (np.arange(npx) + 0.5) * 2 * half / npx
    ys = cy + half - (np.arange(npx) + 0.5) * 2 * half / npx
    return np.meshgrid(xs, ys)


def shade_rgba(planet, P, ok, px_blocks):
    S = planet.sample(np.where(ok[..., None], P, np.nan))
    shade = hillshade(np.where(ok, S[..., 0], np.nan), px_blocks)
    rgb = colorize(np.nan_to_num(S), shade)
    return np.concatenate([rgb, ok.astype(float)[..., None]], -1)


def fold_plane_guyou(X, Y):
    """Guyou's p2 wallpaper group: any plane point to the fundamental domain [-1, 3) x [-1, 1] (the slide
    by 4 and the half-turns about (1, +-1) generate it)."""
    X = np.mod(np.asarray(X, float) + 1, 4) - 1
    Y = np.mod(np.asarray(Y, float) + 1, 4) - 1
    turned = Y > 1
    X = np.where(turned, 2 - X, X)
    Y = np.where(turned, 2 - Y, Y)
    X = np.mod(X + 1, 4) - 1
    return X, Y, turned


def plane_jumps(P, pxb):
    """Largest planet step between neighbouring pixels, and pixels whose step is a tear (> 6 pixels)."""
    Pn = np.nan_to_num(P)
    ok = np.isfinite(P[..., 0])
    jump = np.zeros(ok.shape)
    for dr, dc in ((0, 1), (1, 0)):
        nb = np.roll(Pn, (-dr, -dc), axis=(0, 1))
        okn = np.roll(ok, (-dr, -dc), axis=(0, 1))
        jj = np.arccos(np.clip((Pn * nb).sum(-1), -1, 1)) * R16
        jj[-1, :] = 0
        jj[:, -1] = 0
        jump = np.maximum(jump, np.where(ok & okn, jj, 0))
    return jump, jump > 6 * pxb


def coast_lambda(L_at, Vt, bis_deg, planet):
    """The turn that puts a coast through the corner Vt (land share near 0.5 within 5 degrees, land nearby),
    for the corner close-ups."""
    nud = 1e-6 * np.array([math.cos(math.radians(bis_deg)), math.sin(math.radians(bis_deg))])
    L0 = L_at(0.0)
    P0 = survey.normalize(L0.F(np.array([Vt[0] + nud[0]]), np.array([Vt[1] + nud[1]])))[0]
    best = None
    for lam in np.arange(0, 360, 1.0):
        pv = zrot(math.radians(lam)) @ P0
        s5 = float(planet.land(disc_points(pv, math.radians(5), 160)).mean())
        s15 = float(planet.land(disc_points(pv, math.radians(15), 300)).mean())
        score = abs(s5 - 0.5) + 0.5 * max(0.0, 0.3 - s15)
        if best is None or score < best[0]:
            best = (score, lam, s5, s15)
    return best[1], best[2], best[3]


def corner_study(L, Vt, cone_deg, bis_deg, planet, out, tag, half_blocks=1500.0, npx=None, d_blocks=450.0):
    """A corner up close: the faces round it (missing wedge transparent), the single raw copy the engine's
    edge order puts in the missing wedge, what a player d blocks away looking at it sees, walkers, a ring."""
    npx = npx or int(600 * PXS)
    u = L.unit_blocks
    half = half_blocks / u
    cx, cy = Vt
    X, Y = window_grid(cx, cy, half, npx)
    px_blocks = 2 * half_blocks / npx
    ins = L.inside(X, Y)
    P = L.F(X, Y)
    ok = np.isfinite(P[..., 0]) & ins
    save(shade_rgba(planet, P, ok, px_blocks), f"{out}/corner-{tag}-nbhd.webp", q=86)
    # the single raw copy (engine's edge order, deep bands)
    t2e, e2t = engine_conv(L)
    ex, ez = t2e(X, Y)
    edges = engine_edges_unit(L, 1.0)
    fx, fz, first, closed = engine_fold(edges, ex, ez)
    FX, FY = e2t(fx, fz)
    okr = closed & L.inside(FX, FY)
    Pr = L.F(np.where(okr, FX, 0), np.where(okr, FY, 0))
    Pr[~okr] = np.nan
    save(shade_rgba(planet, Pr, okr, px_blocks), f"{out}/corner-{tag}-raw.webp", q=86)
    src = {}
    outside = okr & ~ins
    for ei in np.unique(first[(first >= 0) & outside]):
        src[edges[ei][0]] = round(float(((first == ei) & outside).sum() / max(1, outside.sum())), 3)
    _, tear = plane_jumps(Pr, px_blocks)
    T = np.zeros(okr.shape + (4,))
    T[tear] = [1.0, 0.15, 0.1, 1.0]
    save(T, f"{out}/corner-{tag}-rawtear.webp", q=92)
    tear_px = int(tear[2:-2, 2:-2].sum())
    # what a player sees: rays from P, d blocks from the corner on the bisector, looking at it
    bis = np.array([math.cos(math.radians(bis_deg)), math.sin(math.radians(bis_deg))])
    P0 = np.array(Vt, float) + bis * d_blocks / u
    up = -bis
    right = np.array([up[1], -up[0]])
    vb0, vb1, va = -900.0, 2100.0, 1500.0
    ndir, step = int(4000 * PXS), 4.0 / max(PXS, 0.25)
    rmax = math.hypot(va, vb1) + 10
    ang = TAU * np.arange(ndir) / ndir
    pos, _, _ = march_rays(L, P0, ang, int(rmax / step), step / u)
    A_ = -va + (np.arange(npx) + 0.5) * 2 * va / npx
    B_ = vb1 - (np.arange(npx) + 0.5) * (vb1 - vb0) / npx
    AA, BB = np.meshgrid(A_, B_)
    dx = AA * right[0] + BB * up[0]
    dy = AA * right[1] + BB * up[1]
    phi = np.mod(np.arctan2(dy, dx), TAU)
    rr = np.hypot(dx, dy)
    j = np.mod(np.round(phi / TAU * ndir).astype(int), ndir)
    k = np.clip(np.round(rr / step).astype(int), 0, pos.shape[1] - 1)
    W_ = pos[j, k].astype(float)
    okv = np.isfinite(W_[..., 0])
    Pv = L.F(np.where(okv, W_[..., 0], 0), np.where(okv, W_[..., 1], 0))
    Pv[~okv] = np.nan
    okv &= np.isfinite(Pv[..., 0])
    save(shade_rgba(planet, Pv, okv, (vb1 - vb0) / npx), f"{out}/corner-{tag}-view.webp", q=86)
    th = math.radians(cone_deg)
    qa, qb = AA, BB - d_blocks
    psi = np.arctan2(qa, -qb)  # angle round V from the direction V -> P
    apsi = np.abs(psi)
    Tn = np.zeros(okv.shape + (4,))
    second = apsi > th / 2
    dup = (apsi > th - math.pi) & ~second
    Tn[second] = [0.15, 0.45, 1.0, 0.34]
    Tn[dup] = [1.0, 0.62, 0.1, 0.18]
    save(Tn, f"{out}/corner-{tag}-viewtint.webp", q=90)
    # check: every second copy shows the same planet point as its first image (psi -+ theta, same radius)
    rng = np.random.default_rng(3)
    cand = np.argwhere(second & okv)
    errs = []
    if len(cand):
        for (r_, c_) in cand[rng.choice(len(cand), min(3000, len(cand)), replace=False)]:
            ps = psi[r_, c_]
            ps2 = ps - math.copysign(th, ps)
            rad = math.hypot(qa[r_, c_], qb[r_, c_])
            a2, b2 = rad * math.sin(ps2), d_blocks - rad * math.cos(ps2)
            if not (-va <= a2 <= va and vb0 <= b2 <= vb1):
                continue
            c2 = int((a2 + va) / (2 * va) * npx)
            r2 = int((vb1 - b2) / (vb1 - vb0) * npx)
            if 0 <= r2 < npx and 0 <= c2 < npx and okv[r2, c2]:
                errs.append(float(np.arccos(np.clip(Pv[r_, c_] @ Pv[r2, c2], -1, 1)) * R16))
    # the view's own tear: neighbouring pixels far apart on the planet (behind a cube corner)
    _, vtear = plane_jumps(Pv, (vb1 - vb0) / npx)
    view = {"window_ab": [-va, vb0, va, vb1], "d": d_blocks, "npx": npx, "lost_px": int((~okv).sum()),
            "tear_px": int(vtear[2:-2, 2:-2].sum()),
            "dup_check_blocks": [round(float(np.median(errs)), 2), round(float(np.percentile(errs, 95)), 2), len(errs)] if errs else None,
            "second_share": round(float(second.mean()), 3)}
    # walkers either side, a ring round the corner (V on the walker's left)
    off, back = 300.0 / u, 1000.0 / u
    wk = {"A": walk(L, (cx - off, cy - back), (0, 1), 2600 / u, ds=0.0006),
          "B": walk(L, (cx + off, cy - back), (0, 1), 2600 / u, ds=0.0006)}
    rho = 600.0 / u
    a0 = math.radians(bis_deg)
    ring = walk(L, (cx + rho * math.cos(a0), cy + rho * math.sin(a0)), (-math.sin(a0), math.cos(a0)), th * rho,
                ds=0.0004, curv=1.0 / rho)
    log(tag, "ring blocks", ring["blocks"], "planet", ring["planet_blocks"], "close", ring["close_blocks"],
        "felt", ring["felt_turn_deg"], "view", view)
    return {"V": list(Vt), "window_t": [cx - half, cy - half, cx + half, cy + half], "half_blocks": half_blocks,
            "npx": npx, "raw_sources": src, "raw_tear_px": tear_px, "view": view, "P_t": P0.tolist(), "up": up.tolist(),
            "walkers": wk, "ring": ring, "rho_blocks": 600.0, "cone": cone_deg}


def block_grid(kind):
    """Block cells round a corner with the engine's integer edge table at a production size: which stored
    cell each band cell shows (every claiming edge; the first wins), and the neighbour relations of the
    cells touching the corner."""
    if kind == "cube":
        s = 3776
        edges = cube_cross_edges(s, 64, True)
        V = (s, 0)

        def inside(x, z):
            if 0 <= z < s and 0 <= x < 4 * s:
                return f"F{x // s}"
            if 0 <= x < s and -s <= z < 0:
                return "N"
            if 0 <= x < s and s <= z < 2 * s:
                return "S"
            return None
    else:
        h = 6528
        edges = guyou_edges(h, 64, True)
        V = (0, -h // 2)

        def inside(x, z):
            if -h <= x < h and -h // 2 <= z < h // 2:
                return "W" if x < 0 else "E"
            return None

    def claims(x, z):
        out = []
        for e in edges:
            if bool(beyond(e, np.array(x), np.array(z))):
                fx, fz = e[7].apply_cell(x, z)
                out.append({"edge": e[0], "to": [int(fx), int(fz)], "k": e[7].k, "face": inside(int(fx), int(fz))})
        return out

    def fold(x, z):
        for _ in range(8):
            if inside(x, z):
                return x, z
            c = claims(x, z)
            if not c:
                return None
            x, z = c[0]["to"]
        return None

    cells = []
    for dz in range(-4, 4):
        for dx in range(-4, 4):
            x, z = V[0] + dx, V[1] + dz
            f = inside(x, z)
            cells.append({"x": x, "z": z, "face": f, "claims": [] if f else claims(x, z)})
    dirs = {"N": (0, -1), "NE": (1, -1), "E": (1, 0), "SE": (1, 1), "S": (0, 1), "SW": (-1, 1), "W": (-1, 0), "NW": (-1, -1)}
    touching = []
    for (dx, dz) in ((-1, -1), (0, -1), (-1, 0), (0, 0)):
        x, z = V[0] + dx, V[1] + dz
        if not inside(x, z):
            continue
        nb = {}
        for nm, (ddx, ddz) in dirs.items():
            nx_, nz_ = x + ddx, z + ddz
            cl = [] if inside(nx_, nz_) else claims(nx_, nz_)
            f = fold(nx_, nz_)
            nb[nm] = {"raw": [nx_, nz_], "stored": None if f is None else [int(f[0]), int(f[1])],
                      "n_claims": len(cl), "alt": [c["to"] for c in cl[1:]]}
        notes = []
        for nm, v in nb.items():
            if v["stored"] == [x, z]:
                notes.append(f"its {nm} neighbour is itself")
            if v["n_claims"] > 1 and v["stored"] is not None and len({tuple(a) for a in v["alt"]} | {tuple(v["stored"])}) > 1:
                notes.append(f"its {nm} neighbour cell is claimed by {v['n_claims']} edges that disagree")
        seen = {}
        for nm, v in nb.items():
            if v["stored"] is not None:
                seen.setdefault(tuple(v["stored"]), []).append(nm)
        for k_, v in seen.items():
            if len(v) > 1:
                notes.append(f"its {' and '.join(v)} neighbours are the same block ({k_[0]}, {k_[1]})")
        touching.append({"x": x, "z": z, "face": inside(x, z), "nb": nb, "notes": notes})
    return {"V": list(V), "cells": cells, "touching": touching, "size": 3776 if kind == "cube" else 6528}


def tiling_images(Lg, Lc, planet, out):
    """The plane tiled by Guyou copies; the Guyou corner's full turn (two copies, consistent); the cube
    corner's missing quarter with its two candidates; the cube net's raw surroundings under the engine."""
    res = {}
    X0, X1, Y0, Y1, W, H = -5.0, 7.0, -3.0, 3.0, int(1600 * PXS), int(800 * PXS)
    xs = X0 + (np.arange(W) + 0.5) * (X1 - X0) / W
    ys = Y1 - (np.arange(H) + 0.5) * (Y1 - Y0) / H
    X, Y = np.meshgrid(xs, ys)
    FX, FY, _ = fold_plane_guyou(X, Y)
    P = Lg.F(FX, FY)
    ok = np.isfinite(P[..., 0])
    pxb = (X1 - X0) / W * Lg.unit_blocks
    save(shade_rgba(planet, P, ok, pxb), f"{out}/tiling-guyou.webp", q=84)
    jump, tear = plane_jumps(P, pxb)
    res["guyou"] = {"window_t": [X0, Y0, X1, Y1], "px": [W, H], "px_blocks": round(pxb, 1),
                    "max_step_blocks": round(float(jump.max()), 1), "tear_px": int(tear.sum())}
    NC = int(600 * PXS)
    X, Y = window_grid(1.0, 1.0, 1.0, NC)
    FX, FY, _ = fold_plane_guyou(X, Y)
    P = Lg.F(FX, FY)
    pxb = 2 / NC * Lg.unit_blocks
    save(shade_rgba(planet, P, np.isfinite(P[..., 0]), pxb), f"{out}/tiling-guyou-corner.webp", q=86)
    jump, tear = plane_jumps(P, pxb)
    res["guyou_corner"] = {"window_t": [0.0, 0.0, 2.0, 2.0], "tear_px": int(tear.sum())}
    # cube corner (1, 1): three faces and the missing quarter's two candidates
    X, Y = window_grid(1.0, 1.0, 1.0, NC)
    q = (X > 1) & (Y > 1)
    ins = Lc.inside(X, Y) & ~q
    XA, YA = Y, 2 - X           # through N's east edge: face 1, turned
    XB, YB = 2 - Y, X           # through face 1's top edge: N, turned
    PA = Lc.F(np.where(q, XA, X), np.where(q, YA, Y))
    PB = Lc.F(np.where(q, XB, X), np.where(q, YB, Y))
    pxb = 2 / NC * Lc.unit_blocks
    imA = shade_rgba(planet, PA, np.isfinite(PA[..., 0]) & (ins | q), pxb)
    imB = shade_rgba(planet, PB, np.isfinite(PB[..., 0]) & (ins | q), pxb)
    rr, cc = np.mgrid[0:NC, 0:NC]
    stripe = ((rr + cc) // max(2, int(22 * PXS))) % 2 == 0
    mix = np.where((q & stripe)[..., None], imA, np.where(q[..., None], imB, imA))
    save(mix, f"{out}/tiling-cube-mix.webp", q=86)
    for nm, im in (("a", imA), ("b", imB)):
        im2 = im.copy()
        im2[~q, 3] = 0
        save(im2, f"{out}/tiling-cube-{nm}.webp", q=86)
    _, tA = plane_jumps(np.where((ins | q)[..., None], PA, np.nan), pxb)
    _, tB = plane_jumps(np.where((ins | q)[..., None], PB, np.nan), pxb)
    res["cube_corner"] = {"window_t": [0.0, 0.0, 2.0, 2.0], "tear_px_a": int(tA.sum()), "tear_px_b": int(tB.sum())}
    # the cube net and its raw surroundings under the engine's edge order (deep bands)
    X0, X1, Y0, Y1, W, H = -1.0, 5.0, -2.0, 3.0, int(1500 * PXS), int(1250 * PXS)
    xs = X0 + (np.arange(W) + 0.5) * (X1 - X0) / W
    ys = Y1 - (np.arange(H) + 0.5) * (Y1 - Y0) / H
    X, Y = np.meshgrid(xs, ys)
    t2e, e2t = engine_conv(Lc)
    ex, ez = t2e(X, Y)
    fx, fz, first, closed = engine_fold(engine_edges_unit(Lc, 1.5), ex, ez)
    FX, FY = e2t(fx, fz)
    ok = closed & Lc.inside(FX, FY)
    P = Lc.F(np.where(ok, FX, 0), np.where(ok, FY, 0))
    P[~ok] = np.nan
    pxb = (X1 - X0) / W * Lc.unit_blocks
    save(shade_rgba(planet, P, ok, pxb), f"{out}/rawext-cube.webp", q=84)
    _, tear = plane_jumps(P, pxb)
    from scipy import ndimage
    T = np.zeros(ok.shape + (4,))
    T[ndimage.binary_dilation(tear, iterations=1)] = [1.0, 0.1, 0.1, 1.0]
    save(T, f"{out}/rawext-cube-tear.webp", q=92)
    res["cube_rawext"] = {"window_t": [X0, Y0, X1, Y1], "px": [W, H], "tear_px": int(tear.sum()),
                          "covered": round(float(ok.mean()), 3), "px_blocks": round(pxb, 1)}
    return res


# ------------------------------------------------------------------------------------------
# globe overlays, meshes, seams, self-check


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
    if L.fold is None:
        return None
    n = L.mesh_cells
    tiles = []
    for (x0, y0, w, h, _) in L.tiles:
        xs = x0 + np.linspace(0, 1, n + 1) * w
        ys = y0 + np.linspace(0, 1, n + 1) * h
        X, Y = np.meshgrid(xs, ys)
        eps = 1e-6 * max(w, h)
        P = survey.normalize(np.nan_to_num(L.F(np.clip(X, x0 + eps, x0 + w - eps), np.clip(Y, y0 + eps, y0 + h - eps))))
        q = np.clip(np.round(P * 32767), -32767, 32767).astype("<i2")
        tiles.append({"x0": x0, "y0": y0, "w": w, "h": h, "nx": n, "ny": n, "xyz": base64.b64encode(q.tobytes()).decode()})
    hinges = [{"parent": a, "child": b, "p0": list(map(float, p0)), "p1": list(map(float, p1))} for (a, b, p0, p1) in L.hinges]
    return {"tiles": tiles, "hinges": hinges, "fold": L.fold, "radius": math.sqrt(L.area / (4 * math.pi))}


def seams_json(L):
    return [{"pair": s.pair, "kind": s.kind, "side": s.partner,
             "a0": [round(float(v) * L.unit_blocks, 2) for v in s.a0],
             "a1": [round(float(v) * L.unit_blocks, 2) for v in s.a1],
             "turn": kind_of(s.R)[1]} for s in L.seams]


def self_check(L):
    worst = 0.0
    t = np.linspace(0.02, 0.98, 50)
    for s in L.seams:
        P = s.a0[None, :] + t[:, None] * (s.a1 - s.a0)[None, :]
        inside = L.F(P[:, 0] - 1e-9 * s.n[0], P[:, 1] - 1e-9 * s.n[1])
        gx, gy = s.g(P[:, 0] + 1e-9 * s.n[0], P[:, 1] + 1e-9 * s.n[1])
        other = L.F(gx, gy)
        worst = max(worst, float(np.nanmax(np.linalg.norm(inside - other, axis=-1))))
    return worst


# ------------------------------------------------------------------------------------------
# corner placement, spawn


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
        shares = [float(planet.land(disc_points(c, rad)).mean()) for c in corners_fn(lam)]
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
    out = []
    for zs in (1, -1):
        for dl in (math.pi / 2, -math.pi / 2):
            a = lam + dl
            out.append(np.array([math.cos(a) * math.cos(math.pi / 4), math.sin(a) * math.cos(math.pi / 4),
                                 zs * math.sin(math.pi / 4)]))
    return out


def seam_dist(L, X, Y):
    best = np.inf
    for s in L.seams:
        d = s.a1 - s.a0
        t = min(1.0, max(0.0, ((X - s.a0[0]) * d[0] + (Y - s.a0[1]) * d[1]) / (d @ d)))
        best = min(best, math.hypot(X - s.a0[0] - t * d[0], Y - s.a0[1] - t * d[1]))
    return best * L.unit_blocks


def corner_dist(L, X, Y):
    return min(math.hypot(X - cx, Y - cy) for (cx, cy, _, _) in L.corners) * L.unit_blocks


def pick_spawn(planet, mains, R):
    """One planet point for both worlds: land within 25 degrees of the equator, as far from every seam and
    corner of both worlds as possible."""
    from scipy.spatial import cKDTree
    look = {}
    for L in mains:
        r = R[L.id]
        m = r["okh"]
        look[L.id] = (cKDTree(r["p"][m]), np.stack([r["Xh"][m], r["Yh"][m]], 1))
    best = None
    for la in np.arange(-25, 25.1, 1.0):
        for lo in np.arange(-180, 180, 2.0):
            q = survey.ll2xyz(np.radians(la), np.radians(lo))
            if not planet.land(q[None, :])[0]:
                continue
            score, where = np.inf, {}
            for L in mains:
                tree, XY = look[L.id]
                _, i = tree.query(q)
                X, Y = XY[i]
                where[L.id] = (float(X), float(Y))
                score = min(score, seam_dist(L, X, Y), corner_dist(L, X, Y))
            if best is None or score > best[0]:
                best = (score, float(la), float(lo), where)
    return best


# ------------------------------------------------------------------------------------------


def main():
    hm, out = sys.argv[1], sys.argv[2]
    cache = sys.argv[3] if len(sys.argv) > 3 else out
    os.makedirs(out, exist_ok=True)
    planet = Planet(hm)

    sw_cube = sweep(planet, cube_corner_dirs, np.radians(np.arange(0, 90, 1.0)))
    sw_guyou = sweep(planet, guyou_corner_dirs, np.radians(np.arange(0, 360, 2.0)))
    best_cube = min(sw_cube, key=lambda r: (r[1], r[2]))
    best_guyou = min(sw_guyou, key=lambda r: (r[1], r[2]))
    log("best cube lam0", best_cube, "best guyou lamc", best_guyou)
    lam0 = math.radians(best_cube[0])
    lamg = math.radians(best_guyou[0])

    mpath = os.path.join(cache, "moser-cube-512.npz")
    if os.path.exists(mpath):
        z = np.load(mpath)
        s_, G, G0 = z["s"], z["G"], z["G0"]
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

    def mk_guyou(lam, lid="guyou-petroff"):
        L = guyou(lid, survey.hemi_petroff, math.radians(lam))
        L.mc_unit = 6544 / 2
        name_seams(L, ["W square", "E square"])
        return L

    def mk_cube(lam, lid="cube-moser-cross"):
        L = cube_net(lid, CROSS, face_moser, math.radians(lam))
        L.mc_unit = 3776
        name_seams(L, ["F0", "F1", "F2", "F3", "N", "S"])
        return L

    Lg, Lc = mk_guyou(best_guyou[0]), mk_cube(best_cube[0])
    mains = [Lg, Lc]
    only = os.environ.get("ONLY", "")
    data = {"generated": time.strftime("%Y-%m-%d %H:%M UTC", time.gmtime()), "R16": R16,
            "lam0_deg": best_cube[0], "lamc_deg": best_guyou[0],
            "sweep_cube": sw_cube, "sweep_guyou": sw_guyou, "layouts": {}, "stats": {}}
    R = {}
    for L in mains:
        chk = self_check(L)
        ech = engine_check(L)
        log(L.id, "seam closure", chk, "engine table vs survey seams", ech)
        info, R[L.id] = render_layout(L, planet, out, px=int(1400 * PXS), hpx=int(700 * PXS))
        d = {"cls": L.cls, "unit_blocks": L.unit_blocks, "mc_unit": L.mc_unit,
             "bbox": [v * L.unit_blocks for v in L.bbox],
             "tiles": [[t[0] * L.unit_blocks, t[1] * L.unit_blocks, t[2] * L.unit_blocks, t[3] * L.unit_blocks] for t in L.tiles],
             "seams": seams_json(L), "seam_closure_rad": chk, "engine_vs_survey_rad": ech, "seam_mc": seam_mc(L),
             "internal": [[[float(v) * L.unit_blocks for v in a0], [float(v) * L.unit_blocks for v in a1]] for (a0, a1) in L.internal],
             "corners": [[X * L.unit_blocks, Y * L.unit_blocks, v, c] for (X, Y, v, c) in L.corners],
             "img": info, "tissot": tissot(L, 3 if L.cls == "cube" else 4), "overlay": overlays(L), "mesh": mesh(L)}
        st = distortion_stats(L, R[L.id])
        y0 = 0.5 if L.cls == "cube" else 0.0
        st["bow"] = [parallel_bow(L, R[L.id], la, y0, 1.0) for la in (15.0, 30.0)]
        data["stats"][L.id] = st
        log(L.id, "stats", st)
        data["layouts"][L.id] = d

    # a common spawn, walking distances, compass
    sp = pick_spawn(planet, mains, R)
    log("spawn", sp[:3], sp[3])
    data["spawn"] = {"lat": sp[1], "lon": sp[2], "clearance_blocks": round(sp[0])}
    for L in mains:
        S0 = sp[3][L.id]
        D, pd, cerr, filled = distance_field(L, R[L.id], S0, step_blocks=10.0 / max(PXS, 0.1) ** 0.5, ndir=int(4320 * PXS))
        dist_images(L, R[L.id], D, pd, cerr, out)
        okh, w = R[L.id]["okh"], R[L.id]["w"]
        m = okh & np.isfinite(D) & (pd > 300)
        ratio = D / np.maximum(pd, 1)
        data["stats"][L.id]["dist"] = {
            "spawn_t": [round(S0[0], 5), round(S0[1], 5)], "spawn_mc": [int(round(v)) for v in to_mc(L, S0[0], S0[1])],
            "spawn_blocks": [round(S0[0] * L.unit_blocks, 1), round(S0[1] * L.unit_blocks, 1)],
            "max_walk": round(float(np.nanmax(np.where(okh, D, np.nan)))),
            "q50_walk": round(wq(D[okh], w[okh], [0.5])[0]),
            "ratio_q10_50_90": [round(v, 3) for v in wq(ratio[m], w[m], [0.1, 0.5, 0.9])],
            "ratio_gt_1_2": round(float(w[m & (ratio > 1.2)].sum() / w[m].sum()), 4),
            "ratio_lt_0_85": round(float(w[m & (ratio < 0.85)].sum() / w[m].sum()), 4),
            "compass_gt_45": round(float(w[m & (cerr > 45)].sum() / w[m].sum()), 4),
            "compass_gt_90": round(float(w[m & (cerr > 90)].sum() / w[m].sum()), 4),
            "compass_q50": round(wq(cerr[m], w[m], [0.5])[0], 1),
            "filled_px_share": round(filled, 4), "planet_max": round(math.pi * R16)}
        log(L.id, "dist", data["stats"][L.id]["dist"])

    # walks and straight trips on the finalists
    for L in mains:
        W = {}
        S0 = sp[3][L.id]
        if L.cls == "cube":
            W["equator"] = walk(L, (1e-6, 0.5), (1, 0), 4.0)
            W["pole"] = walk(L, (0.5, 0.5), (0, 1), 2.0)
            W["poleB"] = walk(L, (1.5, 0.5), (0, 1), 2.0)
            W["cornerA"] = walk(L, (0.9, 0.5), (0, 1), 1.6)
            W["cornerB"] = walk(L, (1.1, 0.5), (0, 1), 1.6)
        else:
            W["equator"] = walk(L, (-1 + 1e-6, 0.0), (1, 0), 4.0)
            W["pole"] = walk(L, (0.0, 0.0), (0, 1), 2.0)
            W["poleB"] = walk(L, (0.5, 0.0), (0, 1), 2.0)
            W["cornerA"] = walk(L, (0.9, 0.0), (0, 1), 1.6)
            W["cornerB"] = walk(L, (1.1, 0.0), (0, 1), 1.6)
        W["diag"] = walk(L, S0, (math.sqrt(0.5), math.sqrt(0.5)), 24000 / L.unit_blocks, ds=0.003)
        data["layouts"][L.id]["walks"] = W
        data["layouts"][L.id]["trips"] = travel_table(L, S0, [0.0, 15.0, 30.0, 45.0, 60.0, 90.0], 60000.0)
        log(L.id, "walks", {k: (v["blocks"], v["planet_deg"], len(v["crossings"])) for k, v in W.items()})

    # corners up close (turned so a coast runs past the corner), and the same corner on the chosen worlds
    data["corners"] = {}
    for L, mk, Vt, cone, bis in ((Lg, mk_guyou, (1.0, 1.0), 180.0, 270.0), (Lc, mk_cube, (1.0, 1.0), 270.0, 225.0)):
        lam, s5, s15 = coast_lambda(lambda lm: mk(lm, L.id + "-c"), Vt, bis, planet)
        log(L.id, "coast lambda", lam, s5, s15)
        Lk = mk(lam, L.id + "-c")
        cs = corner_study(Lk, Vt, cone, bis, planet, out, L.id)
        cs.update({"lambda": lam, "land5": round(s5, 3), "land15": round(s15, 3)})
        nud = 1e-6 * np.array([math.cos(math.radians(bis)), math.sin(math.radians(bis))])
        pv = survey.normalize(L.F(np.array([Vt[0] + nud[0]]), np.array([Vt[1] + nud[1]])))[0]
        cs["globe_ll"] = latlon([pv])[0]
        u = L.unit_blocks
        rho = 600.0 / u
        a0 = math.radians(bis)
        cs["globe_ring"] = walk(L, (Vt[0] + rho * math.cos(a0), Vt[1] + rho * math.sin(a0)), (-math.sin(a0), math.cos(a0)),
                                math.radians(cone) * rho, ds=0.0004, curv=1.0 / rho)["track"]
        cs["globe_A"] = walk(L, (Vt[0] - 300 / u, Vt[1] - 1000 / u), (0, 1), 2600 / u, ds=0.0006)["track"]
        cs["globe_B"] = walk(L, (Vt[0] + 300 / u, Vt[1] - 1000 / u), (0, 1), 2600 / u, ds=0.0006)["track"]
        cs["blocks"] = block_grid(L.cls)
        data["corners"][L.id] = cs

    data["tiling"] = tiling_images(Lg, Lc, planet, out)
    log("tiling", data["tiling"])

    # the rest, thumbnails for the appendix
    data["appendix"] = {}
    if only != "main":
        appx = [lambert("lambert"), guyou("guyou-peirce", survey.hemi_peirce, lamg), peirce_square("peirce-square", survey.hemi_peirce),
                guyou("guyou-gringorten", survey.hemi_gringorten, lamg), guyou("guyou-collignon", survey.hemi_collignon, lamg),
                cube_net("cube-moser-3x2", NET32, face_moser, lam0), cube_net("cube-qsc-cross", CROSS, survey.face_rosca, lam0),
                cube_net("cube-healpix-cross", CROSS, None, lam0, tile_fns=healpix_tile_fns()),
                cube_net("cube-eac-cross", CROSS, survey.face_eac, lam0), cube_net("cube-conformal-cross", CROSS, face_conf, lam0)]
        for L in appx:
            info, _ = render_layout(L, planet, out, px=int(640 * PXS), full=False)
            data["appendix"][L.id] = {"cls": L.cls, "bbox": [v * L.unit_blocks for v in L.bbox], "img": info,
                                      "seams": seams_json(L), "corners": [[X * L.unit_blocks, Y * L.unit_blocks, v, c] for (X, Y, v, c) in L.corners],
                                      "seam_closure_rad": self_check(L)}

    log("globe texture")
    nlon, nlat = 2048, 1024
    lon = (np.arange(nlon) + 0.5) / nlon * TAU - math.pi
    lat = math.pi / 2 - (np.arange(nlat) + 0.5) / nlat * math.pi
    LO, LA = np.meshgrid(lon, lat)
    P = survey.ll2xyz(LA, LO)
    S = planet.sample(P)
    shade = hillshade(S[..., 0], TAU * R16 / nlon, cosfix=np.cos(LA))
    save(colorize(np.nan_to_num(S), shade), f"{out}/globe.webp", q=85)
    with open(f"{out}/data.js", "w") as f:
        f.write("window.WS_DATA = ")
        json.dump(data, f, separators=(",", ":"))
        f.write(";\n")
    log("done", os.path.getsize(f"{out}/data.js"), "bytes of data.js")


if __name__ == "__main__":
    main()
