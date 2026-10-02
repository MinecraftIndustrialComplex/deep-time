#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The grid-artifact audit (P1-23): numeric tests for the imprint of the simulation's grids on
what the game reads. Run through tools/review/grid-audit.sh (numpy, scipy, pillow); the method is
written up in docs/research/notes/grid-artifact-audit.md.

Every test compares a product with what a field without a grid would give:

- orientation   the direction of the gradient (or of a boundary's normal), folded to 0..180 deg.
                a4 = <cos 4θ> is 0 for an isotropic field, positive when slopes face along the
                block axes (lines run N-S and E-W), negative for the diagonals. a8 = <cos 8θ> is
                positive when both the axes and the diagonals are preferred (D8 routing).
- phase         the mean |first difference| and |second difference| of a field by the position of
                the sample inside a coarser cell (index mod p). Without an imprint every phase has
                the same mean: the figure is max/min − 1. Periods that are no grid (3, 5, 7) are the
                control.
- spectrum      the same a4 from the power spectrum, by wavelength band.
- outliers      the mean |second difference| of every column and every row against its
                neighbours: a seam (the antimeridian, a tile edge) is a column that stands out.
- runs          for categorical maps: the share of boundary edges in straight axis-aligned runs,
                against a random isotropic map of the same boundary density.
"""

import argparse
import json
import math
import os
import struct
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

R = {}  # results


def say(*a):
    print(*a, flush=True)


def put(path, value):
    d = R
    keys = path.split("/")
    for k in keys[:-1]:
        d = d.setdefault(k, {})
    d[keys[-1]] = value


def rnd(v, n=4):
    if isinstance(v, (list, tuple, np.ndarray)):
        return [rnd(x, n) for x in v]
    if isinstance(v, (float, np.floating)):
        return None if not math.isfinite(v) else round(float(v), n)
    if isinstance(v, np.integer):
        return int(v)
    return v


# ---------------------------------------------------------------- orientation


def orient_stats(theta, w):
    """a4, a8 and the shares near the axes and the diagonals of orientations theta (radians, any
    range; folded to 180 deg) with weights w."""
    sw = float(w.sum())
    if sw <= 0:
        return None
    a4 = float((w * np.cos(4 * theta)).sum() / sw)
    a8 = float((w * np.cos(8 * theta)).sum() / sw)
    deg = np.degrees(theta) % 90.0  # 0 = an axis, 45 = a diagonal
    off_axis = np.minimum(deg, 90.0 - deg)
    off_diag = np.abs(deg - 45.0)
    d180 = np.degrees(theta) % 180.0
    along_x = np.minimum(d180, 180.0 - d180) <= 2  # the slope faces east or west: a N-S line
    along_z = np.abs(d180 - 90.0) <= 2  # the slope faces north or south: an E-W line
    return {
        "a4": a4,
        "a8": a8,
        "axis_pm2": float(w[off_axis <= 2].sum() / sw),  # isotropic 0.0444
        "diag_pm2": float(w[off_diag <= 2].sum() / sw),  # isotropic 0.0444
        "ns_lines_pm2": float(w[along_x].sum() / sw),  # isotropic 0.0222
        "ew_lines_pm2": float(w[along_z].sum() / sw),  # isotropic 0.0222
        "n": int(theta.size),
    }


def gradient(f, s):
    """Central differences with step s cells; x wraps, rows do not (edges NaN)."""
    gx = np.roll(f, -s, axis=1) - np.roll(f, s, axis=1)
    gz = np.full_like(f, np.nan)
    gz[s:-s] = f[2 * s :] - f[: -2 * s]
    return gx, gz


def orient_field(f, mask, s=1, weight="mag", tile=256, min_mag=0.0):
    """Orientation statistics of f's gradient over mask, with the spread of a4 over tiles.
    min_mag: the smallest difference counted, for fields stored in steps (a difference of one or
    two steps has only the directions 0, 45 and 90 deg)."""
    gx, gz = gradient(f.astype(np.float64), s)
    mag = np.hypot(gx, gz)
    ok = mask & np.isfinite(mag) & (mag > min_mag)
    th = np.arctan2(gz, gx)
    w = mag if weight == "mag" else np.ones_like(mag)
    st = orient_stats(th[ok], w[ok])
    if st is None:
        return None
    # Per-tile a4, for a standard error and the worst tile.
    h, wd = f.shape
    c4 = np.where(ok, w * np.cos(4 * th), 0.0)
    ww = np.where(ok, w, 0.0)
    nj, ni = h // tile, wd // tile
    t4 = c4[: nj * tile, : ni * tile].reshape(nj, tile, ni, tile).sum(axis=(1, 3))
    tw = ww[: nj * tile, : ni * tile].reshape(nj, tile, ni, tile).sum(axis=(1, 3))
    cnt = ok[: nj * tile, : ni * tile].reshape(nj, tile, ni, tile).sum(axis=(1, 3))
    good = cnt > 0.5 * tile * tile
    if good.sum() >= 4:
        a = t4[good] / tw[good]
        st["tiles"] = int(good.sum())
        st["a4_tile_mean"] = float(a.mean())
        st["a4_tile_se"] = float(a.std(ddof=1) / math.sqrt(a.size))
        jj, ii = np.nonzero(good)
        k = int(np.argmax(np.abs(a)))
        st["worst_tile"] = [int(ii[k] * tile), int(jj[k] * tile), float(a[k])]
    return st


# ---------------------------------------------------------------- phase


def phase_test(f, mask, periods=(2, 4, 8, 16, 32, 3, 5, 7), origin=(0, 0)):
    """max/min − 1 of the mean |Δ| and |Δ²| by index mod p, along x and along z."""
    f = f.astype(np.float64)
    out = {}
    h, w = f.shape
    for axis, name in ((1, "x"), (0, "z")):
        d1 = np.abs(np.roll(f, -1, axis=axis) - f)
        d2 = np.abs(np.roll(f, -1, axis=axis) - 2 * f + np.roll(f, 1, axis=axis))
        m1 = mask & np.roll(mask, -1, axis=axis)
        m2 = m1 & np.roll(mask, 1, axis=axis)
        if axis == 0:
            m1[-1] = False
            m2[-1] = False
            m2[0] = False
        n = f.shape[axis]
        idx = np.arange(n)
        for p in periods:
            ph = (idx + (origin[0] if axis == 1 else origin[1])) % p
            res = {}
            for label, d, m in (("d1", d1, m1), ("d2", d2, m2)):
                s = np.where(m, d, 0.0).sum(axis=1 - axis)
                c = m.sum(axis=1 - axis)
                means = np.array(
                    [s[ph == k].sum() / max(c[ph == k].sum(), 1) for k in range(p)]
                )
                lo = means.min()
                res[label] = {
                    "excess": float(means.max() / lo - 1) if lo > 0 else float("inf"),
                    "max_phase": int(means.argmax()),
                    "min_phase": int(means.argmin()),
                    "means": rnd(means.tolist(), 5),
                }
            out[f"{name}_p{p}"] = res
    return out


def phase_summary(pt):
    """One line per period: the larger excess of x and z, for d1 and d2."""
    rows = {}
    for key, v in pt.items():
        p = key.split("_p")[1]
        r = rows.setdefault(p, {"d1": 0.0, "d2": 0.0})
        r["d1"] = max(r["d1"], v["d1"]["excess"])
        r["d2"] = max(r["d2"], v["d2"]["excess"])
    return {p: rnd(r) for p, r in sorted(rows.items(), key=lambda kv: int(kv[0]))}


# ---------------------------------------------------------------- spectrum


def spectrum_bands(f, mask, cell_blocks, tile=512, min_land=0.9):
    """a4 and a8 of the power spectrum by wavelength band, averaged over land tiles."""
    h, w = f.shape
    win = np.hanning(tile)[:, None] * np.hanning(tile)[None, :]
    k = np.fft.fftfreq(tile)  # cycles per cell
    kx, kz = np.meshgrid(k, k)
    kr = np.hypot(kx, kz)
    phi = np.arctan2(kz, kx)
    edges = [4, 8, 16, 32, 64, 128, 256, 512]  # wavelength, blocks
    acc = {}
    n = 0
    yy, xx = np.mgrid[0:tile, 0:tile]
    A = np.stack([xx.ravel(), yy.ravel(), np.ones(tile * tile)], axis=1)
    for j0 in range(0, h - tile + 1, tile):
        for i0 in range(0, w - tile + 1, tile):
            m = mask[j0 : j0 + tile, i0 : i0 + tile]
            if m.mean() < min_land:
                continue
            t = f[j0 : j0 + tile, i0 : i0 + tile].astype(np.float64)
            coef, *_ = np.linalg.lstsq(A, t.ravel(), rcond=None)
            t = t - (A @ coef).reshape(tile, tile)
            P = np.abs(np.fft.fft2(t * win)) ** 2
            n += 1
            for lo, hi in zip(edges[:-1], edges[1:]):
                # wavelength in blocks = cell_blocks / kr
                sel = (kr > cell_blocks / hi) & (kr <= cell_blocks / lo) & (kr <= 0.5)
                p = P[sel]
                s = p.sum()
                if s <= 0:
                    continue
                # Less what the band's own modes give with equal power (the FFT's modes lie on a
                # square lattice, which is not isotropic for the few modes of a long wavelength).
                a = acc.setdefault(f"{lo}-{hi}", {"a4": [], "a8": []})
                a["a4"].append(float((p * np.cos(4 * phi[sel])).sum() / s - np.cos(4 * phi[sel]).mean()))
                a["a8"].append(float((p * np.cos(8 * phi[sel])).sum() / s - np.cos(8 * phi[sel]).mean()))
    out = {"tiles": n}
    for band, a in acc.items():
        a4, a8 = np.array(a["a4"]), np.array(a["a8"])
        out[band] = {
            "a4": float(a4.mean()),
            "a4_se": float(a4.std(ddof=1) / math.sqrt(a4.size)) if a4.size > 1 else None,
            "a8": float(a8.mean()),
            "a8_se": float(a8.std(ddof=1) / math.sqrt(a8.size)) if a8.size > 1 else None,
        }
    return out


# ---------------------------------------------------------------- outliers


def outlier_scan(f, mask=None, half=32):
    """Robust z-score of every column's and row's mean |second difference across it| against the
    running median of its neighbours. Returns the largest, where, and the seam's."""
    f = f.astype(np.float64)
    out = {}
    for axis, name in ((1, "columns"), (0, "rows")):
        d2 = np.abs(np.roll(f, -1, axis=axis) - 2 * f + np.roll(f, 1, axis=axis))
        m = np.ones(f.shape, bool) if mask is None else mask.copy()
        m &= np.roll(m, 1, axis=axis) & np.roll(m, -1, axis=axis)
        if axis == 0:
            m[0] = m[-1] = False
        s = np.where(m, d2, 0.0).sum(axis=1 - axis)
        c = m.sum(axis=1 - axis)
        ok = c > 0.02 * f.shape[1 - axis]
        prof = np.where(ok, s / np.maximum(c, 1), np.nan)
        mode = "wrap" if axis == 1 else "nearest"
        filled = np.where(ok, prof, np.nanmedian(prof))
        med = ndimage.median_filter(filled, size=2 * half + 1, mode=mode)
        resid = np.where(ok, prof - med, np.nan)
        mad = np.nanmedian(np.abs(resid - np.nanmedian(resid)))
        z = resid / (1.4826 * mad) if mad > 0 else resid * 0
        k = int(np.nanargmax(z))
        top = np.argsort(-np.nan_to_num(z, nan=-1e9))[:8]
        out[name] = {
            "max_z": float(z[k]),
            "at": k,
            "top": [[int(t), rnd(float(z[t]), 1)] for t in top],
            "n_over_6": int(np.nansum(z > 6)),
            "n": int(ok.sum()),
        }
        if axis == 1:
            out[name]["seam_z"] = [rnd(float(z[0]), 2), rnd(float(z[-1]), 2)]
            # Period of the columns/rows that stand out (a tile edge repeats).
            big = np.nonzero(np.nan_to_num(z, nan=0) > 6)[0]
            out[name]["over_6_at"] = big[:40].tolist()
        else:
            big = np.nonzero(np.nan_to_num(z, nan=0) > 6)[0]
            out[name]["over_6_at"] = big[:40].tolist()
    return out


def seam_step(f, mask=None):
    """Mean |step| across the seam (last column → first) over the mean |step| between interior
    columns, on the cells of mask."""
    f = f.astype(np.float64)
    d = np.abs(np.roll(f, -1, axis=1) - f)  # d[:, i] = |f[i+1] − f[i]|; the last is the seam
    m = np.ones(f.shape, bool) if mask is None else (mask & np.roll(mask, -1, axis=1))
    m &= np.isfinite(d)
    col = np.where(m, d, 0).sum(axis=0) / np.maximum(m.sum(axis=0), 1)
    cnt = m.sum(axis=0)
    inner = col[:-1][cnt[:-1] > 0]
    return {
        "seam": float(col[-1]),
        "interior_mean": float(inner.mean()),
        "interior_p99": float(np.percentile(inner, 99)),
        "ratio": float(col[-1] / inner.mean()) if inner.mean() > 0 else None,
        "seam_cells": int(cnt[-1]),
    }


# ---------------------------------------------------------------- categorical


OFFS = [(a, b) for b in range(-2, 3) for a in range(-2, 3) if (a, b) != (0, 0)]


def cat_boundary(c, mask, wrap=True):
    """Boundary cells of a categorical map and the orientation of their normals: the vector sum
    of the offsets (5 × 5) to cells of another class."""
    nx = np.zeros(c.shape)
    nz = np.zeros(c.shape)
    for a, b in OFFS:
        other = np.roll(np.roll(c, -a, axis=1), -b, axis=0) != c
        nx += a * other
        nz += b * other
    edge = np.zeros(c.shape, bool)
    for a, b in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        edge |= np.roll(np.roll(c, -a, axis=1), -b, axis=0) != c
    ok = edge & mask & ((nx != 0) | (nz != 0))
    ok[:3] = ok[-3:] = False
    if not wrap:
        ok[:, :3] = ok[:, -3:] = False
    return ok, np.arctan2(nz, nx)


def run_shares(c, mask, wrap=True):
    """Share of the boundary edges that lie in straight axis-aligned runs of at least 4, 8 and 16
    edges, and the boundary density (edges per cell)."""
    total = 0
    counts = {4: 0, 8: 0, 16: 0}
    # Vertical edges between (j, i) and (j, i+1): runs along j. Horizontal ones: runs along i.
    for axis in (1, 0):
        e = (np.roll(c, -1, axis=axis) != c) & mask & np.roll(mask, -1, axis=axis)
        if axis == 0:
            e[-1] = False
        elif not wrap:
            e[:, -1] = False
        # A run continues only while the pair of classes is the same (one boundary, not a corner
        # between three units).
        lo = np.minimum(c, np.roll(c, -1, axis=axis)).astype(np.int64)
        hi = np.maximum(c, np.roll(c, -1, axis=axis)).astype(np.int64)
        pair = np.where(e, lo * 100003 + hi + 1, 0)
        run_axis = 1 - axis
        p = pair if run_axis == 0 else pair.T
        p = np.ascontiguousarray(p)
        # Run lengths along axis 0 of p.
        same = (p[1:] == p[:-1]) & (p[1:] != 0)
        length = np.zeros(p.shape, np.int32)
        length[0] = p[0] != 0
        for r in range(1, p.shape[0]):
            length[r] = np.where(p[r] != 0, np.where(same[r - 1], length[r - 1] + 1, 1), 0)
        # A run's length is its last cell's count: propagate backwards.
        full = length.copy()
        for r in range(p.shape[0] - 2, -1, -1):
            cont = same[r]
            full[r] = np.where(cont, full[r + 1], full[r])
        total += int((p != 0).sum())
        for k in counts:
            counts[k] += int((full >= k).sum())
    area = int(mask.sum())
    return {
        "edges": total,
        "density": total / max(area, 1),
        "run_ge4": counts[4] / max(total, 1),
        "run_ge8": counts[8] / max(total, 1),
        "run_ge16": counts[16] / max(total, 1),
    }


def control_map(shape, sigma, classes=5, seed=7, blocky=1):
    """A random isotropic categorical map: smoothed white noise cut at its quantiles. With blocky
    > 1, the same map made at 1/blocky the size and replicated: what a nearest lookup gives."""
    rng = np.random.default_rng(seed)
    h, w = shape
    if blocky > 1:
        small = control_map((h // blocky + 1, w // blocky + 1), sigma / blocky, classes, seed)
        return np.repeat(np.repeat(small, blocky, axis=0), blocky, axis=1)[:h, :w]
    g = ndimage.gaussian_filter(rng.standard_normal((h, w)), sigma, mode="wrap")
    q = np.quantile(g, np.linspace(0, 1, classes + 1)[1:-1])
    return np.digitize(g, q).astype(np.int32)


def cat_test(c, mask, wrap=True, control_shape=(512, 512)):
    """Orientation and straight-run statistics of a categorical map, beside an isotropic control
    of the same boundary density and a blocky (×4 nearest) control."""
    c = np.where(np.isfinite(c), c, -9).astype(np.int32)
    ok, th = cat_boundary(c, mask, wrap)
    if ok.sum() < 200:
        return {"boundary_cells": int(ok.sum())}
    out = {"map": orient_stats(th[ok], np.ones(int(ok.sum())))}
    out["map"].update(run_shares(c, mask, wrap))
    # Control: the isotropic map whose boundary density is closest.
    best = None
    full = np.ones(control_shape, bool)
    for sigma in (1.5, 2, 3, 4, 6, 8, 12, 16, 24, 32):
        cm = control_map(control_shape, sigma)
        rs = run_shares(cm, full)
        if best is None or abs(math.log(rs["density"] / out["map"]["density"])) < best[0]:
            best = (abs(math.log(rs["density"] / out["map"]["density"])), sigma, cm, rs)
    _, sigma, cm, rs = best
    okc, thc = cat_boundary(cm, full)
    out["control"] = orient_stats(thc[okc], np.ones(int(okc.sum())))
    out["control"].update(rs)
    out["control"]["sigma"] = sigma
    bm = control_map(control_shape, sigma * 4, blocky=4)
    okb, thb = cat_boundary(bm, full)
    out["blocky_control"] = orient_stats(thb[okb], np.ones(int(okb.sum())))
    out["blocky_control"].update(run_shares(bm, full))
    return out


# ---------------------------------------------------------------- pictures


def hillshade(h, zscale=1.0):
    gx = np.gradient(h, axis=1) * zscale
    gz = np.gradient(h, axis=0) * zscale
    # Light from the north-west, 45 deg up.
    lx, lz, ly = -0.5, -0.5, 0.707
    n = 1.0 / np.sqrt(gx * gx + gz * gz + 1.0)
    s = (-gx * lx - gz * lz + ly) * n
    return np.clip(s, 0, 1)


def save_shade(path, h, sea=None, scale=1, extra=None):
    s = hillshade(h)
    rgb = np.stack([s * 235, s * 228, s * 205], axis=-1)
    if sea is not None:
        wet = h < sea
        rgb[wet] = np.stack([s * 60 + 20, s * 110 + 40, s * 150 + 70], axis=-1)[wet]
    if extra is not None:
        extra(rgb)
    img = Image.fromarray(np.clip(rgb, 0, 255).astype(np.uint8))
    if scale != 1:
        img = img.resize((img.width * scale, img.height * scale), Image.NEAREST)
    img.save(path)


PALETTE = np.array(
    [[(37 * k + 60) % 256, (91 * k + 30) % 256, (151 * k + 90) % 256] for k in range(256)],
    dtype=np.uint8,
)


def save_cat(path, c, scale=1):
    idx = np.where(np.isfinite(c), c, 255).astype(np.int64) % 256
    img = Image.fromarray(PALETTE[idx])
    if scale != 1:
        img = img.resize((img.width * scale, img.height * scale), Image.NEAREST)
    img.save(path)


def save_gray(path, f, lo=None, hi=None, scale=1):
    lo = np.nanmin(f) if lo is None else lo
    hi = np.nanmax(f) if hi is None else hi
    g = np.clip((f - lo) / max(hi - lo, 1e-9), 0, 1) * 255
    img = Image.fromarray(np.nan_to_num(g).astype(np.uint8))
    if scale != 1:
        img = img.resize((img.width * scale, img.height * scale), Image.NEAREST)
    img.save(path)


# ---------------------------------------------------------------- inputs


class Planet:
    def __init__(self, arrays, world):
        self.arrays = arrays
        self.world = world
        self.manifest = json.load(open(os.path.join(world, "manifest.json")))
        sc = self.manifest["scale"]
        self.C = sc["circumference_blocks"]
        self.sea = sc["sea_level_y"]
        self.a = np.load(os.path.join(arrays, "planet.npy"), mmap_mode="r")
        self.names = json.load(open(os.path.join(arrays, "planet.npy.json")))["channels"]
        self.step = 2
        self._cache = {}

    def ch(self, name, sub=1):
        key = (name, sub)
        if key not in self._cache:
            k = self.names.index(name)
            self._cache[key] = np.array(self.a[::sub, ::sub, k])
        return self._cache[key]

    def drop(self, name, sub=1):
        self._cache.pop((name, sub), None)

    def lat_of_row(self, j, cell_blocks):
        z = -self.C / 4 + j * cell_blocks
        return -360.0 * z / self.C


def read_rivers(world):
    p = os.path.join(world, "rivers", "rivers.dtrv")
    if not os.path.exists(p):
        return None
    b = open(p, "rb").read()
    assert b[:4] == b"DTRV"
    (c, tile, tx, tz, nv, nk, ne) = struct.unpack_from("<7I", b, 16)
    table = 16 + 48
    secs = [struct.unpack_from("<QII", b, table + 16 * k) for k in range(4)]
    v = np.frombuffer(b, dtype="<f4", count=nv * 6, offset=secs[0][0]).reshape(nv, 6)
    ch = np.frombuffer(b, dtype="<u4", count=nk * 4, offset=secs[1][0]).reshape(nk, 4)
    return {"C": c, "v": v, "chains": ch}


# ---------------------------------------------------------------- the parts


def part_elevation(P, out):
    say("== elevation (L3 points, every 2nd block)")
    E = P.ch("elevation")
    land = E >= P.sea
    h, w = E.shape
    lat = np.abs(P.lat_of_row(np.arange(h), 2))[:, None] * np.ones((1, w))
    play = lat < 75.0
    put("elevation/land_share", float(land.mean()))
    for s in (1, 2, 4, 8, 16):
        st = orient_field(E, land & play, s)
        put(f"elevation/orientation/land_step{s * 2}blocks", rnd(st, 5))
        say(f"  slope orientation, step {s * 2:3d} blocks: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} "
            f"axis±2° {st['axis_pm2']:.4f} diag±2° {st['diag_pm2']:.4f} (isotropic 0, 0, 0.0444, 0.0444); "
            f"tiles a4 {st.get('a4_tile_mean', float('nan')):+.4f} ± {st.get('a4_tile_se', float('nan')):.4f}")
    for lo, hi in ((0, 30), (30, 60), (60, 75), (75, 90)):
        m = land & (lat >= lo) & (lat < hi)
        st = orient_field(E, m, 1)
        if st:
            put(f"elevation/orientation/by_latitude/{lo}-{hi}", rnd(st, 5))
            say(f"  |lat| {lo}-{hi}: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} axis±2° {st['axis_pm2']:.4f}")
    # Which axis: the E-W share of the slope direction by latitude (pole stretch).
    gx, gz = gradient(E.astype(np.float64), 1)
    for lo, hi in ((0, 30), (30, 60), (60, 75), (75, 90)):
        m = land & (lat >= lo) & (lat < hi) & np.isfinite(gz)
        if m.sum():
            ex, ez = float((gx[m] ** 2).mean()), float((gz[m] ** 2).mean())
            put(f"elevation/gradient_energy/{lo}-{hi}", {"x": ex, "z": ez, "x_over_z": ex / ez})
            say(f"  |lat| {lo}-{hi}: <gx²>/<gz²> = {ex / ez:.3f} (1 for an isotropic field in block space)")
    del gx, gz
    pt = phase_test(E, land & play)
    put("elevation/phase", pt)
    put("elevation/phase_summary", phase_summary(pt))
    say("  phase excess (max/min − 1) by period in L3 cells:", json.dumps(phase_summary(pt)))
    sp = spectrum_bands(E, land & play, 2)
    put("elevation/spectrum", rnd(sp, 5))
    for band, v in sp.items():
        if band != "tiles":
            say(f"  spectrum {band:>8} blocks: a4 {v['a4']:+.4f} ± {v['a4_se'] or 0:.4f}  a8 {v['a8']:+.4f} ± {v['a8_se'] or 0:.4f}")
    sc = outlier_scan(E)
    put("elevation/outliers", sc)
    say("  outlier scan:", json.dumps(sc))
    ss = seam_step(E)
    put("elevation/seam", rnd(ss, 5))
    say("  seam:", json.dumps(rnd(ss, 4)))
    # Coast: the slope's direction at the sea-level contour.
    coast = np.zeros(E.shape, bool)
    for a, b in ((1, 0), (0, 1)):
        nb = np.roll(np.roll(land, -a, axis=1), -b, axis=0)
        coast |= land != nb
    coast &= play
    for s in (1, 2, 4, 8, 16):
        st = orient_field(E, coast, s, weight="one", min_mag=2.0)
        put(f"coast/orientation/step{s * 2}blocks", rnd(st, 5))
        say(f"  coast normal, step {s * 2:3d} blocks: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} "
            f"axis±2° {st['axis_pm2']:.4f} (N-S lines {st['ns_lines_pm2']:.4f}, E-W lines {st['ew_lines_pm2']:.4f}) diag±2° {st['diag_pm2']:.4f} n {st['n']}")
    ct = cat_test(land.astype(np.float32)[::1, ::1][1024:3072, :4096], np.ones((2048, 4096), bool), wrap=False)
    put("coast/land_sea_map_L3", rnd(ct, 5))
    say("  land/sea map (L3 cells) straight runs:", json.dumps(rnd(ct, 4)))


def part_detail(P, out):
    say("== detail term A·N")
    E = P.ch("elevation")
    AN = P.ch("detail")
    A = P.ch("detail/amplitude")
    land = E >= P.sea
    m = land & (A >= 1.0)
    put("detail/land_share_A_ge_1_block", float(m.sum() / max(land.sum(), 1)))
    put("detail/land_share_A_ge_3_blocks", float((land & (A >= 3.0)).sum() / max(land.sum(), 1)))
    put("detail/A_land_mean", float(A[land].mean()))
    put("detail/abs_AN_land_p50_p90_p99", rnd(np.percentile(np.abs(AN[land]), [50, 90, 99]).tolist()))
    N = np.where(A > 0.25, AN / np.maximum(A, 1e-6), 0.0)
    # The lattice: x = −C/2 + 2i is a multiple of 32 when i mod 16 == 0; z = −C/4 + 2j likewise.
    pt = phase_test(N, m, periods=(4, 8, 16, 3, 5, 7))
    put("detail/phase_N", pt)
    put("detail/phase_N_summary", phase_summary(pt))
    say("  N phase excess by period in L3 cells (16 = the 32-block lattice):", json.dumps(phase_summary(pt)))
    st = orient_field(N, m, 1)
    put("detail/orientation_N", rnd(st, 5))
    say(f"  N gradient orientation: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} axis±2° {st['axis_pm2']:.4f} diag±2° {st['diag_pm2']:.4f}")
    # How much of the block-scale relief the detail is.
    dE = np.abs(np.roll(E, -1, axis=1) - E)[m].mean()
    dAN = np.abs(np.roll(AN, -1, axis=1) - AN)[m].mean()
    put("detail/mean_step_2blocks", {"E": float(dE), "AN": float(dAN)})
    say(f"  mean |step over 2 blocks| where A ≥ 1: E {dE:.3f}, A·N {dAN:.3f} blocks; land with A ≥ 1: {R['detail']['land_share_A_ge_1_block']:.3f}")


def part_water(P, out):
    say("== lakes and rivers")
    E = P.ch("elevation")
    L = P.ch("water_level")
    wet = np.isfinite(L)
    put("lakes/cells", int(wet.sum()))
    # Shore pairs: a lake cell beside a dry cell. Cut by the mask when the dry cell's ground is
    # more than a block under the lake's surface (the shore is then not the terrain's contour).
    cut = tot = 0
    axis_edges = {"x": 0, "z": 0}
    for a, b, name in ((1, 0, "x"), (-1, 0, "x"), (0, 1, "z"), (0, -1, "z")):
        dry_nb = ~np.roll(np.roll(wet, -a, axis=1), -b, axis=0)
        e_nb = np.roll(np.roll(E, -a, axis=1), -b, axis=0)
        pair = wet & dry_nb & (e_nb >= P.sea - 1)
        tot += int(pair.sum())
        cut += int((pair & (e_nb < L - 1.0)).sum())
        axis_edges[name] += int(pair.sum())
    put("lakes/shore_pairs", tot)
    put("lakes/shore_cut_by_mask_share", cut / max(tot, 1))
    say(f"  lake shore pairs {tot}, cut by the mask (dry ground > 1 block under the level): {cut / max(tot, 1):.4f}")
    shore = wet & ~ndimage.binary_erosion(wet, structure=np.ones((3, 3), bool))
    for s in (1, 4):
        st = orient_field(E, shore, s, weight="one", min_mag=2.0)
        if st:
            put(f"lakes/shore_orientation/step{2 * s}blocks", rnd(st, 5))
            say(f"  lake shore normal, step {2 * s} blocks: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} axis±2° {st['axis_pm2']:.4f} diag±2° {st['diag_pm2']:.4f} n {st['n']}")
    rv = read_rivers(P.world)
    if rv is None:
        say("  no river file")
        return
    v, chains = rv["v"], rv["chains"]
    put("rivers/vertices", int(len(v)))
    put("rivers/chains", int(len(chains)))
    for stride in (1, 4, 16, 64):
        th, wl = [], []
        for first, count, _, _ in chains:
            x = v[first : first + count, 0].astype(np.float64)
            z = v[first : first + count, 1].astype(np.float64)
            if count <= stride:
                continue
            dx, dz = x[stride:] - x[:-stride], z[stride:] - z[:-stride]
            ln = np.hypot(dx, dz)
            ok = ln > 0
            th.append(np.arctan2(dz[ok], dx[ok]))
            wl.append(ln[ok])
        th, wl = np.concatenate(th), np.concatenate(wl)
        st = orient_stats(th, wl)
        deg = np.degrees(th) % 45.0
        off = np.minimum(deg, 45.0 - deg)
        st["d8_pm2"] = float(wl[off <= 2].sum() / wl.sum())  # isotropic 0.0889
        st["d8_pm0_5"] = float(wl[off <= 0.5].sum() / wl.sum())  # isotropic 0.0222
        put(f"rivers/heading/stride{stride}", rnd(st, 5))
        say(f"  river heading over {stride:2d} segments: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} "
            f"within 2° of a D8 direction {st['d8_pm2']:.4f} (isotropic 0.0889), within 0.5° {st['d8_pm0_5']:.4f} (0.0222)")
        if stride == 1:
            hist, _ = np.histogram(np.degrees(th) % 180.0, bins=36, range=(0, 180), weights=wl)
            put("rivers/heading_hist_5deg", rnd((hist / hist.sum()).tolist(), 4))
    # Straight runs and sinuosity.
    runs = []
    sinu = []
    total = 0.0
    for first, count, _, _ in chains:
        x = v[first : first + count, 0].astype(np.float64)
        z = v[first : first + count, 1].astype(np.float64)
        dx, dz = np.diff(x), np.diff(z)
        ln = np.hypot(dx, dz)
        total += ln.sum()
        hd = np.degrees(np.arctan2(dz, dx))
        start = 0
        for k in range(1, len(hd) + 1):
            if k == len(hd) or abs((hd[k] - hd[start] + 180) % 360 - 180) > 3.0:
                runs.append(ln[start:k].sum())
                start = k
        cum = np.concatenate([[0], np.cumsum(ln)])
        # Sinuosity over windows of about 128 blocks of path.
        k0 = 0
        while k0 < len(x) - 1:
            k1 = int(np.searchsorted(cum, cum[k0] + 128.0))
            if k1 >= len(x):
                break
            chord = math.hypot(x[k1] - x[k0], z[k1] - z[k0])
            if chord > 0:
                sinu.append((cum[k1] - cum[k0]) / chord)
            k0 = k1
    runs = np.array(runs)
    put("rivers/length_blocks", float(total))
    for L0 in (16, 32, 64, 128):
        put(f"rivers/share_in_straight_runs_ge_{L0}_blocks", float(runs[runs >= L0].sum() / total))
    put("rivers/longest_straight_run_blocks", float(runs.max()))
    sinu = np.array(sinu)
    put("rivers/sinuosity_128_blocks", {"p10": float(np.percentile(sinu, 10)), "p50": float(np.percentile(sinu, 50)), "p90": float(np.percentile(sinu, 90)), "share_below_1_02": float((sinu < 1.02).mean())})
    say(f"  rivers: {total:.0f} blocks; in straight runs (heading within 3°) ≥ 32 blocks {R['rivers']['share_in_straight_runs_ge_32_blocks']:.3f}, "
        f"≥ 64 {R['rivers']['share_in_straight_runs_ge_64_blocks']:.3f}, ≥ 128 {R['rivers']['share_in_straight_runs_ge_128_blocks']:.3f}; longest {runs.max():.0f}")
    say(f"  sinuosity over 128 blocks: p10 {np.percentile(sinu, 10):.3f} p50 {np.percentile(sinu, 50):.3f} p90 {np.percentile(sinu, 90):.3f}; share under 1.02: {(sinu < 1.02).mean():.3f}")


def part_climate(P, out):
    say("== biome inputs (L2), ice cover (L2), climate (L1)")
    E2 = P.ch("elevation", 4)
    land2 = E2 >= P.sea
    h, w = E2.shape
    lat = np.abs(P.lat_of_row(np.arange(h), 8))[:, None] * np.ones((1, w))
    play = lat < 75
    for name in ("climate/temperature", "climate/vegetation", "climate/continents", "climate/erosion", "climate/ridges"):
        f = P.ch(name, 4)
        m = land2 & play
        pt = phase_test(f, m, periods=(2, 4, 8, 3, 5, 7))
        st = orient_field(f, m, 1)
        rowmean = np.where(m, f, 0).sum(axis=1) / np.maximum(m.sum(axis=1), 1)
        fit = np.broadcast_to(rowmean[:, None], f.shape)
        var = float(f[m].var())
        r2 = 1.0 - float(((f - fit)[m] ** 2).mean()) / var if var > 0 else None
        vals = np.unique(f[m]).size
        put(f"biome_inputs/{name}", {"phase_summary": phase_summary(pt), "phase": pt, "orientation": rnd(st, 5), "latitude_r2": r2, "distinct_values": int(vals), "seam": rnd(seam_step(f), 5)})
        say(f"  {name}: phase excess {json.dumps(phase_summary(pt))}")
        say(f"      orientation a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} axis±2° {st['axis_pm2']:.4f} (N-S lines {st['ns_lines_pm2']:.4f}, E-W lines {st['ew_lines_pm2']:.4f}); latitude explains R² {r2 if r2 is None else round(r2, 3)}; distinct values {vals}; seam ratio {seam_step(f)['ratio']:.2f}")
    ice = P.ch("climate/ice_cover", 4)
    icy = ice > 0
    put("ice/land_share", float((icy & land2).sum() / max(land2.sum(), 1)))
    if icy.sum() > 500:
        ct = cat_test(icy.astype(np.float32), land2 & play)
        put("ice/edge", rnd(ct, 5))
        say("  ice cover edge (L2 cells):", json.dumps(rnd(ct, 4)))
        # Banding: the ice edge's rows.
        edge = icy & ~ndimage.binary_erosion(icy) & land2
        rows = edge.sum(axis=1)
        top = np.sort(rows)[::-1]
        put("ice/edge_cells", int(edge.sum()))
        put("ice/edge_share_in_top_8_rows", float(top[:8].sum() / max(edge.sum(), 1)))
        pt = phase_test(ice, land2 & play & icy, periods=(2, 4, 3, 5))
        put("ice/phase_summary", phase_summary(pt))
        say(f"  ice: land share {R['ice']['land_share']:.3f}; edge cells in the 8 busiest rows {R['ice']['edge_share_in_top_8_rows']:.3f}; phase {json.dumps(phase_summary(pt))}")
    E1 = P.ch("elevation", 8)
    land1 = E1 >= P.sea
    for name in ("climate/t_mean_c", "climate/p_mm"):
        f = P.ch(name, 8)
        m = land1 & np.isfinite(f)
        rowmean = np.where(m, f, 0).sum(axis=1) / np.maximum(m.sum(axis=1), 1)
        var = float(f[m].var())
        r2 = 1.0 - float(((f - rowmean[:, None])[m] ** 2).mean()) / var
        st = orient_field(np.nan_to_num(f), m, 1)
        put(f"climate_l1/{name}", {"latitude_r2": r2, "orientation": rnd(st, 5), "seam": rnd(seam_step(np.nan_to_num(f)), 5)})
        say(f"  {name}: latitude R² {r2:.3f}; a4 {st['a4']:+.4f}; axis±2° {st['axis_pm2']:.4f}; seam ratio {seam_step(np.nan_to_num(f))['ratio']:.2f}")
    k = P.ch("climate/koppen", 8)
    ct = cat_test(k, land1)
    put("climate_l1/koppen_boundaries", rnd(ct, 5))
    say("  Köppen boundaries (L1 cells):", json.dumps(rnd(ct, 4)))


def part_rock(P, out):
    say("== rock (the column rule every 8th block)")
    a = np.load(os.path.join(P.arrays, "columns8.npy"), mmap_mode="r")
    names = json.load(open(os.path.join(P.arrays, "columns8.npy.json")))["channels"]
    H = np.array(a[:, :, 0])
    land = H >= P.sea
    h, w = H.shape
    lat = np.abs(P.lat_of_row(np.arange(h), 8))[:, None] * np.ones((1, w))
    play = lat < 75
    for name in ("rock_top", "rock_8", "rock_32", "rock_96"):
        c = np.array(a[:, :, names.index(name)])
        ct = cat_test(c, land & play)
        put(f"rock/{name}", rnd(ct, 5))
        say(f"  {name}:", json.dumps(rnd(ct, 4)))
    st2 = np.array(a[:, :, names.index("second_top")])
    thick = H - st2
    m = land & play & np.isfinite(thick)
    # 8-block samples: period 2 = L1 (16 blocks), 8 = the smoothing lattice (64 blocks).
    pt = phase_test(np.nan_to_num(thick), m, periods=(2, 4, 8, 16, 3, 5, 7))
    put("rock/first_horizon_thickness_phase", pt)
    put("rock/first_horizon_thickness_phase_summary", phase_summary(pt))
    say("  first horizon's thickness, phase excess by period in 8-block samples (2 = L1, 8 = the 64-block smoothing lattice):", json.dumps(phase_summary(pt)))
    st = orient_field(np.nan_to_num(st2), m, 1, min_mag=30.0)
    put("rock/second_top_orientation_steep", rnd(st, 5))
    if st:
        say(f"  second horizon's top, slope orientation where it changes by 30 blocks and more in 16: a4 {st['a4']:+.4f} a8 {st['a8']:+.4f} axis±2° {st['axis_pm2']:.4f} n {st['n']}")
    sc = outlier_scan(np.nan_to_num(st2), m)
    put("rock/second_top_outliers", sc)
    say("  second top outlier scan:", json.dumps(sc))
    wk = np.array(a[:, :, names.index("water_kind")])
    put("rock/water_kind_counts", {str(int(k)): int((wk == k).sum()) for k in np.unique(wk)})
    # The layers nothing in the game reads yet (they would show if a surface rule did).
    for name, sub in (("surface/class_top", 4), ("meta/province", 8), ("meta/plate_id", 8)):
        c = P.ch(name, sub)
        E = P.ch("elevation", sub)
        lat2 = np.abs(P.lat_of_row(np.arange(E.shape[0]), 2 * sub))[:, None] * np.ones((1, E.shape[1]))
        ct = cat_test(c, (E >= P.sea) & (lat2 < 75))
        put(f"unread_layers/{name}", rnd(ct, 5))
        say(f"  {name} (not read by the game):", json.dumps(rnd(ct, 4)))


def part_points(P, out):
    say("== volcanoes and deposits")
    p = os.path.join(P.world, "volcanoes", "active.json")
    E = P.ch("elevation")
    if os.path.exists(p):
        vj = json.load(open(p))
        vs = vj["volcanoes"] if isinstance(vj, dict) else vj
        xs = np.array([[v.get("x", v.get("summit_x")), v.get("z", v.get("summit_z"))] for v in vs], dtype=np.float64)
        put("volcanoes/count", len(vs))
        for g in (2, 8, 16):
            on = float(((xs % g) == 0).all(axis=1).mean())
            put(f"volcanoes/summits_on_{g}_block_grid", on)
            say(f"  volcano summits on the {g}-block grid: {on:.3f} (chance {1 / g / g:.4f})")
        put("volcanoes/keys", sorted(vs[0].keys()) if vs else [])
        # Roundness: elevation on a ring at half the basal radius; the share of the azimuthal
        # variance in harmonics 4 and 8 (the grid's) among 1..12.
        m4 = []
        m8 = []
        h, w = E.shape
        for v in vs:
            x, z = v.get("x", v.get("summit_x")), v.get("z", v.get("summit_z"))
            r = 0.5 * float(v.get("radius_m", 0) or 0) / P.manifest["scale"]["m_per_block_h"]
            if r < 6:
                continue
            phi = np.linspace(0, 2 * np.pi, 96, endpoint=False)
            ii = np.round((x + r * np.cos(phi) + P.C / 2) / 2).astype(int) % w
            jj = np.clip(np.round((z + r * np.sin(phi) + P.C / 4) / 2).astype(int), 0, h - 1)
            ring = E[jj, ii].astype(np.float64)
            F = np.abs(np.fft.rfft(ring - ring.mean())) ** 2
            tot = F[1:13].sum()
            if tot > 0:
                m4.append(F[4] / tot)
                m8.append(F[8] / tot)
        if m4:
            put("volcanoes/ring_harmonic_share", {"m4_mean": float(np.mean(m4)), "m8_mean": float(np.mean(m8)), "n": len(m4), "chance": 1 / 12})
            say(f"  cone rings: harmonic 4 holds {np.mean(m4):.3f}, harmonic 8 {np.mean(m8):.3f} of the azimuthal variance (1..12; even share 0.083), n {len(m4)}")
    p = os.path.join(P.arrays, "deposits.json")
    if os.path.exists(p):
        d = json.load(open(p))
        sites = d["sites"]
        put("deposits/sites", len(sites))
        if sites:
            c = np.array([[s["center"][0], s["center"][2]] for s in sites], dtype=np.float64)
            for g in (2, 8, 16, 32):
                on = float(((c % g) == 0).all(axis=1).mean())
                put(f"deposits/site_centres_on_{g}_block_grid", on)
                say(f"  deposit site centres on the {g}-block grid: {on:.3f} (chance {1 / g / g:.4f}), n {len(sites)}")
            ph = (c % 16).astype(int)
            put("deposits/site_centre_phase16_hist_x", np.bincount(ph[:, 0], minlength=16).tolist())
            put("deposits/site_centre_phase16_hist_z", np.bincount(ph[:, 1], minlength=16).tolist())
        bodies = [b["body"] for b in d["bodies"]]
        put("deposits/bodies", len(bodies))
        if bodies:
            put("deposits/body_keys", sorted(bodies[0].keys()))
            bc = np.array([[b["center"][0], b["center"][2]] for b in bodies if "center" in b], dtype=np.float64)
            if len(bc):
                for g in (2, 8, 16):
                    on = float((np.abs(bc / g - np.round(bc / g)) < 1e-6).all(axis=1).mean())
                    put(f"deposits/body_centres_on_{g}_block_grid", on)
                    say(f"  deposit body centres on the {g}-block grid: {on:.3f}, n {len(bc)}")


def part_boxes(P, out):
    """Block-scale boxes (grid-audit.sh boxes): the same tests at one block, and pictures."""
    bj = os.path.join(P.arrays, "boxes.json")
    if not os.path.exists(bj):
        say("no boxes.json")
        return
    for name, box in json.load(open(bj)).items():
        p = os.path.join(P.arrays, f"box-{name}.npy")
        if not os.path.exists(p):
            continue
        a = np.load(p)
        names = json.load(open(p + ".json"))["channels"]
        g = lambda n: a[:, :, names.index(n)]
        H, E, AN = g("surface"), g("elevation"), g("detail")
        x0, z0 = box[0], box[1]
        land = H >= P.sea
        say(f"== box {name} {box}: land {land.mean():.3f}")
        res = {"box": box, "land": float(land.mean())}
        if land.mean() > 0.2:
            # Phases in absolute block coordinates: 16 = L1, 8 = L2, 32 = the detail lattice.
            org = (x0 + P.C // 2, z0 + P.C // 4)
            pt = phase_test(H, land, periods=(2, 4, 8, 16, 32, 3, 5, 7), origin=org)
            res["H_phase"] = phase_summary(pt)
            pt = phase_test(AN, land & (np.abs(AN) > 0), periods=(8, 16, 32, 5, 7), origin=org)
            res["AN_phase"] = phase_summary(pt)
            st = orient_field(H, land, 1)
            res["H_orientation"] = rnd(st, 5)
            say(f"  H phase {json.dumps(res['H_phase'])}; A·N phase {json.dumps(res['AN_phase'])}")
        for cn in ("rock_top", "rock_8", "rock_32", "water_kind"):
            c = g(cn)
            ct = cat_test(c, land if cn != "water_kind" else np.ones(c.shape, bool), wrap=False)
            res[cn] = rnd(ct, 5)
            if "map" in ct:
                say(f"  {cn}: a4 {ct['map']['a4']:+.3f} (control {ct['control']['a4']:+.3f}, blocky {ct['blocky_control']['a4']:+.3f}); "
                    f"runs ≥ 8: {ct['map']['run_ge8']:.3f} (control {ct['control']['run_ge8']:.3f}, blocky {ct['blocky_control']['run_ge8']:.3f}); "
                    f"≥ 16: {ct['map']['run_ge16']:.3f} (control {ct['control']['run_ge16']:.3f})")
            # Where the class changes fall inside an L1 cell (16 blocks).
            if cn.startswith("rock"):
                for axis, an in ((1, "x"), (0, "z")):
                    ch = (np.roll(c, -1, axis=axis) != c) & land
                    idx = (np.arange(c.shape[axis]) + (x0 if axis == 1 else z0)) % 16
                    cnt = np.array([ch.sum(axis=1 - axis)[idx == k].sum() for k in range(16)], dtype=float)
                    if cnt.sum() > 500:
                        res[f"{cn}_change_phase16_{an}"] = {"max_over_mean": float(cnt.max() / cnt.mean()), "min_over_mean": float(cnt.min() / cnt.mean())}
        put(f"boxes/{name}", res)
        d = os.path.join(out, "boxes")
        os.makedirs(d, exist_ok=True)
        save_shade(os.path.join(d, f"{name}-terrain.png"), H, P.sea, scale=2)
        save_gray(os.path.join(d, f"{name}-detail.png"), AN, -3, 3, scale=2)
        save_cat(os.path.join(d, f"{name}-rock_top.png"), g("rock_top"), scale=2)
        save_cat(os.path.join(d, f"{name}-rock_32.png"), g("rock_32"), scale=2)
        save_cat(os.path.join(d, f"{name}-water.png"), g("water_kind") * 40 + 3, scale=2)
        for cn in ("climate/temperature", "climate/ice_cover"):
            if cn in names:
                save_gray(os.path.join(d, f"{name}-{cn.split('/')[1]}.png"), g(cn), scale=2)


def load_field(P, folder, name):
    d = os.path.join(P.arrays, folder)
    idx = json.load(open(os.path.join(d, "index.json")))[name]
    a = np.fromfile(os.path.join(d, idx["file"]), dtype="<f4").reshape(idx["h"], idx["w"])
    return a, idx["r"]


def part_stages(P, out):
    """The same tests on the terrain of each stage, to say which stage a pattern comes from."""
    say("== stages: the terrain after each stage")
    sea_m = P.manifest["notes"]["surface"]["sea_level_m"]
    amp = [f for f in os.listdir(P.arrays) if f.startswith("fields-") and "amplify" in f][0]
    rows = [
        ("plates (L1)", "plates/elevation_m"),
        ("surface (L1)", "surface/elevation_m"),
        ("young (L2)", "young/elevation_m"),
    ]
    for label, name in rows:
        f, r = load_field(P, amp, name)
        land = f > sea_m
        lat = np.abs(P.lat_of_row(np.arange(f.shape[0]), r))[:, None] * np.ones((1, f.shape[1]))
        m = land & (lat < 75)
        res = {}
        for s in (1, 2, 4):
            st = orient_field(f, m, s)
            res[f"slope_step{2 * s * r}blocks"] = rnd(st, 5)
        coast = np.zeros(f.shape, bool)
        for a, b in ((1, 0), (0, 1)):
            coast |= land != np.roll(np.roll(land, -a, axis=1), -b, axis=0)
        coast &= lat < 75
        for s in (1, 2):
            st = orient_field(f, coast, s, weight="one", min_mag=1.0)
            res[f"coast_step{2 * s * r}blocks"] = rnd(st, 5)
        ct = cat_test(land.astype(np.float32), lat < 75)
        res["land_sea_map"] = rnd(ct, 5)
        res["phase"] = phase_summary(phase_test(f, m, periods=(2, 4, 8, 3, 5, 7)))
        put(f"stages/{label}", res)
        a = res[f"slope_step{2 * r}blocks"]
        c = res[f"coast_step{2 * r}blocks"]
        say(f"  {label:13s} slope a4 {a['a4']:+.4f} a8 {a['a8']:+.4f} axis±2° {a['axis_pm2']:.4f} | coast a4 {c['a4']:+.4f} a8 {c['a8']:+.4f} axis±2° {c['axis_pm2']:.4f} diag±2° {c['diag_pm2']:.4f} | "
            f"land/sea runs ≥ 8: {ct['map']['run_ge8']:.3f} (control {ct['control']['run_ge8']:.3f}, blocky {ct['blocky_control']['run_ge8']:.3f})")
        say(f"                phase {json.dumps(res['phase'])}")
    # The final terrain, subsampled to the L1 and L2 points: what the amplifier kept of them.
    E = P.ch("elevation")
    for sub, label in ((8, "final at the L1 points"), (4, "final at the L2 points")):
        f = E[::sub, ::sub]
        land = f >= P.sea
        ct = cat_test(land.astype(np.float32), np.ones(f.shape, bool))
        put(f"stages/{label}", {"land_sea_map": rnd(ct, 5)})
        say(f"  {label}: land/sea runs ≥ 8: {ct['map']['run_ge8']:.3f} (control {ct['control']['run_ge8']:.3f})")


def part_pick(P, out):
    """Overview pictures and the block-scale boxes worth looking at (boxes.json)."""
    say("== overview and boxes")
    E = P.ch("elevation")
    h, w = E.shape
    C = P.C
    half = 256  # a box is 512 × 512 blocks

    def box_at(i, j, cell):
        x = -C // 2 + int(i) * cell
        z = -C // 4 + int(j) * cell
        z = max(-C // 4 + half, min(C // 4 - half, z))
        return [x - half, z - half, x + half, z + half]

    boxes = {}
    land = E >= P.sea
    lat = np.abs(P.lat_of_row(np.arange(h), 2))
    # Tiles of 256 L3 cells (512 blocks).
    T = 256
    nj, ni = h // T, w // T
    tl = land[: nj * T, : ni * T].reshape(nj, T, ni, T).mean(axis=(1, 3))
    gx, gz = gradient(E.astype(np.float64), 1)
    slope = np.nan_to_num(np.hypot(gx, gz))
    ts = np.where(land, slope, 0)[: nj * T, : ni * T].reshape(nj, T, ni, T).mean(axis=(1, 3))
    tlat = lat[: nj * T].reshape(nj, T).mean(axis=1)[:, None] * np.ones((1, ni))
    play = tlat < 70

    def best(score, ok, tile_blocks=2 * T):
        sc = np.where(ok, score, -np.inf)
        j, i = np.unravel_index(np.argmax(sc), sc.shape)
        x0, z0 = -C // 2 + i * tile_blocks, -C // 4 + j * tile_blocks
        return [int(x0), int(z0), int(x0 + 512), int(z0 + 512)]

    boxes["mountain"] = best(ts, play & (tl > 0.95))
    boxes["plain"] = best(-ts, play & (tl > 0.98) & (tlat < 45))
    boxes["coast"] = best(-np.abs(tl - 0.5), play & (tlat < 50))
    boxes["highlat"] = best(ts, (tlat > 62) & (tlat < 72) & (tl > 0.8))
    # The seam: a row with land at the first and last columns.
    seam_land = land[:, 0] & land[:, -1] & (lat < 70)
    if seam_land.any():
        rows = np.nonzero(seam_land)[0]
        j = int(rows[len(rows) // 2])
        boxes["seam"] = box_at(0, j, 2)
    # Rivers: the longest straight run.
    rv = read_rivers(P.world)
    if rv is not None:
        v = rv["v"]
        bestrun = (0, 0, 0)
        for first, count, _, _ in rv["chains"]:
            x = v[first : first + count, 0].astype(np.float64)
            z = v[first : first + count, 1].astype(np.float64)
            hd = np.degrees(np.arctan2(np.diff(z), np.diff(x)))
            ln = np.hypot(np.diff(x), np.diff(z))
            start = 0
            for k in range(1, len(hd) + 1):
                if k == len(hd) or abs((hd[k] - hd[start] + 180) % 360 - 180) > 3.0:
                    L = ln[start:k].sum()
                    if L > bestrun[0] and abs(z[start]) < C / 4 * 70 / 90:
                        bestrun = (L, 0.5 * (x[start] + x[k]), 0.5 * (z[start] + z[k]))
                    start = k
        if bestrun[0] > 0:
            x = int(((bestrun[1] + C / 2) % C) - C / 2)
            boxes["river"] = [x - half, int(bestrun[2]) - half, x + half, int(bestrun[2]) + half]
            put("rivers/longest_straight_run_at", [x, int(bestrun[2]), float(bestrun[0])])
    # Lakes: the tile with the most lake shore.
    L = P.ch("water_level")
    wet = np.isfinite(L)
    shore = wet & ~ndimage.binary_erosion(wet, structure=np.ones((3, 3), bool))
    tsh = shore[: nj * T, : ni * T].reshape(nj, T, ni, T).sum(axis=(1, 3))
    boxes["lake"] = best(tsh.astype(float), play)
    # Biome borders: where the temperature input's steps run along the axes.
    t = P.ch("climate/temperature", 4)
    tx, tz = gradient(t.astype(np.float64), 1)
    mag = np.nan_to_num(np.hypot(tx, tz))
    th = np.degrees(np.arctan2(np.nan_to_num(tz), tx)) % 90.0
    axis = (np.minimum(th, 90 - th) <= 2) & (mag > 0.05)
    land2 = E[::4, ::4] >= P.sea
    T2 = 64
    n2j, n2i = h // 4 // T2, w // 4 // T2
    ta = (axis & land2)[: n2j * T2, : n2i * T2].reshape(n2j, T2, n2i, T2).sum(axis=(1, 3))
    boxes["biome"] = best(ta.astype(float), play & (tl > 0.9))
    # Ice edge.
    ice = P.ch("climate/ice_cover", 4) > 0
    edge = ice & ~ndimage.binary_erosion(ice) & land2
    te = edge[: n2j * T2, : n2i * T2].reshape(n2j, T2, n2i, T2).sum(axis=(1, 3))
    boxes["ice"] = best(te.astype(float), (tlat < 72) & (tl > 0.8))
    # Rock: the column of the second horizon's top that stands out most, at its busiest row.
    a = np.load(os.path.join(P.arrays, "columns8.npy"), mmap_mode="r")
    names = json.load(open(os.path.join(P.arrays, "columns8.npy.json")))["channels"]
    st2 = np.nan_to_num(np.array(a[:, :, names.index("second_top")]))
    H8 = np.array(a[:, :, 0])
    d2 = np.abs(np.roll(st2, -1, axis=1) - 2 * st2 + np.roll(st2, 1, axis=1))
    d2[H8 < P.sea] = 0
    col = R.get("rock", {}).get("second_top_outliers", {}).get("columns", {}).get("at")
    if col is not None:
        prof = ndimage.uniform_filter1d(d2[:, col], 32)
        j = int(np.argmax(prof))
        boxes["rock_line"] = box_at(col, j, 8)
    # A volcano on land: the tallest.
    p = os.path.join(P.world, "volcanoes", "active.json")
    if os.path.exists(p):
        vs = [v for v in json.load(open(p))["volcanoes"] if not v.get("oceanic")]
        if vs:
            v = max(vs, key=lambda v: v["height_m"])
            boxes["volcano"] = [int(v["x"]) - half, int(v["z"]) - half, int(v["x"]) + half, int(v["z"]) + half]
    json.dump(boxes, open(os.path.join(P.arrays, "boxes.json"), "w"), indent=1)
    put("boxes_picked", boxes)
    say("  boxes:", json.dumps(boxes))
    # Overviews (2048 × 1024, 8 blocks a pixel).
    d = os.path.join(out, "overview")
    os.makedirs(d, exist_ok=True)

    def mark(rgb):
        for b in boxes.values():
            i0, j0 = (b[0] + C // 2) // 8, (b[1] + C // 4) // 8
            i1, j1 = (b[2] + C // 2) // 8, (b[3] + C // 4) // 8
            cols = np.arange(i0, i1) % 2048
            rgb[j0:j1, i0 % 2048] = (255, 0, 0)
            rgb[j0:j1, (i1 - 1) % 2048] = (255, 0, 0)
            rgb[j0, cols] = (255, 0, 0)
            rgb[j1 - 1, cols] = (255, 0, 0)

    save_shade(os.path.join(d, "terrain.png"), E[::4, ::4] / 4.0, P.sea / 4.0, extra=mark)
    save_gray(os.path.join(d, "temperature.png"), t)
    save_gray(os.path.join(d, "vegetation.png"), P.ch("climate/vegetation", 4))
    save_gray(os.path.join(d, "continents.png"), P.ch("climate/continents", 4))
    save_gray(os.path.join(d, "erosion.png"), P.ch("climate/erosion", 4))
    save_gray(os.path.join(d, "ridges.png"), P.ch("climate/ridges", 4))
    save_gray(os.path.join(d, "ice_cover.png"), P.ch("climate/ice_cover", 4))
    save_gray(os.path.join(d, "second_top.png"), np.where(H8 >= P.sea, st2, np.nan), -352, 400)
    save_cat(os.path.join(d, "rock_top.png"), np.array(a[:, :, names.index("rock_top")]))
    save_cat(os.path.join(d, "rock_32.png"), np.array(a[:, :, names.index("rock_32")]))
    save_cat(os.path.join(d, "koppen.png"), P.ch("climate/koppen", 4))


PARTS = {
    "elevation": part_elevation,
    "detail": part_detail,
    "water": part_water,
    "climate": part_climate,
    "rock": part_rock,
    "points": part_points,
    "stages": part_stages,
    "pick": part_pick,
    "boxes": part_boxes,
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--world", required=True)
    ap.add_argument("--arrays", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("parts", nargs="*")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    P = Planet(a.arrays, a.world)
    parts = a.parts or [p for p in PARTS if p not in ("boxes", "pick")]
    path = os.path.join(a.out, "results.json")
    if os.path.exists(path):
        R.update(json.load(open(path)))
    for p in parts:
        PARTS[p](P, a.out)
        json.dump(R, open(path, "w"), indent=1, default=rnd)
    say("results:", path)


if __name__ == "__main__":
    sys.exit(main())
