#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The delta census of the deltas lane (V1 gap 5, stage 1) over one or more world-data folders.

Per folder: sea mouths, sea-reaching rivers, distributaries and mouths per river, the delta land
(the `delta` raster's raised cells, which stand above the sea), its share of the land, and the
share of the 25 largest sea-reaching rivers with a delta (a plain at the mouth, distributaries).

    python3 -I tools/review/delta_census.py <name>=<world-data> [<name>=<world-data> ...] [--json out.json] [--top out.json]

Needs numpy and lz4 (on the VM: nix shell --impure --expr '(builtins.getFlake "nixpkgs")
.legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.lz4])' -c python3 ...).
"""

import json
import struct
import sys
from pathlib import Path

import lz4.block
import numpy as np

NO_CHAIN = 0xFFFFFFFF
MOUTH_SEA = 0


def read_dtr(path: Path):
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


def read_rivers(path: Path):
    """`rivers/rivers.dtrv` (layout 2): vertices (x, z, w, hw, depth, q), chains (first, count,
    mouth word, into, from)."""
    data = path.read_bytes()
    assert data[:4] == b"DTRV", path
    fixed = struct.unpack_from("<7I", data, 16)
    c, nv, nc = fixed[0], fixed[4], fixed[5]
    assert struct.unpack_from("<I", data, 16 + 48)[0] == 2, f"{path}: river file layout 2 expected"
    table = 16 + 52
    s = [struct.unpack_from("<QII", data, table + 16 * k) for k in range(4)]
    verts = np.frombuffer(data[s[0][0]:s[0][0] + s[0][1]], dtype="<f4").reshape(nv, 6)
    chains = np.frombuffer(data[s[1][0]:s[1][0] + s[1][1]], dtype="<u4").reshape(nc, 5)
    return verts, chains, c


def bilinear_at(arr: np.ndarray, grid: dict, bx: np.ndarray, bz: np.ndarray) -> np.ndarray:
    """`arr` read bilinear at blocks (bx, bz), the game's read of a climate layer (rows bz,
    columns bx; x wraps, z clamps), as `tools/review/biome_land_census.py` reads it."""
    w, hh, r = grid["w"], grid["h"], grid["r"]
    u = (bx - grid["origin_block_x"]) / r
    v = np.clip((bz - grid["origin_block_z"]) / r, 0, hh - 1)
    i0 = np.floor(u).astype(np.int64)
    j0 = np.floor(v).astype(np.int64)
    fu = (u - i0).astype(np.float32)
    fv = (v - j0).astype(np.float32)
    j1 = np.minimum(j0 + 1, hh - 1)
    ia, ib = i0 % w, (i0 + 1) % w
    a = arr[j0][:, ia] * (1 - fu)[None, :] + arr[j0][:, ib] * fu[None, :]
    b = arr[j1][:, ia] * (1 - fu)[None, :] + arr[j1][:, ib] * fu[None, :]
    return a * (1 - fv)[:, None] + b * fv[:, None]


def dilate(m: np.ndarray, k: int) -> np.ndarray:
    """A square dilation by k cells (x wraps, z clamps)."""
    out = m.copy()
    for s in range(1, k + 1):
        out |= np.roll(m, s, axis=1) | np.roll(m, -s, axis=1)
    m2 = out.copy()
    for s in range(1, k + 1):
        out[s:] |= m2[:-s]
        out[:-s] |= m2[s:]
    return out


def plain_biomes(world: Path, man: dict, plain: np.ndarray, el: np.ndarray, ed: dict) -> dict:
    """Ocean biomes on the plains (BIO-185): the plain's dry columns (the world's rule,
    `water::sea_covers`: top block under sea level - 1 and not closed) whose continentalness, read
    bilinear from L2 at the column as the game reads it, is under the ocean | coast edge; and the
    sea's columns within an L2 cell of the plain's dry ones that read the coast's or the land's."""
    r = world / "rasters"
    cpath = r / "climate" / "continents.dtr"
    if not cpath.exists():
        return {}
    sea_y = int(man["scale"]["sea_level_y"])
    bands = man["notes"]["bake"]["climate"]["reference"]["bands"]
    edge = float(bands["continents"][3])
    er_b = bands.get("erosion", [])
    e6 = float(er_b[6]) if len(er_b) > 6 else None
    g = ed["grid"]
    h = el.astype(np.float32) * ed.get("scale", 1.0) + ed.get("offset", 0.0)
    sea = (np.ceil(h).astype(np.int32) - 1) < sea_y - 1
    cbp = r / "closed_basin.dtr"
    if cbp.exists():
        cb, cbd = read_dtr(cbp)
        if (cbd["grid"]["w"], cbd["grid"]["h"]) == (g["w"], g["h"]):
            sea &= cb == 0
    co, cod = read_dtr(cpath)
    c = co.astype(np.float32) * cod.get("scale", 1.0) + cod.get("offset", 0.0)
    cg = cod["grid"]
    bx = g["origin_block_x"] + np.arange(g["w"], dtype=np.float64) * g["r"]
    bz = g["origin_block_z"] + np.arange(g["h"], dtype=np.float64) * g["r"]
    cont = bilinear_at(c, cg, bx, bz)
    dry_plain = plain & ~sea
    ocean = dry_plain & (cont < edge)
    reach = max(int(round(cg["r"] / g["r"])) - 1, 0)
    beside = dilate(dry_plain, reach) & sea
    coast_on_sea = beside & (cont >= edge)
    out = {
        "plain_dry_columns": int(dry_plain.sum()),
        "plain_ocean_biome_columns": int(ocean.sum()),
        "plain_ocean_biome_share": round(float(ocean.sum()) / max(int(dry_plain.sum()), 1), 4),
        "sea_columns_beside_plain": int(beside.sum()),
        "sea_columns_beside_plain_coast_or_land": int(coast_on_sea.sum()),
        "all_ocean_biome_on_dry": int((~sea & (cont < edge)).sum()),
        "all_land_biome_under_sea": int((sea & (cont >= edge)).sum()),
    }
    ep = r / "climate" / "erosion.dtr"
    if e6 is not None and ep.exists():
        er, erd = read_dtr(ep)
        ev = bilinear_at(er.astype(np.float32) * erd.get("scale", 1.0) + erd.get("offset", 0.0),
                         erd["grid"], bx, bz)
        out["plain_dry_in_swamp_band_e6"] = int((dry_plain & (ev >= e6)).sum())
    return out


def census(world: Path) -> dict:
    man = json.loads((world / "manifest.json").read_text())
    sc = man["scale"]
    sea_raw = sc["sea_level_y"] * 16
    m_h = sc["m_per_block_h"]
    water = man["notes"]["bake"]["water"]
    st = water.get("stats", {})
    verts, chains, _c = read_rivers(world / "rivers" / "rivers.dtrv")
    mouth = chains[:, 2] & 0xFF
    frm = chains[:, 4]
    first = chains[:, 0].astype(np.int64)
    count = chains[:, 1].astype(np.int64)

    def root(k: int) -> int:
        seen = 0
        while frm[k] != NO_CHAIN and seen < 64:
            k = int(frm[k])
            seen += 1
        return k

    sea_chains = np.nonzero(mouth == MOUTH_SEA)[0]
    heads = [int(k) for k in sea_chains if frm[k] == NO_CHAIN]
    dist = [int(k) for k in range(len(chains)) if frm[k] != NO_CHAIN]
    per_root: dict[int, int] = {}
    for k in dist:
        r = root(k)
        per_root[r] = per_root.get(r, 0) + 1
    # A river's size: the most discharge on its main stem chain (a split cuts the stem's own q
    # below the apex, so the end is not the measure).
    qmax = {k: float(verts[first[k]:first[k] + count[k], 5].max()) for k in heads}
    ranked = sorted(heads, key=lambda k: (-qmax[k], k))

    out = {
        "world": str(world),
        "method": water.get("method"),
        "chains": int(len(chains)),
        "sea_mouth_chains": int(len(sea_chains)),
        "sea_rivers": len(heads),
        "distributaries": len(dist),
        "rivers_with_distributaries": len(per_root),
        "land_km2": st.get("land_km2"),
    }
    hist = {}
    for k in heads:
        n = 1 + per_root.get(k, 0)
        hist[n] = hist.get(n, 0) + 1
    out["mouths_per_river_hist"] = {str(k): v for k, v in sorted(hist.items())}
    for thr in (50, 100, 300, 1000):
        big = [k for k in heads if qmax[k] >= thr]
        out[f"rivers_q_ge_{thr}"] = len(big)
        out[f"rivers_q_ge_{thr}_with_distributaries"] = sum(1 for k in big if k in per_root)

    # The delta plains: the raster's raised cells.
    dpath = world / "rasters" / "delta.dtr"
    plain = None
    if dpath.exists():
        d, desc = read_dtr(dpath)
        g = desc["grid"]
        plain = d > 0
        el, eld = read_dtr(world / "rasters" / "elevation.dtr")
        above = plain & (el.astype(np.int32) > sea_raw)
        cell_km2 = (g["r"] * m_h / 1000.0) ** 2
        land = int((el.astype(np.int32) >= sea_raw).sum())
        out["delta_cells"] = int(plain.sum())
        out["delta_cells_above_sea"] = int(above.sum())
        out["delta_km2_grid"] = round(float(plain.sum()) * cell_km2, 1)
        out["land_cells_grid"] = land
        out["delta_share_of_land_cells"] = round(float(plain.sum()) / max(land, 1), 5)
        out["delta_raise_blocks_max"] = round(float(d.max()) / 16.0, 2)
        out["grid"] = {k: g[k] for k in ("name", "w", "h", "r", "origin_block_x", "origin_block_z")}
        out["plain_biomes"] = plain_biomes(world, man, plain, el, eld)
    top = []
    for rank, k in enumerate(ranked[:25], 1):
        f, n = first[k], count[k]
        last = verts[f + n - 1]
        rec = {
            "rank": rank,
            "chain": k,
            "q_max": round(qmax[k], 1),
            "mouth_x": round(float(last[0])),
            "mouth_z": round(float(last[1])),
            "distributaries": per_root.get(k, 0),
        }
        if plain is not None:
            g = out["grid"]
            # A plain at the mouth: a raised cell within 6 cells of the stem's last 30 vertices.
            hit = False
            for v in verts[max(f, f + n - 30):f + n]:
                i = int((v[0] - g["origin_block_x"]) // g["r"])
                j = int((v[1] - g["origin_block_z"]) // g["r"])
                i0, i1 = max(i - 6, 0), min(i + 7, g["w"])
                j0, j1 = max(j - 6, 0), min(j + 7, g["h"])
                if plain[j0:j1, i0:i1].any():
                    hit = True
                    break
            rec["plain"] = hit
        top.append(rec)
    out["top25"] = top
    out["top25_with_distributaries"] = sum(1 for r in top if r["distributaries"] > 0)
    if plain is not None:
        out["top25_with_plain"] = sum(1 for r in top if r.get("plain"))
    sd = st.get("deltas")
    if sd:
        out["bake_stats_deltas"] = {k: v for k, v in sd.items() if k not in ("largest", "fit_unsplit_plains")}
        out["bake_largest"] = sd.get("largest", [])[:10]
        out["fit_unsplit"] = sd.get("fit_unsplit_plains", [])
    return out


def main() -> int:
    args = sys.argv[1:]
    js = top = None
    if "--json" in args:
        i = args.index("--json")
        js = args[i + 1]
        del args[i:i + 2]
    if "--top" in args:
        i = args.index("--top")
        top = args[i + 1]
        del args[i:i + 2]
    res = {}
    for a in args:
        name, path = a.split("=", 1)
        res[name] = census(Path(path))
    keys = ["method", "chains", "sea_mouth_chains", "sea_rivers", "distributaries",
            "rivers_with_distributaries", "mouths_per_river_hist", "rivers_q_ge_50",
            "rivers_q_ge_50_with_distributaries", "rivers_q_ge_100",
            "rivers_q_ge_100_with_distributaries", "rivers_q_ge_300",
            "rivers_q_ge_300_with_distributaries", "delta_cells", "delta_cells_above_sea",
            "delta_km2_grid", "delta_share_of_land_cells", "delta_raise_blocks_max",
            "top25_with_distributaries", "top25_with_plain", "land_km2"]
    for k in keys:
        print(f"{k:42s}" + "".join(f" {str(res[n].get(k, '-')):>28s}" for n in res))
    for n, r in res.items():
        print(f"== {n} top 25")
        for t in r["top25"]:
            print("  ", t)
        if r.get("plain_biomes"):
            print("   plain biomes:", json.dumps(r["plain_biomes"]))
        if "bake_stats_deltas" in r:
            print("   bake stats:", json.dumps(r["bake_stats_deltas"]))
            for b in r["bake_largest"]:
                print("   largest:", json.dumps(b))
            for b in r.get("fit_unsplit", []):
                print("   unsplit:", json.dumps(b))
    if js:
        Path(js).write_text(json.dumps(res, indent=1))
    if top:
        Path(top).write_text(json.dumps({n: r["top25"] for n, r in res.items()}, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
