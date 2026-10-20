#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Lakes and rivers of a run, as the map draws them and as the bake made them.

Ben, 2026-10-20: "the majority of the world surface (that isn't mountains) appears to be getting
dominated by rivers & lakes". This counts, for every keyframe of a world-data folder:
  - the lakes (open and frozen) as a share of the land and of the ice-free land,
  - the largest lake (8-connected, wrapping east-west) as a share of the planet,
  - the share of the land the `rivers` layer draws as river (pixels touched, and the ink: a
    pixel's share of river when a thread narrower than a pixel is blended over the land),
from the `rivers` layer's colours (crates/deeptime-core/src/water_view.rs: LAND with a hill shade,
SEA, LAKE, FROZEN, PLAYA; anything else is a river on the `rivers` ramp) and the frame's
`climate/ice_thickness_m` layer (under 1 m of ice is #141414 on the `ice` ramp). The map is the
Lambert cylindrical equal-area projection (keyframes::render), so every pixel is the same area
and shares are pixel counts.

With --bake it also reads the bake's own water (what the game places): `notes.bake.water.stats`
of manifest.json, the `water_level` and `elevation` rasters (L3), the land ice (`climate/ice_cover`)
and `rivers/rivers.dtrv` (each segment's length times its channel width, `2 hw`), so the
in-game lake share, largest lake and river share sit beside the map's.

    python3 tools/review/water_census.py --world <world-data> [--bake] [--every N] [--json out.json]

Needs numpy, scipy and lz4 (on the VM: nix shell --impure --expr
'(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages
(p: [p.numpy p.scipy p.lz4])' -c python3 ...).
"""

import argparse
import json
import struct
import sys
from pathlib import Path

import lz4.block
import numpy as np
from scipy import ndimage

SEA = (0x2C, 0x40, 0x56)
LAKE = (0x5C, 0xC0, 0xEA)
FROZEN = (0xA9, 0xD3, 0xE6)
PLAYA = (0xF0, 0xD9, 0xA8)
LAND = (0xDD, 0xD9, 0xCD)
NO_ICE = (0x14, 0x14, 0x14)
EARTH_KM2 = 510.07e6


def read_dtk(path: Path, want: set[str]) -> tuple[dict, dict]:
    """The layers of a `.dtk` keyframe named in `want` as (h, w, 4) uint8 arrays (FORMAT.md)."""
    data = path.read_bytes()
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
    return out, {"n": n, "t": t, "stage": names[0], "w": w, "h": h}


def is_rgb(img: np.ndarray, rgb) -> np.ndarray:
    return (img[..., 0] == rgb[0]) & (img[..., 1] == rgb[1]) & (img[..., 2] == rgb[2])


def is_land_shade(img: np.ndarray) -> np.ndarray:
    """LAND times a hill-shade factor in [0.6, 1.2] (water_view.rs `render`)."""
    f = img[..., 0].astype(np.float32) / LAND[0]
    ok = (f >= 0.59) & (f <= 1.21)
    for c in (1, 2):
        ok &= np.abs(img[..., c].astype(np.float32) - LAND[c] * f) <= 1.5
    return ok


def largest_component(mask: np.ndarray) -> int:
    """Pixels of the largest 8-connected component of `mask`, wrapping east-west."""
    if not mask.any():
        return 0
    lab, nlab = ndimage.label(mask, structure=np.ones((3, 3), bool))
    sizes = np.bincount(lab.ravel())
    sizes[0] = 0
    # Join labels across the seam (column 0 with the last column, 8-connected).
    parent = list(range(nlab + 1))

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    h = mask.shape[0]
    left, right = lab[:, 0], lab[:, -1]
    for y in range(h):
        if left[y] == 0:
            continue
        for dy in (-1, 0, 1):
            yy = y + dy
            if 0 <= yy < h and right[yy]:
                a, b = find(left[y]), find(right[yy])
                if a != b:
                    parent[a] = b
    total = {}
    for k in range(1, nlab + 1):
        r = find(k)
        total[r] = total.get(r, 0) + int(sizes[k])
    return max(total.values())


# The `rivers` ramp of ramps.json (log10 m3/s, colour), sampled for the ink estimate.
RIVER_STOPS = [(0.3, (0x4A, 0xA0, 0xF2)), (1.3, (0x2F, 0x7F, 0xE0)), (2.3, (0x1C, 0x5C, 0xC4)),
               (3.5, (0x0F, 0x3F, 0x95))]


def river_ramp(n: int = 64) -> np.ndarray:
    xs = np.array([s[0] for s in RIVER_STOPS])
    cs = np.array([s[1] for s in RIVER_STOPS], dtype=np.float32)
    v = np.linspace(xs[0], xs[-1], n)
    return np.stack([np.interp(v, xs, cs[:, c]) for c in range(3)], -1)


def river_ink(riv: np.ndarray, river: np.ndarray, land_dry: np.ndarray) -> np.ndarray:
    """How much of each river pixel is river: water_view.rs draws a river narrower than a pixel
    as a thread blended at its coverage over the land, so a pixel is `base (1 - a) + c a` with c
    on the ramp. The base is the mean of the dry land pixels within two pixels (LAND where there
    are none); a and c are the ramp colour that explains the pixel best. A hard-edged river pixel
    (the drawing before 2026-10-20) is a = 1."""
    img = riv[..., :3].astype(np.float32)
    k = np.ones((5, 5), np.float32)
    wsum = ndimage.convolve(land_dry.astype(np.float32), k, mode="wrap")
    base = np.empty_like(img)
    for c in range(3):
        s = ndimage.convolve(img[..., c] * land_dry, k, mode="wrap")
        base[..., c] = np.where(wsum > 0, s / np.maximum(wsum, 1e-6), LAND[c])
    ys, xs = np.nonzero(river)
    p = img[ys, xs]
    b = base[ys, xs]
    ramp = river_ramp()
    best_a = np.zeros(len(p), np.float32)
    best_e = np.full(len(p), np.inf, np.float32)
    for col in ramp:
        d = col[None, :] - b
        a = np.clip(((p - b) * d).sum(1) / np.maximum((d * d).sum(1), 1e-6), 0.0, 1.0)
        e = ((b + a[:, None] * d - p) ** 2).sum(1)
        better = e < best_e
        best_e[better] = e[better]
        best_a[better] = a[better]
    ink = np.zeros(river.shape, np.float32)
    ink[ys, xs] = best_a
    return ink


def frame_stats(layers: dict, ice: np.ndarray | None) -> dict | None:
    riv = layers.get("rivers")
    if riv is None:
        return None
    valid = riv[..., 3] > 0
    sea = is_rgb(riv, SEA) & valid
    lake = is_rgb(riv, LAKE) & valid
    frozen = is_rgb(riv, FROZEN) & valid
    playa = is_rgb(riv, PLAYA) & valid
    land_dry = is_land_shade(riv) & valid
    river = valid & ~(sea | lake | frozen | playa | land_dry)
    land = valid & ~sea
    water = lake | frozen
    ink = river_ink(riv, river, land_dry)
    planet = int(valid.sum())
    n_land = int(land.sum())
    out = {
        "planet_px": planet,
        "land_share_of_planet": n_land / planet if planet else None,
        "lake_share_of_land": int(water.sum()) / n_land if n_land else None,
        "frozen_share_of_lake": int(frozen.sum()) / max(int(water.sum()), 1),
        "playa_share_of_land": int(playa.sum()) / n_land if n_land else None,
        "river_share_of_land": int(river.sum()) / n_land if n_land else None,
        "river_ink_share_of_land": float(ink.sum()) / n_land if n_land else None,
        "largest_lake_share_of_planet": largest_component(water) / planet if planet else None,
    }
    if ice is not None:
        iv = ice[..., 3] > 0
        icy = iv & ~is_rgb(ice, NO_ICE)
        free = land & ~icy
        nf = int(free.sum())
        out["ice_share_of_land"] = int((land & icy).sum()) / n_land if n_land else None
        out["lake_share_of_ice_free_land"] = int((water & free).sum()) / nf if nf else None
        out["river_share_of_ice_free_land"] = int((river & free).sum()) / nf if nf else None
    return out


def read_dtr(path: Path) -> tuple[np.ndarray, dict]:
    """A whole tile pack as its stored integers and its description (FORMAT.md, "Tile pack")."""
    data = path.read_bytes()
    assert data[:4] == b"DTR1", path
    header_len = struct.unpack_from("<I", data, 4)[0]
    desc = json.loads(data[16:header_len])
    g = desc["grid"]
    w, h, tile = g["w"], g["h"], desc["tile"]
    dt = {"i16": np.int16, "u8": np.uint8, "f32": np.float32, "u16": np.uint16}[desc["dtype"]]
    size = np.dtype(dt).itemsize
    tx, tz = -(-w // tile), -(-h // tile)
    index_off = (header_len + 7) // 8 * 8
    out = np.zeros((h, w), dtype=dt)
    for t in range(tx * tz):
        off, length, _crc = struct.unpack_from("<QII", data, index_off + 16 * t)
        x0, z0 = (t % tx) * tile, (t // tx) * tile
        tw, th = min(tile, w - x0), min(tile, h - z0)
        raw = data[off:off + length]
        if desc["codec"] == "lz4_block":
            raw = lz4.block.decompress(raw, uncompressed_size=tw * th * size)
        out[z0:z0 + th, x0:x0 + tw] = np.frombuffer(raw, dtype=dt).reshape(th, tw)
    return out, desc


def read_rivers(path: Path) -> tuple[np.ndarray, np.ndarray, int]:
    """`rivers/rivers.dtrv`: the vertices (x, z, w, hw, depth, q) and the chains (first, count)."""
    data = path.read_bytes()
    assert data[:4] == b"DTRV", path
    fixed = struct.unpack_from("<7I", data, 16)
    c, nv, nc = fixed[0], fixed[4], fixed[5]
    table = 16 + 48
    s = [struct.unpack_from("<QII", data, table + 16 * k) for k in range(4)]
    verts = np.frombuffer(data[s[0][0]:s[0][0] + s[0][1]], dtype="<f4").reshape(nv, 6)
    chains = np.frombuffer(data[s[1][0]:s[1][0] + s[1][1]], dtype="<u4").reshape(nc, 4)
    return verts, chains, c


def bake_stats(world: Path) -> dict:
    man = json.loads((world / "manifest.json").read_text())
    water = man["notes"]["bake"]["water"]
    st = water["stats"]
    scale = man["scale"]
    m_h = scale.get("m_per_block_h", 300.0)
    sea_y = scale.get("sea_level_y", 63.0)
    km2_per_block = (m_h / 1000.0) ** 2
    out = {
        "land_km2": st["land_km2"],
        "lake_area_km2": st["lake_area_km2"],
        "lake_share_of_land": st["lake_share_of_land"],
        "largest_lake_km2": st["largest_lake_km2"],
        "lakes": st["lakes"],
        "chains": st.get("chains"),
        "basins": st.get("basins"),
    }
    # The planet in the bake's own accounting: the hydrology counts a block as m_h² and the
    # play map is C blocks round and C / 2 from pole to pole (`z_edges` "half_turn"), so the
    # planet is C² / 2 blocks (π / 2 times the sphere of circumference C: the projection is
    # not equal-area, and every share here is of blocks).
    verts, chains, c = read_rivers(world / "rivers" / "rivers.dtrv")
    planet_km2 = c * (c / 2.0) * km2_per_block
    out["planet_km2"] = planet_km2
    out["largest_lake_share_of_planet"] = st["largest_lake_km2"] / planet_km2
    out["largest_lake_earth_equiv_km2"] = st["largest_lake_km2"] / planet_km2 * EARTH_KM2
    # Rivers: each segment's length times its channel width (2 hw), blocks².
    area = 0.0
    length = 0.0
    hw_len = []
    for first, count, _mouth, _into in chains:
        v = verts[first:first + count]
        if len(v) < 2:
            continue
        dx = np.diff(v[:, 0])
        dz = np.diff(v[:, 1])
        # Across the seam a segment is short the other way round.
        dx = np.where(dx > c / 2, dx - c, np.where(dx < -c / 2, dx + c, dx))
        seg = np.hypot(dx, dz)
        hw = 0.5 * (v[:-1, 3] + v[1:, 3])
        area += float((seg * 2.0 * hw).sum())
        length += float(seg.sum())
        hw_len.append(np.stack([hw, seg], 1))
    hw_len = np.concatenate(hw_len) if hw_len else np.zeros((0, 2))
    out["river_km"] = length * m_h / 1000.0
    out["river_area_km2"] = area * km2_per_block
    out["river_share_of_land"] = out["river_area_km2"] / st["land_km2"]
    out["drainage_density_km_per_km2"] = out["river_km"] / st["land_km2"]
    if len(hw_len):
        order = np.argsort(hw_len[:, 0])
        cum = np.cumsum(hw_len[order, 1])
        def wq(p):
            return float(2.0 * hw_len[order[np.searchsorted(cum, p * cum[-1])], 0])
        out["channel_width_blocks_by_length"] = {"p10": wq(0.1), "p50": wq(0.5), "p90": wq(0.9), "p99": wq(0.99)}
    # Ice-free land from the bake's rasters, when present.
    layers = man.get("layers", {})
    try:
        wl, wdesc = read_dtr(world / "rasters" / "water_level.dtr")
        el, edesc = read_dtr(world / "rasters" / "elevation.dtr")
        nodata = wdesc.get("nodata")
        sea_raw = sea_y * 16 if edesc.get("scale", 1.0) != 1.0 else sea_y
        wet = wl != (nodata if nodata is not None else np.iinfo(wl.dtype).min)
        out["water_level_desc"] = {k: wdesc.get(k) for k in ("dtype", "scale", "offset", "nodata", "grid")}
        out["elevation_desc"] = {k: edesc.get(k) for k in ("dtype", "scale", "offset", "nodata")}
        e_blocks = el.astype(np.float32) * edesc.get("scale", 1.0) + edesc.get("offset", 0.0)
        land = e_blocks > sea_y - 1.0
        out["raster_land_cells"] = int(land.sum())
        out["raster_lake_cells"] = int((wet & land).sum())
        out["raster_lake_share_of_land"] = int((wet & land).sum()) / max(int(land.sum()), 1)
        del sea_raw
    except (FileNotFoundError, KeyError) as e:
        out["raster_error"] = str(e)
    for name in ("climate/ice_cover", "rasters/climate/ice_cover.dtr"):
        p = world / "rasters" / "climate" / "ice_cover.dtr"
        if p.exists():
            ic, idesc = read_dtr(p)
            out["ice_cover_desc"] = {k: idesc.get(k) for k in ("dtype", "scale", "offset", "nodata", "grid")}
            out["ice_cover_values"] = {int(k): int(v) for k, v in zip(*np.unique(ic, return_counts=True))} if ic.dtype == np.uint8 else None
            break
    out["layers_named"] = [k for k in layers if "ice" in k or "water" in k or "lake" in k]
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--world", required=True, type=Path)
    ap.add_argument("--bake", action="store_true", help="also the bake's own water")
    ap.add_argument("--every", type=int, default=1, help="every Nth keyframe")
    ap.add_argument("--json", type=Path, help="write everything here")
    a = ap.parse_args()
    index = json.loads((a.world / "keyframes" / "index.json").read_text())
    frames = [f for f in index["frames"] if "rivers" in f["layers"]]
    rows = []
    last_ice = None
    for i, f in enumerate(frames):
        if i % a.every and i != len(frames) - 1:
            continue
        want = {"rivers", "climate/ice_thickness_m", "young/ice_thickness_m"}
        layers, meta = read_dtk(a.world / f["path"], want)
        ice = layers.get("climate/ice_thickness_m", layers.get("young/ice_thickness_m"))
        if ice is not None:
            last_ice = ice
        st = frame_stats(layers, ice if ice is not None else last_ice)
        if st is None:
            continue
        st.update({"n": meta["n"], "myr_ago": -meta["t"] / 1e6, "stage": meta["stage"]})
        rows.append(st)
        print(json.dumps({k: (round(v, 5) if isinstance(v, float) else v) for k, v in st.items()}), flush=True)

    def summary(key, sel):
        v = np.array([r[key] for r in sel if r.get(key) is not None], dtype=float)
        if not len(v):
            return None
        return {"p10": float(np.percentile(v, 10)), "p50": float(np.median(v)), "p90": float(np.percentile(v, 90)),
                "max": float(v.max()), "n": int(len(v))}

    plate = [r for r in rows if r["stage"].startswith("plates")]
    keys = ["lake_share_of_land", "lake_share_of_ice_free_land", "largest_lake_share_of_planet",
            "river_share_of_land", "river_ink_share_of_land", "river_share_of_ice_free_land", "ice_share_of_land",
            "land_share_of_planet"]
    summ = {"all_plate_frames": {k: summary(k, plate) for k in keys},
            "first_150_myr": {k: summary(k, [r for r in plate if r["myr_ago"] > 850]) for k in keys},
            "after_150_myr": {k: summary(k, [r for r in plate if r["myr_ago"] <= 850]) for k in keys},
            "last_300_myr": {k: summary(k, [r for r in plate if r["myr_ago"] <= 300]) for k in keys},
            "final_frames": [r for r in rows if not r["stage"].startswith("plates")]}
    print("SUMMARY " + json.dumps(summ))
    out = {"frames": rows, "summary": summ}
    if a.bake:
        b = bake_stats(a.world)
        print("BAKE " + json.dumps(b))
        out["bake"] = b
    if a.json:
        a.json.write_text(json.dumps(out, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
