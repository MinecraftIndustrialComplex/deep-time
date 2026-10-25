#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's own history on V1 item 1's land measures: the census's land share, largest landmass, land
above 2 km and coast with high ground, read off the PALEOMAP PaleoDEMs (Scotese & Wright 2018, 1 x 1
degree, every 5 Myr, 0 to 540 Ma) and, as the control for the pipeline, off a present-day relief grid
with the same code. Docs task of 2026-10-23 (docs/science/realism-checks.md, ledger RLC).

    curl -sSLO https://www.earthbyte.org/webdav/ftp/Data_Collections/Scotese_Wright_2018_PaleoDEM/Scotese_Wright_2018_Maps_1-88_1degX1deg_PaleoDEMS_nc.zip
    curl -sSLO https://oceania.generic-mapping-tools.org/server/earth/earth_relief/earth_relief_15m_p.grd
    # each in its own new directory; the zip unpacks to a folder of Map*.nc files
    python3 -I tools/review/earth-paleodem.py paleodem <folder of Map*.nc> [N]
    python3 -I tools/review/earth-paleodem.py relief earth_relief_15m_p.grd [N]

Needs numpy, scipy and h5py (nix shell --impure --expr '(builtins.getFlake "nixpkgs").legacyPackages.
${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy p.h5py])' -c python3 -I ...). Light:
a few seconds a map at N = 40,000 on one core; run it niced (systemd-run --user --scope -p
CPUQuota=100% nice -n 19).

Per map, the same definitions as tools/review/pool/measure_v1census.py (it reuses continents_census.py's
helpers, not a copy), on a grid that is 111 km at the equator instead of the census's 16 km:

  land       cells above 0 m, area-weighted (a share of the planet)
  largest    the largest landmass (8-connected on the grid, joined over the x seam), area-weighted
  above_2km  land above 2 km, a share of the land (area-weighted on the grid)
  coast_hi   N Fibonacci points sampled from the grid at their nearest node (earth-coast.py's way);
             continents_census.coast_metrics: the coast is land within 1.6 point spacings of the sea, and
             the share of it with land above 2 km within 200 km inland, over land. Since 2026-10-25
             (RLC-055) the sea is the world ocean (a closed body of sea is land) and the 200 km are a
             path over land; coast_hi_old is the first version's rule (any sea, the straight line within
             one landmass), coast_hi_ocean the world ocean with the straight line. At N = 40,000 the
             spacing is 113 km, so the coast band is 181 km wide where the census's (N = 332,800, spacing
             39.1 km) is 62.6 km: compare maps with each other and with the control, not with the
             census's own numbers, or run N = 332,800 (a 1 degree map then has several points a node).

