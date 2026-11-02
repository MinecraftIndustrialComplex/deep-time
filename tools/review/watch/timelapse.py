#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""A run's crust-budget maps as pictures, one per snapshot (measurement only): timelapse.py <run folder> <out dir> [W]

For looking at a run's behaviour, not only its numbers (Ben, 2026-11-01: "are you actually analyzing the behaviour of
runs while they are going to see if there is anything that looks off?"). Each snapshot `maps/t*.bin` becomes
`<out>/t<Myr>.png`, three equirectangular panels W x W/2 (default 720) stacked: the crust's class (ocean floor shaded by
depth; continental crust by arcstock.py's classes: the start's crust, island-arc crust, collision belts, the other
converted crust, hyper-extended crust drawn over them), the height against that step's sea (blue under, green to brown
to white over), and the plates (one colour each, continental crust brighter). Each pixel shows the visible sample
nearest it. Prints each snapshot's continental and hyper-extended crust as shares of the planet's samples. Needs numpy and scipy; writes PNG with zlib (no imaging library)."""
import struct
import sys
import zlib
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "freeboard"))
import thinning as th  # noqa: E402
from arcstock import classes  # noqa: E402


def png(path, rgb):
    h, w, _ = rgb.shape
    raw = b"".join(b"\x00" + rgb[y].astype(np.uint8).tobytes() for y in range(h))

    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    Path(path).write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                           + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b""))


def ramp(x, stops):
    """Piecewise-linear colours: stops = [(value, (r, g, b)), ...] ascending."""
    xs = np.array([s[0] for s in stops], float)
    out = np.zeros(x.shape + (3,))
    for c in range(3):
        out[..., c] = np.interp(x, xs, [s[1][c] for s in stops])
    return out


def main(run, out, w=720):
    run, out = Path(run), Path(out)
    out.mkdir(parents=True, exist_ok=True)
    h = w // 2
    lat = (0.5 - (np.arange(h) + 0.5) / h) * 180.0
    lon = ((np.arange(w) + 0.5) / w - 0.5) * 360.0
    LON, LAT = np.meshgrid(lon, lat)
    px = th.xyz(LAT.ravel(), LON.ravel())
    ser = th.sea_series(run)
    names = sorted((run / "maps").glob("t*.bin"), key=lambda p: int(p.stem[1:]))
    # The planet's samples: every one is visible at a resampling's snapshot (the run's last); a
    # share of the visible samples between resamplings rises as floor goes down and gaps open.
    planet = max(f.stat().st_size for f in names) // (4 * 11)
    for f in names:
        t = int(f.stem[1:])
        a, pp = th.read(run, t)
        p = th.xyz(a[:, 0], a[:, 1])
        _, idx = cKDTree(p).query(px)
        sea = th.sea_at(ser, t)
        cont, start, island = classes(a, pp)
        belt = cont & ~start & ~island & (a[:, 5] > 1.5)
        own = pp[:, 0:7].sum(1) - np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None)
        hyper = cont & (own - pp[:, 0:2].sum(1) < 10_000.0)
        z = a[:, 2] - sea
        # Panel 1: the class.
        c1 = ramp(np.clip(z, -6000, 0), [(-6000, (10, 20, 70)), (-3000, (30, 70, 150)), (0, (110, 170, 220))])
        col = {"start": (200, 170, 110), "island": (235, 130, 40), "belt": (150, 60, 40),
               "other": (140, 90, 170), "hyper": (240, 190, 200)}
        other = cont & ~start & ~island & ~belt
        for m, k in ((start, "start"), (island, "island"), (belt, "belt"), (other, "other"), (hyper, "hyper")):
            c1[m] = col[k]
        # Panel 2: the height against the sea.
        c2 = np.where((z < 0)[:, None],
                      ramp(np.clip(z, -6000, 0), [(-6000, (10, 20, 70)), (0, (120, 180, 230))]),
                      ramp(np.clip(z, 0, 5000), [(0, (60, 140, 60)), (800, (170, 170, 80)),
                                                 (2500, (140, 90, 50)), (5000, (255, 255, 255))]))
        # Panel 3: the plates.
        pf = run / "maps" / "plate" / f.name
        if pf.exists():
            pl = np.fromfile(pf, "<u4")
            rng = np.random.default_rng(7)
            pal = rng.integers(40, 200, size=(int(pl.max()) + 1, 3))
            c3 = pal[pl].astype(float)
            c3[cont] = np.minimum(c3[cont] + 70, 255)
        else:
            c3 = np.full((len(a), 3), 128.0)
        img = np.concatenate([c[idx].reshape(h, w, 3) for c in (c1, c2, c3)], axis=0)
        png(out / f"t{t:05d}.png" if t >= 0 else out / f"t-{-t:04d}.png", np.clip(img, 0, 255))
        print(f"{f.name}: cont {cont.sum() / planet:.3f} hyper {hyper.sum() / planet:.3f} of the planet, "
              f"sea {sea:.0f} m")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 720)
