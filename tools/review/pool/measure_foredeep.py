#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `foredeep` measure: how wide and deep the flexural foreland basins are (gap 6,
docs/science/physics-gap-audit.md "Gap 6"; docs/science/flexure.md "Foreland basins"), from one pool
run folder, printed as a JSON object on stdout.

    python3 -I measure_foredeep.py <run folder> [--fields DIR]

Two parts.

From the record (rec.json, every run has it): the plate check's own foredeep, read in front of each
collision belt when it stood highest (`belts::foredeep`: bins of `surface/flexure_m` one sample
spacing wide beyond the wedge's toe, the stretch below -100 m that starts within two bins of it).

  rec_n_high        belts that stood 3 km or higher (the plate check's "belt above 3 km")
  rec_has_share     share of those with a foredeep at all (width above 0)
  rec_width_p50     their foredeep width, median, km (the plate check's foredeep_width_km_p50)
  rec_depth_p50     their foredeep depth below local isostasy, median, m (foredeep_depth_m_p50)
  rec_wedge_p50     their wedge width, median, km

From the final frame's fields (runs made with this measure asked for; NaN otherwise): every cell of an
active collision's footprint (`plates/collision_mm_yr` above 0 without the FORELAND flag: the belt and
the wedge) and every foreland cell (FORELAND: the incoming plate beyond the wedge's toe, to 600 km).
Each foreland cell is binned by its distance to the nearest footprint cell, 40 km bins (one L1 cell,
Earth-normalized), and the median of each bin's fields makes the basin's profile:

  fd_width_km       where the median flexure first crosses 0 beyond the footprint, km (linear between
                    bin centres; 0 when the first bin is not below 0)
  fd_depth_m        the deepest median flexure before that crossing, m below local isostasy
  fd_bulge_m        the highest median flexure after it, m, and fd_bulge_km where it is
  fd_fill_m         median strata thickness (`surface/sediment_m`) of the cells before the crossing, m
  fd_fill_far_m     the same after it, m (the foreland's background)
  fd_te_fp, fd_te_fl  median elastic thickness of the footprint and of the foreland cells, km
  fd_n_fp, fd_n_fl  footprint and foreland cells
  fdh_*             the same over the foreland cells whose nearest footprint is a belt that stands 3 km
                    or higher (its connected footprint's highest cell)

Note the depth is the flexure, not the fill: a basin filled to its old level with sediment of density
rho_s stands (1 - rho_s / rho_m) of the fill below the local isostasy of its column, about a quarter, so
DeCelles & Giles' 2 to 8 km of foredeep fill is about 0.5 to 2 km of flexure.

Needs numpy and scipy (the pool resolves them with nix). Version 1.
"""
import argparse
import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402
from scipy.spatial import cKDTree  # noqa: E402

import continents_census as cc  # noqa: E402

FORELAND = 128  # plates::state::FORELAND
BIN_KM = 40.0
BINS = 16
HIGH_M = 3000.0  # belts::FOREDEEP_BELT_M
MIN_CELLS = 3
NAN = float("nan")


def med(x):
    x = np.asarray(x, float)
    x = x[np.isfinite(x)]
    return float(np.median(x)) if x.size else NAN


def from_record(run):
    out = {k: NAN for k in ("rec_n_high", "rec_has_share", "rec_width_p50", "rec_depth_p50", "rec_wedge_p50")}
    try:
        rec = json.loads((Path(run) / "rec.json").read_text())
    except (OSError, ValueError):
        return out
    high = [b for b in rec.get("belts") or [] if (b.get("peak_height_m") or 0.0) >= HIGH_M]
    widths = [b["foredeep_width_km"] for b in high if b.get("foredeep_width_km") is not None]
    depths = [b["foredeep_depth_m"] for b in high if b.get("foredeep_depth_m") is not None]
    out["rec_n_high"] = float(len(high))
    out["rec_has_share"] = float(np.mean([w > 0 for w in widths])) if widths else NAN
    out["rec_width_p50"] = med(widths)
    out["rec_depth_p50"] = med(depths)
    out["rec_wedge_p50"] = med([b.get("wedge_width_km") for b in high if b.get("wedge_width_km") is not None])
    return out


def profile(d_km, flex, sed, te_fl, te_fp, n_fp, prefix):
    out = {}
    b = np.floor(d_km / BIN_KM).astype(int)
    keep = b < BINS
    b, flex, sed, te_fl = b[keep], flex[keep], sed[keep], te_fl[keep]
    centre = (np.arange(BINS) + 0.5) * BIN_KM
    m = np.full(BINS, np.nan)
    for k in range(BINS):
        sel = b == k
        if sel.sum() >= MIN_CELLS:
            m[k] = med(flex[sel])
    known = np.nonzero(np.isfinite(m))[0]
    width, depth, cross = NAN, NAN, None
    if known.size:
        first = known[0]
        if m[first] >= 0.0:
            width, depth, cross = 0.0, 0.0, first
        else:
            depth = 0.0
            prev = first
            for k in known:
                if m[k] >= 0.0:
                    # Linear between the last bin below 0 and this one.
                    x0, x1, y0, y1 = centre[prev], centre[k], m[prev], m[k]
                    width = x0 + (x1 - x0) * (-y0) / (y1 - y0)
                    cross = k
                    break
                depth = max(depth, -m[k])
                prev = k
            else:
                width = centre[known[-1]] + 0.5 * BIN_KM
                cross = BINS
    bulge_m, bulge_km = NAN, NAN
    if cross is not None and cross < BINS:
        after = [k for k in known if k >= cross]
        if after:
            k = max(after, key=lambda j: m[j])
            bulge_m, bulge_km = float(m[k]), float(centre[k])
    inside = b < (cross if cross is not None else 0)
    out[prefix + "width_km"] = float(width)
    out[prefix + "depth_m"] = float(depth)
    out[prefix + "bulge_m"] = bulge_m
    out[prefix + "bulge_km"] = bulge_km
    out[prefix + "fill_m"] = med(sed[inside]) if inside.any() else NAN
    out[prefix + "fill_far_m"] = med(sed[~inside]) if (~inside).any() else NAN
    out[prefix + "te_fl"] = med(te_fl)
    out[prefix + "te_fp"] = med(te_fp)
    out[prefix + "n_fl"] = float(d_km.size)
    out[prefix + "n_fp"] = float(n_fp)
    return out


def from_fields(fields_dir):
    keys = ("width_km", "depth_m", "bulge_m", "bulge_km", "fill_m", "fill_far_m", "te_fl", "te_fp", "n_fl", "n_fp")
    blank = {p + k: NAN for p in ("fd_", "fdh_") for k in keys}
    need = ("plates/collision_mm_yr", "plates/flags", "surface/flexure_m", "surface/sediment_m",
            "surface/elastic_thickness_km", "elevation")
    try:
        f, _g, e = cc.load_dump(fields_dir)
    except (OSError, ValueError):
        return blank
    if any(n not in f for n in need):
        return blank
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    ncell = int(ok.sum())
    spacing = math.sqrt(cc.EARTH_KM2 / ncell)
    p = cc.xyz(lat[ok], lon[ok])
    coll = np.nan_to_num(f["plates/collision_mm_yr"][ok], nan=0.0)
    flags = np.rint(np.nan_to_num(f["plates/flags"][ok], nan=0.0)).astype(int)
    fl = (flags & FORELAND) != 0
    fp = (coll > 0.0) & ~fl
    if not fp.any() or not fl.any():
        out = dict(blank)
        out["fd_n_fp"] = out["fdh_n_fp"] = float(fp.sum())
        out["fd_n_fl"] = out["fdh_n_fl"] = float(fl.sum())
        return out
    flex = f["surface/flexure_m"][ok].astype(float)
    sed = f["surface/sediment_m"][ok].astype(float)
    te = f["surface/elastic_thickness_km"][ok].astype(float)
    z = (f["elevation"][ok] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    fpi = np.nonzero(fp)[0]
    fli = np.nonzero(fl)[0]
    d, j = cKDTree(p[fpi]).query(p[fli])
    d_km = cc.arc_km(d)
    out = profile(d_km, flex[fli], sed[fli], te[fli], te[fpi], fpi.size, "fd_")
    # The footprints that stand 3 km or higher: connected footprint pieces by their highest cell.
    comp = cc.components(p, fp, spacing)
    high_comp = {c for c in np.unique(comp[fp]) if np.nanmax(z[(comp == c)]) >= HIGH_M}
    near_comp = comp[fpi[j]]
    sel = np.isin(near_comp, list(high_comp)) if high_comp else np.zeros(fli.size, bool)
    fph = fpi[np.isin(comp[fpi], list(high_comp))] if high_comp else fpi[:0]
    if sel.any():
        out.update(profile(d_km[sel], flex[fli][sel], sed[fli][sel], te[fli][sel], te[fph], fph.size, "fdh_"))
    else:
        out.update({"fdh_" + k: NAN for k in keys})
        out["fdh_n_fp"] = float(fph.size)
        out["fdh_n_fl"] = 0.0
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("run")
    ap.add_argument("--fields")
    a = ap.parse_args()
    out = from_record(a.run)
    out.update(from_fields(a.fields or str(Path(a.run) / "fields")))
    print(json.dumps({k: (None if isinstance(v, float) and not math.isfinite(v) else v) for k, v in out.items()},
                     sort_keys=True))


if __name__ == "__main__":
    main()
