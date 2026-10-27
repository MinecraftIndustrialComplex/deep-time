#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Maps of the coast with high ground for a few pool runs (coast-high lane, diagnosis only):
coasthi_map.py <out dir> <run folder> [...]. One PNG per run on the census's Fibonacci points
(equirectangular): sea by depth, land grey, land above 2 km brown, the qualified coast red, the
rest of the coast green, the collision regime purple, trench and arc regime black, back-arc cyan.
Needs numpy, scipy, matplotlib."""
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import continents_census as cc  # noqa: E402


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for run in sys.argv[2:]:
        f, _g, e = cc.load_dump(Path(run) / "fields")
        h, w, r = e["h"], e["w"], e["r"]
        frow, fcol, fp = cc.fib_sample(h, w, r)
        _lat, ok_r = cc.lat_of_rows(h, r, w * r)
        sp = float(np.sqrt(cc.EARTH_KM2 / (int(ok_r.sum()) * w)))
        z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[frow, fcol].astype(float)
        geo = cc.coast_geometry(fp, z, sp)
        # Raster on the census grid (equal-area rows), the Fibonacci coast flags painted on their cells.
        zg = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
        rg = np.rint(np.nan_to_num(f["regime"], nan=0.0)).astype(int)
        img = np.zeros((h, w, 3))
        d = np.clip(-zg / 6000.0, 0, 1)
        sea_g = zg <= 0
        img[sea_g] = np.stack([0.55 - 0.4 * d[sea_g], 0.7 - 0.45 * d[sea_g], 0.95 - 0.35 * d[sea_g]], 1)
        img[~sea_g] = (0.80, 0.80, 0.74)
        img[zg > 1000] = (0.68, 0.62, 0.50)
        img[zg > 2000] = (0.55, 0.35, 0.20)
        img[zg > 3500] = (0.30, 0.17, 0.08)
        reg_img = img.copy()
        for codes, c in (((5,), (0.65, 0.0, 0.85)), ((2,), (0, 0, 0)), ((3,), (0.9, 0.5, 0.0)), ((4,), (0.0, 0.8, 0.9)),
                         ((6,), (0.1, 0.6, 0.1)), ((9,), (0.95, 0.95, 0.2))):
            reg_img[np.isin(rg, codes)] = c
        reg_img[(zg > 2000)] = reg_img[(zg > 2000)] * 0.5
        cimg = img.copy()
        cf = geo["coast"] & ~geo["has"]
        cimg[frow[cf], fcol[cf]] = (0.2, 0.75, 0.2)
        cimg[frow[geo["has"]], fcol[geo["has"]]] = (1.0, 0.0, 0.0)
        fig, axs = plt.subplots(2, 1, figsize=(24, 16), dpi=80)
        q = geo["has"].sum() / max(geo["coast"].sum(), 1)
        axs[0].imshow(cimg, interpolation="nearest", aspect="auto")
        axs[0].set_title(f"{Path(run).name}: M5 {q:.3f}. Red: coast with high ground; green: other coast;"
                         " browns: land above 1, 2, 3.5 km (grid rows equal-area)")
        axs[1].imshow(reg_img, interpolation="nearest", aspect="auto")
        axs[1].set_title("regime: purple collision, black trench, orange arc, cyan back-arc, green rift,"
                         " yellow passive margin; darkened where above 2 km")
        fig.tight_layout()
        fig.savefig(out / f"{Path(run).name}.png")
        plt.close(fig)
        print(out / f"{Path(run).name}.png")


if __name__ == "__main__":
    main()
