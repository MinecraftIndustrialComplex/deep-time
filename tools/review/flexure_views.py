#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Review pictures of the surface stage's flexure (docs/design/flexure.md): the planet, the
largest volcanic load on ocean floor with its moat, and the highest collision belt with the basin
in front of it, each with sections through it.

    deeptime dump-fields --in run.dtcp --out fields --fields surface/elevation_m,\\
        surface/flexure_m,surface/deflection_m,surface/elastic_thickness_km,plates/crust_type,regime,plates/flags
    python3 tools/review/flexure_views.py --fields fields --sea 0 --out DIR [--label on]

Writes planet.png, moat.png, belt.png and numbers.json. Needs numpy and matplotlib (on the VM:
nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}
.python3.withPackages (p: [p.numpy p.matplotlib])').
"""

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from matplotlib.colors import LinearSegmentedColormap, TwoSlopeNorm  # noqa: E402

# Two hues and a neutral middle: below local isostasy (basins, moats) blue, above it red.
DIVERGING = LinearSegmentedColormap.from_list("flex", ["#1f4e8c", "#7fa6d6", "#d9d9d9", "#e39a7a", "#a8321e"])
INK = "#333333"
HOTSPOT, COLLISION = 7, 5  # regime codes (plates/state.rs)


def load(folder: Path) -> dict:
    index = json.loads((folder / "index.json").read_text())
    return {k: np.fromfile(folder / e["file"], dtype="<f4").reshape(e["h"], e["w"]) for k, e in index.items()}


def relief(z: np.ndarray) -> np.ndarray:
    """Land light to dark with height, sea light to dark with depth."""
    land = matplotlib.colormaps["YlOrBr"](0.08 + 0.85 * np.clip(z / 6000, 0, 1))[..., :3]
    sea = matplotlib.colormaps["Blues"](0.25 + 0.7 * np.clip(-z / 7000, 0, 1))[..., :3]
    return np.where((z > 0)[..., None], land, sea)


def window(a: np.ndarray, j: int, i: int, half: int) -> np.ndarray:
    h, w = a.shape
    rows = np.clip(np.arange(j - half, j + half + 1), 0, h - 1)
    cols = np.arange(i - half, i + half + 1) % w
    return a[np.ix_(rows, cols)]


def close_up(f: dict, z: np.ndarray, j: int, i: int, half: int, km: float, title: str, path: Path) -> dict:
    """A map of the ground and of the flexure round (j, i), and W–E and N–S sections through it."""
    zw, fw, dw = (window(a, j, i, half) for a in (z, f["surface/flexure_m"], f["surface/deflection_m"]))
    ext = [-half * km, half * km, half * km, -half * km]
    fig, ax = plt.subplots(2, 2, figsize=(13, 10.5))
    ax[0, 0].imshow(relief(zw), extent=ext, interpolation="nearest")
    ax[0, 0].contour(np.linspace(ext[0], ext[1], zw.shape[1]), np.linspace(ext[3], ext[2], zw.shape[0]), zw,
                     levels=[0], colors=INK, linewidths=0.7)
    ax[0, 0].set_title("Ground (land brown, sea blue; line: the coast)", color=INK)
    span = max(200.0, float(np.percentile(np.abs(fw), 99)))
    im = ax[0, 1].imshow(fw, extent=ext, cmap=DIVERGING, norm=TwoSlopeNorm(0, -span, span), interpolation="nearest")
    fig.colorbar(im, ax=ax[0, 1], shrink=0.8, label="m above (+) or below (−) local isostasy")
    ax[0, 1].set_title("Flexure: what the plate adds to local isostasy", color=INK)
    x = np.arange(-half, half + 1) * km
    for a, (name, zs, fs, ds) in zip(ax[1], (("West–east", zw[half], fw[half], dw[half]),
                                              ("North–south", zw[:, half], fw[:, half], dw[:, half]))):
        a.axhline(0, color="#999999", linewidth=0.8)
        a.plot(x, zs / 1000, color="#8c5a2b", linewidth=2, label="ground")
        a.plot(x, (zs - fs) / 1000, color="#8c5a2b", linewidth=1.2, linestyle=":", label="ground under local isostasy")
        a.plot(x, -ds / 1000, color="#1f4e8c", linewidth=2, label="the plate since the start: down (−), up (+)")
        a.set_title(f"{name} section", color=INK)
        a.set_xlabel("km (plate kilometres: Earth-normalized)")
        a.set_ylabel("km")
        a.grid(color="#e6e6e6", linewidth=0.6)
        a.legend(frameon=False, fontsize=9)
    for a in ax[0]:
        a.set_xlabel("km")
        a.axhline(0, color=INK, linewidth=0.4)
        a.axvline(0, color=INK, linewidth=0.4)
    fig.suptitle(title, color=INK, fontsize=13)
    fig.tight_layout()
    fig.savefig(path, dpi=110)
    plt.close(fig)
    return {"ground_m": [float(v) for v in zw[half]], "flexure_m": [float(v) for v in fw[half]],
            "ground_ns_m": [float(v) for v in zw[:, half]], "flexure_ns_m": [float(v) for v in fw[:, half]],
            "km_per_cell": km}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--fields", type=Path, required=True)
    ap.add_argument("--sea", type=float, default=0.0)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--label", default="")
    ap.add_argument("--model-radius-km", type=float, default=6370.0)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    f = load(args.fields)
    z = f["surface/elevation_m"] - args.sea
    for name in ("surface/flexure_m", "surface/deflection_m", "surface/elastic_thickness_km"):
        f.setdefault(name, np.zeros_like(z))  # a run without flexure: only the ground is drawn
    flex, te = f["surface/flexure_m"], f["surface/elastic_thickness_km"]
    h, w = z.shape
    km = 2 * np.pi * args.model_radius_km / w
    numbers: dict = {"km_per_cell": km}

    fig, ax = plt.subplots(3, 1, figsize=(14, 21))
    ext = [-180, 180, -90, 90]
    ax[0].imshow(relief(z), extent=ext, interpolation="nearest")
    ax[0].set_title(f"{args.label} ground at the end of the run (land brown, sea blue)", color=INK)
    span = max(1.0, float(np.percentile(np.abs(flex), 99.5)))
    im = ax[1].imshow(flex, extent=ext, cmap=DIVERGING, norm=TwoSlopeNorm(0, -span, span), interpolation="nearest")
    fig.colorbar(im, ax=ax[1], shrink=0.7, label="m above (+) or below (−) local isostasy")
    ax[1].contour(np.linspace(-180, 180, w), np.linspace(90, -90, h), z, levels=[0], colors=INK, linewidths=0.3)
    ax[1].set_title("Flexure: blue where the plate is pulled down by loads beside it (basins, moats), "
                    "red where it holds a load up or bulges", color=INK)
    im = ax[2].imshow(te, extent=ext, cmap="Greys", vmin=0, interpolation="nearest")
    fig.colorbar(im, ax=ax[2], shrink=0.7, label="km")
    ax[2].contour(np.linspace(-180, 180, w), np.linspace(90, -90, h), z, levels=[0], colors="#a8321e", linewidths=0.3)
    ax[2].set_title("Effective elastic thickness (dark: stiff old ocean floor and cratons; red line: the coast)",
                    color=INK)
    fig.tight_layout()
    fig.savefig(args.out / "planet.png", dpi=90)
    plt.close(fig)

    lat = 90 - (np.arange(h) + 0.5) * 180 / h
    inner = (np.abs(lat) < 70)[:, None]
    ocean = f["plates/crust_type"] < 0.5
    hot = np.abs(f["regime"] - HOTSPOT) < 0.5
    # A hotspot's load inside a plate: stiff floor all round it, not an arc at a plate's weak edge.
    cand = np.where(ocean & hot & inner, flex, -np.inf)
    j, i = np.unravel_index(int(np.argmax(cand)), cand.shape)
    for flat in np.argsort(cand, axis=None)[::-1][:400]:
        jj, ii = np.unravel_index(int(flat), cand.shape)
        if np.isfinite(cand[jj, ii]) and window(te, jj, ii, 6).min() > 12 and window(ocean, jj, ii, 6).all():
            j, i = jj, ii
            break
    half = 16
    sec = close_up(f, z, j, i, half, km,
                   f"{args.label} The largest volcanic load on ocean floor (cell {i}, {j}; Te {te[j, i]:.0f} km)",
                   args.out / "moat.png")
    ring = window(flex, j, i, half)
    numbers["moat"] = {"cell": [int(i), int(j)], "te_km": float(te[j, i]), "peak_m": float(z[j, i]),
                       "held_up_m": float(flex[j, i]), "deepest_moat_m": float(ring.min()), **sec}

    coll = (np.abs(f["regime"] - COLLISION) < 0.5) & inner
    cand = np.where(coll, z, -np.inf)
    if np.isfinite(cand.max()):
        j, i = np.unravel_index(int(np.argmax(cand)), cand.shape)
        half = 28
        sec = close_up(f, z, j, i, half, km,
                       f"{args.label} The highest collision belt (cell {i}, {j}; {z[j, i]:.0f} m)",
                       args.out / "belt.png")
        numbers["belt"] = {"cell": [int(i), int(j)], "peak_m": float(z[j, i]),
                           "deepest_basin_m": float(window(flex, j, i, half).min()), **sec}
    # The worst margin: the window along a plate boundary with the most cells that zigzag (the
    # flexure follow-up, tools/review/flexure_artifacts.py), ground and flexure with sections.
    if "plates/plate_id" in f:
        import sys
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from flexure_artifacts import zigzag  # noqa: E402
        pid = f["plates/plate_id"]
        edge = (pid != np.roll(pid, 1, 1)) | (pid != np.roll(pid, -1, 1))
        edge[1:] |= pid[1:] != pid[:-1]
        zz = (zigzag(z) & ~np.repeat((np.abs(lat) > 70)[:, None], w, 1)).astype(float)
        dens = zz.copy()
        for _ in range(9):
            dens = (dens + np.roll(dens, 1, 1) + np.roll(dens, -1, 1)) / 3
            dens[1:-1] = (dens[:-2] + dens[1:-1] + dens[2:]) / 3
        dens[~edge] = 0
        j, i = np.unravel_index(int(np.argmax(dens)), dens.shape)
        half = 9
        sec = close_up(f, z, j, i, half, km, f"{args.label} The margin with the most cell-scale zigzag (cell {i}, {j})",
                       args.out / "margin.png")
        numbers["margin"] = {"cell": [int(i), int(j)], "zigzag_cells_in_window": int(window(zz, j, i, half).sum()),
                             **sec}
    # The north polar band: ground and flexure, and a section along the meridian through the
    # strongest flexure there (the stripes of the first build ran east-west near the pole).
    band = int(h * 20 / 180)
    fig, ax = plt.subplots(3, 1, figsize=(14, 11))
    ext = [-180, 180, 90 - 180 * band / h, 90]
    ax[0].imshow(relief(z[:band]), extent=ext, aspect="auto", interpolation="nearest")
    ax[0].set_title(f"{args.label} north of 70°: ground", color=INK)
    span = max(50.0, float(np.percentile(np.abs(flex[:band]), 99.5)))
    im = ax[1].imshow(flex[:band], extent=ext, aspect="auto", cmap=DIVERGING, norm=TwoSlopeNorm(0, -span, span),
                      interpolation="nearest")
    fig.colorbar(im, ax=ax[1], shrink=0.8, label="m above (+) or below (−) local isostasy")
    ax[1].set_title("north of 70°: flexure", color=INK)
    ip = int(np.argmax(np.abs(flex[:band]).max(axis=0)))
    ax[2].plot(lat[:band], z[:band, ip], color="#8c5a2b", linewidth=2, label="ground")
    ax[2].plot(lat[:band], flex[:band, ip], color="#1f4e8c", linewidth=1.5, label="flexure")
    ax[2].set_xlabel("latitude, °")
    ax[2].set_ylabel("m")
    ax[2].set_title(f"meridian section at column {ip}", color=INK)
    ax[2].grid(color="#e6e6e6", linewidth=0.6)
    ax[2].legend(frameon=False)
    fig.tight_layout()
    fig.savefig(args.out / "pole.png", dpi=90)
    plt.close(fig)
    # Roughness of the ground: how far a cell stands from the mean of its four neighbours.
    mean4 = (np.roll(z, 1, 1) + np.roll(z, -1, 1) + np.vstack([z[:1], z[:-1]]) + np.vstack([z[1:], z[-1:]])) / 4
    dev = np.abs(z - mean4)
    for name, mask in (("land", (z > 0) & inner), ("sea", (z <= 0) & inner)):
        numbers[f"roughness_{name}_p50_m"] = float(np.median(dev[mask]))
        numbers[f"roughness_{name}_p99_m"] = float(np.percentile(dev[mask], 99))
    # How much of the planet is off local isostasy, and by how much.
    wgt = np.cos(np.radians(lat))[:, None] * np.ones((1, w))
    for name, mask in (("below_100m", flex < -100), ("below_500m", flex < -500), ("above_100m", flex > 100),
                       ("above_500m", flex > 500)):
        numbers[name] = float(wgt[mask].sum() / wgt.sum())
    numbers["flexure_min_m"], numbers["flexure_max_m"] = float(flex.min()), float(flex.max())
    (args.out / "numbers.json").write_text(json.dumps(numbers, indent=1))
    print(json.dumps({k: v for k, v in numbers.items() if not isinstance(v, dict)}))
    for k in ("moat", "belt", "margin"):
        if k in numbers:
            print(k, {a: b for a, b in numbers[k].items() if not isinstance(b, list)})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
