#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Offline what-if for a pack placement table (design aid for the Terralith table, 2026-10-04).

    tools/review/terralith_emulate.py <name> [--jar <Terralith jar>] [--runs <dir>] [--exp <name>,...]

Reads `<runs>/<name>-layers.npy` (`terralith-biomes.sh dump`: the baked biome inputs on the census's
grid), `<runs>/<name>-mic-climate_mapping.json` (the world's MIC mapping) and `<runs>/<name>-mic-biomemap.json`
(the game's census of the same world), and the pack's overworld parameter list. It answers the
climate search for every land column as the game would (the mapping's five tables, then the nearest
parameter entry, the volcanic biomes kept out), checks that against the census (agreement share), then
applies each experiment's change to the band coordinates of the classes it names (as the placement
table would) and prints the biome shares per class before and after. Nothing here is shipped; the
real numbers come from a bake and the in-game census.
"""
import argparse
import base64
import glob
import json
import math
import struct
import zipfile
from collections import Counter

import numpy as np

PL = "data/minecraft/worldgen/multi_noise_biome_source_parameter_list/overworld.json"
KEYS = ("temperature", "humidity", "continentalness", "erosion", "weirdness", "depth")
REF = {
    "temperature": [-1.0, -0.45, -0.15, 0.2, 0.55, 1.0],
    "vegetation": [-1.0, -0.35, -0.1, 0.1, 0.3, 1.0],
    "continents": [-1.2, -1.05, -0.455, -0.19, -0.11, 0.03, 0.3, 1.0],
    "erosion": [-1.0, -0.78, -0.375, -0.2225, 0.05, 0.45, 0.55, 1.0],
}
CODES = ["Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc", "Cfa",
         "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF", "sea"]
VOLCANIC = {"terralith:yellowstone", "terralith:volcanic_peaks", "terralith:volcanic_crater", "terralith:caldera",
            "terralith:ashen_savanna", "terralith:basalt_cliffs"}


def rng(v):
    if isinstance(v, (int, float)):
        return (float(v), float(v))
    if isinstance(v, list):
        return (float(v[0]), float(v[1]))
    return (float(v["min"]), float(v["max"]))


def load_entries(jar):
    pl = json.loads(zipfile.ZipFile(glob.glob(jar)[0]).read(PL))
    entries = pl.get("lithostitched:biomes") or pl.get("biomes")
    names, lo, hi, off = [], [], [], []
    for e in entries:
        if e["biome"] in VOLCANIC:
            continue
        names.append(e["biome"])
        r = [rng(e["parameters"][k]) for k in KEYS]
        lo.append([a for a, _ in r])
        hi.append([b for _, b in r])
        off.append(float(e["parameters"].get("offset", 0)))
    return names, np.array(lo), np.array(hi), np.array(off)


def nearest(pts, lo, hi, off):
    out = np.empty(len(pts), dtype=np.int32)
    for s in range(0, len(pts), 1500):
        p = pts[s:s + 1500, None, :]
        d = np.maximum(0.0, np.maximum(lo[None] - p, p - hi[None]))
        dist = (d * d).sum(axis=2) + off[None] ** 2
        out[s:s + 1500] = dist.argmin(axis=1)
    return out


def table(t):
    f, to = np.array(t["from"]), np.array(t["to"])
    odd = t.get("odd", False)

    def apply(v):
        a = np.abs(v) if odd else v
        r = np.interp(a, f, to)
        r = np.where(a < f[0], to[0] + (a - f[0]), r)
        r = np.where(a > f[-1], to[-1] + (a - f[-1]), r)
        return np.copysign(r, v) if odd else r
    return apply


def band_coord(edges, v):
    e = np.array(edges)
    n = len(e) - 1
    k = np.clip(np.searchsorted(e, v, side="left") - 1, 0, n - 1)
    b = k + (v - e[k]) / (e[k + 1] - e[k])
    return np.clip(b, 0, n)


def band_value(edges, b):
    e = np.array(edges)
    n = len(e) - 1
    b = np.clip(b, 0, n)
    k = np.minimum(np.floor(b).astype(int), n - 1)
    return e[k] + (b - k) * (e[k + 1] - e[k])


def experiments():
    """Each: class codes -> function(cols dict) -> cols dict (band coords / reference values, masked)."""
    def am_wet(c):
        c["h"] = np.maximum(c["h"], 4.05)
        return c

    def cs_dry(c):
        c["h"] = np.minimum(c["h"], 0.95)
        return c

    def cs_dry_half(c):
        c["h"] = np.full_like(c["h"], 0.5)
        return c

    def bsk(lo, hi, dry):
        def f(c):
            c["e"] = np.clip(c["e"], lo, hi)
            if dry:
                c["h"] = np.minimum(c["h"], 0.95)
            return c
        return f

    def et_fixed(h, elo=None):
        def f(c):
            c["h"] = np.full_like(c["h"], h)
            if elo is not None:
                c["e"] = np.maximum(c["e"], elo)
            return c
        return f

    def cloud(lat_max, h_min, cmin, emin, t=1.9, hum=3.5):
        def f(c):
            m = (c["lat"] < lat_max) & (c["height_m"] > h_min)
            c["t"] = np.where(m, t, c["t"])
            c["h"] = np.where(m, np.maximum(c["h"], hum), c["h"])
            c["e"] = np.where(m, np.maximum(c["e"], emin), c["e"])
            c["c"] = np.where(m & (c["cb"] >= 4), np.maximum(c["c"], cmin), c["c"])
            return c
        return f

    def bsk_fixed(h, lo, hi):
        def f(c):
            c["h"] = np.full_like(c["h"], h)
            c["e"] = np.clip(c["e"], lo, hi)
            return c
        return f

    def et_pos(h=None):
        def f(c):
            c["w"] = np.abs(c["w"])
            if h is not None:
                c["h"] = np.full_like(c["h"], h)
            return c
        return f

    return {
        "bsk_h03": (["BSk"], bsk_fixed(0.3, 0.12, 0.40)),
        "bsk_h03_narrow": (["BSk"], bsk_fixed(0.3, 0.12, 0.38)),
        "bsk_h03_free": (["BSk"], bsk_fixed(0.3, -2.0, 2.0)),
        "et_pos": (["ET"], et_pos()),
        "et_pos_15": (["ET"], et_pos(1.5)),
        "am_wet": (["Am"], am_wet),
        "cs_dry": (["Csa", "Csb", "Csc"], cs_dry),
        "cs_half": (["Csa", "Csb", "Csc"], cs_dry_half),
        "bsk_steppe": (["BSk"], bsk(0.05, 0.40, False)),
        "bsk_steppe_dry": (["BSk"], bsk(0.05, 0.40, True)),
        "bsk_wide_dry": (["BSk"], bsk(-0.10, 0.40, True)),
        "bsk_low_dry": (["BSk"], bsk(0.10, 0.30, True)),
        "et_15": (["ET"], et_fixed(1.5)),
        "et_15_flat": (["ET"], et_fixed(1.5, 0.06)),
        "cloud": (["Cfb", "Cwb", "Cfc", "Cwc"], cloud(25, 1000, 0.35, 0.62)),
        "cloud_t15": (["Cfb", "Cwb", "Cfc", "Cwc"], cloud(25, 1000, 0.35, 0.62, t=1.5)),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--runs", default="/Volumes/Graphide/ban/deeptime-runs/terralith-biomes")
    ap.add_argument("--jar", default="/Volumes/Graphide/ban/mic/mods/deep-time/smoke/mic/server/mods/Terralith_1.21.1_v2.6.2_Neoforge.jar")
    ap.add_argument("--exp", default="")
    ap.add_argument("--top", type=int, default=8)
    ap.add_argument("--m-per-block", type=float, default=35.0)
    ap.add_argument("--sea-y", type=float, default=63.0)
    a = ap.parse_args()
    base = f"{a.runs}/{a.name}"
    arr = np.load(f"{base}-layers.npy")
    meta = json.load(open(f"{base}-layers.npy.json"))
    ch = {n: i for i, n in enumerate(meta["channels"])}
    rows, cols = arr.shape[:2]
    mp = json.load(open(f"{base}-mic-climate_mapping.json"))["tables"]
    names, lo, hi, off = load_entries(a.jar)
    get = lambda n: arr[:, :, ch[n]].reshape(-1).astype(np.float64)
    T, V, C, E, W = (get("climate/" + n) for n in ("temperature", "vegetation", "continents", "erosion", "ridges"))
    K = np.nan_to_num(get("climate/koppen"), nan=31).round().astype(int)
    H = get(meta["channels"][0])
    z0 = meta["box"][1]
    z = np.repeat(z0 + meta["step"] * np.arange(rows), cols).astype(np.float64)
    R = 16384 / (2 * math.pi)
    lat = np.degrees(np.arcsin(np.clip(z / R, -1, 1)))
    land = (K != 31) & (H > a.sea_y)
    idx = np.nonzero(land)[0]
    print(f"grid {rows}x{cols}, land columns {len(idx)}, entries {len(names)}")

    def answer(t, v, c, e, w):
        pts = np.stack([table(mp["climate/temperature"])(t), table(mp["climate/vegetation"])(v),
                        table(mp["climate/continents"])(c), table(mp["climate/erosion"])(e),
                        table(mp["climate/ridges"])(w), np.zeros_like(t)], axis=1)
        return nearest(pts, lo, hi, off)

    base_ans = answer(T[idx], V[idx], C[idx], E[idx], W[idx])
    # Validation against the census.
    try:
        m = json.load(open(f"{base}-mic-biomemap.json"))
        bio = struct.unpack("<%dH" % (m["w"] * m["h"]), base64.b64decode(m["biome_u16le"]))
        pal = [p["id"] for p in m["palette"]]
        game = np.array([pal[bio[i]] for i in idx]) if m["w"] * m["h"] == rows * cols else None
        if game is not None:
            emu = np.array([names[j] for j in base_ans])
            ok = (emu == game) | np.isin(game, list(VOLCANIC))
            print(f"agreement with the census: {ok.mean():.3f} of {len(idx)} land columns")
    except FileNotFoundError:
        print("no census to check against")
    by_class = {}
    for code in set(K[idx]):
        by_class[code] = idx[K[idx] == code]

    def shares(ans_idx, sel):
        cnt = Counter(names[j] for j in ans_idx[sel])
        n = sum(cnt.values())
        return n, ", ".join(f"{b.replace('terralith:', 't:').replace('minecraft:', '')} {100 * v / n:.0f}" for b, v in cnt.most_common(a.top))

    for ename in [e for e in a.exp.split(",") if e]:
        classes, fn = experiments()[ename]
        print(f"== {ename}")
        for cname in classes:
            code = CODES.index(cname)
            sel = K[idx] == code
            if not sel.any():
                print(f"  {cname}: no columns")
                continue
            j = idx[sel]
            cdict = {
                "t": band_coord(REF["temperature"], T[j]), "h": band_coord(REF["vegetation"], V[j]),
                "c": C[j], "cb": band_coord(REF["continents"], C[j]), "e": E[j], "w": W[j],
                "lat": np.abs(lat[j]), "height_m": (H[j] - a.sea_y) * a.m_per_block,
            }
            cdict = fn(cdict)
            new = answer(band_value(REF["temperature"], cdict["t"]), band_value(REF["vegetation"], cdict["h"]),
                         cdict["c"], cdict["e"], cdict["w"])
            n, before = shares(base_ans, sel)
            _, after = shares(new, np.ones(len(new), bool))
            print(f"  {cname} ({n}): before {before}\n  {' ' * len(cname)}        after  {after}")


if __name__ == "__main__":
    main()
