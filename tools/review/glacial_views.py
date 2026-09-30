#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Review figures for the glacial landforms (surface/glacial.rs, surface/amplify/glacial.rs).

Finds the most glaciated mountain window and the best fjord coast of a run from its young
landscape's fields, and draws, for every L3 terrain given, a hill-shaded map of each window and
cross-sections through its thickest ice, in blocks at 1:1 (what a player sees).

    deeptime stage young --in deposits.dtcp --out y.dtcp          # glaciers on
    deeptime dump-fields --in y.dtcp --out fields \\
        --fields young/ice_thickness_m,young/ice_surface_m,young/elevation_m,young/glacial_erosion_m,young/till_m
    deeptime stage amplify --in y.dtcp --out a.dtcp               # a.dtcp.work/rasters/elevation.dtr
    python3 tools/review/glacial_views.py --fields fields --l3 on=a.dtcp.work \\
        --l3 off=a0.dtcp.work --out review/out/glacial

Needs numpy, matplotlib and lz4 (on the Mac: nix shell --impure --expr
'(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages
(p: [p.numpy p.matplotlib p.lz4])' -c python3 …).
"""

import argparse
import json
import struct
from pathlib import Path

import lz4.block
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

SEA_Y = 63.0


def read_dtr(path: Path) -> tuple[np.ndarray, dict]:
    """A whole tile pack as physical values (FORMAT.md, "Tile pack")."""
    data = path.read_bytes()
    assert data[:4] == b"DTR1", path
    header_len = struct.unpack_from("<I", data, 4)[0]
    desc = json.loads(data[16:header_len])
    g = desc["grid"]
    w, h, tile = g["w"], g["h"], desc["tile"]
    dt = {"i16": np.int16, "u8": np.uint8, "f32": np.float32, "u16": np.uint16}[desc["dtype"]]
    size = np.dtype(dt).itemsize
    tx, tz = -(-w // tile), -(-h // tile)
    index_off = (header_len + 7) // 8 * 8
    out = np.zeros((h, w), dtype=np.float32)
    for t in range(tx * tz):
        off, length, _crc = struct.unpack_from("<QII", data, index_off + 16 * t)
        x0, z0 = (t % tx) * tile, (t // tx) * tile
        tw, th = min(tile, w - x0), min(tile, h - z0)
        raw = data[off:off + length]
        if desc["codec"] == "lz4_block":
            raw = lz4.block.decompress(raw, uncompressed_size=tw * th * size)
        arr = np.frombuffer(raw, dtype=dt).reshape(th, tw).astype(np.float32)
        out[z0:z0 + th, x0:x0 + tw] = arr * desc.get("scale", 1.0) + desc.get("offset", 0.0)
    return out, desc


def read_fields(folder: Path) -> tuple[dict, dict]:
    index = json.loads((folder / "index.json").read_text())
    fields = {}
    for name, e in index.items():
        fields[name] = np.fromfile(folder / e["file"], dtype="<f4").reshape(e["h"], e["w"])
    return fields, index


def best_windows(score: np.ndarray, size: int, count: int) -> list[tuple[int, int]]:
    """Top-left corners of the `count` best non-overlapping size × size windows (x wraps)."""
    h, w = score.shape
    s = np.nan_to_num(score)
    ext = np.concatenate([s, s[:, :size]], axis=1)
    c = np.cumsum(np.cumsum(ext, axis=0), axis=1)
    c = np.pad(c, ((1, 0), (1, 0)))
    sums = c[size:, size:] - c[:-size, size:] - c[size:, :-size] + c[:-size, :-size]
    sums = sums[: h - size + 1, :w]
    picks = []
    for _ in range(count):
        j, i = np.unravel_index(np.argmax(sums), sums.shape)
        if sums[j, i] <= 0:
            break
        picks.append((int(i), int(j)))
        for dj in range(-size, size + 1):
            for di in range(-size, size + 1):
                jj, ii = j + dj, (i + di) % w
                if 0 <= jj < sums.shape[0]:
                    sums[jj, ii] = -1
    return picks


def crop(a: np.ndarray, x0: int, y0: int, n: int) -> np.ndarray:
    cols = np.arange(x0, x0 + n) % a.shape[1]
    return a[y0:y0 + n][:, cols]


def hillshade(y: np.ndarray, cell_blocks: float) -> np.ndarray:
    gy, gx = np.gradient(y, cell_blocks)
    az, alt = np.radians(315), np.radians(40)
    slope = np.arctan(np.hypot(gx, gy))
    aspect = np.arctan2(-gx, gy)
    return np.clip(np.sin(alt) * np.cos(slope) + np.cos(alt) * np.sin(slope) * np.cos(az - aspect), 0, 1)


def colour(y: np.ndarray, cell_blocks: float) -> np.ndarray:
    land = y >= SEA_Y
    rel = np.clip((y - SEA_Y) / 250.0, 0, 1)
    cmap = matplotlib.colormaps["terrain"]
    rgb = cmap(0.25 + 0.75 * rel)[..., :3]
    sea = matplotlib.colormaps["Blues_r"](np.clip((SEA_Y - y) / 120.0, 0, 1) * 0.6 + 0.2)[..., :3]
    rgb = np.where(land[..., None], rgb, sea)
    hs = hillshade(y, cell_blocks)[..., None]
    return np.clip(rgb * (0.35 + 0.75 * hs), 0, 1)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--fields", type=Path, required=True, help="dump-fields output of the young stage")
    ap.add_argument("--l3", action="append", required=True, help="label=folder holding rasters/elevation.dtr")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--window", type=int, default=48, help="window edge in L2 cells")
    ap.add_argument("--mv", type=float, default=20.0, help="metres per block vertically")
    ap.add_argument("--sea", type=float, default=None, help="sea level, m (default: guessed 0)")
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)

    f, index = read_fields(args.fields)
    ice = f["young/ice_thickness_m"]
    zs = f.get("young/ice_surface_m")
    z2 = f["young/elevation_m"]
    ero = f.get("young/glacial_erosion_m", ice)
    sea_m = args.sea if args.sea is not None else 0.0
    runs = []
    for spec in args.l3:
        label, folder = spec.split("=", 1)
        y, desc = read_dtr(Path(folder) / "rasters" / "elevation.dtr")
        runs.append((label, y))
    l2_r = index["young/ice_thickness_m"]["r"]
    l3_r = (l2_r * ice.shape[1]) // runs[0][1].shape[1]
    k = l2_r // l3_r
    n2 = args.window

    land = z2 > sea_m
    # Valley glaciers, not ice sheets: glacial erosion where ground within 3 L2 cells stands above
    # the ice surface (the ice is confined).
    zmax = z2.copy()
    for dj in range(-3, 4):
        for di in range(-3, 4):
            zmax = np.maximum(zmax, np.roll(np.roll(z2, dj, axis=0), di, axis=1))
    surf = np.where(np.isfinite(zs), zs, -1e9) if zs is not None else z2 + ice
    confined = np.clip((zmax - surf) / 300.0, 0.0, 1.0)
    mountain = np.where(land & (ice > 0), ero * confined, 0.0)
    # Fjords: sea cells the glaciers cut that land surrounds on most sides (inlets, not open
    # coast), away from the pole rows.
    land_n = np.zeros_like(ice)
    for dj in range(-2, 3):
        for di in range(-2, 3):
            land_n += np.roll(np.roll(land.astype(np.float32), dj, axis=0), di, axis=1)
    inlet = np.clip((land_n - 10.0) / 8.0, 0.0, 1.0)
    coast = np.zeros_like(ice)
    below = (z2 < sea_m) & (ero > 0)
    coast[below] = (ero * inlet)[below]
    edge = max(1, ice.shape[0] // 20)
    coast[:edge] = 0
    coast[-edge:] = 0
    views = [("glaciated range", w) for w in best_windows(mountain, n2, 2)]
    views += [("fjord coast", w) for w in best_windows(coast, n2, 2)]

    report = []
    for vi, (kind, (x0, y0)) in enumerate(views):
        name = f"view{vi}"
        ice_w = crop(ice, x0, y0, n2)
        ncol = len(runs) + 1
        fig, axes = plt.subplots(1, ncol, figsize=(5.5 * ncol, 5.8), squeeze=False)
        ice_up = np.kron(ice_w, np.ones((k, k)))
        for ax, (label, y) in zip(axes[0], runs):
            yy = crop(y, x0 * k, y0 * k, n2 * k)
            ax.imshow(colour(yy, l3_r), interpolation="nearest")
            ax.contour(yy, levels=np.arange(80, 640, 20), colors="black", linewidths=0.25, alpha=0.5)
            ax.contour(ice_up, levels=[1.0], colors="cyan", linewidths=0.7)
            ax.set_title(f"{label}", fontsize=10)
            ax.set_xticks([])
            ax.set_yticks([])
        ax = axes[0][-1]
        im = ax.imshow(ice_up, cmap="Blues", interpolation="nearest")
        fig.colorbar(im, ax=ax, fraction=0.046, label="glacial-state ice, m")
        ax.set_title(f"{kind}: ice at L2 ({x0}, {y0})", fontsize=10)
        ax.set_xticks([])
        ax.set_yticks([])
        fig.tight_layout()
        fig.savefig(args.out / f"{name}-map.png", dpi=110)
        plt.close(fig)

        # A zoom on the thickest ice: the middle third of the window at full resolution.
        jj, ii = np.unravel_index(np.argmax(ice_w), ice_w.shape)
        zn = n2 * k // 3
        zx0 = int(np.clip((x0 + ii) * k - zn // 2, x0 * k, (x0 + n2) * k - zn))
        zy0 = int(np.clip((y0 + jj) * k - zn // 2, y0 * k, (y0 + n2) * k - zn))
        fig, axes = plt.subplots(1, len(runs), figsize=(6 * len(runs), 6), squeeze=False)
        for ax, (label, y) in zip(axes[0], runs):
            yy = crop(y, zx0, zy0, zn)
            ax.imshow(colour(yy, l3_r), interpolation="nearest")
            ax.contour(yy, levels=np.arange(80, 640, 10), colors="black", linewidths=0.3, alpha=0.5)
            ax.set_title(f"{label} (zoom, {zn * l3_r} blocks across)", fontsize=10)
            ax.set_xticks([])
            ax.set_yticks([])
        fig.tight_layout()
        fig.savefig(args.out / f"{name}-zoom.png", dpi=110)
        plt.close(fig)

        # Cross-sections through the thickest ice of the window, both ways, in blocks at 1:1.
        jj, ii = np.unravel_index(np.argmax(ice_w), ice_w.shape)
        cj, ci = (y0 + jj) * k, (x0 + ii) * k
        half = n2 * k // 2
        fig, axes = plt.subplots(2, 1, figsize=(12, 8))
        for ax, horizontal in zip(axes, [True, False]):
            for label, y in runs:
                if horizontal:
                    cols = np.arange(ci - half, ci + half) % y.shape[1]
                    prof = y[cj, cols]
                else:
                    rows = np.clip(np.arange(cj - half, cj + half), 0, y.shape[0] - 1)
                    prof = y[rows, ci % y.shape[1]]
                ax.plot(np.arange(len(prof)) * l3_r, prof, label=label, lw=1.2)
            if zs is not None:
                if horizontal:
                    cols2 = np.arange((ci - half) // k, (ci + half) // k) % zs.shape[1]
                    s = zs[cj // k, cols2]
                else:
                    rows2 = np.clip(np.arange((cj - half) // k, (cj + half) // k), 0, zs.shape[0] - 1)
                    s = zs[rows2, (ci // k) % zs.shape[1]]
                sy = SEA_Y + (s - sea_m) / args.mv
                ax.plot(np.arange(len(sy)) * l2_r, sy, color="#6ab0e0", lw=1, ls="--", label="glacial ice surface")
            ax.axhline(SEA_Y, color="#2060a0", lw=0.6)
            ax.set_aspect("equal")
            ax.set_ylabel("block y")
            ax.set_title(("west-east" if horizontal else "north-south") + f" through L3 ({ci}, {cj})")
            ax.legend(fontsize=8, loc="upper right")
        axes[1].set_xlabel("blocks")
        fig.tight_layout()
        fig.savefig(args.out / f"{name}-section.png", dpi=110)
        plt.close(fig)
        c_blocks = l3_r * runs[0][1].shape[1]
        bx, bz = int(-c_blocks // 2 + ci * l3_r), int(-c_blocks // 4 + cj * l3_r)
        report.append({"kind": kind, "l2": [x0, y0], "l3_centre": [int(ci), int(cj)],
                       "block_centre": [bx, bz],
                       "ice_max_m": float(ice_w.max()), "map": f"{name}-map.png",
                       "section": f"{name}-section.png"})
        print(f"{name}: {kind} L2 ({x0},{y0}) ice max {ice_w.max():.0f} m, section at L3 ({ci},{cj}), "
              f"block ({bx},{bz})")

    html = ["<!doctype html><meta charset=utf-8><title>Glacial landforms</title>",
            "<style>body{font:14px system-ui;background:#111;color:#ddd;margin:16px}img{max-width:100%}</style>",
            "<h1>Glacial landforms</h1>"]
    for r in report:
        html.append(f"<h2>{r['kind']} (L2 {r['l2']}, ice up to {r['ice_max_m']:.0f} m)</h2>")
        html.append(f"<img src='{r['map']}'><br><img src='{r['map'].replace('-map', '-zoom')}'><br><img src='{r['section']}'>")
    (args.out / "index.html").write_text("\n".join(html))
    (args.out / "views.json").write_text(json.dumps(report, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
