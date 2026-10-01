#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Planet-scale before/after figures for the glacial landforms: the deep-time surface (L1) of two
runs side by side with their last glaciers, and the land hypsometry of both.

    deeptime dump-fields --in before/checkpoints/4-deposits.dtcp --out f_before \\
        --fields surface/elevation_m,climate/glacier
    deeptime dump-fields --in after/checkpoints/4-deposits.dtcp --out f_after \\
        --fields surface/elevation_m,surface/ice_thickness_m,surface/glacial_erosion_m_myr,climate/glacier
    python3 tools/review/glacial_planet.py --run before=f_before --run after=f_after --sea 25 --out DIR

Needs numpy and matplotlib (see glacial_views.py for the Mac's nix shell).
"""

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402


def load(folder: Path) -> dict:
    index = json.loads((folder / "index.json").read_text())
    return {k: np.fromfile(folder / e["file"], dtype="<f4").reshape(e["h"], e["w"])
            for k, e in index.items()}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--run", action="append", required=True, help="label=dump-fields folder")
    ap.add_argument("--sea", type=float, default=0.0)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    runs = [(s.split("=", 1)[0], load(Path(s.split("=", 1)[1]))) for s in args.run]

    fig, axes = plt.subplots(len(runs), 1, figsize=(14, 7.2 * len(runs)), squeeze=False)
    stats = {}
    for ax, (label, f) in zip(axes[:, 0], runs):
        z = f["surface/elevation_m"] - args.sea
        h, w = z.shape
        lat = 90 - (np.arange(h) + 0.5) * 180 / h
        wgt = np.cos(np.radians(lat))[:, None] * np.ones((1, w))
        land = z > 0
        rgb = np.where(land[..., None],
                       matplotlib.colormaps["terrain"](0.25 + 0.75 * np.clip(z / 5000, 0, 1))[..., :3],
                       matplotlib.colormaps["Blues_r"](0.2 + 0.6 * np.clip(-z / 6000, 0, 1))[..., :3])
        ax.imshow(rgb, extent=[-180, 180, -90, 90], interpolation="nearest")
        ice = f.get("surface/ice_thickness_m")
        if ice is None:
            ice = f.get("climate/glacier")
        if ice is not None:
            ax.contour(np.linspace(-180, 180, w), lat, ice > 0, levels=[0.5], colors="white", linewidths=0.6)
        ax.set_title(f"{label}: deep-time surface at the present (white: glaciers)")
        zl = z[land]
        wl = wgt[land]
        order = np.argsort(zl)
        cum = np.cumsum(wl[order]) / wl.sum()
        q = lambda p: float(zl[order][np.searchsorted(cum, p)])  # noqa: E731
        s = {"land_frac": float(wl.sum() / wgt.sum()), "z50": q(0.5), "z90": q(0.9), "z99": q(0.99),
             "zmax": float(zl.max())}
        glac = f.get("climate/glacier")
        if glac is not None:
            s["glaciated_land_frac"] = float((wgt * (glac > 0) * land).sum() / wl.sum())
        if "surface/glacial_erosion_m_myr" in f:
            s["glacial_erosion_max_m_myr"] = float(f["surface/glacial_erosion_m_myr"].max())
        stats[label] = s
    fig.tight_layout()
    fig.savefig(args.out / "planet-maps.png", dpi=90)
    plt.close(fig)

    fig, ax = plt.subplots(figsize=(8, 5))
    for label, f in runs:
        z = f["surface/elevation_m"] - args.sea
        h, w = z.shape
        lat = 90 - (np.arange(h) + 0.5) * 180 / h
        wgt = (np.cos(np.radians(lat))[:, None] * np.ones((1, w)))[z > 0]
        zl = z[z > 0]
        order = np.argsort(zl)[::-1]
        ax.plot(np.cumsum(wgt[order]) / wgt.sum(), zl[order], label=label)
    ax.set_xlabel("share of the land above")
    ax.set_ylabel("elevation above sea, m")
    ax.set_title("Land hypsometry")
    ax.legend()
    fig.tight_layout()
    fig.savefig(args.out / "planet-hypsometry.png", dpi=100)
    plt.close(fig)
    (args.out / "planet-stats.json").write_text(json.dumps(stats, indent=1))
    print(json.dumps(stats, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
