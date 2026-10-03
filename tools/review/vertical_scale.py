#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The vertical-scale study's numbers and pictures (docs/research/notes/vertical-scale-study.md).

Run through tools/review/vertical-scale.sh on the test VM (numpy, scipy, matplotlib, pillow).

    join    --work W --name N --world-data WD     strips of `deeptime heightmap` -> W/N.h16.npy (terrain
                                                  height H x 16, int16), W/N.wl16.npy (lake level x 16),
                                                  W/N.json (the manifest's scale)
    refjoin --work W --name N TILE.npy…           squares of `deeptime-inspect heights` -> W/N.ref.npy, side by
                                                  side with a no-data column between them
    stats   --work W --planet N [--scales …] [--ref name=label]… --out F.json
                                                  slope, relief and coast statistics of the planet at each
                                                  vertical scale and of each reference world (W/<name>.ref.npy)
    places  --work W --planet N --world-data WD --out F.json    the places the pictures show
    render  --work W --planet N --places F.json [--ref name=label]… --out DIR
                                                  oblique views and profiles of each place at each scale
    strata  --work W --planet N --world-data WD --bin deeptime --out F.json
                                                  rock-layer thickness in blocks down sampled land columns (the
                                                  column evaluator), and what falls under one block at each scale
    charts  --stats F.json [--more G.json=Label]… --out DIR              the comparison charts
    page    --stats F.json --pictures DIR/pictures.json [--more …] [--strata S.json] --out DIR    index.html

A vertical scale other than the bake's is applied to the baked terrain height H (blocks, the game's
density zero) as H' = sea + (H - sea) * mv0 / mv: the same planet in metres, drawn at mv metres per
block. `compare` checks that against a planet really baked at another scale.