The PaleoDEM files are a "first draft" by their author's own words (the highest ranges "likely
overestimated", young features on old maps); this is a published model of Earth's past, not
observation. Its elevations are 40 m steps. The 0 Ma map's Antarctic and Greenland heights are read
as they come (see the printed maxima).
"""
import re
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import continents_census as cc  # noqa: E402

PLANET_MKM2 = cc.EARTH_KM2 / 1e6


def read_grid(path):
    """(z, lat, lon) of a netCDF-4 grid with variables z, lat (or y) and lon (or x), z[lat, lon]."""
    import h5py
    with h5py.File(path, "r") as f:
        z = f["z"][()].astype(np.float64)
        lat = np.asarray(f["lat"][()] if "lat" in f else f["y"][()], dtype=np.float64)
        lon = np.asarray(f["lon"][()] if "lon" in f else f["x"][()], dtype=np.float64)
        at = {k: f["z"].attrs[k] for k in f["z"].attrs}
    z = z * float(np.ravel(at.get("scale_factor", 1.0))[0]) + float(np.ravel(at.get("add_offset", 0.0))[0])
    if lat[0] > lat[-1]:
        lat, z = lat[::-1], z[::-1]
    return np.nan_to_num(z, nan=-1000.0), lat, lon


def row_weights(lat):
    """Each row's share of the planet's area: the band half a grid step either side of its latitude."""
    step = float(np.median(np.diff(lat)))
    lo = np.radians(np.clip(lat - step / 2, -90, 90))
    hi = np.radians(np.clip(lat + step / 2, -90, 90))
    w = np.sin(hi) - np.sin(lo)
    return w / w.sum()


def fib(n):
    i = np.arange(n) + 0.5
    return np.degrees(np.arcsin(1 - 2 * i / n)), np.degrees((np.pi * (1 + 5 ** 0.5) * i) % (2 * np.pi) - np.pi)


def measure(z, lat, lon, n):
    # The grid without its duplicated seam column (gridline registration: the first and last longitudes agree).
    keep = np.ones(lon.size, bool)
    if lon.size > 1 and abs((lon[-1] - lon[0]) - 360.0) < 0.5 * abs(lon[1] - lon[0]):
        keep[-1] = False
    zg, lg = z[:, keep], lon[keep]
    w = np.repeat(row_weights(lat)[:, None], zg.shape[1], 1)
    w = w / w.sum()
    land = zg > 0
    lab = cc.label_wrap(land)
    sizes = np.bincount(lab.ravel(), weights=w.ravel())
    sizes[0] = 0.0
    land_share = float(w[land].sum())
    # The points, for the coast measure.
    plat, plon = fib(n)
    iy = np.clip(np.searchsorted(lat, plat), 0, lat.size - 1)
    ix = np.clip(np.searchsorted(lon, plon), 0, lon.size - 1)
    zp = z[iy, ix]
    p = cc.xyz(plat, plon)
    sp = float(np.sqrt(cc.EARTH_KM2 / n))
    import contextlib
    import io
    with contextlib.redirect_stdout(io.StringIO()):
        coast = cc.coast_metrics(p, zp, sp)
        old = cc.coast_metrics(p, zp, sp, closed_as_land=False, over_land=False)
        ocean = cc.coast_metrics(p, zp, sp, over_land=False)
    return {
        "land": land_share,
        "largest": float(sizes.max()),
        "above_2km": float(w[land & (zg > 2000)].sum() / max(land_share, 1e-9)),
        "coast_hi": float(coast["coast_with_hi"]),
        "coast_hi_old": float(old["coast_with_hi"]),
        "coast_hi_ocean": float(ocean["coast_with_hi"]),
        "land_pts": float(coast["land"]),
        "zmax": float(z.max()),
    }


def age_of(path):
    m = re.search(r"_(\d+(?:\.\d+)?)Ma\.nc$", path.name)
    return float(m.group(1)) if m else None


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(2)
    n = int(sys.argv[3]) if len(sys.argv) > 3 else 40_000
    head = "Ma      land  largest  Mkm2   >2km  coast_hi  old(v1)  ocean  zmax"
    if sys.argv[1] == "relief":
        z, lat, lon = read_grid(sys.argv[2])
        r = measure(z, lat, lon, n)
        print(f"relief N={n}: land {r['land']:.4f}; largest {r['largest']:.4f}; above 2 km {r['above_2km']:.4f};"
              f" coast with high ground {r['coast_hi']:.4f} (version 1 rule {r['coast_hi_old']:.4f}, world ocean with"
              f" the straight line {r['coast_hi_ocean']:.4f}); max {r['zmax']:.0f} m")
        # By rock: the two ice sheets (Antarctica, Greenland) kept out of the shares, as PLT-360's control.
        import contextlib
        import io
        plat, plon = cc.fib_points(n)
        sheets = cc.ice_sheets(plat, plon)
        zp = z[np.clip(np.searchsorted(lat, plat), 0, lat.size - 1), np.clip(np.searchsorted(lon, plon), 0, lon.size - 1)]
        sp = float(np.sqrt(cc.EARTH_KM2 / n))
        with contextlib.redirect_stdout(io.StringIO()):
            rock = cc.coast_metrics(cc.xyz(plat, plon), zp, sp, exclude=sheets)
            rock_old = cc.coast_metrics(cc.xyz(plat, plon), zp, sp, exclude=sheets, closed_as_land=False, over_land=False)
        print(f"relief N={n} by rock (without Antarctica and Greenland): above 2 km {rock['hi_of_land']:.4f};"
              f" coast with high ground {rock['coast_with_hi']:.4f} (version 1 rule {rock_old['coast_with_hi']:.4f})")
    elif sys.argv[1] == "paleodem":
        files = sorted((p for p in Path(sys.argv[2]).glob("*.nc") if age_of(p) is not None), key=age_of)
        print(f"PaleoDEMs of {sys.argv[2]}, N={n} (spacing {np.sqrt(cc.EARTH_KM2 / n):.0f} km)")
        print(head)
        for p in files:
            z, lat, lon = read_grid(p)
            r = measure(z, lat, lon, n)
            print(f"{age_of(p):5.1f} {r['land']:7.4f} {r['largest']:8.4f} {r['largest'] * PLANET_MKM2:6.1f}"
                  f" {r['above_2km']:7.4f} {r['coast_hi']:9.4f} {r['coast_hi_old']:8.4f} {r['coast_hi_ocean']:6.4f}"
                  f" {r['zmax']:6.0f}")
    else:
        print(__doc__)
        sys.exit(2)


if __name__ == "__main__":
    main()
