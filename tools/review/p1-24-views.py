#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-24 quality pictures: what each faster-quick-mode candidate's planet looks like.

For every baked world-data folder given (label=folder), writes into --out:
  <label>-screen.png    the last keyframe's satellite layer, 1024 x 326: what the Create World
                        screen shows at the end of the run
  <label>-planet.png    the whole planet's terrain height (deeptime heightmap, a sample every 16
                        blocks), hill-shaded: coastlines, ranges and basins at map scale
  <label>-ground.png    the most rugged 1024 x 1024-block land window, every block, hill-shaded:
                        what a player walks on
  <label>-coast.png     the 1024 x 1024 window with the most coastline, every block
and views.json (the windows, their relief, land share and number of distinct drainage lines).
Each candidate is a different planet (a coarser grid or a longer step changes the whole history),
so the windows are each planet's own most rugged or most coastal ones, not the same place.

    python3 tools/review/p1-24-views.py --bin target/release/deeptime --out <dir> base=<world> ...

Needs numpy, matplotlib and lz4 (on the VM: nix shell --impure --expr
'(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages
(p: [p.numpy p.matplotlib p.lz4])' -c python3 ...).
"""

import argparse
import json
import struct
import subprocess
from pathlib import Path

import lz4.block
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

SEA_Y = 63.0
WIN = 1024


def last_keyframe(world: Path, layer: str) -> np.ndarray | None:
    """The newest frame's RGBA8 `layer` (FORMAT.md "Keyframes", DTKF)."""
    index = json.loads((world / "keyframes" / "index.json").read_text())
    frames = [f for f in index["frames"] if layer in f["layers"]]
    if not frames:
        return None
    f = max(frames, key=lambda f: f["n"])
    data = (world / f["path"]).read_bytes()
    assert data[:4] == b"DTKF", f["path"]
    n, w, h, nl = struct.unpack_from("<IIII", data, 16)
    table = 16 + 24
    sections = [struct.unpack_from("<QII", data, table + 16 * k) for k in range(1 + nl)]
    off, ln, _ = sections[0]
    names, p = [], off
    for _ in range(1 + nl):
        (sl,) = struct.unpack_from("<I", data, p)
        names.append(data[p + 4:p + 4 + sl].decode())
        p += 4 + sl
    k = names[1:].index(layer)
    off, ln, _ = sections[1 + k]
    raw = lz4.block.decompress(data[off:off + ln], uncompressed_size=w * h * 4)
    return np.frombuffer(raw, np.uint8).reshape(h, w, 4)


def heightmap(binary: str, world: Path, box, step: int, tmp: Path) -> np.ndarray:
    out = tmp / f"hm-{step}-{box[0]}-{box[1]}.npy"
    subprocess.run([binary, "heightmap", "--world-data", str(world), "--box",
                    ",".join(str(v) for v in box), "--step", str(step), "--out", str(out)],
                   check=True, stdout=subprocess.DEVNULL)
    return np.load(out)[..., 0]


def hillshade(y: np.ndarray, cell: float) -> np.ndarray:
    gy, gx = np.gradient(y, cell)
    az, alt = np.radians(315), np.radians(40)
    slope = np.arctan(np.hypot(gx, gy))
    aspect = np.arctan2(-gx, gy)
    return np.clip(np.sin(alt) * np.cos(slope) + np.cos(alt) * np.sin(slope) * np.cos(az - aspect), 0, 1)


def colour(y: np.ndarray, cell: float, relief: float) -> np.ndarray:
    land = y >= SEA_Y
    rgb = matplotlib.colormaps["terrain"](0.25 + 0.75 * np.clip((y - SEA_Y) / relief, 0, 1))[..., :3]
    sea = matplotlib.colormaps["Blues_r"](np.clip((SEA_Y - y) / 120.0, 0, 1) * 0.6 + 0.2)[..., :3]
    rgb = np.where(land[..., None], rgb, sea)
    return np.clip(rgb * (0.35 + 0.75 * hillshade(y, cell)[..., None]), 0, 1)


def save(path: Path, rgb: np.ndarray, title: str):
    h, w = rgb.shape[:2]
    fig = plt.figure(figsize=(w / 100, h / 100 + 0.3), dpi=100)
    ax = fig.add_axes([0, 0, 1, h / (h + 30)])
    ax.imshow(rgb, interpolation="nearest")
    ax.axis("off")
    fig.text(0.01, 1 - 12 / (h + 30), title, fontsize=9, va="center")
    fig.savefig(path)
    plt.close(fig)


def windows(over: np.ndarray, step: int, x0: int, z0: int):
    """The most rugged and the most coastal WIN-block windows on the overview (stride WIN/2)."""
    k = WIN // step
    best_r, best_c = (-1.0, None), (-1.0, None)
    for r in range(0, over.shape[0] - k + 1, k // 2):
        for c in range(0, over.shape[1] - k + 1, k // 2):
            w = over[r:r + k, c:c + k]
            land = w >= SEA_Y
            share = land.mean()
            if share > 0.8:
                score = float(np.std(w[land]))
                if score > best_r[0]:
                    best_r = (score, (x0 + c * step, z0 + r * step))
            coast = float((land[1:, :] != land[:-1, :]).mean() + (land[:, 1:] != land[:, :-1]).mean())
            if 0.3 < share < 0.7 and coast > best_c[0]:
                best_c = (coast, (x0 + c * step, z0 + r * step))
    return best_r[1], best_c[1]


def drainage_lines(y: np.ndarray) -> int:
    """A rough count of valley lines: land cells lower than both neighbours along x or along z."""
    land = y[1:-1, 1:-1] >= SEA_Y
    vx = (y[1:-1, 1:-1] < y[1:-1, :-2]) & (y[1:-1, 1:-1] < y[1:-1, 2:])
    vz = (y[1:-1, 1:-1] < y[:-2, 1:-1]) & (y[1:-1, 1:-1] < y[2:, 1:-1])
    return int(((vx | vz) & land).sum())


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--bin", default="target/release/deeptime")
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--circumference", type=int, default=16384)
    ap.add_argument("worlds", nargs="+", help="label=world-data folder")
    a = ap.parse_args()
    a.out.mkdir(parents=True, exist_ok=True)
    tmp = a.out / "tmp"
    tmp.mkdir(exist_ok=True)
    c = a.circumference
    zo = 2624 if c == 16384 else 5248
    report = {}
    for spec in a.worlds:
        label, world = spec.split("=", 1)
        world = Path(world)
        r = {}
        sat = last_keyframe(world, "satellite")
        if sat is not None:
            plt.imsave(a.out / f"{label}-screen.png", sat)
        step = 16
        over = heightmap(a.bin, world, (0, -zo, c, zo), step, tmp)
        save(a.out / f"{label}-planet.png", colour(over, step, 400.0), f"{label}: whole planet, a sample every {step} blocks")
        r["land_share"] = round(float((over >= SEA_Y).mean()), 3)
        rugged, coast = windows(over, step, 0, -zo)
        for name, origin in (("ground", rugged), ("coast", coast)):
            if origin is None:
                continue
            x, z = origin
            y = heightmap(a.bin, world, (x, z, x + WIN, z + WIN), 1, tmp)
            land = y >= SEA_Y
            relief = float(np.percentile(y[land], 99) - np.percentile(y[land], 1)) if land.any() else 0.0
            save(a.out / f"{label}-{name}.png", colour(y, 1.0, max(relief, 60.0)),
                 f"{label}: {WIN} x {WIN} blocks at x {x}, z {z}; land relief {relief:.0f} blocks")
            r[name] = {"x": x, "z": z, "relief_blocks": round(relief), "land_share": round(float(land.mean()), 3),
                       "valley_cells": drainage_lines(y)}
        report[label] = r
        print(label, json.dumps(r))
    (a.out / "views.json").write_text(json.dumps(report, indent=1) + "\n")
    for f in tmp.iterdir():
        f.unlink()
    tmp.rmdir()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