Definitions (every world, in block space):
- top block T = ceil(H) - 1 (the format's `top_y`); a reference world's T is its highest terrain
  block (trees, plants, snow layers and fluids skipped).
- the sea fills blocks up to SEA_TOP = sea_level_y - 1; a column is wet when water stands on it (sea,
  lake); land is every other column. The visible surface V is T on land and the water's top block on
  wet columns.
- slope: the gradient of V by central differences (run 2 blocks), as an angle; hillside slope: the
  same over a run of 16 blocks.
- step: the largest difference of V to the four neighbours.
- relief within w: max - min of V in the w x w square around a column.
- coast: land columns touching the open sea (sea-level water after an opening with a 15-block square,
  so rivers and creeks at sea level are not coast); their cliff height is the highest V within 8
  blocks, above the sea surface.
"""

import argparse
import json
import math
import os
import sys

import numpy as np

NODATA = -32768
SCALES = [20, 25, 30, 40, 50, 60, 80]


# ----------------------------------------------------------------------------------------------
# Data
# ----------------------------------------------------------------------------------------------

def join(a):
    m = json.load(open(os.path.join(a.world_data, "manifest.json")))
    sc = m["scale"]
    c = sc["circumference_blocks"]
    h = c // 2
    h16 = np.lib.format.open_memmap(os.path.join(a.work, a.name + ".h16.npy"), mode="w+", dtype="<i2", shape=(h, c))
    wl16 = np.lib.format.open_memmap(os.path.join(a.work, a.name + ".wl16.npy"), mode="w+", dtype="<i2", shape=(h, c))
    for row in range(0, h, 1024):
        s = np.load(os.path.join(a.work, f"{a.name}.strip.{row}.npy"))
        surf = s[:, :, 0]
        h16[row:row + s.shape[0]] = np.clip(np.rint(surf * 16), -32767, 32767).astype("<i2")
        w = s[:, :, 3]
        wl16[row:row + s.shape[0]] = np.where(np.isfinite(w), np.clip(np.rint(w * 16), -32767, 32767), NODATA).astype("<i2")
    h16.flush()
    wl16.flush()
    meta = {"scale": sc, "source": m.get("source"), "created_utc": m.get("created_utc"),
            "notes_surface": {k: m.get("notes", {}).get("surface", {}).get(k) for k in ("land_frac", "max_elevation_m", "sea_level_m")},
            "young": m.get("notes", {}).get("young", {}).get("after")}
    json.dump(meta, open(os.path.join(a.work, a.name + ".json"), "w"), indent=1)
    print(f"join: {h} x {c} -> {a.name}.h16.npy")


def refjoin(a):
    tiles = [np.load(t) for t in a.tiles]
    rows = max(t.shape[0] for t in tiles)
    cols = sum(t.shape[1] for t in tiles) + len(tiles) - 1
    out = np.full((rows, cols, 2), NODATA, np.int16)
    c = 0
    for t in tiles:
        out[:t.shape[0], c:c + t.shape[1]] = t
        c += t.shape[1] + 1
    np.save(os.path.join(a.work, a.name + ".ref.npy"), out)
    full = int((out[:, :, 0] != NODATA).sum())
    print(f"refjoin: {len(tiles)} squares, {full} columns with terrain -> {a.name}.ref.npy")


class Planet:
    def __init__(self, work, name):
        self.meta = json.load(open(os.path.join(work, name + ".json")))
        sc = self.meta["scale"]
        self.c = sc["circumference_blocks"]
        self.sea_y = sc["sea_level_y"]
        self.mv0 = sc["m_per_block_v"]
        self.mh = sc["m_per_block_h"]
        self.h16 = np.load(os.path.join(work, name + ".h16.npy"), mmap_mode="r")
        self.wl16 = np.load(os.path.join(work, name + ".wl16.npy"), mmap_mode="r")
        self.rows = self.h16.shape[0]
        self.name = name

    def lat_rows(self, cap_deg=75.0):
        """Row range inside the pole cap: z = -lat / 360 * C."""
        half = int(cap_deg / 360.0 * self.c)
        mid = self.rows // 2
        return mid - half, mid + half

    def box(self, x0, z0, x1, z1):
        """H and lake level (blocks, float) of block box x0..x1, z0..z1 (world coordinates; x wraps)."""
        r0, r1 = z0 + self.rows // 2, z1 + self.rows // 2
        cols =(np.arange(x0, x1) + self.c // 2) % self.c
        h = self.h16[r0:r1][:, cols].astype(np.float32) / 16.0
        wl = self.wl16[r0:r1][:, cols]
        w = np.where(wl == NODATA, np.nan, wl.astype(np.float32) / 16.0)
        return h, w


def surfaces(h, wl, sea_y, mv0, mv, warp=None):
    """Top block T, visible surface V, wet and sea masks of terrain height h (blocks at mv0) drawn at
    mv metres per block (or through `warp`, metres above the sea -> blocks above the sea)."""
    if warp is not None:
        f = lambda v: sea_y + warp((v - sea_y) * mv0)
    else:
        k = mv0 / mv
        f = lambda v: sea_y + (v - sea_y) * k
    hs = f(h)
    t = (np.ceil(hs) - 1).astype(np.int16)
    sea_top = sea_y - 1
    sea = t < sea_top
    v = np.where(sea, sea_top, t).astype(np.int16)
    wet = sea.copy()
    if wl is not None:
        lake_top = np.ceil(f(wl)) - 1
        lake = np.isfinite(wl) & (lake_top > t)
        v = np.where(lake, lake_top, v).astype(np.int16)
        wet |= lake
    return t, v, wet, sea


def ref_surfaces(arr):
    t = arr[:, :, 0]
    w = arr[:, :, 1]
    valid = t != NODATA
    wet = valid & (w != NODATA) & (w > t)
    v = np.where(wet, w, t).astype(np.int16)
    return t, v, wet, valid


# ----------------------------------------------------------------------------------------------
# Statistics
# ----------------------------------------------------------------------------------------------

def pct(v, qs=(50, 90, 99)):
    if v.size == 0:
        return [float("nan")] * len(qs)
    return [float(x) for x in np.percentile(v, qs)]


def deg(g):
    return math.degrees(math.atan(g))


def measure(v, land, sea, valid, sea_top, wrap):
    """The statistics of one surface. v int16 visible surface; land/sea/valid masks."""
    from scipy import ndimage
    out = {}
    vf = v.astype(np.float32)
    mode = "wrap" if wrap else "nearest"

    def shift(a, d, axis):
        if axis == 1 and wrap:
            return np.roll(a, d, axis=1)
        r = np.roll(a, d, axis=axis)
        return r

    def interior(margin):
        """Columns whose square of half-width `margin` is inside the data."""
        m = valid.copy()
        if margin > 0:
            m[:margin] = False
            m[-margin:] = False
            if not wrap:
                m[:, :margin] = False
                m[:, -margin:] = False
            if not valid.all():
                m &= ndimage.minimum_filter(valid.view(np.uint8), size=2 * margin + 1, mode="constant", cval=0) > 0
        return m

    n_land = int(land.sum())
    out["columns"] = int(valid.sum())
    out["land_columns"] = n_land
    out["land_share"] = n_land / max(1, int(valid.sum()))

    # Slope (run 2) and step.
    for run, key in ((1, "slope"), (8, "hillside")):
        gx = (shift(vf, -run, 1) - shift(vf, run, 1)) / (2 * run)
        gz = (shift(vf, -run, 0) - shift(vf, run, 0)) / (2 * run)
        g = np.hypot(gx, gz)
        del gx, gz
        m = land & interior(run)
        gv = g[m]
        del g
        p = pct(gv, (50, 90, 99, 75, 95))
        out[key] = {
            "median_deg": deg(p[0]), "p90_deg": deg(p[1]), "p99_deg": deg(p[2]),
            "p75_deg": deg(p[3]), "p95_deg": deg(p[4]),
            "mean_rise_per_block": float(gv.mean()) if gv.size else float("nan"),
            "share_over_30": float((gv > math.tan(math.radians(30))).mean()),
            "share_over_45": float((gv > 1.0).mean()),
            "share_over_60": float((gv > math.tan(math.radians(60))).mean()),
            "share_over_75": float((gv > math.tan(math.radians(75))).mean()),
        }
        del gv
    step = np.zeros(v.shape, np.int16)
    for d, ax in ((1, 0), (-1, 0), (1, 1), (-1, 1)):
        np.maximum(step, np.abs(v - shift(v, d, ax)), out=step)
    m = land & interior(1)
    sv = step[m]
    del step
    p = pct(sv)
    out["step"] = {
        "median": p[0], "p90": p[1], "p99": p[2], "mean": float(sv.mean()),
        "share_flat": float((sv == 0).mean()),
        "share_ge2": float((sv >= 2).mean()), "share_ge4": float((sv >= 4).mean()),
        "share_ge8": float((sv >= 8).mean()), "share_ge16": float((sv >= 16).mean()),
    }
    del sv

    # Relief.
    for w in (16, 64, 256):
        hi = ndimage.maximum_filter(v, size=(w, w), mode=("nearest", mode))
        lo = ndimage.minimum_filter(v, size=(w, w), mode=("nearest", mode))
        r = (hi - lo)
        del hi, lo
        m = land & interior(w // 2)
        rv = r[m]
        del r
        p = pct(rv, (50, 90, 99, 100))
        out[f"relief{w}"] = {"median": p[0], "p90": p[1], "p99": p[2], "max": p[3]}
        del rv

    # Coast.
    open_sea = ndimage.maximum_filter(ndimage.minimum_filter(sea.view(np.uint8), size=15, mode=("nearest", mode)),
                                      size=15, mode=("nearest", mode))
    open_sea &= sea.view(np.uint8)
    near_sea = ndimage.maximum_filter(open_sea, size=3, mode=("nearest", mode)) > 0
    del open_sea
    shore = land & near_sea & interior(8)
    hi = ndimage.maximum_filter(v, size=(17, 17), mode=("nearest", mode))
    cv = (hi[shore] - sea_top).astype(np.int32)
    del hi
    p = pct(cv)
    out["coast"] = {
        "shore_columns": int(shore.sum()),
        "cliff_median": p[0], "cliff_p90": p[1], "cliff_p99": p[2],
        "share_ge8": float((cv >= 8).mean()) if cv.size else float("nan"),
        "share_ge16": float((cv >= 16).mean()) if cv.size else float("nan"),
        "share_ge32": float((cv >= 32).mean()) if cv.size else float("nan"),
        "cliff_mean": float(cv.mean()) if cv.size else float("nan"),
    }
    # Height of the land above the sea surface, blocks.
    hv = (v[land] - sea_top).astype(np.int32)
    p = pct(hv, (50, 90, 99, 100))
    out["height"] = {"median": p[0], "p90": p[1], "p99": p[2], "max": p[3]}
    return out


def planet_measure(pl, mv, warp=None, cap=75.0):
    r0, r1 = pl.lat_rows(cap)
    h = pl.h16[r0:r1].astype(np.float32) / 16.0
    wl16 = pl.wl16[r0:r1]
    wl = np.where(wl16 == NODATA, np.nan, wl16.astype(np.float32) / 16.0)
    t, v, wet, sea = surfaces(h, wl, pl.sea_y, pl.mv0, mv, warp)
    del h, wl, t
    valid = np.ones(v.shape, bool)
    out = measure(v, ~wet, sea, valid, pl.sea_y - 1, wrap=True)
    out["lake_share_of_land"] = float((wet & ~sea).sum() / max(1, (~sea).sum()))
    return out


def warp_named(name):
    """Non-linear mappings, metres above the sea -> blocks above the sea. `knee:a:b:h0:w` draws a metres
    per block up to about h0 and b above, blending over w metres (the slope dy/dh goes smoothly from
    1/a to 1/b); below the sea the mapping is a metres per block."""
    kind, *rest = name.split(":")
    if kind != "knee":
        raise SystemExit(f"unknown warp {name}")
    a, b, h0, w = (float(x) for x in rest)

    def f(hm):
        hm = np.asarray(hm, dtype=np.float64)
        # dy/dh = 1/b + (1/a - 1/b) * s(h), s a logistic falling from 1 to 0 around h0.
        # Its integral from 0: h/b + (1/a - 1/b) * (S(h) - S(0)), S(h) = -w * log(1 + exp(-(h - h0)/w)) + … .
        def big_s(x):
            # integral of 1/(1+exp((x-h0)/w)) = x - w*log(1+exp((x-h0)/w))
            return x - w * np.logaddexp(0.0, (x - h0) / w)
        up = hm / b + (1 / a - 1 / b) * (big_s(hm) - big_s(0.0))
        return np.where(hm >= 0, up, hm / a).astype(np.float32)
    return f


def warp_label(name):
    kind, a, b, h0, w = name.split(":")
    return f"non-linear, {a} m per block below about {float(h0) / 1000:g} km and {b} m above"


def stats(a):
    pl = Planet(a.work, a.planet)
    res = {"planet": {"name": a.planet, "meta": pl.meta}, "scales": {}, "warps": {}, "refs": {}}
    # --reuse: the planet's rows of an earlier output (they take minutes; the references are quick).
    old = json.load(open(a.reuse)) if a.reuse else {"scales": {}, "warps": {}}
    for mv in a.scales:
        if str(mv) in old["scales"]:
            res["scales"][str(mv)] = old["scales"][str(mv)]
            continue
        print(f"stats: planet at {mv} m per block", flush=True)
        res["scales"][str(mv)] = planet_measure(pl, mv)
    for w in a.warp or []:
        if w in old["warps"]:
            res["warps"][w] = old["warps"][w]
            continue
        print(f"stats: planet through {w}", flush=True)
        res["warps"][w] = planet_measure(pl, pl.mv0, warp_named(w))
    for spec in a.ref or []:
        name, label = spec.split("=", 1)
        arr = np.load(os.path.join(a.work, name + ".ref.npy"))
        t, v, wet, valid = ref_surfaces(arr)
        sea_top = 62
        sea = wet & (v == sea_top)
        print(f"stats: reference {name}", flush=True)
        r = measure(v, valid & ~wet, sea, valid, sea_top, wrap=False)
        r["label"] = label
        r["y_range"] = [int(t[valid].min()), int(t[valid].max())]
        res["refs"][name] = r
    json.dump(res, open(a.out, "w"), indent=1)
    print(f"stats -> {a.out}")


def compare(a):
    """A planet baked at another scale against the base planet rescaled to it."""
    base, other = Planet(a.work, a.planet), Planet(a.work, a.other)
    mv = other.mv0
    res = {"rescaled": planet_measure(base, mv), "baked": planet_measure(other, mv), "mv": mv}
    r0, r1 = base.lat_rows()
    hb = base.sea_y + (base.h16[r0:r1].astype(np.float32) / 16.0 - base.sea_y) * (base.mv0 / mv)
    ho = other.h16[r0:r1].astype(np.float32) / 16.0
    d = ho - hb
    land = hb >= base.sea_y - 1
    res["difference_blocks"] = {
        "land_mean": float(d[land].mean()), "land_rms": float(np.sqrt((d[land] ** 2).mean())),
        "land_p50_abs": float(np.percentile(np.abs(d[land]), 50)), "land_p99_abs": float(np.percentile(np.abs(d[land]), 99)),
        "land_max_abs": float(np.abs(d[land]).max()),
        "all_rms": float(np.sqrt((d ** 2).mean())),
    }
    json.dump(res, open(a.out, "w"), indent=1)
    print(json.dumps(res["difference_blocks"], indent=1))


# ----------------------------------------------------------------------------------------------
# Rock layers
# ----------------------------------------------------------------------------------------------

def strata(a):
    """Thickness in blocks of the rock units down sampled land columns (the part of each unit between
    the top block and the world floor), and the share under one block at each scale."""
    import subprocess
    from concurrent.futures import ThreadPoolExecutor
    pl = Planet(a.work, a.planet)
    r0, r1 = pl.lat_rows()
    rng = np.random.default_rng(7)
    cols = []
    while len(cols) < a.n:
        r = int(rng.integers(r0, r1))
        c = int(rng.integers(0, pl.c))
        if pl.h16[r, c] / 16.0 >= pl.sea_y + 1:
            cols.append((c - pl.c // 2, r - pl.rows // 2))

    def one(xz):
        o = subprocess.run([a.bin, "inspect-column", "--world-data", a.world_data, str(xz[0]), str(xz[1])],
                           capture_output=True, text=True)
        return json.loads(o.stdout) if o.returncode == 0 else None

    with ThreadPoolExecutor(4) as ex:
        res = [r for r in ex.map(one, cols) if r]
    floor = pl.meta["scale"]["min_y"]
    thick, cls, per_col, depth_top = [], [], [], []
    for d in res:
        top_y = d["top_y"]
        n = 0
        for u in d["strata"]:
            t, b = min(u["top"], top_y), max(u["bottom"], floor)
            if t < b:
                continue
            thick.append(t - b + 1)
            cls.append(u["class"])
            depth_top.append(top_y - t)
            n += 1
        per_col.append(n)
    thick = np.array(thick, float)
    depth_top = np.array(depth_top, float)
    out = {"columns": len(res), "units": int(thick.size), "units_per_column": pct(np.array(per_col), (50, 90, 100)),
           "thickness_blocks": pct(thick, (10, 25, 50, 75, 90)), "scales": {}}
    near = depth_top <= 64
    for mv in a.scales:
        t = thick * pl.mv0 / mv
        out["scales"][str(mv)] = {
            "median_blocks": float(np.median(t)),
            "share_under_1": float((t < 1).mean()), "share_under_2": float((t < 2).mean()),
            "share_under_half": float((t < 0.5).mean()),
            "near_surface_share_under_1": float((t[near] < 1).mean()) if near.any() else float("nan"),
        }
    by = {}
    for t, k in zip(thick, cls):
        by.setdefault(k, []).append(t)
    out["by_class"] = {k: {"units": len(v), "median_blocks": float(np.median(v)), "p10_blocks": float(np.percentile(v, 10))}
                       for k, v in sorted(by.items(), key=lambda kv: np.median(kv[1]))}
    json.dump(out, open(a.out, "w"), indent=1)
    print(json.dumps(out, indent=1))


# ----------------------------------------------------------------------------------------------
# Places
# ----------------------------------------------------------------------------------------------

def block_sum(m, k):
    r, c = m.shape[0] // k * k, m.shape[1] // k * k
    return m[:r, :c].reshape(r // k, k, c // k, k).sum(axis=(1, 3))


def places(a):
    from scipy import ndimage
    pl = Planet(a.work, a.planet)
    half_z = pl.rows // 2
    r0, r1 = pl.lat_rows(60.0)
    h = pl.h16[r0:r1].astype(np.float32) / 16.0
    t, v, wet, sea = surfaces(h, None, pl.sea_y, pl.mv0, pl.mv0)
    land = ~wet
    out = {}

    def world(row, col):
        return int(col - pl.c // 2), int(row + r0 - half_z)

    # Stratovolcanoes: the tallest standing above the sea on land, from the world data.
    vol = json.load(open(os.path.join(a.world_data, "volcanoes", "active.json")))
    vs = vol["volcanoes"] if isinstance(vol, dict) else vol
    cand = []
    for e in vs:
        if e.get("kind") != "stratovolcano":
            continue
        x, z = int(round(e["x"])), int(round(e["z"]))
        row, col = z + half_z - r0, (x + pl.c // 2) % pl.c
        rad = int(round(e.get("radius_m", 28000.0) / pl.mh))
        m = rad + 70
        if not (m <= row < h.shape[0] - m) or not (m <= col < pl.c - m):
            continue
        # A clean cone stands above everything on the circle at its basal radius, on ground near or
        # above the sea.
        peak = float(h[row - 10:row + 10, col - 10:col + 10].max())
        ang = np.linspace(0, 2 * np.pi, 96, endpoint=False)
        ring = h[(row + rad * np.sin(ang)).astype(int), (col + rad * np.cos(ang)).astype(int)]
        if peak < pl.sea_y + 60 or np.median(ring) < pl.sea_y - 2:
            continue
        cand.append((round(peak - float(ring.max()), 1), e["height_m"], x, z, e.get("radius_m"), round(peak - float(np.median(ring)), 1)))
    cand.sort(reverse=True)
    out["volcano_candidates"] = cand[:8]
    for key, c in zip(("volcano", "volcano_b"), cand):
        out[key] = {
            "x": c[2], "z": c[3], "height_m": c[1], "radius_m": c[4],
            "what": "stratovolcano" if key == "volcano" else "a second stratovolcano",
        }
    vmask = np.zeros(h.shape, bool)
    for e in vs:
        x, z = int(round(e["x"])), int(round(e["z"]))
        row, col = z + half_z - r0, (x + pl.c // 2) % pl.c
        if 0 <= row < h.shape[0]:
            vmask[max(0, row - 120):row + 120, max(0, col - 120):col + 120] = True

    # High range: the 64-block cell with the highest mean land height away from volcanoes, then the peak.
    k = 64
    hm = np.where(land & ~vmask, h, 0)
    cells = block_sum(hm, k) / (k * k)
    cells[:3] = 0
    cells[-3:] = 0
    cells[:, :3] = 0
    cells[:, -3:] = 0
    i = np.unravel_index(np.argmax(cells), cells.shape)
    x, z = world(i[0] * k + k // 2, i[1] * k + k // 2)
    out["range"] = {"x": x, "z": z, "what": "high range"}

    # Coast: the cell with most shoreline columns under a cliff (highest V within 8 blocks >= 16 above the sea).
    near_sea = ndimage.maximum_filter(sea.view(np.uint8), size=3) > 0
    hi = ndimage.maximum_filter(v, size=(17, 17))
    cliff = land & near_sea & (hi - (pl.sea_y - 1) >= 16) & ~vmask
    cells = block_sum(cliff, 128).astype(np.float64)
    cells[:2] = 0
    cells[-2:] = 0
    cells[:, :2] = 0
    cells[:, -2:] = 0
    i = np.unravel_index(np.argmax(cells), cells.shape)
    x, z = world(i[0] * 128 + 64, i[1] * 128 + 64)
    out["coast"] = {"x": x, "z": z, "what": "steep coast and continental margin", "cliff_columns": int(cells[i])}
    # Valley: deepest narrow hollows on land (closing over 33 blocks minus the surface), within 45 degrees.
    q0, q1 = pl.lat_rows(45.0)
    q0, q1 = q0 - r0, q1 - r0
    vv = v[q0:q1]
    closed = ndimage.grey_closing(vv, size=(33, 33))
    depth = (closed - vv)
    del closed
    inland = ndimage.minimum_filter(land[q0:q1].view(np.uint8), size=65) > 0
    deep = (depth >= 12) & inland & ~vmask[q0:q1]
    cells = block_sum(deep, 128).astype(np.float64)
    cells[:, :2] = 0
    cells[:, -2:] = 0
    i = np.unravel_index(np.argmax(cells), cells.shape)
    x, z = world(i[0] * 128 + 64 + q0, i[1] * 128 + 64)
    out["valley"] = {"x": x, "z": z, "what": "incised valleys", "deep_columns": int(cells[i]),
                     "deepest_blocks": int(depth[i[0] * 128:(i[0] + 1) * 128, i[1] * 128:(i[1] + 1) * 128].max())}
    json.dump(out, open(a.out, "w"), indent=1)
    print(json.dumps(out, indent=1))


# ----------------------------------------------------------------------------------------------
# Pictures
# ----------------------------------------------------------------------------------------------

# Height tints, metres above the sea -> colour (lowland green, upland olive and tan, rock, snow).
TINT = [(-1, (196, 186, 140)), (30, (122, 160, 86)), (400, (104, 146, 78)), (1000, (140, 150, 88)),
        (1800, (160, 140, 100)), (2600, (140, 128, 116)), (3400, (156, 154, 152)), (4200, (236, 238, 242)),
        (9000, (250, 250, 252))]
STONE = np.array((118, 118, 120), np.float32)
SKY = (223, 231, 240)


def tint(m):
    xs = [p[0] for p in TINT]
    out = np.zeros(m.shape + (3,), np.float32)
    for ch in range(3):
        out[..., ch] = np.interp(m, xs, [p[1][ch] for p in TINT])
    return out


def oblique(v, wet, metres, depth_blocks, s=2, pitch_deg=30.0, base=None, top=None):
    """An oblique orthographic view from the south: every column a block-wide pillar, painted back to
    front. v the visible surface (blocks), wet where it is water, metres the land's height for the tint,
    depth_blocks the water depth. base/top fix the vertical range so pictures of one place align."""
    nz, nx = v.shape
    p = math.radians(pitch_deg)
    ys, zs = s * math.cos(p), s * math.sin(p)
    base = (int(v.min()) - 2) if base is None else base
    top = int(v.max()) if top is None else top
    hmax = top - base
    hh = int(nz * zs + hmax * ys) + 6
    img = np.empty((hh, nx * s, 3), np.uint8)
    img[:] = SKY
    vf = v.astype(np.float32)
    gx = np.gradient(vf, axis=1)
    gz = np.gradient(vf, axis=0)
    # Light from the upper left (north-west), 40 degrees up.
    lx, lz, ly = -0.55, -0.45, 0.70
    n = np.sqrt(gx * gx + gz * gz + 1)
    lam = np.clip((-gx * lx - gz * lz + ly) / n, 0, 1) / ly
    shade = np.clip(0.50 + 0.50 * lam, 0.35, 1.25)
    col = tint(metres)
    steep = np.clip((np.hypot(gx, gz) - 1.2) / 1.5, 0, 1)[..., None]
    col = col * (1 - steep) + STONE * steep
    water = np.stack([np.interp(depth_blocks, [0, 6, 40], c) for c in ((92, 62, 38), (150, 118, 80), (196, 160, 122))], -1)
    col = np.where(wet[..., None], water, col)
    shade = np.where(wet, 1.0, shade)
    top_rgb = np.clip(col * shade[..., None], 0, 255)
    drop = np.zeros(v.shape, np.float32)
    drop[:-1] = np.clip(vf[:-1] - vf[1:], 0, None)
    w_side = np.clip((drop - 1) / 3, 0, 1)[..., None]
    side_rgb = np.where(wet[..., None], water * 0.72, (col * (1 - w_side) + STONE * w_side) * 0.60)
    side_rgb[-1] = np.where(wet[-1][..., None], water[-1] * 0.6, np.array((78, 70, 62), np.float32))  # the cut face of the box
    rows = np.arange(hh)[:, None]
    for z in range(nz):
        topf = (hmax - (vf[z] - base)) * ys + z * zs
        t0 = np.floor(topf).astype(np.int64)
        t1 = np.maximum(t0 + 1, np.floor(topf + zs).astype(np.int64) + 1)
        b = min(hh, int(hmax * ys + (z + 1) * zs) + 2)
        t0p, t1p = np.repeat(t0, s), np.repeat(t1, s)
        lo = max(0, int(t0.min()))
        r = rows[lo:b]
        sub = img[lo:b]
        m_side = (r >= t1p) & (r < b)
        sub[m_side] = np.broadcast_to(np.repeat(side_rgb[z], s, axis=0)[None], sub.shape)[m_side]
        m_top = (r >= t0p) & (r < t1p)
        sub[m_top] = np.broadcast_to(np.repeat(top_rgb[z], s, axis=0)[None], sub.shape)[m_top]
    return img


def flank_angle(profile):
    """The mean slope of the steeper flank of a cone profile between 20 % and 80 % of its height."""
    i = int(np.argmax(profile))
    best = 0.0
    for side in (profile[:i + 1][::-1], profile[i:]):
        if side.size < 4:
            continue
        lo = float(np.percentile(side, 5))
        hgt = float(side[0]) - lo
        if hgt <= 0:
            continue
        a = np.argmax(side <= lo + 0.8 * hgt)
        b = np.argmax(side <= lo + 0.2 * hgt)
        if b > a:
            best = max(best, math.degrees(math.atan((side[a] - side[b]) / (b - a))))
    return best


def box_numbers(v, wet, sea_top):
    land = ~wet
    vf = v.astype(np.float32)
    g = np.hypot(np.gradient(vf, axis=1), np.gradient(vf, axis=0))
    gl = g[land] if land.any() else np.zeros(1)
    return {
        "peak_above_sea": int(v.max() - sea_top),
        "relief": int(v.max() - v.min()),
        "slope_median_deg": deg(float(np.percentile(gl, 50))),
        "slope_p90_deg": deg(float(np.percentile(gl, 90))),
        "slope_p99_deg": deg(float(np.percentile(gl, 99))),
        "share_over_45": float((gl > 1).mean()),
    }


def render(a):
    from PIL import Image
    pl = Planet(a.work, a.planet)
    plc = json.load(open(a.places))
    os.makedirs(a.out, exist_ok=True)
    half = a.side // 2
    sea_top = pl.sea_y - 1
    pictures = {"scales": a.scales, "places": [], "refs": [], "side": a.side, "warps": a.warp or []}
    profiles = {}
    for key in ("volcano", "volcano_b", "range", "coast", "valley"):
        if key not in plc:
            continue
        p = plc[key]
        x, z = p["x"], p["z"]
        h, wl = pl.box(x - half, z - half, x + half, z + half)
        metres = (h - pl.sea_y) * pl.mv0
        entry = {"key": key, "what": p["what"], "x": x, "z": z, "km": a.side * pl.mh / 1000.0, "views": []}
        variants = [(str(mv), mv, None) for mv in a.scales] + [(w, pl.mv0, warp_named(w)) for w in (a.warp or [])]
        surf = {name: surfaces(h, wl, pl.sea_y, pl.mv0, mv, warp) for name, mv, warp in variants}
        base = min(int(s[1].min()) for s in surf.values()) - 2
        top = max(int(s[1].max()) for s in surf.values())
        row = half
        if key in ("volcano", "volcano_b", "range"):
            row = int(np.unravel_index(np.argmax(h), h.shape)[0])
        prof = {}
        for name, mv, warp in variants:
            t, v, wet, sea = surf[name]
            depth = np.clip(v - t, 0, None).astype(np.float32)
            img = oblique(v, wet, metres, depth, s=a.px, base=base, top=top)
            f = f"{key}-{name.replace(':', '_')}.png"
            Image.fromarray(img).save(os.path.join(a.out, f), optimize=True)
            nums = box_numbers(v, wet, sea_top)
            if key.startswith("volcano"):
                nums["flank_deg"] = flank_angle(v[row].astype(np.float32))
            entry["views"].append({"variant": name, "file": f, **nums})
            prof[name] = {"terrain": t[row].tolist(), "surface": v[row].tolist()}
            print(f"render: {f} {nums}", flush=True)
        profiles[key] = {"row_z": z - half + row, "x0": x - half, "lines": prof, "sea_top": sea_top, "what": p["what"],
                         "through": "its highest point" if row != half else "the middle of the box"}
        pictures["places"].append(entry)
    for spec in a.ref or []:
        name, label = spec.split("=", 1)
        arr = np.load(os.path.join(a.work, name + ".ref.npy"))
        t, v, wet, valid = ref_surfaces(arr)
        from scipy import ndimage
        land = valid & ~wet
        k = 64
        # The highest ground, and the highest coast, of the reference area (boxes wholly inside one square).
        picks = []
        inside = ndimage.minimum_filter(valid.view(np.uint8), size=a.side + 2, mode="constant", cval=0) > 0
        nr, nc = v.shape[0] // k, v.shape[1] // k
        inside = inside[k // 2::k, k // 2::k][:nr, :nc]
        hm = np.where(land, v, 0)
        cells = block_sum(hm, k) / (k * k)
        cells[~inside] = 0
        i = np.unravel_index(np.argmax(cells), cells.shape)
        picks.append(("mountains", "its highest ground", i[0] * k + k // 2, i[1] * k + k // 2))
        sea = wet & (v == 62)
        near_sea = ndimage.maximum_filter(sea.view(np.uint8), size=3) > 0
        hi = ndimage.maximum_filter(v, size=(17, 17))
        cliff = land & near_sea & (hi - 62 >= 16)
        cells = block_sum(cliff, k).astype(np.float64)
        cells[~inside] = 0
        i = np.unravel_index(np.argmax(cells), cells.shape)
        picks.append(("coast", "its steepest coast", i[0] * k + k // 2, i[1] * k + k // 2))
        for pk, what, r, c in picks:
            r = min(max(r, half), v.shape[0] - half)
            c = min(max(c, half), v.shape[1] - half)
            vb, tb, wb = v[r - half:r + half, c - half:c + half], t[r - half:r + half, c - half:c + half], wet[r - half:r + half, c - half:c + half]
            metres = (tb.astype(np.float32) - 62) * 20.0
            depth = np.clip(vb - tb, 0, None).astype(np.float32)
            img = oblique(vb, wb, metres, depth, s=a.px)
            f = f"ref-{name}-{pk}.png"
            Image.fromarray(img).save(os.path.join(a.out, f), optimize=True)
            nums = box_numbers(vb, wb, 62)
            row = int(np.unravel_index(np.argmax(vb), vb.shape)[0])
            pictures["refs"].append({"name": name, "label": label, "pick": pk, "what": what, "file": f, **nums})
            profiles[f"ref-{name}-{pk}"] = {"lines": {label: {"terrain": tb[row].tolist(), "surface": vb[row].tolist()}}, "sea_top": 62,
                                            "what": f"{label}: {what}"}
            print(f"render: {f} {nums}", flush=True)
    json.dump(pictures, open(os.path.join(a.out, "pictures.json"), "w"), indent=1)
    json.dump(profiles, open(os.path.join(a.out, "profiles.json"), "w"))
    profile_charts(profiles, a.scales, a.out)


# ----------------------------------------------------------------------------------------------
# Charts (matplotlib; the palette is the dataviz reference instance, light surface)
# ----------------------------------------------------------------------------------------------

SURFACE, INK, INK2, MUTED, GRID, AXIS = "#fcfcfb", "#0b0b0b", "#52514e", "#898781", "#e1e0d9", "#c3c2b7"
BLUE, ORANGE, AQUA, YELLOW = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"
# One hue, light to dark, for the ordered scales (20 m lightest … 80 m darkest).
RAMP = ["#86b6ef", "#6da7ec", "#5598e7", "#2a78d6", "#1c5cab", "#104281", "#0d366b"]
REF_COLOURS = [ORANGE, AQUA, YELLOW]


def style(ax):
    ax.set_facecolor(SURFACE)
    for s in ("top", "right"):
        ax.spines[s].set_visible(False)
    for s in ("left", "bottom"):
        ax.spines[s].set_color(AXIS)
    ax.tick_params(colors=MUTED, labelsize=8, length=3)
    ax.grid(True, color=GRID, linewidth=0.6)
    ax.set_axisbelow(True)


def profile_charts(profiles, scales, out):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    files = {}
    for key, p in profiles.items():
        lines = p["lines"]
        n = len(next(iter(lines.values()))["surface"])
        ymax = max(max(l["surface"]) for l in lines.values())
        ymin = min(min(l["terrain"]) for l in lines.values())
        ymin = max(ymin, p["sea_top"] - 40)
        w_in = 10.0
        h_in = max(1.6, w_in * (ymax - ymin + 12) / n + 0.7)
        fig, ax = plt.subplots(figsize=(w_in, h_in), dpi=110)
        fig.patch.set_facecolor(SURFACE)
        style(ax)
        xs = np.arange(n)
        names = list(lines)
        for i, name in enumerate(names):
            if name in [str(s) for s in scales]:
                colour = RAMP[min(len(RAMP) - 1, [str(s) for s in scales].index(name))]
                label = f"{name} m"
            elif name.startswith("knee"):
                kn = [n for n in names if n.startswith("knee")].index(name)
                colour = (ORANGE, "#b0431a")[kn % 2]
                label = "{1}→{2} m".format(*name.split(":"))
            else:
                colour, label = INK2, name
            y = np.array(lines[name]["terrain"], float) + 1
            ax.plot(xs, y, color=colour, linewidth=1.6, drawstyle="steps-mid", label=label)
            j = int(np.argmax(y))
            ax.annotate(label, (xs[j], y[j]), xytext=(4, 3), textcoords="offset points", fontsize=7.5, color=INK2)
        ax.axhline(p["sea_top"] + 1, color=AXIS, linewidth=1)
        ax.set_aspect("equal", adjustable="box")
        ax.set_xlim(0, n)
        ax.set_ylim(ymin, ymax + 12)
        ax.set_xlabel("blocks along the section (west to east)", fontsize=8, color=INK2)
        ax.set_ylabel("y", fontsize=8, color=INK2)
        ax.set_title(f"{p['what']}: the west to east section through {p.get('through', 'its highest point')}, drawn one block = one block",
                     fontsize=9, color=INK, loc="left")
        f = f"profile-{key}.png"
        fig.savefig(os.path.join(out, f), bbox_inches="tight", facecolor=SURFACE)
        plt.close(fig)
        files[key] = f
    json.dump(files, open(os.path.join(out, "profile-files.json"), "w"))


METRICS = [
    ("slope", "share_over_45", "Land steeper than 45° between neighbouring columns", "% of land columns"),
    ("slope", "share_over_60", "Land steeper than 60°", "% of land columns"),
    ("hillside", "median_deg", "Hillside slope over 16 blocks, median", "degrees"),
    ("hillside", "p90_deg", "Hillside slope over 16 blocks, steepest tenth (90th percentile)", "degrees"),
    ("relief64", "p90", "Relief within 64 blocks (90th percentile)", "blocks"),
    ("relief256", "p90", "Relief within 256 blocks (90th percentile)", "blocks"),
    ("coast", "cliff_p90", "Coast: highest ground within 8 blocks of the shore (90th percentile)", "blocks above the sea"),
    ("height", "p99", "Height of the land (99th percentile)", "blocks above the sea"),
]


def load_more(specs):
    """--more F.json=Label: other planets' stats files."""
    out = []
    for spec in specs or []:
        f, label = spec.split("=", 1)
        out.append((label, json.load(open(f))))
    return out


def charts(a):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    st = json.load(open(a.stats))
    more = load_more(a.more)
    os.makedirs(a.out, exist_ok=True)
    scales = sorted(st["scales"], key=float)
    xs = [float(s) for s in scales]
    refs = list(st["refs"].items())
    fig, axes = plt.subplots(4, 2, figsize=(11, 13), dpi=110)
    fig.patch.set_facecolor(SURFACE)
    for ax, (grp, key, title, unit) in zip(axes.ravel(), METRICS):
        style(ax)
        mul = 100.0 if key.startswith("share") else 1.0
        ys = [st["scales"][s][grp][key] * mul for s in scales]
        ax.plot(xs, ys, color=BLUE, linewidth=2, marker="o", markersize=5, label=a.label)
        for x, y in zip(xs, ys):
            ax.annotate(f"{y:.3g}", (x, y), xytext=(0, 6), textcoords="offset points", ha="center", fontsize=7, color=INK2)
        for label, other in more:
            ys2 = [other["scales"][s][grp][key] * mul for s in scales if s in other["scales"]]
            xs2 = [float(s) for s in scales if s in other["scales"]]
            ax.plot(xs2, ys2, color="#104281", linewidth=1.6, marker="s", markersize=4, label=label)
            for x, y in zip(xs2, ys2):
                ax.annotate(f"{y:.3g}", (x, y), xytext=(0, -11), textcoords="offset points", ha="center", fontsize=7, color=INK2)
        for i, (name, r) in enumerate(refs):
            y = r[grp][key] * mul
            ax.axhline(y, color=REF_COLOURS[i % 3], linewidth=1.6, linestyle=(0, (5, 2)) if i else "-", label=r["label"])
            ax.annotate(f"{r['label']} {y:.3g}", (xs[-1], y), xytext=(2, 3 if i % 2 == 0 else -9), textcoords="offset points",
                        ha="right", fontsize=7.5, color=INK2)
        ax.set_title(title, fontsize=9.5, color=INK, loc="left")
        ax.set_ylabel(unit, fontsize=8, color=INK2)
        ax.set_xlabel("metres per block, vertical", fontsize=8, color=INK2)
        ax.set_xticks(xs)
        ax.set_ylim(bottom=0)
    h, l = axes[0, 0].get_legend_handles_labels()
    fig.legend(h, l, loc="upper center", ncol=len(l), frameon=False, fontsize=9, labelcolor=INK2)
    fig.tight_layout(rect=(0, 0, 1, 0.97))
    fig.savefig(os.path.join(a.out, "chart-scales.png"), facecolor=SURFACE)
    plt.close(fig)
    print("charts -> chart-scales.png")


# ----------------------------------------------------------------------------------------------
# Page
# ----------------------------------------------------------------------------------------------

CSS = """
:root{--bg:#f9f9f7;--card:#fcfcfb;--ink:#0b0b0b;--ink2:#52514e;--muted:#898781;--line:#e1e0d9}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--ink);font:15px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif}
main{max-width:1500px;margin:0 auto;padding:24px 16px 80px}
h1{font-size:26px;margin:0 0 4px}h2{font-size:19px;margin:36px 0 8px}h3{font-size:15px;margin:22px 0 6px}
p{max-width:920px;color:var(--ink2);margin:6px 0}
.row{display:flex;gap:10px;overflow-x:auto;padding-bottom:8px}
figure{margin:0;flex:0 0 auto;background:var(--card);border:1px solid var(--line);border-radius:8px;padding:8px}
figure img{display:block;width:420px;height:auto;image-rendering:auto}
figcaption{font-size:12.5px;color:var(--ink2);margin-top:6px;max-width:420px}
figcaption b{color:var(--ink);font-size:14px}
table{border-collapse:collapse;font-size:13px;background:var(--card);margin:8px 0}
th,td{border:1px solid var(--line);padding:4px 9px;text-align:right;font-variant-numeric:tabular-nums}
th{font-weight:600;color:var(--ink2);background:#f3f2ee}td:first-child,th:first-child{text-align:left}
tr.now td{background:#fff5ee}tr.ref td{background:#eef8f3}
.wide{overflow-x:auto}.chart{max-width:100%;border:1px solid var(--line);border-radius:8px;background:var(--card)}
.note{font-size:13px;color:var(--muted)}
"""


def fmt(v, d=1):
    return "–" if v is None or (isinstance(v, float) and math.isnan(v)) else f"{v:.{d}f}"


def stats_table(st, label="Deep Time", more=()):
    rows = []
    head = ("<tr><th>World</th><th>Exaggeration</th><th>Slope median</th><th>p90</th><th>p99</th><th>&gt;30°</th><th>&gt;45°</th><th>&gt;60°</th><th>&gt;75°</th>"
            "<th>Hillside median</th><th>Hillside p90</th><th>Level with all four neighbours</th><th>Drop ≥2</th><th>≥4</th><th>≥8</th><th>Relief 16 med / p90 / p99</th><th>Relief 64</th><th>Relief 256</th>"
            "<th>Coast cliff med / p90 / p99</th><th>Land height med / p90 / p99 / max</th></tr>")

    def row(label, r, cls="", ex=""):
        s, h, t = r["slope"], r["hillside"], r["step"]
        rel = lambda k: f"{fmt(r[k]['median'], 0)} / {fmt(r[k]['p90'], 0)} / {fmt(r[k]['p99'], 0)}"
        c, hg = r["coast"], r["height"]
        return (f"<tr class='{cls}'><td>{label}</td><td>{ex}</td><td>{fmt(s['median_deg'])}°</td><td>{fmt(s['p90_deg'])}°</td><td>{fmt(s['p99_deg'])}°</td>"
                f"<td>{fmt(100 * s['share_over_30'])}%</td><td>{fmt(100 * s['share_over_45'])}%</td><td>{fmt(100 * s['share_over_60'], 2)}%</td><td>{fmt(100 * s['share_over_75'], 2)}%</td>"
                f"<td>{fmt(h['median_deg'])}°</td><td>{fmt(h['p90_deg'])}°</td><td>{fmt(100 * t['share_flat'])}%</td><td>{fmt(100 * t['share_ge2'])}%</td><td>{fmt(100 * t['share_ge4'])}%</td><td>{fmt(100 * t['share_ge8'], 2)}%</td>"
                f"<td>{rel('relief16')}</td><td>{rel('relief64')}</td><td>{rel('relief256')}</td>"
                f"<td>{fmt(c['cliff_median'], 0)} / {fmt(c['cliff_p90'], 0)} / {fmt(c['cliff_p99'], 0)}</td>"
                f"<td>{fmt(hg['median'], 0)} / {fmt(hg['p90'], 0)} / {fmt(hg['p99'], 0)} / {fmt(hg['max'], 0)}</td></tr>")
    mh = st["planet"]["meta"]["scale"]["m_per_block_h"]
    mv0 = st["planet"]["meta"]["scale"]["m_per_block_v"]
    for name, one in [(label, st)] + list(more):
        for s in sorted(one["scales"], key=float):
            rows.append(row(f"{name}, {s} m per block", one["scales"][s], "now" if float(s) == mv0 else "", f"{mh / float(s):.3g}×"))
        for w, r in one.get("warps", {}).items():
            rows.append(row(f"{name}, {warp_label(w)}", r, "", "varies"))
    for name, r in st["refs"].items():
        rows.append(row(r["label"], r, "ref", "1×"))
    return "<div class='wide'><table>" + head + "".join(rows) + "</table></div>"


def costs_table(st, strata):
    """What a real height or thickness is in blocks at each scale, and what a real angle is drawn as."""
    sc = st["planet"]["meta"]["scale"]
    mh, sea = sc["m_per_block_h"], sc["sea_level_y"]
    top = st["planet"]["meta"]["notes_surface"]["max_elevation_m"]
    scales = [float(x) for x in sorted(st["scales"], key=float)]
    drawn = lambda real, mv: math.degrees(math.atan(mh / mv * math.tan(math.radians(real))))
    lines = [
        ("Vertical exaggeration", lambda mv: f"{mh / mv:.3g}×"),
        ("Mount Everest, 8,849 m: blocks above the sea", lambda mv: f"{8849 / mv:.0f}"),
        (f"This planet's highest point, {top:,.0f} m: blocks above the sea (y)", lambda mv: f"{top / mv:.0f} (y {sea + top / mv:.0f})"),
        ("A 3 km volcano: blocks tall", lambda mv: f"{3000 / mv:.0f}"),
        ("A 1 km deep canyon: blocks deep", lambda mv: f"{1000 / mv:.0f}"),
        ("A 100 m river valley: blocks deep", lambda mv: f"{100 / mv:.1f}"),
        ("A 100 m rock layer: blocks thick", lambda mv: f"{100 / mv:.1f}"),
        ("Ore 1.5 km down: blocks below the surface", lambda mv: f"{1500 / mv:.0f}"),
        ("Abyssal plain, 4 km deep: y", lambda mv: f"{sea - 4000 / mv:.0f}"),
        ("Trench, 10 km deep: y", lambda mv: f"{sea - 10000 / mv:.0f}"),
        ("A real 6° volcano flank is drawn at", lambda mv: f"{drawn(6, mv):.0f}°"),
        ("A real 18° slope is drawn at", lambda mv: f"{drawn(18, mv):.0f}°"),
        ("A real 30° scree slope is drawn at", lambda mv: f"{drawn(30, mv):.0f}°"),
        ("The real slope that is drawn at 45°", lambda mv: f"{math.degrees(math.atan(mv / mh)):.1f}°"),
    ]
    if strata:
        lines.append(("Rock layers under one block thick (sampled land columns)",
                      lambda mv: f"{100 * strata['scales'][f'{mv:.0f}']['share_under_1']:.0f}%" if f"{mv:.0f}" in strata["scales"] else "–"))
        lines.append(("Median rock layer: blocks thick",
                      lambda mv: f"{strata['scales'][f'{mv:.0f}']['median_blocks']:.1f}" if f"{mv:.0f}" in strata["scales"] else "–"))
    head = "<tr><th>At this many metres per block</th>" + "".join(f"<th>{mv:.0f}</th>" for mv in scales) + "</tr>"
    body = "".join("<tr><td>" + name + "</td>" + "".join(f"<td>{f(mv)}</td>" for mv in scales) + "</tr>" for name, f in lines)
    return "<div class='wide'><table>" + head + body + "</table></div>"


def page(a):
    st = json.load(open(a.stats))
    more = load_more(a.more)
    strata = json.load(open(a.strata)) if a.strata else None
    pics = json.load(open(a.pictures))
    pf = json.load(open(os.path.join(os.path.dirname(a.pictures), "profile-files.json")))
    mh = st["planet"]["meta"]["scale"]["m_per_block_h"]
    out = [f"<!doctype html><html lang='en'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'>"
           f"<title>Vertical scale study</title><style>{CSS}</style></head><body><main>",
           "<h1>Vertical scale: how tall should a block be?</h1>",
           f"<p>One baked planet ({a.label}; {mh:.0f} m per block across, baked at {st['planet']['meta']['scale']['m_per_block_v']:.0f} m per block up) "
           "drawn at each candidate vertical scale, beside vanilla's terrain and the pack's own. Every picture is drawn from the heightmap with one "
           "renderer (an oblique view from the south, one pillar per block column, no trees), so shapes compare directly; colours follow real "
           "elevation, not block height, and each place keeps one frame so the pictures of a row line up. "
           "The study and its reasoning: docs/research/notes/vertical-scale-study.md.</p>",
           "<h2>Where each scale sits against vanilla and the pack</h2>",
           "<img class='chart' src='chart-scales.png' alt='Deep Time statistics at each vertical scale against the reference worlds'>",
           "<h2>The numbers</h2>",
           "<p>Slopes are angles in block space on dry land (between neighbouring columns: a run of 2 blocks, so the angles come in steps; "
           "“hillside”: a run of 16). “Drop” is the largest height difference to a neighbouring column. Relief is highest minus lowest visible "
           "surface in a square of that many blocks. Coast cliff is the highest ground within 8 blocks of a column on the open sea's shore, above "
           "the water. Heights are blocks above the sea. The highlighted row is the scale the planet was baked at; the green rows are the references.</p>",
           stats_table(st, a.label, more),
           "<h2>What a real size becomes</h2>",
           costs_table(st, strata)]
    for p in pics["places"]:
        out.append(f"<h2>{p['what'].capitalize()} <span class='note'>x {p['x']}, z {p['z']}; {pics['side']} blocks ({p['km']:.0f} km) across</span></h2>")
        out.append("<div class='row'>")
        for v in p["views"]:
            name = v["variant"]
            title = f"{name} m per block ({mh / float(name):.3g}×)" if name.replace('.', '').isdigit() else warp_label(name).capitalize()
            extra = f"; flank {v['flank_deg']:.0f}°" if "flank_deg" in v else ""
            out.append(f"<figure><img loading='lazy' src='{v['file']}' alt='{p['what']} at {name}'><figcaption><b>{title}</b><br>"
                       f"highest point {v['peak_above_sea']} blocks above the sea; relief {v['relief']} blocks{extra}<br>"
                       f"slope median {v['slope_median_deg']:.0f}°, p90 {v['slope_p90_deg']:.0f}°, p99 {v['slope_p99_deg']:.0f}°; "
                       f"{100 * v['share_over_45']:.1f}% steeper than 45°</figcaption></figure>")
        out.append("</div>")
        if p["key"] in pf:
            out.append(f"<img class='chart' loading='lazy' src='{pf[p['key']]}' alt='section through the {p['what']}'>")
    if pics["refs"]:
        out.append("<h2>The references, same renderer and box size</h2><div class='row'>")
        for v in pics["refs"]:
            out.append(f"<figure><img loading='lazy' src='{v['file']}' alt='{v['label']}'><figcaption><b>{v['label']}</b>: {v['what']}<br>"
                       f"highest point {v['peak_above_sea']} blocks above the sea; relief {v['relief']} blocks<br>"
                       f"slope median {v['slope_median_deg']:.0f}°, p90 {v['slope_p90_deg']:.0f}°, p99 {v['slope_p99_deg']:.0f}°; "
                       f"{100 * v['share_over_45']:.1f}% steeper than 45°</figcaption></figure>")
        out.append("</div>")
        for v in pics["refs"]:
            k = f"ref-{v['name']}-{v['pick']}"
            if k in pf:
                out.append(f"<img class='chart' loading='lazy' src='{pf[k]}' alt='section, {v['label']}'>")
    extra = os.path.join(a.out, "extra.html")
    if os.path.exists(extra):
        out.append(open(extra).read())
    out.append("</main></body></html>")
    open(os.path.join(a.out, "index.html"), "w").write("\n".join(out))
    print("page -> index.html")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("join")
    p.add_argument("--work", required=True); p.add_argument("--name", required=True); p.add_argument("--world-data", required=True)
    p = sub.add_parser("refjoin")
    p.add_argument("--work", required=True); p.add_argument("--name", required=True); p.add_argument("tiles", nargs="+")
    p = sub.add_parser("stats")
    p.add_argument("--work", required=True); p.add_argument("--planet", required=True)
    p.add_argument("--scales", type=lambda s: [float(x) if "." in x else int(x) for x in s.split(",")], default=SCALES)
    p.add_argument("--ref", action="append"); p.add_argument("--warp", action="append"); p.add_argument("--out", required=True)
    p.add_argument("--reuse")
    p = sub.add_parser("compare")
    p.add_argument("--work", required=True); p.add_argument("--planet", required=True); p.add_argument("--other", required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("places")
    p.add_argument("--work", required=True); p.add_argument("--planet", required=True); p.add_argument("--world-data", required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("render")
    p.add_argument("--work", required=True); p.add_argument("--planet", required=True); p.add_argument("--places", required=True)
    p.add_argument("--scales", type=lambda s: [int(x) for x in s.split(",")], default=[20, 30, 40, 50, 60, 80])
    p.add_argument("--ref", action="append"); p.add_argument("--warp", action="append")
    p.add_argument("--side", type=int, default=320); p.add_argument("--px", type=int, default=2); p.add_argument("--out", required=True)
    p = sub.add_parser("strata")
    p.add_argument("--work", required=True); p.add_argument("--planet", required=True); p.add_argument("--world-data", required=True)
    p.add_argument("--bin", required=True); p.add_argument("--n", type=int, default=1500)
    p.add_argument("--scales", type=lambda s: [int(x) for x in s.split(",")], default=[20, 30, 40, 50, 60, 80])
    p.add_argument("--out", required=True)
    p = sub.add_parser("charts")
    p.add_argument("--stats", required=True); p.add_argument("--out", required=True)
    p.add_argument("--label", default="Deep Time"); p.add_argument("--more", action="append")
    p = sub.add_parser("page")
    p.add_argument("--stats", required=True); p.add_argument("--pictures", required=True); p.add_argument("--out", required=True)
    p.add_argument("--label", default="Deep Time"); p.add_argument("--more", action="append"); p.add_argument("--strata")
    a = ap.parse_args()
    {"join": join, "refjoin": refjoin, "stats": stats, "compare": compare, "places": places, "strata": strata, "render": render, "charts": charts, "page": page}[a.cmd](a)


if __name__ == "__main__":
    sys.exit(main())
