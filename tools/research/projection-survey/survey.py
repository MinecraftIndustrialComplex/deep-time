#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Distortion, seam and crease measurements for seamless world projections.

Research tool for docs/research/notes/seamless-projection-survey.md (2026-10-03). Not shipped.

Every candidate is a *world*: a set of rectangular tiles in a global (x, y) plane, an inverse map
F(x, y) -> unit vector on the planet, and a table of seams (a world edge, the isometry that glues the
outside of it to the inside). From that alone the script measures:

  * area distortion: blocks per unit planet area relative to the mean, planet-area-weighted quantiles;
  * angular distortion: the shape ratio a/b (singular values of dF) and 2*omega = 2 asin((a-b)/(a+b));
  * continuity across every seam (|F(p) - F(g(p))|, which must be 0: checks the seam table);
  * creases: across every seam, tile edge, tile midline and tile diagonal, the one-sided Jacobians
    J- and J+ are compared. eps = ||pinv(J-) J+ - I|| is the relative jump of the derivative (0 for a
    map that is C1 there); "bend" is the largest angle by which a straight world line is bent on the
    planet as it crosses (equivalently how much a straight planet feature kinks in the world);
  * straight-walk drift: the geodesic curvature on the planet of the image of a straight world line,
    as degrees of heading per 1,000 blocks on a planet of the 16k circumference.

Run on the test VM (numpy, scipy):
  nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.x86_64-linux.python3.withPackages
      (p: [p.numpy p.scipy p.mpmath])' -c python3 tools/research/projection-survey/survey.py [names...]
