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
from the manifest's climate series, and the ocean's water volume under that sea level.

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

    # (3) High ground by the coast.
    sea_m = ok & ~land
    p_sea = xyz(lat[sea_m], lon[sea_m])
    tree = cKDTree(p_sea)
    p_land = xyz(lat[land], lon[land])
    dist, _ = tree.query(p_land, k=1)
    dist_km = 2 * np.arcsin(np.clip(dist / 2, 0, 1)) * EARTH_R_KM
    zl = z[land]
    hi = zl > 2000
    near = dist_km <= 200
    o = oro[land]
    names = {0: "none", 1: "Andean", 2: "Himalayan"}
    print(f"land above 2 km {hi.mean():.4f} of the land; of it within 200 km of the sea {(hi & near).sum() / max(hi.sum(), 1):.4f}")
    for k, nm in names.items():
        m = np.round(o) == k
        if (hi & m).sum():
            print(f"  orogeny {nm:9s}: share of the high ground {(hi & m).sum() / hi.sum():.3f};"
                  f" of it within 200 km of the sea {(hi & m & near).sum() / (hi & m).sum():.3f}")
    # Coast cells: land cells with a sea neighbour (4-connected, x wraps).
    sea_pad = np.pad(~land & ok, ((1, 1), (0, 0)), constant_values=False)
    nb = (np.roll(~land & ok, 1, 1) | np.roll(~land & ok, -1, 1) | sea_pad[:-2] | sea_pad[2:])
    coast = land & nb
    p_hi = xyz(lat[land][hi], lon[land][hi])
    if p_hi.size:
        th_tree = cKDTree(p_hi)
        dc, _ = th_tree.query(xyz(lat[coast], lon[coast]), k=1)
        dc_km = 2 * np.arcsin(np.clip(dc / 2, 0, 1)) * EARTH_R_KM
        oc = oro[coast]
        print(f"coast cells {coast.sum()}; with land above 2 km within 200 km inland {(dc_km <= 200).mean():.4f}"
              f" (crude: the high ground's nearest cell, not checked to be inland of that coast)")
        for k, nm in names.items():
            m = np.round(oc) == k
            if m.sum():
                print(f"  coast crust orogeny {nm:9s}: share of the coast {m.mean():.3f}; with high ground within 200 km"
                      f" {(dc_km[m] <= 200).mean():.3f}")


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
    print("n  t_Myr  land  cont  cont_wet(shelf,deeper)  drowned_of_planet  land_on_ocean  sea_m  ice_vol_Mkm3"
          "  ocean_vol_1e9km3  sea_for_const_vol_m  masses>1Mkm2  largest_Mkm2")
    base_vol = None
    for fr in idx["frames"]:
        if "elevation" not in fr["layers"] or "crust_age" not in fr["layers"]:
            continue
        lay, t, stage = read_dtk(folder / fr["path"], {"elevation", "crust_age"})
        h, w = lay["elevation"].shape[:2]
        e = decode_ramp(lay["elevation"], ramps["elevation"]["stops"])
        ca = lay["crust_age"][..., :3].astype(int)
        cont = (ca[..., 0] == 0x9A) & (ca[..., 1] == 0x9A) & (ca[..., 2] == 0x9A)
        # The sea and the land are told apart exactly by the ramp's step at 63.
        land_px = decode_ramp(lay["elevation"], ramps["elevation"]["stops"]) >= SEA_Y
        z = (e - SEA_Y) * M_PER_BLOCK_V
        ok = np.isfinite(e)
        n = ok.sum()
        land = ok & land_px
        wet = cont & ok & ~land
        shelf = wet & (z > -200)
        tm = t / 1e6
        k = int(np.argmin(np.abs(rows[:, ci["t_myr"]] - tm)))
        sea = rows[k, ci["sea_level_m"]] if abs(rows[k, ci["t_myr"]] - tm) <= 2.1 else float("nan")
        ice_v = rows[k, ci["ice_volume_km3"]] if abs(rows[k, ci["t_myr"]] - tm) <= 2.1 else float("nan")
        # Ocean water over the basins: Σ depth below sea level, Earth-normalised km3.
        cell_km2 = EARTH_KM2 / n
        vol = float(np.where(ok & ~land, -z, 0).sum() * cell_km2 / 1000.0)
        # The water held as ice (0.917) goes back to the sea for a constant total.
        water = vol + (0.917 * ice_v if np.isfinite(ice_v) else 0.0)
        if base_vol is None:
            base_vol = water
        # The sea level (relative to this frame's) that would hold base_vol of water: solve on the
        # frame's own ground (absolute heights = z + sea; heights relative to the frame's sea z).
        zz = z[ok]
        target = base_vol - (0.917 * ice_v if np.isfinite(ice_v) else 0.0)
        lo, hi = -3000.0, 3000.0
        for _ in range(50):
            mid = (lo + hi) / 2
            v = float(np.clip(mid - zz, 0, None).sum() * cell_km2 / 1000.0)
            if v < target:
                lo = mid
            else:
                hi = mid
        dsea = (lo + hi) / 2
        lab = label_wrap(land)
        ls = np.bincount(lab.ravel())
        ls[0] = 0
        print(f"{fr['n']:2d} {tm:7.0f} {stage[:8]:8s} {land.sum() / n:.4f} {(cont & ok).sum() / n:.4f}"
              f"  {wet.sum() / max((cont & ok).sum(), 1):.3f}({shelf.sum() / max((cont & ok).sum(), 1):.3f},"
              f"{(wet & ~shelf).sum() / max((cont & ok).sum(), 1):.3f})  {wet.sum() / n:.4f}"
              f"  {(land & ~cont).sum() / max(land.sum(), 1):.3f}  {sea:7.1f}  {ice_v / 1e6:6.2f}"
              f"  {vol / 1e9:6.3f}  {sea + dsea:7.1f} ({dsea:+.0f})  {int((ls * cell_km2 >= 1e6).sum()):3d}"
              f"  {ls.max() * cell_km2 / 1e6:6.1f}")


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
