#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Close-up of a `deeptime heightmap` box (step 1): hillshade of H and of E, an oblique view at the
game's own scale (one block per unit on every axis, as a player sees it), transects, and the
radially averaged spectrum. A tool of the sea-floor spikes study (2026-10-20).

    closeup.py <heightmap.npy> <out.png> [--title T]
"""

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from matplotlib.colors import LightSource  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("npy")
    ap.add_argument("out")
    ap.add_argument("--title", default="")
    ap.add_argument("--sea", type=float, default=63.0)
    args = ap.parse_args()
    a = np.load(args.npy)
    meta = json.loads(Path(args.npy + ".json").read_text())
    x0, z0, x1, z1 = meta["box"]
    step = meta["step"]
    ch = {n: i for i, n in enumerate(meta["channels"])}
    H = a[:, :, ch["surface"]].astype(np.float64)
    E = a[:, :, ch["elevation"]].astype(np.float64)
    D = a[:, :, ch["detail"]].astype(np.float64)
    ls = LightSource(azdeg=315, altdeg=35)
    ext = [x0, x1, z1, z0]
    fig = plt.figure(figsize=(20, 20))
    ax = fig.add_subplot(2, 2, 1)
    ax.imshow(ls.shade(H, cmap=plt.cm.gist_earth, vert_exag=1, blend_mode="soft", vmin=H.min(), vmax=max(H.max(), args.sea)), extent=ext)
    ax.contour(np.linspace(x0, x1, H.shape[1]), np.linspace(z0, z1, H.shape[0]), H, levels=[args.sea], colors="red", linewidths=0.8)
    ax.set_title(f"H (game terrain) hillshade, 1:1 blocks; red: sea level  [{H.min():.0f}..{H.max():.0f}]")
    ax = fig.add_subplot(2, 2, 2)
    im = ax.imshow(D, cmap="RdBu", extent=ext, vmin=-8, vmax=8)
    fig.colorbar(im, ax=ax, shrink=0.7)
    ax.set_title(f"detail term A*N (blocks), RMS {D.std():.2f}")
    # Oblique view at game scale over the central 192 x 192 blocks.
    ax = fig.add_subplot(2, 2, 3, projection="3d")
    n = min(H.shape[0], H.shape[1], 192 // step)
    r0 = (H.shape[0] - n) // 2
    c0 = (H.shape[1] - n) // 2
    sub = H[r0:r0 + n, c0:c0 + n]
    X, Z = np.meshgrid(x0 + (c0 + np.arange(n)) * step, z0 + (r0 + np.arange(n)) * step)
    rgb = ls.shade(sub, cmap=plt.cm.gist_earth, vert_exag=1, blend_mode="soft")
    ax.plot_surface(X, Z, sub, facecolors=rgb, rstride=1, cstride=1, linewidth=0, antialiased=False, shade=False)
    span = n * step
    ax.set_box_aspect((span, span, max(sub.max() - sub.min(), 1)))
    ax.view_init(elev=20, azim=-60)
    ax.set_title("oblique view, 1 block = 1 unit on all axes (centre 192 blocks)")
    ax = fig.add_subplot(2, 2, 4)
    mid = H.shape[0] // 2
    xs = x0 + np.arange(H.shape[1]) * step
    ax.plot(xs, H[mid], label=f"H row z={z0 + mid * step}")
    ax.plot(xs, E[mid], label="E", lw=0.8)
    midc = H.shape[1] // 2
    zs = z0 + np.arange(H.shape[0]) * step
    ax.plot(zs - zs[0] + xs[0], H[:, midc], label=f"H column x={x0 + midc * step} (z offset)")
    ax.axhline(args.sea, color="b", lw=0.5)
    ax.set_aspect("equal")
    ax.legend()
    ax.set_title("transects at game scale (1:1)")
    fig.suptitle(args.title)
    fig.savefig(args.out, dpi=70, bbox_inches="tight")


if __name__ == "__main__":
    main()
