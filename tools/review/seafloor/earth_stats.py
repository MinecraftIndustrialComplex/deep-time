#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Sea-floor roughness of ETOPO 2022 (netCDF, lat/lon grid) by the same measures as survey.py
(the sea-floor spikes study, 2026-10-20).

    earth_stats.py <out.json> <file.nc>... [--coarsen K] [--sigma-km S] [--tile-km T] [--max-lat L]

Deep sea floor = surface elevation below -210 m (6 blocks at 35 m). Slopes from central differences
in metres on the sphere (R 6371 km); the high-pass is the terrain minus a Gaussian of sigma S km
(along longitude the sigma in pixels is S / (pixel width at the row's latitude)); tiles are T km
squares (T / pixel height rows by the matching longitude span). Percentiles are area-weighted
(cos lat).
"""

import argparse
import json
import math

import h5py
import numpy as np
from scipy import ndimage

R = 6_371_000.0


def wpct(v, w, ps):
    o = np.argsort(v)
    v, w = v[o], w[o]
    c = np.cumsum(w)
    c /= c[-1]
    return {str(p): float(np.interp(p / 100.0, c, v)) for p in ps}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("files", nargs="+")
    ap.add_argument("--coarsen", type=int, default=1, help="block-mean K x K pixels first")
    ap.add_argument("--sigma-km", type=float, default=4.71)
    ap.add_argument("--tile-km", type=float, default=37.7)
    ap.add_argument("--max-lat", type=float, default=80.0)
    ap.add_argument("--deep-m", type=float, default=-210.0)
    args = ap.parse_args()
    slopes, weights, tiles = [], [], []
    per_file = {}
    for path in args.files:
        with h5py.File(path, "r") as f:
            lat = f["lat"][:].astype(np.float64)
            lon = f["lon"][:].astype(np.float64)
            z = f["z"][:].astype(np.float32)
        k = args.coarsen
        if k > 1:
            h, w = (z.shape[0] // k) * k, (z.shape[1] // k) * k
            z = z[:h, :w].reshape(h // k, k, w // k, k).mean(axis=(1, 3))
            lat = lat[:h].reshape(-1, k).mean(axis=1)
            lon = lon[:w].reshape(-1, k).mean(axis=1)
        dlat = abs(lat[1] - lat[0])
        dlon = abs(lon[1] - lon[0])
        dy = R * math.radians(dlat)
        dxr = (R * math.radians(dlon) * np.cos(np.radians(lat))).astype(np.float32)  # per row
        keep = np.abs(lat) <= args.max_lat
        z = z[keep]
        lat = lat[keep]
        dxr = dxr[keep]
        gy = np.gradient(z, axis=0) / np.float32(dy)
        gx = np.gradient(z, axis=1) / dxr[:, None]
        deg = np.degrees(np.arctan(np.hypot(gx, gy)))
        # High-pass: latitude axis fixed sigma, longitude per row.
        sy = args.sigma_km * 1000.0 / dy
        sm = ndimage.gaussian_filter1d(z, sy, axis=0, mode="nearest")
        global_wrap = abs((lon[-1] - lon[0]) + dlon - 360.0) < 1e-6
        out = np.empty_like(sm)
        for r in range(sm.shape[0]):
            sx = args.sigma_km * 1000.0 / dxr[r]
            out[r] = ndimage.gaussian_filter1d(sm[r], sx, mode="wrap" if global_wrap else "nearest")
        hp = z - out
        deep = z < args.deep_m
        wrow = np.cos(np.radians(lat))
        W = np.broadcast_to(wrow[:, None], z.shape)
        slopes.append(deg[deep])
        weights.append(W[deep])
        # Tiles.
        th = max(2, int(round(args.tile_km * 1000.0 / dy)))
        ft = []
        for r0 in range(0, z.shape[0] - th + 1, th):
            tw = max(2, int(round(args.tile_km * 1000.0 / dxr[r0 + th // 2])))
            for c0 in range(0, z.shape[1] - tw + 1, tw):
                m = deep[r0:r0 + th, c0:c0 + tw]
                if m.mean() < 0.95:
                    continue
                d = deg[r0:r0 + th, c0:c0 + tw][m]
                t = {
                    "lat": float(lat[r0 + th // 2]), "lon": float(lon[c0 + tw // 2]),
                    "depth_m": float(-z[r0:r0 + th, c0:c0 + tw][m].mean()),
                    "rms_m": float(hp[r0:r0 + th, c0:c0 + tw][m].std()),
                    "deg_p50": float(np.percentile(d, 50)), "deg_p95": float(np.percentile(d, 95)),
                    "w": float(wrow[r0 + th // 2]),
                }
                ft.append(t)
        tiles += ft
        s = deg[deep]
        per_file[path.split("/")[-1]] = {
            "pixel_m": [float(dy), float(dxr.mean())],
            "deep_share": float(deep.mean()),
            "deg_pct": wpct(s, W[deep], (50, 90, 95, 99, 99.9)),
            "tile_rms_pct": wpct(np.array([t["rms_m"] for t in ft]), np.array([t["w"] for t in ft]), (10, 50, 90, 99, 100)) if ft else None,
            "worst_tiles": sorted(ft, key=lambda t: -t["rms_m"])[:8],
        }
        print(path, json.dumps(per_file[path.split("/")[-1]])[:1200], flush=True)
    s = np.concatenate(slopes)
    w = np.concatenate(weights)
    rms = np.array([t["rms_m"] for t in tiles])
    tw = np.array([t["w"] for t in tiles])
    res = {
        "args": vars(args),
        "deg_pct": wpct(s, w, (50, 75, 90, 95, 99, 99.9)),
        "share_over": {str(a): float(w[s > a].sum() / w.sum()) for a in (5, 10, 15, 20, 30)},
        "tile_rms_pct": wpct(rms, tw, (10, 50, 90, 95, 99, 100)),
        "tiles": len(tiles),
        "files": per_file,
    }
    with open(args.out, "w") as f:
        json.dump(res, f, indent=1)
    print(json.dumps({k: v for k, v in res.items() if k != "files"}, indent=1))


if __name__ == "__main__":
    main()
