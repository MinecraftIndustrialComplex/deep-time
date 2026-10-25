#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The final land's local relief by orogeny class and age: m4relief.py <run folder> ...

The erosion law was calibrated against local relief (ERO-004, ERO-179: the regions' net rate within 6 % of
Montgomery & Brandon 2002's eq. 2 at 100 to 1,000 m of relief), not against height. So a belt's crust loss rate
(m4cohort.py's E_c) can be read against eq. 2 at the belt's own relief: if they agree, the law does what it was
calibrated to and a belt that erodes slower than Pinet & Souriau 1988's height law does so because it keeps less
relief for its height; if E_c falls well under eq. 2, something takes the crust loss away between the surface and
the column (deposition on the belt, the feedback's averaging).

Reads the pool's fields dump (`fields/`, the L1 grid: surface/elevation_m, plates/orogeny_type,
plates/orogeny_age_myr, plates/thickness_m, plates/crust_type; the sea from globals.json's climate/sea_level_m).
Relief is ERO-175's: the range over a cell and its four edge neighbours (x wraps, y clamps). Per tag and age band
over the continental land: cells, the height over the sea H (mean), the relief Rz (mean, p50), eq. 2 at each cell's
relief averaged and at the class's mean relief (m/Myr; E = E0 + K Rz / (1 - (Rz / Rc)^2), E0 0.01 mm/yr,
K 2.5e-4 mm/yr per m, Rc 1,500 m, ERO-179), Rz / H, and Pinet & Souriau's 0.061 H. Needs numpy only."""
import json
import sys
from pathlib import Path

import numpy as np

AGES = [0, 10, 20, 30, 50, 100, 150, 250, 1e9]
E0, KMB, RC = 10.0, 0.25, 1500.0  # m/Myr, m/Myr per m of relief, m (ERO-179)


def eq2(rz):
    rz = np.minimum(rz, 0.99 * RC)
    return E0 + KMB * rz / (1.0 - (rz / RC) ** 2)


def load(run):
    f = run / "fields"
    idx = json.loads((f / "index.json").read_text())
    g = json.loads((f / "globals.json").read_text())["globals"]

    def read(name):
        e = idx[name]
        return np.fromfile(f / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)

    return {"z": read("surface/elevation_m"), "tag": np.rint(read("plates/orogeny_type")),
            "age": read("plates/orogeny_age_myr"), "T": read("plates/thickness_m"),
            "kind": read("plates/crust_type"), "sea": float(g["climate/sea_level_m"])}


def relief(z):
    up = np.vstack([z[:1], z[:-1]])
    dn = np.vstack([z[1:], z[-1:]])
    lf, rt = np.roll(z, 1, axis=1), np.roll(z, -1, axis=1)
    st = np.stack([z, up, dn, lf, rt])
    return st.max(axis=0) - st.min(axis=0)


def main():
    runs = [Path(r) for r in sys.argv[1:]]
    rows = {}
    kinds = set()
    for run in runs:
        d = load(run)
        kinds |= set(np.unique(np.rint(d["kind"])).tolist())
        rz = relief(d["z"])
        h = d["z"] - d["sea"]
        land = (h > 0) & (d["kind"] > 0.5)
        for tag in (2, 1, 0):
            for a0, a1 in zip(AGES[:-1], AGES[1:]):
                m = land & (d["tag"] == tag)
                if tag == 0:
                    if a0 > 0:
                        break
                    for tname, tm in (("T>=45", d["T"] >= 45_000), ("T<45", d["T"] < 45_000)):
                        k = m & tm
                        rows.setdefault((tag, tname), []).append((h[k], rz[k]))
                    continue
                k = m & (d["age"] >= a0) & (d["age"] < a1)
                rows.setdefault((tag, f"{a0:g}-{a1:g}"), []).append((h[k], rz[k]))
    print(f"{len(runs)} runs; crust_type values {sorted(kinds)} (continental > 0.5)")
    print("class: cells, H mean m, Rz mean / p50 m, eq. 2 mean of cells / at mean Rz m/Myr, Rz/H, 0.061 H m/Myr")
    for key, parts in rows.items():
        h = np.concatenate([p[0] for p in parts])
        rz = np.concatenate([p[1] for p in parts])
        if h.size < 20:
            continue
        name = {2: "collision", 1: "andean", 0: "untagged"}[key[0]]
        print(f"  {name:9s} {key[1]:>9s}: n {h.size:7d}  H {h.mean():6.0f}  Rz {rz.mean():5.0f} / {np.median(rz):5.0f}"
              f"  eq2 {eq2(rz).mean():6.1f} / {float(eq2(np.array([rz.mean()]))[0]):6.1f}"
              f"  Rz/H {rz.mean() / max(h.mean(), 1):.2f}  PS {0.061 * h.mean():6.1f}")
    # By height alone, all continental land: Rz against H (Earth's old orogens keep a few hundred m at 0.5-1 km).
    print("\nall continental land by H over the sea: cells, Rz mean / p50 m, eq. 2 at the cells m/Myr, 0.061 H")
    allh = np.concatenate([p[0] for ps in rows.values() for p in ps])
    allr = np.concatenate([p[1] for ps in rows.values() for p in ps])
    for h0, h1 in ((0, 250), (250, 500), (500, 750), (750, 1000), (1000, 1500), (1500, 2000), (2000, 3000),
                   (3000, 9000)):
        k = (allh >= h0) & (allh < h1)
        if k.sum() < 20:
            continue
        print(f"  {h0:5d}-{h1:5d} m: n {int(k.sum()):7d}  Rz {allr[k].mean():5.0f} / {np.median(allr[k]):5.0f}"
              f"  eq2 {eq2(allr[k]).mean():6.1f}  PS {0.061 * allh[k].mean():6.1f}")


if __name__ == "__main__":
    main()