"""

import json
import math
import sys

import numpy as np
import scipy.special as sp

TAU = 2 * math.pi
C16K = 16384.0  # the 16k presets' circumference in blocks (ledger SCL-004)
R16K = C16K / TAU  # planet radius in blocks at the 16k mean scale


# --------------------------------------------------------------------------------------------
# sphere helpers


def ll2xyz(lat, lon):
    cl = np.cos(lat)
    return np.stack([cl * np.cos(lon), cl * np.sin(lon), np.sin(lat)], axis=-1)


def normalize(v):
    return v / np.linalg.norm(v, axis=-1, keepdims=True)


def zeta2xyz(z):
    """Stereographic coordinate (from the south pole) to unit vector: zeta = tan(colat/2) e^{i lon}."""
    r2 = np.abs(z) ** 2
    return np.stack([2 * z.real / (1 + r2), 2 * z.imag / (1 + r2), (1 - r2) / (1 + r2)], axis=-1)


def reflect_equator(p):
    q = p.copy()
    q[..., 2] = -q[..., 2]
    return q


# --------------------------------------------------------------------------------------------
# hemisphere-in-a-square maps (pole at the tile centre, equator on the boundary); tile [-1, 1]^2.
# All return unit vectors in the northern hemisphere.


def hemi_collignon(u, v):
    """Lambert azimuthal + Shirley-Chiu concentric square = the Collignon quincuncial octants."""
    au, av = np.abs(u), np.abs(v)
    r = np.maximum(au, av)
    with np.errstate(invalid="ignore", divide="ignore"):
        lam_a = (math.pi / 4) * np.where(au > 0, v / np.where(au > 0, u, 1), 0)
        lam_a = np.where(u < 0, lam_a + math.pi, lam_a)
        lam_b = math.pi / 2 - (math.pi / 4) * np.where(av > 0, u / np.where(av > 0, v, 1), 0)
        lam_b = np.where(v < 0, lam_b + math.pi, lam_b)
    lam = np.where(au >= av, lam_a, lam_b)
    s = 1 - r * r
    lat = np.arcsin(np.clip(s, -1, 1))
    return ll2xyz(lat, lam)


K_HALF = float(sp.ellipk(0.5))


def cn_complex(w, m=0.5):
    """cn(x + iy | m), A&S 16.21.3."""
    x, y = w.real, w.imag
    sn, cn, dn, _ = sp.ellipj(x, m)
    sn1, cn1, dn1, _ = sp.ellipj(y, 1 - m)
    den = cn1 * cn1 + m * sn * sn * sn1 * sn1
    return (cn * cn1 - 1j * sn * dn * sn1 * dn1) / den


def hemi_peirce(u, v):
    """Peirce's conformal hemisphere square: zeta = cn(w | 1/2), pole at w = K (tile centre).

    Tile (u, v) -> Peirce square coordinates X = (u + v)/2, Z = (v - u)/2 in units of the half-side,
    w = K + K (X - i Z); the tile corners go to the Peirce square's edge midpoints (the singular
    points)."""
    X = (u + v) / 2
    Z = (u - v) / 2
    w = K_HALF + K_HALF * (X - 1j * Z)
    z = cn_complex(w)
    return zeta2xyz(z)


# --------------------------------------------------------------------------------------------
# cube face maps: (u, v) in [-1, 1]^2 -> (a, b, c) with the point = a*e1 + b*e2 + c*centre.


def face_gnomonic(u, v):
    return normalize(np.stack([u, v, np.ones_like(u)], axis=-1))


def face_eac(u, v):  # equi-angular cubemap = S2's tangent projection
    return face_gnomonic(np.tan(math.pi / 4 * u), np.tan(math.pi / 4 * v))


def _s2_quad(t):
    s = (t + 1) / 2
    return np.where(s >= 0.5, (1 / 3) * (4 * s * s - 1), (1 / 3) * (1 - 4 * (1 - s) ** 2))


def face_s2quad(u, v):  # S2's default quadratic projection
    return face_gnomonic(_s2_quad(u), _s2_quad(v))


def face_nowell(u, v):  # the "spherified cube" of games (Nowell 2005)
    x, y, z = u, v, np.ones_like(u)
    a = x * np.sqrt(1 - y * y / 2 - z * z / 2 + y * y * z * z / 3)
    b = y * np.sqrt(1 - z * z / 2 - x * x / 2 + z * z * x * x / 3)
    c = z * np.sqrt(1 - x * x / 2 - y * y / 2 + x * x * y * y / 3)
    return normalize(np.stack([a, b, c], axis=-1))




# Petroff 2021 (CC0 code, Zenodo 4661351, new_projection.py): inverse, ported to numpy. Its square is
# [-1,1]^2 with the north pole at the centre and the equator the diamond |x| + |y| = 1.
def petroff_inverse(x_m, y_m):
    pi = math.pi
    phi0 = 3 * pi / 8
    # Petroff's own line, 0 + (y >= 0) + (x <= 0) + 2 * ((y <= 0) & (x < 0)), gives quadrant 4 on the ray
    # y = 0, x < 0 and quadrant 1 on x = 0, y < 0, so points exactly on those two diagonals of a tile came
    # out at the wrong place (one-pixel lines in symmetric grids; found 2026-10-04 by the world-shape page's
    # tear check). The same quadrants, with each boundary given to one side:
    quadrant = np.where((x_m <= 0) & (y_m >= 0), 2, np.where(y_m >= 0, 1, np.where(x_m >= 0, 0, 3)))
    zeta = pi / 4 + quadrant * pi / 2
    x_c = math.sqrt(6) * (x_m * np.cos(zeta) + y_m * np.sin(zeta))
    y_c = 3 * math.sqrt(2) * (y_m * np.cos(zeta) - x_m * np.sin(zeta))
    idx = y_c < -3
    sgnphi = np.where(idx, -1.0, 1.0)
    y_h = np.where(idx, -6 - y_c, y_c)
    A1 = math.asin(1 / math.sqrt(2 - math.cos(phi0) ** 2))
    A2 = math.asin(2 * math.sin(phi0) / math.sqrt(3 - math.cos(2 * phi0)))
    hp = 12 / pi * (A1 + A2 - pi / 2)
    xip = math.atan((pi * (hp - 3) ** 2) / (math.sqrt(3) * (pi * (hp ** 2 - 2 * hp + 45) - 96 * A1 - 48 * A2)))
    rp = np.sqrt(x_c ** 2 + (hp - 3 - y_h) ** 2)
    thp = np.abs(np.arctan2(x_c, hp - 3 - y_h))
    p0p = math.atan(math.sqrt(3) / hp)
    p1p = 7 * pi / 6 - p0p - xip
    p2p = xip - pi / 6
    s0 = thp <= p0p
    s1 = (p0p < thp) & (thp <= p0p + p1p)
    s2 = p0p + p1p < thp
    alp = np.where(s0, thp, np.where(s1, pi - p2p - thp, thp + p2p - pi))
    c = np.where(thp <= p0p + p1p, math.acos(math.cos(phi0) / math.sqrt(2)), pi / 2 - phi0)
    psi0 = A1
    psi1 = pi - 2 * psi0
    G = np.where(s0, psi0, np.where(s1, psi1, psi0))
    Gp = np.where(s0, p0p, np.where(s1, p1p, p2p))
    F = np.where(s0, A2, np.where(s1, pi / 2 - A2, pi / 4))
    ap = np.where(s0, hp, math.sqrt(hp ** 2 + 3) * math.sin(pi / 3 - math.atan(hp / math.sqrt(3))) / math.sin(xip))
    cp = np.where(thp <= p0p + p1p, math.sqrt(hp ** 2 + 3), 3 - hp)
    b = np.where(s0, pi / 4, np.where(s1, math.atan(math.sqrt(2) * math.tan(phi0)),
                                      pi / 2 - math.atan(math.sqrt(2) * math.tan(phi0))))
    bp = Gp - alp
    xp = np.sqrt(rp ** 2 + cp ** 2 - 2 * rp * cp * np.cos(bp))
    with np.errstate(invalid="ignore", divide="ignore"):
        gp = np.where(xp > 0, np.arcsin(np.clip(rp * np.sin(bp) / xp, -1, 1)), 0)
        ep = pi - Gp - gp
        yp = np.where(ep > 0, rp * np.sin(alp) / np.sin(ep), 0)
        up = np.sqrt(np.abs(cp ** 2 + (xp + yp) ** 2 - 2 * cp * (xp + yp) * np.cos(gp)))
        vp = ap - up
        delta = np.arctan((-np.sin(vp * (F + G - pi / 2) / ap)) / (np.cos(b) - np.cos(vp * (F + G - pi / 2) / ap)))
        gamma = F - delta
        cos_xy = 1 / np.sqrt(1 + (np.tan(b) / np.cos(delta)) ** 2)
        x = np.arccos(np.clip(1 - (xp / (xp + yp)) ** 2 * (1 - cos_xy), -1, 1))
        r = np.arccos(np.clip(np.cos(x) * np.cos(c) + np.sin(x) * np.sin(c) * np.cos(gamma), -1, 1))
        beta = np.arcsin(np.clip(np.sin(x) * np.sin(gamma) / np.sin(r), -1, 1))
    alpha = np.where(s0, psi0 - beta, np.where(s1, psi0 + beta, pi - beta))
    phi_h = np.arcsin(np.clip(math.sin(phi0) * np.cos(r) - math.cos(phi0) * np.sin(r) * np.cos(alpha), -1, 1))
    lambda0 = pi / 4 + quadrant * pi / 2
    lam = lambda0 + np.sign(x_c) * np.arctan(np.sin(alpha) * np.sin(r) * math.cos(phi0)
                                             / (np.cos(r) - math.sin(phi0) * np.sin(phi_h)))
    return ll2xyz(sgnphi * phi_h, lam)


def hemi_petroff(u, v):
    return petroff_inverse((u + v) / 2, (v - u) / 2)


# Gringorten 1972 as d3-geo-projection implements it (ISC licence, Mike Bostock): gringortenRaw.invert
# vectorised. Its square [-1,1]^2 is one hemisphere in transverse aspect (rim on the boundary); the
# hemisphere's centre (lon 0, lat 0) is turned to the pole.
def _gring_hexa_inv(x, y):
    x0 = np.zeros_like(x)
    x1 = np.ones_like(x)
    r = np.full_like(x, 0.5)
    for _ in range(60):
        r2 = r * r
        sinPhi = np.sqrt(r)
        z = np.arcsin(1 / np.sqrt(1 + r2))
        v = (1 - r2) + r * (1 + r2) * z
        p2 = (1 - sinPhi) / v
        p = np.sqrt(p2)
        a2 = p2 * (1 + r2)
        h = p * (1 - r2)
        g = np.sqrt(np.maximum(a2 - x * x, 0))
        y0 = y + h + r * g
        x0 = np.where(y0 > 0, r, x0)
        x1 = np.where(y0 > 0, x1, r)
        r = 0.5 * (x0 + x1)
    r2 = r * r
    sinPhi = np.sqrt(r)
    z = np.arcsin(1 / np.sqrt(1 + r2))
    v = (1 - r2) + r * (1 + r2) * z
    p2 = (1 - sinPhi) / v
    p = np.sqrt(p2)
    a2 = p2 * (1 + r2)
    g = np.sqrt(np.maximum(a2 - x * x, 0))
    phi = np.arcsin(sinPhi)
    cosPhi = np.cos(phi)
    secPhi = 1 / cosPhi
    drdPhi = 2 * sinPhi * cosPhi
    dvdPhi = (-3 * r + z * (1 + 3 * r2)) * drdPhi
    dp2dPhi = (-v * cosPhi - (1 - sinPhi) * dvdPhi) / (v * v)
    dpdPhi = 0.5 * dp2dPhi / p
    dhdPhi = (1 - r2) * dpdPhi - 2 * r * p * drdPhi
    zeta = -2 * secPhi * dhdPhi
    mu = -secPhi * drdPhi
    nu = -secPhi * (r * (1 + r2) * dp2dPhi + p2 * (1 + 3 * r2) * drdPhi)
    lam = math.pi / 4 * (x * (zeta + mu * g) + nu * np.arcsin(np.clip(x / np.sqrt(a2), -1, 1)))
    return lam, phi


def hemi_gringorten(u, v):
    x, y = u.astype(float), v.astype(float)
    sx = np.where(x >= 0, 1.0, -1.0)
    sy = np.where(y >= 0, 1.0, -1.0)
    X0 = -sx * x
    Y0 = -sy * y
    with np.errstate(invalid="ignore", divide="ignore"):
        t = (Y0 / X0) < 1
    lam, phi = _gring_hexa_inv(np.where(t, Y0, X0), np.where(t, X0, Y0))
    lam = np.where(t, -math.pi / 2 - lam, lam)
    cphi = np.cos(phi)
    L = sx * (np.arctan2(np.sin(lam) * cphi, -np.sin(phi)) + math.pi)
    P = sy * np.arcsin(np.clip(np.cos(lam) * cphi, -1, 1))
    q = ll2xyz(P, L)
    return np.stack([q[..., 1], q[..., 2], q[..., 0]], axis=-1)


# QSC, O'Neill & Laubscher 1976, inverse as PROJ qsc.cpp (MIT) does it on one face [-1,1]^2.
def face_qsc(u, v):
    au, av = np.abs(u), np.abs(v)
    area = np.where(u >= av, 0, np.where(v >= au, 1, np.where(-u >= av, 2, 3)))
    # rotate (u, v) into area 0 (x >= |y|)
    x = np.select([area == 0, area == 1, area == 2, area == 3], [u, v, -u, -v])
    y = np.select([area == 0, area == 1, area == 2, area == 3], [v, -u, -v, u])
    nu = np.arctan(np.sqrt(x * x + y * y))
    mu = np.arctan2(y, x)
    t = (math.pi / 12) * np.tan(mu)
    th = np.arctan(np.sin(t) / (np.cos(t) - 1 / math.sqrt(2)))
    cosphi = 1 - np.cos(mu) ** 2 * np.tan(nu) ** 2 * (1 - np.cos(np.arctan(1 / np.cos(th))))
    cosphi = np.clip(cosphi, -1, 1)
    sinphi = np.sqrt(1 - cosphi ** 2)
    th = th + area * math.pi / 2
    return np.stack([sinphi * np.cos(th), sinphi * np.sin(th), cosphi], axis=-1)


# Rosca & Plonka 2011 (JCAM 236:1033), Eqs 12-13 then the inverse Lambert azimuthal (Eqs 18-20).
RP_BETA = math.sqrt(math.pi / 6)


def face_rosca(u, v):
    x, y = RP_BETA * u, RP_BETA * v
    ax, ay = np.abs(x), np.abs(y)
    c4 = 2 ** 0.25
    with np.errstate(invalid="ignore", divide="ignore"):
        a1 = np.where(ax > 0, y * math.pi / (12 * np.where(ax > 0, x, 1)), 0)
        a2 = np.where(ay > 0, x * math.pi / (12 * np.where(ay > 0, y, 1)), 0)
    d1 = np.sqrt(math.sqrt(2) - np.cos(a1))
    d2 = np.sqrt(math.sqrt(2) - np.cos(a2))
    X1 = c4 * (x / RP_BETA) * (math.sqrt(2) * np.cos(a1) - 1) / d1
    Y1 = c4 * (x / RP_BETA) * (math.sqrt(2) * np.sin(a1)) / d1
    X2 = c4 * (y / RP_BETA) * (math.sqrt(2) * np.sin(a2)) / d2
    Y2 = c4 * (y / RP_BETA) * (math.sqrt(2) * np.cos(a2) - 1) / d2
    m = ay <= ax
    X = np.where(m, X1, X2)
    Y = np.where(m, Y1, Y2)
    q = X * X + Y * Y
    k = np.sqrt(np.maximum(1 - q / 4, 0))
    return np.stack([k * X, k * Y, 1 - q / 2], axis=-1)


# COBE CSC (Chan & O'Neill 1975) inverse polynomial, Calabretta & Greisen 2002 Eq. 175 (N = 6),
# then gnomonic; nearly equal-area.
CSC_P = {(0, 0): -0.27292696, (1, 0): -0.07629969, (0, 1): -0.02819452, (2, 0): -0.22797056,
         (1, 1): -0.01471565, (0, 2): 0.27058160, (3, 0): 0.54852384, (2, 1): 0.48051509,
         (1, 2): -0.56800938, (0, 3): -0.60441560, (4, 0): -0.62930065, (3, 1): -1.74114454,
         (2, 2): 0.30803317, (1, 3): 1.50880086, (0, 4): 0.93412077, (5, 0): 0.25795794,
         (4, 1): 1.71547508, (3, 2): 0.98938102, (2, 3): -0.93678576, (1, 4): -1.41601920,
         (0, 5): -0.63915306, (6, 0): 0.02584375, (5, 1): -0.53022337, (4, 2): -0.83180469,
         (3, 3): 0.08693841, (2, 4): 0.33887446, (1, 5): 0.52032238, (0, 6): 0.14381585}


def _csc_f(X, Y):
    s = np.zeros_like(X)
    for (i, j), p in CSC_P.items():
        s = s + p * X ** (2 * i) * Y ** (2 * j)
    return X + X * (1 - X * X) * s


def face_cobe(u, v):
    return face_gnomonic(_csc_f(u, v), _csc_f(v, u))


# --------------------------------------------------------------------------------------------
# worlds


class World:
    """tiles: list of (x0, y0, w, h, fn) with fn(X, Y) -> unit vectors for global X, Y in the tile.
    seams: list of (p0, p1, n, g): segment p0-p1 on the world edge, outward normal n, and g mapping
    global points just outside to the glued points inside. lines: interior segments to scan for
    creases (tile edges shared by adjacent tiles, midlines, diagonals), each (p0, p1, n)."""

    def __init__(self, name, tiles, seams, lines, cls, note="", walk_region=None):
        self.name, self.tiles, self.seams, self.lines, self.cls, self.note = (
            name, tiles, seams, lines, cls, note)
        self.area = sum(t[2] * t[3] for t in tiles)
        self.walk_region = walk_region

    def F(self, X, Y):
        X = np.asarray(X, float)
        Y = np.asarray(Y, float)
        out = np.full(X.shape + (3,), np.nan)
        for (x0, y0, w, h, fn) in self.tiles:
            m = (X >= x0 - 1e-12) & (X <= x0 + w + 1e-12) & (Y >= y0 - 1e-12) & (Y <= y0 + h + 1e-12)
            m &= np.isnan(out[..., 0])
            if m.any():
                out[m] = fn(X[m], Y[m])
        return out


def tile_lines(x0, y0, w, h, diag=True):
    """Midlines and diagonals of a tile, for the crease scan."""
    cx, cy = x0 + w / 2, y0 + h / 2
    L = [((cx, y0), (cx, y0 + h), (1, 0)), ((x0, cy), (x0 + w, cy), (0, 1))]
    if diag:
        s = 1 / math.sqrt(2)
        L.append(((x0, y0), (x0 + w, y0 + h), (s, -s)))
        L.append(((x0, y0 + h), (x0 + w, y0), (s, s)))
    return L


def pillowcase(name, hemi, note=""):
    """Guyou's 2:1 layout in the polar aspect: tile N = [-1,1]^2 (north pole at its centre), tile S =
    [1,3]x[-1,1], S being N mirrored across x = 1 and reflected through the equator. Seams: S's east
    edge slides to N's west edge; the long edges y = +-1 are each glued to themselves by a half-turn
    about (1, +-1). (Guyou's own aspect is this one turned on the sphere; the measurements do not
    depend on that turn.)"""

    def fN(X, Y):
        return hemi(X, Y)

    def fS(X, Y):
        return reflect_equator(hemi(2 - X, Y))

    tiles = [(-1, -1, 2, 2, fN), (1, -1, 2, 2, fS)]
    seams = [
        ((3, -1), (3, 1), (1, 0), lambda X, Y: (X - 4, Y)),
        ((-1, -1), (-1, 1), (-1, 0), lambda X, Y: (X + 4, Y)),
        ((-1, 1), (3, 1), (0, 1), lambda X, Y: (2 - X, 2 - Y)),
        ((-1, -1), (3, -1), (0, -1), lambda X, Y: (2 - X, -2 - Y)),
    ]
    lines = [((1, -1), (1, 1), (1, 0))] + tile_lines(-1, -1, 2, 2) + tile_lines(1, -1, 2, 2)
    return World(name, tiles, seams, lines, "pillowcase", note)


# cube net: band tiles k = 0..3 at [k, k+1] x [0, 1], N at [0,1]x[1,2], S at [0,1]x[-1,0].
BAND_FRAMES = []
for _k in range(4):
    _a = _k * math.pi / 2
    BAND_FRAMES.append((np.array([math.cos(_a), math.sin(_a), 0.0]),
                        np.array([-math.sin(_a), math.cos(_a), 0.0]),
                        np.array([0.0, 0.0, 1.0])))
N_FRAME = (np.array([0.0, 0, 1]), np.array([0.0, 1, 0]), np.array([-1.0, 0, 0]))
S_FRAME = (np.array([0.0, 0, -1]), np.array([0.0, 1, 0]), np.array([1.0, 0, 0]))


def _rot_cw(cx, cy):
    return lambda X, Y: (cx + (Y - cy), cy - (X - cx))


def _rot_ccw(cx, cy):
    return lambda X, Y: (cx - (Y - cy), cy + (X - cx))


def _wrapx(g):
    def h(X, Y):
        a, b = g(X, Y)
        return np.mod(a, 4.0), b
    return h


CUBE_SEAMS = [
    ((4, 0), (4, 1), (1, 0), lambda X, Y: (X - 4, Y)),
    ((0, 0), (0, 1), (-1, 0), lambda X, Y: (X + 4, Y)),
    # north tile
    ((1, 1), (1, 2), (1, 0), _rot_cw(1, 1)),
    ((0, 2), (1, 2), (0, 1), lambda X, Y: (3 - X, 3 - Y)),
    ((0, 1), (0, 2), (-1, 0), _wrapx(_rot_ccw(0, 1))),
    # tops of band tiles 1..3 (the other side of the same three seams)
    ((1, 1), (2, 1), (0, 1), _rot_ccw(1, 1)),
    ((2, 1), (3, 1), (0, 1), lambda X, Y: (3 - X, 3 - Y)),
    ((3, 1), (4, 1), (0, 1), lambda X, Y: _rot_cw(0, 1)(X - 4, Y)),
    # south tile
    ((1, -1), (1, 0), (1, 0), _rot_ccw(1, 0)),
    ((0, -1), (1, -1), (0, -1), lambda X, Y: (3 - X, -1 - Y)),
    ((0, -1), (0, 0), (-1, 0), _wrapx(_rot_cw(0, 0))),
    ((1, 0), (2, 0), (0, -1), _rot_cw(1, 0)),
    ((2, 0), (3, 0), (0, -1), lambda X, Y: (3 - X, -1 - Y)),
    ((3, 0), (4, 0), (0, -1), lambda X, Y: _rot_ccw(0, 0)(X - 4, Y)),
]
CUBE_LINES = ([((k, 0), (k, 1), (1, 0)) for k in (1, 2, 3)]
              + [((0, 1), (1, 1), (0, 1)), ((0, 0), (1, 0), (0, 1))]
              + sum([tile_lines(k, 0, 1, 1) for k in range(4)], [])
              + tile_lines(0, 1, 1, 1) + tile_lines(0, -1, 1, 1))


def cube_world(name, face, note=""):
    """A cube map in the cross net: the four equatorial faces in a band (x wraps every 4), the north
    face above band face 0, the south face below it. 7 seam pairs (the band's wrap and three per
    polar face, two quarter-turns and one half-turn)."""

    def mk(frame, x0, y0):
        c, e1, e2 = frame

        def fn(X, Y):
            u = 2 * (X - x0) - 1
            v = 2 * (Y - y0) - 1
            abc = face(u, v)
            return abc[..., 0:1] * e1 + abc[..., 1:2] * e2 + abc[..., 2:3] * c
        return fn

    tiles = [(k, 0, 1, 1, mk(BAND_FRAMES[k], k, 0)) for k in range(4)]
    tiles += [(0, 1, 1, 1, mk(N_FRAME, 0, 1)), (0, -1, 1, 1, mk(S_FRAME, 0, -1))]
    return World(name, tiles, CUBE_SEAMS, CUBE_LINES, "cube", note)


# HEALPix (Calabretta & Roukema 2007; H = 4, K = 3) on the same cross net: band tile k is HEALPix
# x in [-pi + k pi/2, -pi + (k+1) pi/2], |y| <= pi/4; the polar tiles are the four teeth folded into
# a square round the pole (rHEALPix). Formulas written from memory, checked by the equal-area test.


def healpix_inv(x, y):
    ay = np.abs(y)
    eq = ay <= math.pi / 4
    lam = np.array(x, float)
    sinphi = (8 / (3 * math.pi)) * y
    sigma = 2 - 4 * ay / math.pi
    xc = -math.pi + (2 * np.floor(2 * (x + math.pi) / math.pi) + 1) * math.pi / 4
    with np.errstate(invalid="ignore", divide="ignore"):
        lam_p = xc + (x - xc) / np.where(sigma > 0, sigma, 1)
    sinp = np.sign(y) * (1 - sigma * sigma / 3)
    lam = np.where(eq, lam, lam_p)
    sinphi = np.where(eq, sinphi, sinp)
    return ll2xyz(np.arcsin(np.clip(sinphi, -1, 1)), lam)


def healpix_world():
    Q = math.pi / 2

    def band(X, Y):
        return healpix_inv(-math.pi + Q * X, Q * Y - math.pi / 4)

    def polar(north):
        # (p, q) relative to the polar tile's centre, in HEALPix units. Each of the four triangles
        # (bottom, right, top, left) is unfolded through its seam onto the tooth above (north) or
        # below (south) the band tile it is glued to; the turns are the seam table's.
        def fn(X, Y):
            p = Q * (X - 0.5)
            q = Q * (Y - (1.5 if north else -0.5))
            ap, aq = np.abs(p), np.abs(q)
            m_b = q <= -ap
            m_r = (~m_b) & (p >= aq)
            m_t = (~m_b) & (~m_r) & (q >= ap)
            m_l = ~(m_b | m_r | m_t)
            P = np.empty_like(p)
            V = np.empty_like(q)
            kk = np.empty(p.shape, int)
            if north:  # bottom edge faces band tile 0
                table = [(m_b, p, q, 0), (m_r, q, -p, 1), (m_t, -p, -q, 2), (m_l, -q, p, 3)]
            else:      # top edge faces band tile 0
                table = [(m_t, p, q, 0), (m_r, -q, p, 1), (m_b, -p, -q, 2), (m_l, q, -p, 3)]
            for m, a, b, k in table:
                P[m], V[m], kk[m] = a[m], b[m], k
            xc = -math.pi + (2 * kk + 1) * math.pi / 4
            y = (math.pi / 2 + V) if north else (-math.pi / 2 + V)
            return healpix_inv(xc + P, y)
        return fn

    fN = polar(True)
    fS = polar(False)
    tiles = [(k, 0, 1, 1, band) for k in range(4)]
    tiles += [(0, 1, 1, 1, fN), (0, -1, 1, 1, fS)]
    return World("healpix", tiles, CUBE_SEAMS, CUBE_LINES, "cube",
                 "HEALPix / rHEALPix (equatorial zone cylindrical equal-area, Collignon caps)")


def lambert_world(cap_deg=80.0):
    """Today's shipped world: Lambert cylindrical equal-area, x wraps, z = sin(lat); stats over the
    walkable band |lat| <= cap."""
    zc = math.sin(math.radians(cap_deg))

    def fn(X, Y):
        return ll2xyz(np.arcsin(np.clip(Y, -1, 1)), X)

    tiles = [(-math.pi, -zc, TAU, 2 * zc, fn)]
    seams = [((math.pi, -zc), (math.pi, zc), (1, 0), lambda X, Y: (X - TAU, Y))]
    return World(f"lambert-cyl-cap{cap_deg:g}", tiles, seams, [], "cylinder",
                 "shipped; area over the walkable band only")


# --------------------------------------------------------------------------------------------
# measurements


def fundamental(F, X, Y, h):
    p = F(X, Y)
    a = (F(X + h, Y) - F(X - h, Y)) / (2 * h)
    b = (F(X, Y + h) - F(X, Y - h)) / (2 * h)
    return p, a, b


def sv(a, b):
    E = (a * a).sum(-1)
    Fm = (a * b).sum(-1)
    G = (b * b).sum(-1)
    tr = E + G
    disc = np.sqrt(np.maximum((E - G) ** 2 + 4 * Fm * Fm, 0))
    s1 = np.sqrt((tr + disc) / 2)
    s2 = np.sqrt(np.maximum((tr - disc) / 2, 0))
    return s1, s2


def wquant(vals, w, qs):
    o = np.argsort(vals)
    v, ww = vals[o], w[o]
    c = np.cumsum(ww)
    c /= c[-1]
    return [float(v[min(np.searchsorted(c, q), len(v) - 1)]) for q in qs]


QS = [0.0, 0.01, 0.05, 0.10, 0.50, 0.90, 0.95, 0.99, 0.999, 1.0]


def distortion(world, n=500, h=1e-6):
    Xs, Ys, Ws = [], [], []
    for (x0, y0, w, hh, fn) in world.tiles:
        if w >= hh:
            nx, ny = int(round(n * w / hh)), n
        else:
            nx, ny = n, int(round(n * hh / w))
        xs = x0 + (np.arange(nx) + 0.5) * w / nx
        ys = y0 + (np.arange(ny) + 0.5) * hh / ny
        X, Y = np.meshgrid(xs, ys)
        Xs.append(X.ravel())
        Ys.append(Y.ravel())
        Ws.append(np.full(X.size, w * hh / X.size))
    X = np.concatenate(Xs)
    Y = np.concatenate(Ys)
    Wd = np.concatenate(Ws)  # world area per sample
    step = h * max(1.0, max(t[2] for t in world.tiles))
    p, a, b = fundamental(world.F, X, Y, step)
    s1, s2 = sv(a, b)
    ar = s1 * s2  # planet area per world area
    orient = np.sign(np.einsum("ij,ij->i", p, np.cross(a, b)))
    ok = np.isfinite(ar) & (ar > 0)
    planet_w = ar * Wd
    total_planet = planet_w[ok].sum()
    world_area = Wd[ok].sum()
    mean = total_planet / world_area
    rel = ar / mean  # planet per world, relative
    bpa = 1 / rel  # blocks per planet area relative to mean
    shape = s1 / np.maximum(s2, 1e-300)
    om2 = 2 * np.degrees(np.arcsin((s1 - s2) / (s1 + s2)))
    w = planet_w
    res = {
        "planet_area_covered": float(total_planet / (4 * math.pi)),
        "orientation_preserving_share": float(planet_w[ok & (orient > 0)].sum() / total_planet),
        "blocks_per_planet_area_rel_q": dict(zip([str(q) for q in QS], wquant(bpa[ok], w[ok], QS))),
        "area_cv": float(np.sqrt(np.average((rel[ok] - 1) ** 2, weights=Wd[ok]))),
        "shape_ratio_q": dict(zip([str(q) for q in QS], wquant(shape[ok], w[ok], QS))),
        "two_omega_mean_deg": float(np.average(om2[ok], weights=w[ok])),
        "two_omega_max_deg": float(om2[ok].max()),
        "share_shape_le_1.5": float(w[ok & (shape <= 1.5)].sum() / w[ok].sum()),
        "share_shape_le_2": float(w[ok & (shape <= 2)].sum() / w[ok].sum()),
        "share_area_within_x1.25": float(w[ok & (bpa <= 1.25) & (bpa >= 0.8)].sum() / w[ok].sum()),
        "share_area_within_x2": float(w[ok & (bpa <= 2) & (bpa >= 0.5)].sum() / w[ok].sum()),
        "samples": int(ok.sum()),
    }
    # straight-walk drift: geodesic curvature of images of straight world lines, 8 directions.
    sub = np.arange(0, X.size, max(1, X.size // 60000))
    Xs_, Ys_ = X[sub], Y[sub]
    kap_all, wt_all = [], []
    hh = 1e-4 * max(1.0, max(t[2] for t in world.tiles))
    for ang in np.arange(8) * math.pi / 8:
        dx, dy = math.cos(ang), math.sin(ang)
        c0 = world.F(Xs_, Ys_)
        cp = world.F(Xs_ + hh * dx, Ys_ + hh * dy)
        cm = world.F(Xs_ - hh * dx, Ys_ - hh * dy)
        d1 = (cp - cm) / (2 * hh)
        d2 = (cp - 2 * c0 + cm) / (hh * hh)
        speed = np.linalg.norm(d1, axis=-1)
        kg = np.einsum("ij,ij->i", c0, np.cross(d1, d2)) / speed ** 3
        # heading change per world length = kg * speed; world unit -> blocks at the 16k mean scale
        turn = np.abs(kg * speed)
        kap_all.append(turn)
        wt_all.append(ar[sub])
    turn = np.concatenate(kap_all)
    wt = np.concatenate(wt_all)
    good = np.isfinite(turn)
    # world units: the world's area A_w (in its units) holds the planet 4*pi R^2 blocks^2 at the mean
    # scale, so one world unit = R16K * sqrt(4 pi / A_w) blocks.
    unit_blocks = R16K * math.sqrt(4 * math.pi / world.area)
    per_kblock = np.degrees(turn * 1000 / unit_blocks)
    res["walk_drift_deg_per_1000_blocks_q50_q90_q99"] = wquant(per_kblock[good], wt[good], [0.5, 0.9, 0.99])
    return res


def one_sided(Ffun, P, n, h, sign):
    """One-sided derivative of Ffun at P along sign*n from samples at h, 2h, 3h (the point itself
    is not evaluated, so a formula's branch choice exactly on the line does not matter)."""
    f = [Ffun(P[:, 0] + k * sign * h * n[0], P[:, 1] + k * sign * h * n[1]) for k in (1, 2, 3)]
    return sign * (-5 * f[0] + 8 * f[1] - 3 * f[2]) / (2 * h)


def tangential(Ffun, P, n, tang, h, sign):
    """Derivative along the line, taken just off it on one side and extrapolated to the line."""
    out = []
    for k in (1, 2):
        Q = P + k * sign * h * n[None, :]
        out.append((Ffun(Q[:, 0] + h * tang[0], Q[:, 1] + h * tang[1])
                    - Ffun(Q[:, 0] - h * tang[0], Q[:, 1] - h * tang[1])) / (2 * h))
    return 2 * out[0] - out[1]


def jump(world, p0, p1, n, g=None, m=400, h=1e-5, trim=0.02):
    """Compare the two one-sided Jacobians across a line. bend: the largest angle, in the world,
    between the two drawings of one planet direction (how much a straight ridge or coast kinks on
    the map where it crosses); eps = ||pinv(J-) J+ - I||, the relative jump of the derivative."""
    p0, p1, n = np.array(p0, float), np.array(p1, float), np.array(n, float)
    t = np.linspace(trim, 1 - trim, m)
    P = p0[None, :] + t[:, None] * (p1 - p0)[None, :]
    tang = (p1 - p0) / np.linalg.norm(p1 - p0)
    hh = h * max(1.0, np.linalg.norm(p1 - p0))
    F = world.F
    if g is None:
        Fp = F
    else:
        def Fp(X, Y):
            a, b = g(X, Y)
            return F(a, b)
    f_in = F(P[:, 0], P[:, 1])
    f_out = Fp(P[:, 0], P[:, 1])
    gap = np.linalg.norm(f_in - f_out, axis=-1)
    dn_m = one_sided(F, P, n, hh, -1.0)
    dn_p = one_sided(Fp, P, n, hh, +1.0)
    dt_m = tangential(F, P, n, tang, hh, -1.0)
    dt_p = tangential(Fp, P, n, tang, hh, +1.0)
    eps, bend, perp = [], [], []
    angs = np.linspace(0, math.pi, 90, endpoint=False)
    D = np.stack([np.cos(angs), np.sin(angs)])
    for i in range(m):
        Jm = np.stack([dt_m[i], dn_m[i]], axis=1)  # 3x2 in the (tangent, normal) world frame
        Jp = np.stack([dt_p[i], dn_p[i]], axis=1)
        if not (np.isfinite(Jm).all() and np.isfinite(Jp).all()):
            eps.append(np.nan)
            bend.append(np.nan)
            perp.append(np.nan)
            continue
        M = np.linalg.pinv(Jm) @ Jp  # world -> world: J+ = J- M
        eps.append(float(np.linalg.norm(M - np.eye(2), 2)))
        # a planet direction drawn as world direction d on the - side is drawn as M^-1 d on the + side
        B = np.linalg.solve(M, D)
        cosang = (D * B).sum(0) / np.linalg.norm(B, axis=0)
        bend.append(float(np.degrees(np.arccos(np.clip(cosang, -1, 1))).max()))
        # a planet line crossing at right angles on the - side (world direction (0, 1)):
        bn = np.linalg.solve(M, np.array([0.0, 1.0]))
        perp.append(float(np.degrees(np.arccos(np.clip(bn[1] / np.linalg.norm(bn), -1, 1)))))
    eps = np.array(eps)
    bend = np.array(bend)
    perp = np.array(perp)
    return {
        "gap_max": float(np.nanmax(gap)),
        "eps_max": float(np.nanmax(eps)), "eps_median": float(np.nanmedian(eps)),
        "bend_max_deg": float(np.nanmax(bend)), "bend_median_deg": float(np.nanmedian(bend)),
        "perp_bend_max_deg": float(np.nanmax(perp)), "perp_bend_median_deg": float(np.nanmedian(perp)),
    }


def creases(world, m=300):
    out = []
    for (p0, p1, n, g) in world.seams:
        r = jump(world, p0, p1, n, g, m=m)
        r.update({"line": [list(p0), list(p1)], "kind": "seam"})
        out.append(r)
    for (p0, p1, n) in world.lines:
        r = jump(world, p0, p1, n, None, m=m)
        r.update({"line": [list(p0), list(p1)], "kind": "interior"})
        out.append(r)
    return out


def summarize_creases(cr):
    s = [c for c in cr if c["kind"] == "seam"]
    i = [c for c in cr if c["kind"] == "interior"]
    f = lambda L, k: max([c[k] for c in L], default=float("nan"))
    return {
        "seam_gap_max": f(s, "gap_max"),
        "seam_bend_max_deg": f(s, "bend_max_deg"), "seam_eps_max": f(s, "eps_max"),
        "interior_bend_max_deg": f(i, "bend_max_deg"), "interior_eps_max": f(i, "eps_max"),
        "seam_perp_bend_max_deg": f(s, "perp_bend_max_deg"),
        "interior_perp_bend_max_deg": f(i, "perp_bend_max_deg"),
        "creased_lines": [c["line"] + [c["kind"], round(c["bend_max_deg"], 2), round(c["perp_bend_max_deg"], 2),
                                       round(c["eps_max"], 3), round(c["eps_median"], 3)]
                          for c in cr if c["eps_max"] > 0.01],
    }


WORLDS = {
    "lambert": lambda: lambert_world(80.0),
    "guyou": lambda: pillowcase("guyou-peirce", hemi_peirce, "conformal (Peirce 1879 / Guyou 1887)"),
    "collignon": lambda: pillowcase("collignon-quincuncial", hemi_collignon,
                                    "equal-area; = Lambert azimuthal + Shirley-Chiu square"),
    "petroff": lambda: pillowcase("petroff-2021", hemi_petroff, "equal-area, closed form (Petroff 2021)"),
    "gringorten": lambda: pillowcase("gringorten-1972", hemi_gringorten, "equal-area (Gringorten 1972, d3 port)"),
    "healpix": healpix_world,
    "qsc": lambda: cube_world("cube-qsc", face_qsc, "equal-area (O'Neill & Laubscher 1976)"),
    "rosca": lambda: cube_world("cube-rosca-plonka", face_rosca, "equal-area (Rosca & Plonka 2011)"),
    "cobe": lambda: cube_world("cube-cobe-csc", face_cobe, "nearly equal-area polynomial (Chan & O'Neill 1975)"),
    "gnomonic": lambda: cube_world("cube-gnomonic", face_gnomonic),
    "eac": lambda: cube_world("cube-eac-s2tan", face_eac, "equi-angular cubemap = S2 tangent"),
    "s2quad": lambda: cube_world("cube-s2-quadratic", face_s2quad),
    "nowell": lambda: cube_world("cube-nowell", face_nowell),
}


def main(argv):
    names = argv[1:] or list(WORLDS)
    n = 400
    results = {}
    for nm in names:
        if nm.startswith("n="):
            n = int(nm[2:])
            continue
        w = WORLDS[nm]()
        d = distortion(w, n=n)
        cr = creases(w)
        results[w.name] = {"class": w.cls, "note": w.note, "world_area_units": w.area,
                           "distortion": d, "creases": summarize_creases(cr)}
        print(json.dumps({w.name: results[w.name]}, indent=1), flush=True)
    return results


if __name__ == "__main__":
    main(sys.argv)
