#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The white patches, the drowned continents and the coastal high ground of one run, from the L1
fields of its epoch checkpoints and from its keyframes (Ben's run of 2026-10-21, build log
"White patches and drowned continents").

    deeptime dump-fields --in step-N.dtcp --out D --fields elevation,surface/elevation_m,\
        plates/elevation_m,plates/thickness_m,plates/crust_type,plates/orogeny_type,\
        plates/orogeny_age_myr,climate/ice_thickness_m,climate/ela_m,climate/t_warm_c,\
        climate/t_mean_c,climate/koppen,regime
    python3 -I tools/review/continents_census.py fields D [D ...]
    python3 -I tools/review/continents_census.py keyframes <world or staging folder>
    python3 -I tools/review/continents_census.py maps <ensemble out dir> <label> [t_Myr ...]
    python3 -I tools/review/continents_census.py earth crust1.bnds earth_relief_15m_p.grd [N ...]

`fields`: per dump, (1) the white patches as the satellite view draws them (glacier ice of 1 m
or more, perennial snow: warmest month under 1 C, ground above the snow line, Koppen EF), each
connected patch over 0.5 Mkm2 with its area, latitude, ice share, the ground's height above sea
level under the ice and of the ice surface (median, p90, max), the crust's thickness; (2) the
land and the continental crust: land share, continental crust share, the continental crust under
the sea (shelf, under 200 m, and deeper), the thickness of continental crust on land and under
the sea (p10/p50/p90 against CRUST1.0's land, HYP-040), landmasses over 1 Mkm2; (3) the high
ground by the coast: land above 2 km within 200 km of the sea and the coast with land above 2 km
within 200 km inland, by the orogeny of its crust (plates/orogeny_type: none, Andean, Himalayan).

`keyframes`: the same land and crust shares per keyframe, decoded from the frame's `elevation`
(block heights, ramps.json) and `crust_age` (grey is continental crust) layers, with the sea level
from the manifest's climate series, and the ocean's water volume under that sea level, with the
sea level that would hold the first frame's water and Earth's (the numbers for a sea level that
follows the ocean basins' volume, decisions log 2026-10-21).

`maps`: the crust budget's sample maps of an ensemble (tools/review/force-balance-ensemble.sh
with +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=<out>/maps/@RUN@): per snapshot the land,
the continental crust's drowned share, volume and thickness, and at the chosen times the coastal
high ground split by the origin of the crust (crust_budget.rs `origin`: island arc, collision,
Andean, other).

`earth`: the same measures on Earth: CRUST1.0's crust 25 km and thicker (Laske et al. 2013) and
SRTM15+ 15' (earth_relief_15m_p.grd, Tozer et al. 2019) on N Fibonacci points; the downloads and
their URLs are in tools/review/earth-crust.py and earth-coast.py.

Areas are Earth-normalised (the plate model and the climate are Earth-sized): the planet is
510.07 Mkm2, a share times that. Needs numpy and scipy (and lz4 for keyframes).
"""
import json
import struct
import sys
from pathlib import Path

import numpy as np
from scipy import ndimage
from scipy.spatial import cKDTree

EARTH_KM2 = 510.07e6
EARTH_R_KM = 6371.0
SEA_Y = 63.0
M_PER_BLOCK_V = 35.0
KOPPEN_EF = 30
ICE_MIN_M = 1.0
# Earth's water for the constant-volume sea level: the ocean, 1.332e9 km3 (Charette & Smith 2010),
# and the ice sheets' 29.8 Mkm3 of ice (Antarctica 26.9, Greenland 2.9) at 0.917. from-memory.
EARTH_WATER_KM3 = 1.332e9 + 0.917 * 29.8e6


def lat_of_rows(h, r, circumference):
    """Latitude (deg) of each row of a simulation grid and whether it holds planet."""
    big_r = circumference / (2 * np.pi)
    zo = h * r / 2
    z = -zo + r * np.arange(h)
    ok = np.abs(z) <= big_r
    lat = np.degrees(np.arcsin(np.clip(-z / big_r, -1, 1)))
    return lat, ok


def label_wrap(mask):
    """8-connected components, joined across the x seam."""
    lab, n = ndimage.label(mask, structure=np.ones((3, 3), bool))
    parent = np.arange(n + 1)

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    h = mask.shape[0]
    for y in np.nonzero(lab[:, 0])[0]:
        for dy in (-1, 0, 1):
            yy = y + dy
            if 0 <= yy < h and lab[yy, -1]:
                a, b = find(int(lab[y, 0])), find(int(lab[yy, -1]))
                if a != b:
                    parent[a] = b
    roots = np.array([find(k) for k in range(n + 1)])
    return roots[lab]


def xyz(lat_deg, lon_deg):
    la, lo = np.radians(lat_deg), np.radians(lon_deg)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], -1)


def pct(x, q):
    return float(np.percentile(x, q)) if x.size else float("nan")


def chord(km):
    return 2 * np.sin(km / EARTH_R_KM / 2)


def arc_km(c):
    return 2 * np.arcsin(np.clip(c / 2, 0, 1)) * EARTH_R_KM


def components(p, mask, spacing_km):
    """Connected components of the points in `mask`, joined within 1.6 point spacings (-1 elsewhere)."""
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import connected_components
    idx = np.nonzero(mask)[0]
    out = np.full(mask.size, -1)
    if idx.size == 0:
        return out
    pairs = cKDTree(p[idx]).query_pairs(chord(1.6 * spacing_km), output_type="ndarray")
    g = coo_matrix((np.ones(len(pairs)), (pairs[:, 0], pairs[:, 1])), shape=(idx.size, idx.size))
    _, lab = connected_components(g, directed=False)
    out[idx] = lab
    return out


def largest_high(p, z, spacing_km, level=3000.0, exclude=None):
    """The largest connected piece of ground above `level` m (a plateau's stand-in, Tibet's is the
    ground above 3 km): its area (Mkm2, Earth-normalised by the points' share) and median height."""
    m = z > level
    if exclude is not None:
        m &= ~exclude
    comp = components(p, m, spacing_km)
    if not m.any():
        return 0.0, float("nan")
    c = np.bincount(comp[m])
    k = int(np.argmax(c))
    sel = comp == k
    return c[k] / z.size * EARTH_KM2 / 1e6, float(np.median(z[sel]))


def coast_metrics(p, z, spacing_km, groups=None, gnames=(), exclude=None, prefix="  "):
    """The high ground by the coast, on points (unit vectors p, heights above the sea z in m, about
    `spacing_km` apart): the land above 2 km within 200 km of the sea (great circle to the nearest
    sea point), and the coast (land within 1.6 spacings of the sea) with land above 2 km of the same
    landmass within 200 km. `groups` splits both by an origin per point (the high ground by its own,
    the coast by the coast's); points in `exclude` count for the geometry but not in the shares."""
    keep = np.ones(z.size, bool) if exclude is None else ~exclude
    land = z > 0
    d_sea = np.zeros(z.size)
    ds, _ = cKDTree(p[~land]).query(p[land])
    d_sea[land] = arc_km(ds)
    coast = land & (d_sea <= 1.6 * spacing_km)
    hi = land & (z > 2000)
    comp = components(p, land, spacing_km)
    has = np.zeros(z.size, bool)
    for c in np.unique(comp[hi]):
        hm, cm = hi & (comp == c), coast & (comp == c)
        if cm.any():
            d, _ = cKDTree(p[hm]).query(p[cm], distance_upper_bound=chord(200.0))
            has[np.nonzero(cm)[0][np.isfinite(d)]] = True
    near = d_sea <= 200.0
    L, H, C = land & keep, hi & keep, coast & keep
    res = {"land": L.sum() / max(keep.sum(), 1), "hi_of_land": H.sum() / max(L.sum(), 1),
           "hi_near_coast": (H & near).sum() / max(H.sum(), 1), "coast_with_hi": (C & has).sum() / max(C.sum(), 1),
           "land_near_coast": (L & near).sum() / max(L.sum(), 1)}
    print(f"{prefix}land {res['land']:.4f}; above 2 km {res['hi_of_land']:.4f} of the land, of it within 200 km of"
          f" the sea {res['hi_near_coast']:.3f} (all land {res['land_near_coast']:.3f}); coast points {C.sum()},"
          f" with land above 2 km within 200 km on the same landmass {res['coast_with_hi']:.3f}")
    if groups is not None:
        for k, nm in enumerate(gnames):
            g = groups == k
            if not (L & g).any():
                continue
            print(f"{prefix}  {nm:11s}: land {(L & g).sum() / max(L.sum(), 1):.3f}; high ground"
                  f" {(H & g).sum() / max(H.sum(), 1):.3f} of it, its own within 200 km of the sea"
                  f" {(H & g & near).sum() / max((H & g).sum(), 1):.3f} (its land"
                  f" {(L & g & near).sum() / max((L & g).sum(), 1):.3f}); coast {(C & g).sum() / max(C.sum(), 1):.3f},"
                  f" with high ground within 200 km {(C & g & has).sum() / max((C & g).sum(), 1):.3f}")
    return res


def load_dump(d):
    d = Path(d)
    idx = json.loads((d / "index.json").read_text())
    g = json.loads((d / "globals.json").read_text())
    out = {}
    for name, e in idx.items():
        out[name] = np.fromfile(d / e["file"], "<f4").reshape(e["h"], e["w"])
    any_e = next(iter(idx.values()))
    return out, g, any_e


def fields_report(d):
    f, g, e = load_dump(d)
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    ncell = ok.sum()
    cell_km2 = EARTH_KM2 / ncell
    t = g.get("t_years", g.get("globals", {}).get("t_years"))
    sea = g["globals"].get("climate/sea_level_m", float("nan"))
    z = (f["elevation"] - SEA_Y) * M_PER_BLOCK_V  # height above the step's sea level, m
    land = ok & (z > 0)
    cont = ok & (f["plates/crust_type"] > 0.5)
    th = f["plates/thickness_m"] / 1000.0
    ice = np.nan_to_num(f.get("climate/ice_thickness_m", np.zeros_like(z)))
    ela = f.get("climate/ela_m")
    tw = f.get("climate/t_warm_c")
    kop = f.get("climate/koppen")
    oro = np.nan_to_num(f.get("plates/orogeny_type", np.zeros_like(z)))
    print(f"== {d}  t={t}  sea_level_m={sea:.1f}  cells={ncell} ({cell_km2:.0f} km2 each)")

    # (1) The white patches.
    glacier = land & (ice >= ICE_MIN_M)
    snow_line = land & np.isfinite(ela) & (z - ela > 0) if ela is not None else np.zeros_like(land)
    cold = land & (tw < -1.0) if tw is not None else np.zeros_like(land)
    ef = land & (np.round(kop) == KOPPEN_EF) if kop is not None else np.zeros_like(land)
    white = glacier | snow_line | cold | ef
    # The ground under the ice: the elevation field is the ice-free surface (the climate's ice
    # sits on it), so the ice surface is z + ice.
    print(f"land {land.sum() / ncell:.4f} of the planet; white {white.sum() / max(land.sum(), 1):.4f} of"
          f" the land (glacier {glacier.sum() / max(land.sum(), 1):.4f}, above the snow line"
          f" {snow_line.sum() / max(land.sum(), 1):.4f}, warmest month under -1 C"
          f" {cold.sum() / max(land.sum(), 1):.4f}, EF {ef.sum() / max(land.sum(), 1):.4f})")
    lab = label_wrap(white)
    sizes = np.bincount(lab.ravel(), minlength=lab.max() + 1)
    sizes[0] = 0
    order = np.argsort(-sizes)
    print("  patch  area_Mkm2  lat(mean,min,max)  glacier_share  ground_z p50/p90/max m  ice_surface p50/p90/max m"
          "  ice p50/max m  ela_p50 m  crust_km p50/p90  cont_share  orogeny none/And/Him")
    for k in order[:12]:
        if sizes[k] * cell_km2 < 0.5e6:
            break
        m = lab == k
        zz, ii = z[m], ice[m]
        surf = zz + ii
        el = ela[m] if ela is not None else np.array([np.nan])
        o = oro[m]
        print(f"  {k:5d}  {sizes[k] * cell_km2 / 1e6:8.2f}  {lat[m].mean():6.1f} {lat[m].min():6.1f} {lat[m].max():6.1f}"
              f"  {(ii >= ICE_MIN_M).mean():.2f}"
              f"  {pct(zz, 50):6.0f}/{pct(zz, 90):6.0f}/{zz.max():6.0f}"
              f"  {pct(surf, 50):6.0f}/{pct(surf, 90):6.0f}/{surf.max():6.0f}"
              f"  {pct(ii, 50):5.0f}/{ii.max():5.0f}  {np.nanmedian(el):6.0f}"
              f"  {pct(th[m], 50):5.1f}/{pct(th[m], 90):5.1f}  {cont[m].mean():.2f}"
              f"  {(o < 0.5).mean():.2f}/{((o > 0.5) & (o < 1.5)).mean():.2f}/{(o > 1.5).mean():.2f}")

    # Hypsometry of the land by the rock (ice-free ground) and by the surface (ice on top).
    zl, il = z[land], ice[land]
    shares = {k: float((zl > k * 1000).mean()) for k in (1, 2, 3, 4, 5)}
    sshares = {k: float((zl + il > k * 1000).mean()) for k in (2, 4)}
    print("land above 1/2/3/4/5 km (rock), share of land: "
          + " ".join(f"{shares[k]:.4f}" for k in (1, 2, 3, 4, 5))
          + f"; of the planet above 2/4 km: {shares[2] * land.sum() / ncell:.4f} {shares[4] * land.sum() / ncell:.4f}"
          + f"; by the ice surface above 2/4 km: {sshares[2]:.4f} {sshares[4]:.4f}"
          + f"; land median {pct(zl, 50):.0f} m, p90 {pct(zl, 90):.0f} m, max {zl.max():.0f} m")
    # The highest mean elevation of any 1,000 km box (a 1,000 km square around each cell: rows
    # are 16 blocks of the 2,607.6-block radius, 2.443 km each north-south over cos(lat)).
    km_per_block = EARTH_R_KM / (w * r / (2 * np.pi))
    dy = r * km_per_block  # north-south at the equator (rows uniform in sin(lat))
    zpos = np.where(ok, z, np.nan)
    best = (-1e9, 0, 0)
    zs = np.where(np.isfinite(zpos), zpos, 0.0)
    for j in range(h):
        if not ok_r[j]:
            continue
        c = max(np.cos(np.radians(lat_r[j])), 0.05)
        half_rows = int(round(500.0 / (dy / c)))
        half_cols = int(round(500.0 / (dy * c)))
        if half_cols * 2 + 1 >= w:
            continue
        j0, j1 = max(0, j - half_rows), min(h, j + half_rows + 1)
        band = zs[j0:j1].sum(0)
        cs = np.concatenate([band[-half_cols:], band, band[:half_cols]]).cumsum()
        win = cs[2 * half_cols:] - np.concatenate([[0], cs[:-2 * half_cols - 1]])
        n = (j1 - j0) * (2 * half_cols + 1)
        i = int(np.argmax(win))
        if win[i] / n > best[0]:
            best = (win[i] / n, lat_r[j], 360.0 * i / w)
    print(f"highest mean elevation of a 1,000 km box: {best[0]:.0f} m at lat {best[1]:.1f} lon {best[2]:.1f}")

    # (2) Land and continental crust.
    wet_cont = cont & ~land
    depth = -z
    shelf = wet_cont & (depth < 200)
    deep = wet_cont & (depth >= 200)
    lab_l = label_wrap(land)
    ls = np.bincount(lab_l.ravel())
    ls[0] = 0
    big = int((ls * cell_km2 >= 1e6).sum())
    print(f"land {land.sum() / ncell:.4f}; continental crust {cont.sum() / ncell:.4f} of the planet;"
          f" of it under the sea {wet_cont.sum() / max(cont.sum(), 1):.4f} (shelf <200 m"
          f" {shelf.sum() / max(cont.sum(), 1):.4f}, deeper {deep.sum() / max(cont.sum(), 1):.4f});"
          f" drowned continent {wet_cont.sum() / ncell:.4f} of the planet; land on oceanic crust"
          f" {(land & ~cont).sum() / max(land.sum(), 1):.4f} of the land; landmasses over 1 Mkm2: {big}"
          f" (largest {ls.max() * cell_km2 / 1e6:.1f} Mkm2)")
    for name, m in (("land", cont & land), ("drowned", wet_cont), ("all", cont)):
        x = th[m]
        print(f"  continental crust {name:8s}: thickness p10/p25/p50/p75/p90 {pct(x, 10):.1f}/{pct(x, 25):.1f}/"
              f"{pct(x, 50):.1f}/{pct(x, 75):.1f}/{pct(x, 90):.1f} km; elevation p50 {pct(z[m], 50):.0f} m")
    if wet_cont.any():
        d = depth[wet_cont]
        print(f"  drowned continental crust depth p10/p50/p90 {pct(d, 10):.0f}/{pct(d, 50):.0f}/{pct(d, 90):.0f} m")
    # Bands, as the crust budget's EDGES_KM.
    edges = [0, 20, 25, 30, 35, 40, 45, 55, 70, 1e9]
    row = []
    for a, b in zip(edges[:-1], edges[1:]):
        m = cont & (th >= a) & (th < b)
        if m.sum() == 0:
            continue
        row.append(f"[{a}-{b if b < 1e8 else 'inf'} km a={m.sum() / max(cont.sum(), 1):.3f} dry={(m & land).sum() / m.sum():.2f}"
                   f" z50={pct(z[m], 50):.0f}]")
    print("  bands: " + " ".join(row))

    # The continental crust's volume, and the thick crust as earth-crust.py counts Earth's (25 km
    # and thicker, whatever its kind).
    vol = float((th[cont]).sum() * cell_km2)
    thick = ok & (th >= 25.0)
    fb = float(np.quantile(z[ok], 1 - 0.205))
    print(f"SERIES t={t} sea={sea:.1f} land={land.sum() / ncell:.4f} cont={cont.sum() / ncell:.4f}"
          f" cont_wet={wet_cont.sum() / max(cont.sum(), 1):.3f} vol_1e9km3={vol / 1e9:.3f}"
          f" mean_t_km={vol / max(cont.sum() * cell_km2, 1):.2f} t25={thick.sum() / ncell:.4f}"
          f" t25_dry={(thick & land).sum() / max(thick.sum(), 1):.3f} freeboard41_m={fb:.0f}"
          f" hi2={(z[land] > 2000).mean():.4f} hi4={(z[land] > 4000).mean():.4f}"
          f" white={white.sum() / max(land.sum(), 1):.4f} glacier={glacier.sum() / max(land.sum(), 1):.4f}"
          f" thick55_km2={(cont & (th >= 55)).sum() * cell_km2 / 1e6:.1f}M"
          f" thick55_z50={pct(z[cont & (th >= 55)], 50):.0f}")
    a3, z3 = largest_high(xyz(lat[ok], lon[ok]), z[ok], np.sqrt(cell_km2))
    age = f.get("plates/orogeny_age_myr")
    hi_age = age[land & (z > 2000)] if age is not None else np.array([])
    n_age = hi_age.size
    hi_age = hi_age[np.isfinite(hi_age)]
    print(f"  the largest ground above 3 km: {a3:.2f} Mkm2, median {z3:.0f} m; the land above 2 km's last"
          f" orogeny p10/50/90 {pct(hi_age, 10):.0f}/{pct(hi_age, 50):.0f}/{pct(hi_age, 90):.0f} Myr old"
          f" ({hi_age.size} of {n_age} with an age)")
    # The high plateaus' summer: the warmest month and the equilibrium line on land 3.5 to 5 km
    # high, by latitude band (Tibet's and the Altiplano's latitudes), against the ice there.
    if tw is not None and ela is not None:
        for lo_, hi_ in ((10.0, 25.0), (25.0, 45.0), (45.0, 60.0)):
            sel = land & (z > 3500) & (z < 5000) & (np.abs(lat) >= lo_) & (np.abs(lat) < hi_)
            if sel.sum() < 20:
                continue
            print(f"  land 3.5 to 5 km at |lat| {lo_:.0f} to {hi_:.0f}: {sel.sum() * cell_km2 / 1e6:.2f} Mkm2,"
                  f" ground p50 {pct(z[sel], 50):.0f} m; warmest month p10/50/90 {pct(tw[sel], 10):.1f}/"
                  f"{pct(tw[sel], 50):.1f}/{pct(tw[sel], 90):.1f} C; ELA p50 {pct(ela[sel][np.isfinite(ela[sel])], 50):.0f} m;"
                  f" glacier share {(ice[sel] >= ICE_MIN_M).mean():.2f}")
    print("  coastal high ground on the grid's cells, by plates/orogeny_type:")
    coast_metrics(xyz(lat[ok], lon[ok]), z[ok], np.sqrt(cell_km2), np.round(oro[ok]).astype(int),
                  ("none", "Andean", "Himalayan"))
    g = g["globals"]
    keys = [k for k in sorted(g) if k.startswith("plates/") and k.endswith("_km3")]
    print("  plate globals (km3): " + " ".join(f"{k[7:]}={g[k]:.3e}" for k in keys))


# Keyframes.
def read_dtk(path, want):
    import lz4.block
    data = Path(path).read_bytes()
    assert data[:4] == b"DTKF", path
    n, w, h, nl = struct.unpack_from("<IIII", data, 16)
    (t,) = struct.unpack_from("<d", data, 32)
    table = 16 + 24
    sections = [struct.unpack_from("<QII", data, table + 16 * k) for k in range(1 + nl)]
    off, _ln, _ = sections[0]
    names, p = [], off
    for _ in range(1 + nl):
        (sl,) = struct.unpack_from("<I", data, p)
        names.append(data[p + 4:p + 4 + sl].decode())
        p += 4 + sl
    out = {}
    for k, name in enumerate(names[1:]):
        if name in want:
            off, ln, _ = sections[1 + k]
            raw = lz4.block.decompress(data[off:off + ln], uncompressed_size=w * h * 4)
            out[name] = np.frombuffer(raw, np.uint8).reshape(h, w, 4)
    return out, t, names[0]


def hexrgb(s):
    s = s.lstrip("#")
    return np.array([int(s[k:k + 2], 16) for k in (0, 2, 4)], float)


def decode_ramp(img, stops):
    """Physical values of RGB pixels on a piecewise-linear ramp: for each segment, the projection of
    the colour on it, and the segment whose reconstruction is nearest."""
    rgb = img[..., :3].reshape(-1, 3).astype(float)
    best = np.full(rgb.shape[0], np.inf)
    val = np.full(rgb.shape[0], np.nan)
    for (v0, c0), (v1, c1) in zip(stops[:-1], stops[1:]):
        a, b = hexrgb(c0), hexrgb(c1)
        dv = b - a
        if v1 <= v0:
            continue
        L2 = float(dv @ dv)
        tt = np.clip(((rgb - a) @ dv) / L2, 0, 1) if L2 > 0 else np.zeros(rgb.shape[0])
        rec = a + tt[:, None] * dv
        err = ((rgb - rec) ** 2).sum(1)
        better = err < best
        best[better] = err[better]
        val[better] = v0 + tt[better] * (v1 - v0)
    val[img[..., 3].reshape(-1) == 0] = np.nan
    return val.reshape(img.shape[:2])


def keyframes_report(folder, ramps_path):
    folder = Path(folder)
    ramps = json.loads(Path(ramps_path).read_text())["ramps"]
    man = json.loads((folder / "manifest.json").read_text())
    ser = man["notes"]["climate"]["series"]
    cols = ser["columns"]
    rows = np.array([[np.nan if v is None else v for v in r] for r in ser["rows"]], float)
    ci = {c: k for k, c in enumerate(cols)}
    idx = json.loads((folder / "keyframes" / "index.json").read_text())
    print("columns: frame, t (Myr), stage; land; continental crust (share of the planet); the share of the"
          " continental crust under the sea (shelf under 200 m, deeper); drowned continent (share of the"
          " planet); land on oceanic crust (share of land); land above 2 / 4 km (share of land); land under"
          " ice (share of land); freeboard: the median height of the highest 41 % of the planet above the"
          " sea (Earth's continental share, HYP-012), m; sea level (m, the climate series); ice volume"
          " (Mkm3); the ocean's water under that sea level (1e9 km3); the sea level holding the first frame's"
          " water (ice returned at 0.917) and the land there; the same for Earth's water; landmasses over"
          " 1 Mkm2, the largest (Mkm2)")
    base_vol = None
    for fr in idx["frames"]:
        if "elevation" not in fr["layers"] or "crust_age" not in fr["layers"]:
            continue
        want = {"elevation", "crust_age", "climate/ice_thickness_m"}
        lay, t, stage = read_dtk(folder / fr["path"], want)
        h, w = lay["elevation"].shape[:2]
        e = decode_ramp(lay["elevation"], ramps["elevation"]["stops"])
        ca = lay["crust_age"][..., :3].astype(int)
        cont = (ca[..., 0] == 0x9A) & (ca[..., 1] == 0x9A) & (ca[..., 2] == 0x9A)
        # The sea and the land are told apart exactly by the ramp's step at 63.
        land_px = e >= SEA_Y
        z = (e - SEA_Y) * M_PER_BLOCK_V
        ok = np.isfinite(e)
        n = ok.sum()
        land = ok & land_px
        wet = cont & ok & ~land
        shelf = wet & (z > -200)
        if "climate/ice_thickness_m" in lay:
            icev = decode_ramp(lay["climate/ice_thickness_m"], ramps["ice"]["stops"])
            iced = land & (np.nan_to_num(icev) >= 1.0)
            iced_s = f"{iced.sum() / max(land.sum(), 1):.3f}"
        else:
            iced_s = "  -  "
        tm = t / 1e6
        k = int(np.argmin(np.abs(rows[:, ci["t_myr"]] - tm)))
        sea = rows[k, ci["sea_level_m"]] if abs(rows[k, ci["t_myr"]] - tm) <= 2.1 else float("nan")
        ice_v = rows[k, ci["ice_volume_km3"]] if abs(rows[k, ci["t_myr"]] - tm) <= 2.1 else float("nan")
        ice_we = 0.917 * ice_v if np.isfinite(ice_v) else 0.0
        # Ocean water over the basins: Σ depth below sea level, Earth-normalised km3.
        cell_km2 = EARTH_KM2 / n
        vol = float(np.where(ok & ~land, -z, 0).sum() * cell_km2 / 1000.0)
        # The water held as ice (0.917) goes back to the sea for a constant total.
        water = vol + ice_we
        if base_vol is None:
            base_vol = water
        zz = z[ok]

        def level_for(total):
            """The sea level, relative to this frame's, that holds `total` water less the ice, solved
            on the frame's own ground, and the land share there."""
            target = total - ice_we
            lo, hi = -4000.0, 4000.0
            for _ in range(50):
                mid = (lo + hi) / 2
                v = float(np.clip(mid - zz, 0, None).sum() * cell_km2 / 1000.0)
                if v < target:
                    lo = mid
                else:
                    hi = mid
            d = (lo + hi) / 2
            return d, float((zz > d).mean())

        d0, l0 = level_for(base_vol)
        de, le = level_for(EARTH_WATER_KM3)
        lab = label_wrap(land)
        ls = np.bincount(lab.ravel())
        ls[0] = 0
        zl = z[land]
        fb = float(np.quantile(zz, 1 - 0.205))  # the median of the highest 41 %
        print(f"{fr['n']:2d} {tm:7.0f} {stage[:8]:8s} {land.sum() / n:.4f} {(cont & ok).sum() / n:.4f}"
              f"  {wet.sum() / max((cont & ok).sum(), 1):.3f}({shelf.sum() / max((cont & ok).sum(), 1):.3f},"
              f"{(wet & ~shelf).sum() / max((cont & ok).sum(), 1):.3f})  {wet.sum() / n:.4f}"
              f"  {(land & ~cont).sum() / max(land.sum(), 1):.3f}"
              f"  {(zl > 2000).mean():.3f} {(zl > 4000).mean():.3f}  {iced_s}  {fb:5.0f}"
              f"  {sea:7.1f}  {ice_v / 1e6:6.2f}"
              f"  {vol / 1e9:6.3f}  {sea + d0:7.1f} ({d0:+5.0f}) {l0:.3f}  {sea + de:7.1f} ({de:+5.0f}) {le:.3f}"
              f"  {int((ls * cell_km2 >= 1e6).sum()):3d}  {ls.max() * cell_km2 / 1e6:6.1f}")


MAP_FIELDS = 11  # crust_budget.rs MAP_FIELDS: lat, lon, z, thickness, kind, orogeny, age, arc_oc, arc_co, group, flags
ORIGINS = ("island_arc", "collision", "Andean", "other")


def map_origin(m):
    """crust_budget.rs `origin`: island-arc crust (1 km or more of island-arc magma, more than of
    continental-arc magma), then a collision's belt, then an Andean margin, then the rest."""
    o = np.full(m.shape[0], 3)
    o[m[:, 5] == 1] = 2
    o[m[:, 5] == 2] = 1
    o[(m[:, 7] >= 1000.0) & (m[:, 7] > m[:, 8])] = 0
    return o


def maps_report(out_dir, label, times):
    """The crust budget's sample maps of each run `<label>-<start>-<seed>` in an ensemble folder
    (force-balance-ensemble.sh with +DEEPTIME_CRUST_BUDGET_MAPS=<out>/maps/@RUN@): per snapshot
    the land, the continental crust, its drowned share and thickness, and the coastal high ground
    by the origin of its crust, at the climate's sea level of the run's cb_t line."""
    import re
    out_dir = Path(out_dir)
    for md in sorted((out_dir / "maps").glob(f"{label}-*")):
        txt = next(iter(sorted(f for f in out_dir.glob(f"{md.name}-*.txt")
                               if re.fullmatch(re.escape(md.name) + r"-[0-9]+\.txt", f.name))), None)
        seas = {}
        if txt is not None:
            for line in txt.read_text(errors="replace").splitlines():
                if line.startswith("cb_t "):
                    t = re.search(r"\bt=(-?[0-9.]+)", line)
                    s = re.search(r"\bsea_m=(-?[0-9.]+)", line)
                    if t and s:
                        seas[round(float(t.group(1)))] = float(s.group(1))
        files = sorted(md.glob("t*.bin"), key=lambda f: float(f.stem[1:]))
        print(f"== {md.name} ({len(files)} snapshots, sea levels from {txt.name if txt else 'none'})")
        for f in files:
            tm = float(f.stem[1:])
            sea = seas.get(round(tm), 0.0)
            m = np.fromfile(f, "<f4").reshape(-1, MAP_FIELDS).astype(np.float64)
            n = m.shape[0]
            z = m[:, 2] - sea
            th = m[:, 3] / 1000.0
            cont = m[:, 4] > 0.5
            land = z > 0
            wet = cont & ~land
            cell = EARTH_KM2 / n
            line = (f"MAPS {md.name} t={tm:.0f} sea={sea:.0f} land={land.mean():.4f} cont={cont.mean():.4f}"
                    f" cont_wet={wet.sum() / max(cont.sum(), 1):.3f} shelf={(wet & (z > -200)).sum() / max(cont.sum(), 1):.3f}"
                    f" vol_1e9km3={th[cont].sum() * cell / 1e9:.3f} t_land_p10/50/90={pct(th[cont & land], 10):.1f}/"
                    f"{pct(th[cont & land], 50):.1f}/{pct(th[cont & land], 90):.1f}"
                    f" t_wet_p10/50/90={pct(th[wet], 10):.1f}/{pct(th[wet], 50):.1f}/{pct(th[wet], 90):.1f}"
                    f" t25_dry={((th >= 25) & land).sum() / max((th >= 25).sum(), 1):.3f}"
                    f" hi2={(z[land] > 2000).mean():.4f} hi4={(z[land] > 4000).mean():.4f}")
            if any(abs(tm - x) < 1 for x in times):
                p = xyz(m[:, 0], m[:, 1])
                a3, z3 = largest_high(p, z, np.sqrt(cell))
                print(line + f" plateau3_Mkm2={a3:.2f} plateau3_z50={z3:.0f}")
                coast_metrics(p, z, np.sqrt(cell), map_origin(m), ORIGINS, prefix=f"  t={tm:.0f} ")
            else:
                print(line)


def earth_report(bnds, relief, ns):
    """Earth's references on the same measures: CRUST1.0's crust 25 km and thicker (dry share,
    shelf, thickness on land and under the sea) and, on N Fibonacci points of SRTM15+ 15', the land
    above 2 and 4 km, the coastal high ground (with and without Antarctica and Greenland, whose ice
    surfaces stand above 2 km), the median height of the highest 41 % and the highest 1,000 km box."""
    import h5py
    b = np.loadtxt(bnds)
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    w = np.cos(np.radians(lat))
    top, rock, moho = b[:, 0] * 1000, b[:, 2] * 1000, b[:, 8] * 1000
    thick = (rock - moho) / 1000.0
    water = top - b[:, 1] * 1000
    land = (water <= 0.0) & (top > 0.0)
    for lim in (20.0, 25.0, 30.0):
        c = thick >= lim
        wet = c & ~land
        # The water's depth, not the surface (CRUST1.0's top is the sea surface, 0, at sea).
        shelf = wet & (water < 200.0)

        def q(sel):
            o = np.argsort(thick[sel])
            x, ww = thick[sel][o], w[sel][o]
            cc = np.cumsum(ww) / ww.sum()
            return "/".join(f"{np.interp(k, cc, x):.1f}" for k in (0.1, 0.5, 0.9))
        print(f"CRUST1.0 crust >= {lim:.0f} km: {w[c].sum() / w.sum():.4f} of the planet; under the sea"
              f" {w[wet].sum() / w[c].sum():.3f} (shelf <200 m {w[shelf].sum() / w[c].sum():.3f}); thickness p10/50/90"
              f" land {q(c & land)}, under the sea {q(wet)}, all {q(c)} km")
    with h5py.File(relief, "r") as f:
        zg = f["z"][()].astype(np.float64)
        la = f["lat"][()] if "lat" in f else f["y"][()]
        lo = f["lon"][()] if "lon" in f else f["x"][()]
        at = {k: f["z"].attrs[k] for k in f["z"].attrs}
    zg = zg * float(np.ravel(at.get("scale_factor", 1.0))[0]) + float(np.ravel(at.get("add_offset", 0.0))[0])
    # The highest 1,000 km box on the grid (sea floor counted, as for the model).
    dlat = abs(float(la[1] - la[0]))
    best = (-1e9, 0, 0)
    hr = int(round(500.0 / (111.195 * dlat)))
    for j in range(hr, len(la) - hr):
        c = max(np.cos(np.radians(la[j])), 0.05)
        hc = int(round(500.0 / (111.195 * dlat * c)))
        if 2 * hc + 1 >= len(lo):
            continue
        band = zg[j - hr:j + hr + 1].sum(0)
        cs = np.concatenate([band[-hc:], band, band[:hc]]).cumsum()
        win = cs[2 * hc:] - np.concatenate([[0], cs[:-2 * hc - 1]])
        i = int(np.argmax(win))
        v = win[i] / ((2 * hr + 1) * (2 * hc + 1))
        if v > best[0]:
            best = (v, float(la[j]), float(lo[i]))
    print(f"Earth: highest mean elevation of a 1,000 km box {best[0]:.0f} m at lat {best[1]:.1f} lon {best[2]:.1f}")
    for n in ns:
        i = np.arange(n) + 0.5
        plat = np.degrees(np.arcsin(1 - 2 * i / n))
        plon = np.degrees((np.pi * (1 + 5 ** 0.5) * i) % (2 * np.pi) - np.pi)
        iy = np.clip(np.searchsorted(la, plat), 0, len(la) - 1)
        ix = np.clip(np.searchsorted(lo, plon), 0, len(lo) - 1)
        z = zg[iy, ix]
        sheets = (plat < -60.0) | ((plat > 59.5) & (plon > -74.0) & (plon < -10.0) & ~((plat < 66.7) & (plon > -25.0)))
        zl = z[z > 0]
        print(f"Earth N={n}: land {np.mean(z > 0):.4f}; above 1/2/3/4/5 km of land "
              + "/".join(f"{np.mean(zl > k * 1000):.4f}" for k in (1, 2, 3, 4, 5))
              + f"; without the two ice sheets above 2/4 km {np.mean(z[(z > 0) & ~sheets] > 2000):.4f}/"
              f"{np.mean(z[(z > 0) & ~sheets] > 4000):.4f}; freeboard41 {np.quantile(z, 1 - 0.205):.0f} m")
        p = xyz(plat, plon)
        sp = np.sqrt(EARTH_KM2 / n)
        a3, z3 = largest_high(p, z, sp, exclude=sheets)
        print(f"  Earth N={n}: the largest ground above 3 km, without the ice sheets: {a3:.2f} Mkm2, median {z3:.0f} m")
        coast_metrics(p, z, sp, prefix=f"  Earth N={n} all: ")
        coast_metrics(p, z, sp, exclude=sheets, prefix=f"  Earth N={n} without Antarctica and Greenland: ")


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(2)
    if sys.argv[1] == "fields":
        for d in sys.argv[2:]:
            fields_report(d)
    elif sys.argv[1] == "keyframes":
        ramps = Path(__file__).resolve().parents[2] / "crates/deeptime-viewer/assets/ramps.json"
        keyframes_report(sys.argv[2], ramps)
    elif sys.argv[1] == "maps":
        maps_report(sys.argv[2], sys.argv[3], [float(x) for x in sys.argv[4:]] or [0.0])
    elif sys.argv[1] == "earth":
        earth_report(sys.argv[2], sys.argv[3], [int(x) for x in sys.argv[4:]] or [100_000])
