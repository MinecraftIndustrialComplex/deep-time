#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Sea-floor roughness survey of a `deeptime heightmap` array (the sea-floor spikes study, 2026-10-20).

    survey.py <heightmap.npy> <out_dir> [--mpb-h 294.6] [--mpb-v 35] [--sea 63]

Per 128-block tile of deep sea floor (H below sea level by more than 6 blocks in 95 % of samples):
RMS of the high-passed terrain (Gaussian sigma 16 blocks, ~4.7 km, so wavelengths under ~30 km,
the abyssal-hill band) in metres, for H, E and the detail term; the slope distribution in real
degrees and in-game degrees; the dominant wavelength of the high-passed H (radially averaged
power spectrum). Writes tiles.json and overview PNGs.
"""

import argparse
import json
import math
from pathlib import Path

import numpy as np
from scipy import ndimage


def load(path):
    a = np.load(path, mmap_mode="r")
    meta = json.loads(Path(str(path) + ".json").read_text())
    return a, meta


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("npy")
    ap.add_argument("out")
    ap.add_argument("--mpb-h", type=float, default=294.6035301425964)
    ap.add_argument("--mpb-v", type=float, default=35.0)
    ap.add_argument("--sea", type=float, default=63.0)
    ap.add_argument("--tile", type=int, default=128, help="tile size, blocks")
    ap.add_argument("--sigma", type=float, default=16.0, help="high-pass Gaussian sigma, blocks")
    ap.add_argument("--png", action="store_true")
    ap.add_argument("--coarsen", type=int, default=1, help="block-mean K x K samples first (match a coarser baseline)")
    args = ap.parse_args()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    a, meta = load(args.npy)
    step = meta["step"]
    x0, z0, _, _ = meta["box"]
    ch = {n: i for i, n in enumerate(meta["channels"])}
    K = args.coarsen

    def co(f):
        f = np.asarray(f, dtype=np.float64)
        if K == 1:
            return f
        h, w = (f.shape[0] // K) * K, (f.shape[1] // K) * K
        return f[:h, :w].reshape(h // K, K, w // K, K).mean(axis=(1, 3))

    def pick(f):
        return f if K == 1 else np.asarray(f)[: (f.shape[0] // K) * K : K, : (f.shape[1] // K) * K : K]

    H = co(a[:, :, ch["surface"]])
    E = co(a[:, :, ch["elevation"]])
    D = co(a[:, :, ch["detail"]])
    mpb = co(a[:, :, ch["m_per_block"]]) if "m_per_block" in ch else np.full(H.shape, args.mpb_h)
    area_w = mpb ** 2  # ground area a sample stands for
    age = pick(a[:, :, ch["meta/crust_age_myr"]]) if "meta/crust_age_myr" in ch else None
    prov = pick(a[:, :, ch["meta/province"]]) if "meta/province" in ch else None
    step = step * K
    wrap = meta["box"][2] - meta["box"][0] == meta["circumference"]
    mode = ("wrap", "nearest") if wrap else "nearest"
    sig = args.sigma / step

    def hp(f):
        return f - ndimage.gaussian_filter(f, sig, mode=mode)

    hH, hE, hD = hp(H), hp(E), D - ndimage.gaussian_filter(D, sig, mode=mode)
    gz, gx = np.gradient(H, step)
    g = np.hypot(gx, gz)  # blocks per block
    game_deg = np.degrees(np.arctan(g))
    real_deg = np.degrees(np.arctan(g * args.mpb_v / mpb))
    gzE, gxE = np.gradient(E, step)
    gE = np.hypot(gxE, gzE)
    deep = H < args.sea - 6
    n = max(4, args.tile // step)
    rows, cols = H.shape
    tiles = []
    for r0 in range(0, rows - n + 1, n):
        for c0 in range(0, cols - n + 1, n):
            s = (slice(r0, r0 + n), slice(c0, c0 + n))
            if deep[s].mean() < 0.95:
                continue
            m = deep[s]
            t = {
                "x": x0 + c0 * step, "z": z0 + r0 * step,
                "depth_m": float((args.sea - H[s][m]).mean() * args.mpb_v),
                "rms_H_m": float(hH[s][m].std() * args.mpb_v),
                "mpb": float(mpb[s].mean()),
                "rms_E_m": float(hE[s][m].std() * args.mpb_v),
                "rms_D_m": float(hD[s][m].std() * args.mpb_v),
                "range_H_blocks": float(np.ptp(H[s][m])),
                "real_deg_p50": float(np.percentile(real_deg[s][m], 50)),
                "real_deg_p95": float(np.percentile(real_deg[s][m], 95)),
                "game_deg_p50": float(np.percentile(game_deg[s][m], 50)),
                "game_deg_p95": float(np.percentile(game_deg[s][m], 95)),
                "share_game_over_45": float((game_deg[s][m] > 45).mean()),
                "share_E_game_over_45": float((np.degrees(np.arctan(gE[s][m])) > 45).mean()),
            }
            if age is not None:
                t["age_myr"] = float(np.median(age[s]))
            if prov is not None:
                vals, cnt = np.unique(np.asarray(prov[s]), return_counts=True)
                t["province"] = int(vals[cnt.argmax()])
            # Dominant wavelength of the high-passed H (radially averaged power).
            f = hH[s] - hH[s].mean()
            f = f * np.outer(np.hanning(n), np.hanning(n))
            P = np.abs(np.fft.fft2(f)) ** 2
            ky = np.fft.fftfreq(n, d=step)
            kx = np.fft.fftfreq(n, d=step)
            Kr = np.hypot(*np.meshgrid(kx, ky))
            kb = np.linspace(0, Kr.max(), n // 2)
            idx = np.digitize(Kr.ravel(), kb)
            pr = np.bincount(idx, weights=P.ravel(), minlength=len(kb) + 1)[1:len(kb)]
            kc = 0.5 * (kb[1:] + kb[:-1])
            ok = kc > 1.0 / args.tile
            if ok.any():
                k_peak = kc[ok][np.argmax(pr[ok] * kc[ok])]  # peak of k*P(k): the band that carries the variance
                t["lambda_peak_blocks"] = float(1.0 / k_peak)
            tiles.append(t)
    (out / "tiles.json").write_text(json.dumps({"meta": meta, "tiles": tiles}))
    # Planet-wide distributions over deep sea floor.
    m = deep

    def wpct(v, w, ps):
        o = np.argsort(v)
        v, w = v[o], w[o]
        c = np.cumsum(w)
        c /= c[-1]
        return {str(p): float(np.interp(p / 100.0, c, v)) for p in ps}

    rd, aw = real_deg[m], area_w[m]
    summary = {
        "coarsen": K, "baseline_blocks": step,
        "real_deg_pct_area": wpct(rd, aw, (50, 75, 90, 95, 99, 99.9)),
        "share_real_over_area": {str(t): float(aw[rd > t].sum() / aw.sum()) for t in (5, 10, 15, 20, 30)},
        "mpb_pct": {p: float(np.percentile(mpb[m], p)) for p in (1, 50, 99)},
        "deep_samples": int(m.sum()),
        "rms_H_m": float(hH[m].std() * args.mpb_v),
        "rms_E_m": float(hE[m].std() * args.mpb_v),
        "rms_D_m": float(hD[m].std() * args.mpb_v),
        "real_deg_pct": {p: float(np.percentile(real_deg[m], p)) for p in (50, 75, 90, 95, 99, 99.9)},
        "game_deg_pct": {p: float(np.percentile(game_deg[m], p)) for p in (50, 75, 90, 95, 99, 99.9)},
        "share_real_over_5": float((real_deg[m] > 5).mean()),
        "share_real_over_10": float((real_deg[m] > 10).mean()),
        "share_real_over_20": float((real_deg[m] > 20).mean()),
        "share_game_over_45": float((game_deg[m] > 45).mean()),
        "tile_rms_H_pct": {p: float(np.percentile([t["rms_H_m"] for t in tiles], p)) for p in (10, 50, 90, 95, 99, 100)} if tiles else None,
        "tile_rms_H_pct_area": wpct(np.array([t["rms_H_m"] for t in tiles]), np.array([t["mpb"] ** 2 for t in tiles]), (10, 50, 90, 95, 99, 100)) if tiles else None,
    }
    # The sea as a whole: how much of it is shelf (shallower than 200 m) and how deep it is on
    # average, so a sea-floor change that drags the shelves down shows.
    sea = H < args.sea
    sw = area_w[sea]
    summary["sea_share"] = float(sw.sum() / area_w.sum())
    summary["shelf_share_of_sea"] = float(area_w[sea & (H > args.sea - 200.0 / args.mpb_v)].sum() / sw.sum())
    summary["mean_depth_m"] = float(((args.sea - H[sea]) * sw).sum() / sw.sum() * args.mpb_v)
    summary["hadal_share_of_sea"] = float(area_w[sea & (H < args.sea - 6000.0 / args.mpb_v)].sum() / sw.sum())
    summary["max_depth_m"] = float((args.sea - H[sea].min()) * args.mpb_v)
    summary["share_game_over_45_area"] = float(aw[game_deg[m] > 45].sum() / aw.sum())
    (out / "summary.json").write_text(json.dumps(summary, indent=1))
    print(json.dumps(summary, indent=1))
    worst = sorted(tiles, key=lambda t: -t["rms_H_m"])[:25]
    for t in worst:
        print(json.dumps({k: (round(v, 2) if isinstance(v, float) else v) for k, v in t.items()}))
    if args.png:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        fig, ax = plt.subplots(3, 1, figsize=(16, 24))
        ext = [x0, x0 + cols * step, z0 + rows * step, z0]
        ax[0].imshow(np.where(H < args.sea, H, np.nan), cmap="viridis", extent=ext, interpolation="nearest")
        ax[0].set_title("H at sea (blocks)")
        rough = ndimage.gaussian_filter(np.where(deep, hH, 0) ** 2, 8) ** 0.5 * args.mpb_v
        im = ax[1].imshow(np.where(deep, rough, np.nan), cmap="magma", vmin=0, vmax=800, extent=ext, interpolation="nearest")
        fig.colorbar(im, ax=ax[1]); ax[1].set_title("local RMS of high-passed H (m), deep sea floor")
        im = ax[2].imshow(np.where(deep, game_deg, np.nan), cmap="magma", vmin=0, vmax=70, extent=ext, interpolation="nearest")
        fig.colorbar(im, ax=ax[2]); ax[2].set_title("in-game slope (deg), deep sea floor")
        fig.savefig(out / "overview.png", dpi=80, bbox_inches="tight")


if __name__ == "__main__":
    main()
