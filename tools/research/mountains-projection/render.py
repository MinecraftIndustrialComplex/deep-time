#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pictures of docs/research/notes/mountains-and-the-projection.md. Research tool, not shipped; run
on the test VM through run.sh (numpy, scipy, matplotlib, pillow).

    render.py --work W --out O --bin deeptime [--taus 0.000,0.500,1.000] [--size 1024]

Picks three mountain ranges of the planet (low, middle and high latitude) and draws each, 1 block per
pixel, through every map: today's world as baked (terrain height H, with the detail noise), the same
simulation's elevation E redrawn through today's Petroff-Guyou world, through each cube map
(cube_table.py; the corner placement analyse.py chose, O/numbers.json), and a true-shape local view
(Lambert azimuthal equal-area about the range, at the cube world's mean 294.6 m per pixel, north up).
Every redraw samples the same field, the Lambert world's E (bilinear), so the pictures differ only by
the map. Same light, same tint, same vertical scale (blocks); the rings are true circles of 50 and
150 km about the range's centre and the arrow points north.
"""

import argparse
import glob
import json
import math
import os
import subprocess
import sys

import numpy as np
from scipy import ndimage
from scipy.spatial import cKDTree

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "..", "review"))
import geom  # noqa: E402
import vertical_scale as vs  # noqa: E402

BLOCK_M = 294.6


def load(work, name):
    a = np.load(os.path.join(work, name + ".npy"), mmap_mode="r")
    meta = json.load(open(os.path.join(work, name + ".npy.json")))
    return a, {n: i for i, n in enumerate(meta["channels"])}, meta


class LambertField:
    """Bilinear samples of the Lambert world's E at (lat, lon), from the every-2-blocks map."""

    def __init__(self, L, Lc):
        self.E = np.asarray(L[:, :, Lc["elevation"]], np.float32)
        lat = np.asarray(L[:, L.shape[1] // 2, Lc["lat_deg"]], np.float64)
        lon = np.asarray(L[L.shape[0] // 2, :, Lc["lon_deg"]], np.float64)
        rows = np.arange(L.shape[0])
        ok = np.isfinite(lat) & (np.abs(lat) < 80)
        self.rz = np.polyfit(np.sin(np.radians(lat[ok])), rows[ok], 1)
        cols = np.arange(L.shape[1])
        lonu = np.degrees(np.unwrap(np.radians(lon)))
        self.cx = np.polyfit(lonu, cols, 1)
        self.n = L.shape[1]
        self.Ep = np.concatenate([self.E[:, -2:], self.E, self.E[:, :2]], axis=1)

    def at(self, lat, lon):
        r = np.polyval(self.rz, np.sin(np.radians(lat)))
        c = np.mod(np.polyval(self.cx, lon), self.n) + 2
        return ndimage.map_coordinates(self.Ep, [r, c], order=1, mode="nearest")


def pick_ranges(L, Lc, sea, mv):
    E = np.asarray(L[::2, ::2, Lc["elevation"]], np.float32)
    lat = np.asarray(L[::2, ::2, Lc["lat_deg"]], np.float64)
    lon = np.asarray(L[::2, ::2, Lc["lon_deg"]], np.float64)
    sm = ndimage.uniform_filter(np.maximum(E, sea), size=(9, 25), mode="wrap")
    picks = []
    for lo_, hi_, name in ((0, 25, "low"), (30, 50, "mid"), (52, 75, "high")):
        m = (np.abs(lat) >= lo_) & (np.abs(lat) < hi_) & np.isfinite(sm)
        if not m.any():
            continue
        i = np.argmax(np.where(m, sm, -1e9))
        r, c = np.unravel_index(i, sm.shape)
        picks.append({"name": name, "lat": float(lat[r, c]), "lon": float(lon[r, c]),
                      "mean_m_25km": float((sm[r, c] - sea) * mv)})
    return picks


def hillshade(v, metres, wet):
    gx = np.gradient(v, axis=1)
    gz = np.gradient(v, axis=0)
    lx, lz, ly = -0.55, -0.45, 0.70
    n = np.sqrt(gx * gx + gz * gz + 1)
    lam = np.clip((-gx * lx - gz * lz + ly) / n, 0, 1) / ly
    shade = np.clip(0.50 + 0.50 * lam, 0.30, 1.25)
    col = vs.tint(metres)
    steep = np.clip((np.hypot(gx, gz) - 1.2) / 1.5, 0, 1)[..., None]
    col = col * (1 - steep) + vs.STONE * steep
    col = np.where(wet[..., None], np.array((80, 120, 170), np.float32), col)
    shade = np.where(wet, 1.0, shade)
    return np.clip(col * shade[..., None], 0, 255).astype(np.uint8)


def surface_from_E(E, sea, mv):
    wet = E < sea
    v = np.where(wet, sea, E).astype(np.float32)
    metres = np.where(wet, 0, (E - sea) * mv)
    return v, metres, wet


def window_units_cube(cube, p0, size, s_blocks):
    X0, Y0, _ = cube.forward(p0[None])
    off = (np.arange(size) - size / 2 + 0.5) / s_blocks
    X = X0[0] + off[None, :]
    Y = Y0[0] - off[:, None]
    X, Y = np.broadcast_arrays(X, Y)
    return cube.unit(X, Y)


def window_units_laea(p0, size, r_m):
    lat0, lon0 = geom.xyz2ll(p0)
    phi1, lam0 = math.radians(float(lat0)), math.radians(float(lon0))
    off = (np.arange(size) - size / 2 + 0.5) * BLOCK_M / r_m
    x = off[None, :] * np.ones((size, 1))
    y = -off[:, None] * np.ones((1, size))
    rho = np.hypot(x, y)
    c = 2 * np.arcsin(np.clip(rho / 2, -1, 1))
    sc, cc = np.sin(c), np.cos(c)
    with np.errstate(invalid="ignore", divide="ignore"):
        lat = np.arcsin(cc * math.sin(phi1) + np.where(rho > 0, y * sc * math.cos(phi1) / rho, 0))
        lon = lam0 + np.arctan2(x * sc, rho * math.cos(phi1) * cc - y * math.sin(phi1) * sc)
    return geom.ll2xyz(np.degrees(lat), np.degrees(lon))


def window_units_pg(bin_, pg_dir, P, Pc, p0, size, work, tag):
    # the Petroff-Guyou block nearest the range's centre, then that window's lat/lon from the bake
    lat = np.asarray(P[::4, ::4, Pc["lat_deg"]], np.float64)
    lon = np.asarray(P[::4, ::4, Pc["lon_deg"]], np.float64)
    ok = np.isfinite(lat)
    tree = cKDTree(geom.ll2xyz(lat[ok], lon[ok]))
    _, i = tree.query(p0)
    rr, cc = np.argwhere(ok)[i]
    x_c = -6656 + 8 * int(cc)
    z_c = -3328 + 8 * int(rr)
    x0, z0 = x_c - size // 2, z_c - size // 2
    out = os.path.join(work, f"pg-crop-{tag}.npy")
    if not os.path.exists(out):
        subprocess.run([bin_, "heightmap", "--world-data", pg_dir, f"--box={x0},{z0},{x0 + size},{z0 + size}",
                        "--step", "1", "--scale", "--out", out], check=True)
    a = np.load(out)
    ch = {n: i for i, n in enumerate(json.load(open(out + ".json"))["channels"])}
    U = geom.ll2xyz(a[:, :, ch["lat_deg"]], a[:, :, ch["lon_deg"]])
    return U, a[:, :, ch["surface"]], (x_c, z_c)


def north_arrow(U):
    lat, _ = geom.xyz2ll(U)
    n = U.shape[0] // 2
    gy, gx = np.gradient(lat[n - 3:n + 4, n - 3:n + 4])
    d = np.array([gx[3, 3], gy[3, 3]])
    return d / (np.linalg.norm(d) + 1e-12)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--bin", default="deeptime")
    ap.add_argument("--taus", default="0.000,0.500,1.000")
    ap.add_argument("--size", type=int, default=1024)
    ap.add_argument("--aspect", default="free", help="corner placement: free (any orientation) or polar")
    a = ap.parse_args()
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    from PIL import Image

    os.makedirs(a.out, exist_ok=True)
    L, Lc, _ = load(a.work, "lambert-s2")
    P, Pc, _ = load(a.work, "pg-s2")
    man = json.load(open(os.path.join(a.work, "lambert", "manifest.json")))
    sc = man["scale"]
    sea, mv = sc["sea_level_y"], sc["m_per_block_v"]
    r_m = sc["circumference_blocks"] * sc["m_per_block_h"] / (2 * math.pi)
    nums = json.load(open(os.path.join(a.out, "numbers.json")))
    lam0 = nums["corners_this_planet"]["polar_aspect"]["lambda0_deg"]
    Qc = geom.rot_z(-lam0)
    if a.aspect == "free":
        Qc = np.array(nums["corners_this_planet"]["free_rotation"]["rotation_planet_to_canonical"], float)
    s_blocks = r_m * math.sqrt(4 * math.pi / 6) / BLOCK_M  # cube face side, blocks
    field = LambertField(L, Lc)
    ranges = pick_ranges(L, Lc, sea, mv)
    cubes = []
    for t in a.taus.split(","):
        path = os.path.join(a.work, "cube", f"cube-tau{float(t):.3f}.npz")
        if os.path.exists(path):
            cubes.append((float(t), geom.Cube(path, Q=Qc)))
    label = {0.0: "conformal cube", 1.0: "equal-area (Moser) cube"}
    info = {"ranges": ranges, "cube_face_blocks": s_blocks, "aspect": a.aspect,
            "rotation_planet_to_canonical": Qc.tolist(), "panels": []}
    for rg in ranges:
        p0 = geom.ll2xyz(rg["lat"], rg["lon"])
        panels = []
        U, Hbaked, xz = window_units_pg(a.bin, os.path.join(a.work, "pg"), P, Pc, p0, a.size, a.work, rg["name"])
        rg["pg_block"] = xz
        panels.append(("today, as baked (H with detail)", U, np.asarray(Hbaked, np.float32)))
        panels.append(("today: Petroff-Guyou", U, None))
        for t, cube in cubes:
            panels.append((label.get(t, f"cube, Moser's flow to tau = {t:g}"), window_units_cube(cube, p0, a.size, s_blocks), None))
        panels.append(("true shape (azimuthal equal-area)", window_units_laea(p0, a.size, r_m), None))
        rendered = []
        for title, Uw, Hgiven in panels:
            lat, lon = geom.xyz2ll(Uw)
            E = Hgiven if Hgiven is not None else field.at(lat, lon)
            v, metres, wet = surface_from_E(E, sea, mv)
            img = hillshade(v, metres, wet)
            dist_km = np.arccos(np.clip(Uw @ p0, -1, 1)) * r_m / 1000
            rendered.append((title, img, dist_km, north_arrow(Uw), v, metres, wet))
        n = len(rendered)
        cols = min(n, 4)
        rows = (n + cols - 1) // cols
        fig, axs = plt.subplots(rows, cols, figsize=(5 * cols, 5.3 * rows))
        axs = np.atleast_1d(axs).ravel()
        for ax, (title, img, dk, na, *_r) in zip(axs, rendered):
            ax.imshow(img, interpolation="nearest")
            ax.contour(dk, levels=[50, 150], colors="w", linewidths=0.8)
            c = a.size / 2
            ax.annotate("", xy=(c + 60 * na[0], c + 60 * na[1]), xytext=(c, c),
                        arrowprops=dict(arrowstyle="->", color="w", lw=1.5))
            ax.set_title(title, fontsize=9)
            ax.axis("off")
        for ax in axs[n:]:
            ax.axis("off")
        fig.suptitle(f"{rg['name']}-latitude range at {rg['lat']:.1f} N, {rg['lon']:.1f} E: {a.size} x {a.size} blocks, "
                     f"rings 50 and 150 km, arrow north", fontsize=10)
        fig.tight_layout()
        f = os.path.join(a.out, f"range-{rg['name']}-relief.png")
        fig.savefig(f, dpi=100)
        plt.close(fig)
        info["panels"].append(f)
        # Oblique views: the central half of each window, from the south of the window.
        h = a.size // 4
        base = int(sea) - 2
        top = int(max(r[4][h:-h, h:-h].max() for r in rendered)) + 2
        obl = []
        for title, img, dk, na, v, metres, wet in rendered:
            o = vs.oblique(v[h:-h, h:-h], wet[h:-h, h:-h], metres[h:-h, h:-h], np.clip(sea - v[h:-h, h:-h], 0, None),
                           s=1, pitch_deg=30.0, base=base, top=top)
            obl.append((title, o))
        W = max(o.shape[1] for _, o in obl)
        Ht = sum(o.shape[0] + 22 for _, o in obl)
        canvas = Image.new("RGB", (W, Ht), (255, 255, 255))
        from PIL import ImageDraw
        dr = ImageDraw.Draw(canvas)
        y = 0
        for title, o in obl:
            dr.text((6, y + 4), title, fill=(0, 0, 0))
            canvas.paste(Image.fromarray(o), (0, y + 22))
            y += o.shape[0] + 22
        f = os.path.join(a.out, f"range-{rg['name']}-oblique.png")
        canvas.save(f)
        info["panels"].append(f)
        print("rendered", rg, flush=True)
    json.dump(info, open(os.path.join(a.out, "renders.json"), "w"), indent=1)


if __name__ == "__main__":
    main()
