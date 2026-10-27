#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Maps of what the coast's high ground is, by plate (coast-high lane, diagnosis only):
coasthi_map2.py <out dir> <run folder> [...]. On the census grid: sea blue, land grey; land above
2 km in the belt's zone of an open collision (within 400 km of its front, the overriding side)
purple, above 2 km on an overriding plate beyond that zone orange, other land above 2 km brown;
the open collisions' fronts (overriding side) black, incoming side white; the coast with high
ground red, the other coast green. Plate ids from the crust budget's map at the end."""
import json
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from scipy.spatial import cKDTree  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import continents_census as cc  # noqa: E402


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for run in map(Path, sys.argv[2:]):
        f, _g, e = cc.load_dump(run / "fields")
        h, w, r = e["h"], e["w"], e["r"]
        frow, fcol, fp = cc.fib_sample(h, w, r)
        _lat, ok_r = cc.lat_of_rows(h, r, w * r)
        sp = float(np.sqrt(cc.EARTH_KM2 / (int(ok_r.sum()) * w)))
        z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[frow, fcol].astype(float)
        geo = cc.coast_geometry(fp, z, sp)
        m = np.fromfile(run / "maps" / "t0000.bin", "<f4").reshape(-1, cc.MAP_FIELDS).astype(np.float64)
        pid = np.fromfile(run / "maps" / "plate" / "t0000.bin", "<u4").astype(np.int64)
        mp = cc.xyz(m[:, 0], m[:, 1])
        mtree = cKDTree(mp)
        rec = json.loads((run / "rec.json").read_text())
        t_end = rec["steps"][-1]["t"]
        open_ = [p for p in rec["partings"] if p["reason"] == "open" and p["end_myr"] >= t_end - 1e-6]
        lat_sp = float(np.sqrt(cc.EARTH_KM2 / mp.shape[0]))
        f_over, f_under = [], []
        for o in open_:
            a, b = o["over"], o["under"]
            for x, y, acc in ((a, b, f_over), (b, a, f_under)):
                ia = np.nonzero(pid == x)[0]
                if ia.size == 0:
                    continue
                nb = mtree.query_ball_point(mp[ia], cc.chord(1.6 * lat_sp))
                isf = np.fromiter((bool(np.any(pid[np.asarray(q, np.int64)] == y)) for q in nb), bool, ia.size)
                acc.append((x, ia[isf]))
        # Grid cells: each takes the nearest map sample's plate.
        lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
        lat2 = np.repeat(lat_r[:, None], w, 1)
        lon2 = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
        pg = cc.xyz(lat2.ravel(), lon2.ravel())
        _d, kg = mtree.query(pg)
        plate_g = pid[kg].reshape(h, w)
        zg = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
        dfront = np.full(h * w, np.inf)
        over_ids = {x for x, _ in f_over}
        for x, idx in f_over:
            if idx.size == 0:
                continue
            sel = (plate_g.ravel() == x)
            d = cc.arc_km(cKDTree(mp[idx]).query(pg[sel])[0])
            dfront[sel] = np.minimum(dfront[sel], d)
        dfront = dfront.reshape(h, w)
        img = np.zeros((h, w, 3))
        d = np.clip(-zg / 6000.0, 0, 1)
        sea_g = zg <= 0
        img[sea_g] = np.stack([0.55 - 0.4 * d[sea_g], 0.7 - 0.45 * d[sea_g], 0.95 - 0.35 * d[sea_g]], 1)
        img[~sea_g] = (0.80, 0.80, 0.74)
        hi = zg > 2000
        on_over = np.isin(plate_g, list(over_ids))
        img[hi] = (0.55, 0.35, 0.20)
        img[hi & on_over & (dfront > 400)] = (1.0, 0.55, 0.0)
        img[hi & on_over & (dfront <= 400)] = (0.6, 0.0, 0.8)
        cf = geo["coast"] & ~geo["has"]
        img[frow[cf], fcol[cf]] = (0.2, 0.75, 0.2)
        img[frow[geo["has"]], fcol[geo["has"]]] = (1.0, 0.0, 0.0)
        # Plate boundaries (thin grey) and fronts.
        bnd = np.zeros((h, w), bool)
        bnd[:, 1:] |= plate_g[:, 1:] != plate_g[:, :-1]
        bnd[1:, :] |= plate_g[1:, :] != plate_g[:-1, :]
        img[bnd] = img[bnd] * 0.6
        fig, ax = plt.subplots(figsize=(24, 9), dpi=80)
        ax.imshow(img, interpolation="nearest", aspect="auto")
        for acc, c in ((f_over, "k"), (f_under, "w")):
            for _x, idx in acc:
                if idx.size == 0:
                    continue
                la, lo = m[idx, 0], m[idx, 1] % 360.0
                row = (h * r / 2 - (w * r / (2 * np.pi)) * np.sin(np.radians(la))) / r
                ax.scatter(lo / 360.0 * w, row, s=3, c=c, marker="s", linewidths=0)
        q = geo["has"].sum() / max(geo["coast"].sum(), 1)
        ax.set_title(f"{run.name}: M5 {q:.3f}; >2 km: purple belt zone (<=400 km of an open front, overriding side),"
                     f" orange overriding plate beyond it, brown other; fronts black (over) / white (under);"
                     f" red coast with high ground; {len(open_)} open collisions")
        fig.tight_layout()
        fig.savefig(out / f"{run.name}-plates.png")
        plt.close(fig)
        print(out / f"{run.name}-plates.png")


if __name__ == "__main__":
    main()
